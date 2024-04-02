mod command;
mod connected;
mod exposure;
mod properties;

use std::sync::{mpsc::Sender, Arc};

use ccdi_common::{
    CameraParamMessage, CameraParams, ClientMessage, ConnectionState, ExposureCommand,
    ImageParamMessage, ImageParams, IoMessage, LogicStatus, ProcessMessage, StorageDetail,
    StorageMessage, StorageState, ViewState,
};
use ccdi_imager_interface::{DeviceDescriptor, ExposureArea, ImagerDriver};
use log::info;

use crate::ServiceConfig;

use self::{command::execute_command, connected::ConnectedCameraController};

// ============================================ PUBLIC =============================================

pub struct CameraController {
    driver: Box<dyn ImagerDriver>,
    state: State,
    detail: String,
    connected: Option<ConnectedCameraController>,
    view: Option<ViewState>,
    image_params: ImageParams,
    camera_params: CameraParams,
    process_tx: Sender<ProcessMessage>,
    storage_tx: Sender<StorageMessage>,
    storage_status: StorageState,
    config: Arc<ServiceConfig>,
    trigger_active: bool,
    storage_detail: StorageDetail,
    turnning_off: bool,
}

impl CameraController {
    pub fn new(
        driver: Box<dyn ImagerDriver>,
        process_tx: Sender<ProcessMessage>,
        storage_tx: Sender<StorageMessage>,
        config: Arc<ServiceConfig>,
    ) -> Self {
        Self {
            driver,
            state: State::Error,
            connected: None,
            detail: String::from("Started"),
            view: None,
            image_params: ImageParams::new(
                config.render_size,
                ExposureArea {
                    x: config.roi.x,
                    y: config.roi.y,
                    width: config.roi.width,
                    height: config.roi.height,
                },
            ),
            camera_params: CameraParams::new(),
            process_tx,
            storage_tx,
            storage_status: StorageState::Unknown,
            config,
            trigger_active: false,
            storage_detail: Default::default(),
            turnning_off: false,
        }
    }

    pub fn periodic(&mut self) -> (Vec<ClientMessage>, Vec<IoMessage>) {
        if self.turnning_off {
            return (vec![], vec![]);
        }

        let old_state = self.state;

        self.state = match self.state {
            State::Error => self.handle_error_state(),
            State::Connected => self.handle_connected_state(),
        };

        if self.state != old_state {
            info!("Camera state {:?} -> {:?}", old_state, self.state);
        }

        let new_view = self.get_view();

        let mut messages = vec![];

        if let Some(sview) = self.view.as_ref()
        {
            if sview != &new_view {
                messages.push(ClientMessage::View(Box::new(new_view)));
            }
        }

        if let Some(ref mut camera) = self.connected {
            messages.append(&mut camera.flush_messages());
        }

        let states = vec![IoMessage::SetExposureActive(self.exposure_active())];

        (messages, states)
    }

    pub fn get_view(&self) -> ViewState {
        let into_state = |value: bool| match value {
            false => ConnectionState::Disconnected,
            true => ConnectionState::Established,
        };

        ViewState {
            detail: self.detail.clone(),
            status: LogicStatus {
                camera: self.connection_state(),
                exposure: self
                    .connected
                    .as_ref()
                    .map(|cam| cam.exposure_status())
                    .unwrap_or(ConnectionState::Disconnected),
                storage: self.storage_status.clone(),
                trigger: into_state(self.trigger_active),
                required: into_state(self.camera_params.trigger_required),
                save: into_state(self.storage_detail.storage_enabled),
                loop_enabled: into_state(self.camera_params.loop_enabled),
            },
            camera_properties: self.connected.as_ref().map(|cam| cam.get_properties()),
            image_params: self.image_params.clone(),
            camera_params: self.camera_params.clone(),
            config: self.config.gui.clone(),
            storage_detail: self.storage_detail.clone(),
        }
    }

    pub fn update_image_params(&mut self, message: ImageParamMessage) {
        match message {
            ImageParamMessage::SetRoi((x, y, w, h)) => {
                info!("New ROI: X {} Y {}, {} x {}", x, y, w, h);
                self.image_params.x = x;
                self.image_params.y = y;
                self.image_params.w = w;
                self.image_params.h = h;
            }
            ImageParamMessage::SetFlipX(value) => self.image_params.flipx = value,
            ImageParamMessage::SetFlipY(value) => self.image_params.flipy = value,
            ImageParamMessage::SetPercentilePix(value) => self.image_params.percentile_pix = value,
            ImageParamMessage::SetPixelTgt(value) => self.image_params.pixel_tgt = value,
            ImageParamMessage::SetPixelTol(value) => self.image_params.pixel_tol = value,
            ImageParamMessage::SetRenderingType(rendering) => {
                self.image_params.rendering = rendering
            }
        }
        if let Some(camera) = self.connected.as_mut() {
            camera.update_image_params(self.image_params.clone());
        }
    }

