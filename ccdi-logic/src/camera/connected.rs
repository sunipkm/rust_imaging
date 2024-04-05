use std::sync::{Arc, mpsc::Sender};

use ccdi_common::{
    CameraParams, ClientMessage, ConnectionState, ExposureCommand, ImageParams, OptExposureConfig, ProcessMessage, StorageMessage
};
use ccdi_imager_interface::{ImagerDevice, ImagerProperties, TemperatureRequest};
use nanocv::ImgSize;

use super::{properties::PropertiesController, exposure::ExposureController};

// ============================================ PUBLIC =============================================

pub struct ConnectedCameraController {
    device: Box<dyn ImagerDevice>,
    properties: PropertiesController,
    exposure: ExposureController,
    messages: Vec<ClientMessage>,
    last_temperature_set: Option<f64>,
}

impl ConnectedCameraController {
    pub fn new(
        mut device: Box<dyn ImagerDevice>,
        render_size: ImgSize,
        process_tx: Sender<ProcessMessage>,
        storage_tx: Sender<StorageMessage>,
        opt: OptExposureConfig,
    ) -> Result<Self, String> {
        let properties = PropertiesController::new(device.as_mut())?;

        let exposure = ExposureController::new(
            render_size, properties.get_properties().basic, process_tx, storage_tx, opt
        );

        let last_temperature_set = None;
        Ok(Self {properties, exposure, device, messages: vec![], last_temperature_set})
    }

    pub fn close(mut self) {
        self.device.close()
    }

    pub fn periodic(&mut self, temperature: f64) -> Result<(), String> {
        self.messages.append(&mut self.exposure.periodic(self.device.as_mut())?);

        if self.last_temperature_set != Some(temperature) {
            self.device.set_temperature(
                TemperatureRequest { temperature: temperature as f32, speed: 3.0 }
            )?;
            self.last_temperature_set = Some(temperature);
        }

        self.properties
            .read_properties(self.device.as_mut())
            .map_err(|_| "Periodic read properties failed".to_string())

    }

    pub fn get_properties(&self) -> Arc<ImagerProperties> {
        self.properties.get_properties()
    }

    // Probably don't need update_image_params here...?

    pub fn update_camera_params(&mut self, params: CameraParams) {
        self.exposure.update_camera_params(params);
    }

    pub fn update_image_params(&mut self, params: ImageParams) {
        self.exposure.update_image_params(params);
    }

    pub fn exposure_command(&mut self, command: ExposureCommand) -> Result<(), String> {
        self.exposure.exposure_command(self.device.as_mut(), command)
    }

    pub fn flush_messages(&mut self) -> Vec<ClientMessage> {
        let mut result = Vec::new();
        result.append(&mut self.messages);
        result

    }

    pub fn exposure_status(&self) -> ConnectionState {
        match self.exposure.exposure_active() {
            false => ConnectionState::Disconnected,
            true => ConnectionState::Established
        }
    }

    pub fn update_trigger_status(&mut self, value: bool) {
        self.exposure.update_trigger_status(value);
    }

    pub fn turn_off(&mut self) {
        self.device.close();
    }
}