    pub fn update_camera_params(&mut self, message: CameraParamMessage) {
        use CameraParamMessage::*;

        match message {
            EnableLoop(value) => self.camera_params.loop_enabled = value,
            SetGain(gain) => self.camera_params.gain = gain,
            SetTemp(temp) => self.camera_params.temperature = temp,
            SetHeatingPwm(temp) => self.camera_params.heating_pwm = temp,
            SetTime(time) => self.camera_params.time = time,
            // SetRenderingType(rendering) => self.camera_params.rendering = rendering,
            SetTriggerRequired(value) => self.camera_params.trigger_required = value,
            SetAutoExp(value) => {
                info!("Autoexposure: {}", value);
                self.camera_params.autoexp = value;
            }
            // SetFlipX(value) => self.camera_params.flipx = value,
            // SetFlipY(value) => self.camera_params.flipy = value,
            // SetPercentilePix(value) => self.camera_params.percentile_pix = value,
            // SetPixelTgt(value) => self.camera_params.pixel_tgt = value,
            // SetPixelTol(value) => self.camera_params.pixel_tol = value,
            // SetRoi((x, y, w, h)) => {
            //     info!("New ROI: X {} Y {}, {} x {}", x, y, w, h);
            //     self.camera_params.x = x;
            //     self.camera_params.y = y;
            //     self.camera_params.w = w;
            //     self.camera_params.h = h;
            // }
        }

        if let Some(camera) = self.connected.as_mut() {
            camera.update_camera_params(self.camera_params.clone());
        }
    }

    pub fn exposure_command(&mut self, command: ExposureCommand) {
        match self.connected.as_mut() {
            None => self.set_detail("Not connected - cannot handle exposure command"),
            Some(connected) => match connected.exposure_command(command) {
                Ok(_) => {}
                Err(message) => self.set_detail(&format!("Exposure command failed: {}", message)),
            },
        }
    }

    pub fn update_storage_status(&mut self, message: StorageState) {
        self.storage_status = message;
    }

    pub fn update_storage_detail(&mut self, detail: StorageDetail) {
        self.storage_detail = detail;
    }

    pub fn update_trigger_status(&mut self, value: bool) {
        self.trigger_active = value;

        if let Some(ref mut camera) = self.connected {
            camera.update_trigger_status(value);
        }
    }

    pub fn turn_off(&mut self) {
        if let Some(ref mut camera) = self.connected {
            camera.turn_off();
        }

        self.turnning_off = true;
        info!("Abort requested, executing abort.");
        execute_command(&self.config.turn_off_command);
        std::process::abort();
    }
}

// =========================================== PRIVATE =============================================

impl CameraController {
    fn exposure_active(&self) -> bool {
        let exposure_status = self
            .connected
            .as_ref()
            .map(|cam| cam.exposure_status())
            .unwrap_or(ConnectionState::Disconnected);

        matches!(exposure_status, ConnectionState::Established)
    }

    fn connection_state(&self) -> ConnectionState {
        match self.state {
            State::Error => ConnectionState::Connecting,
            State::Connected => ConnectionState::Established,
        }
    }

    fn set_detail(&mut self, detail: &str) {
        if detail != self.detail {
            info!("Detail updated: {}", detail);
        }

        self.detail = detail.to_owned();
    }

    fn handle_error_state(&mut self) -> State {
        if let Some(old_device) = self.connected.take() {
            old_device.close();
            self.set_detail("Closing old device");
        }

        match self.driver.list_devices() {
            Err(_) => {
                self.set_detail("Could not list devices");
                State::Error
            }
            Ok(devices) => match devices.as_slice() {
                [] => {
                    self.set_detail("No devices present in list");
                    State::Error
                }
                [device_id, ..] => self.connect_and_init(device_id),
            },
        }
    }

    fn connect_and_init(&mut self, id: &DeviceDescriptor) -> State {
        match self.driver.connect_device(id, &self.config.roi) {
            Err(_) => {
                self.set_detail("Connect device failed");
                State::Error
            }
            Ok(device) => {
                self.set_detail("Device connected, reading basic info");

                match ConnectedCameraController::new(
                    device,
                    self.config.render_size,
                    self.process_tx.clone(),
                    self.storage_tx.clone(),
                ) {
                    Ok(connected) => {
                        self.set_detail("Camera initialized");
                        self.connected = Some(connected);
                        State::Connected
                    }
                    Err(message) => {
                        self.set_detail(&format!("Init failed: {}", message));
                        self.connected = None;
                        State::Error
                    }
                }
            }
        }
    }

    fn handle_connected_state(&mut self) -> State {
        if let Some(ref mut controller) = self.connected {
            match controller.periodic(self.camera_params.temperature) {
                Ok(_) => State::Connected,
                Err(message) => {
                    self.set_detail(&format!("Periodic task failed: {}", message));
                    self.connected = None;
                    State::Error
                }
            }
        } else {
            State::Error
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum State {
    Error,
    Connected,
}
