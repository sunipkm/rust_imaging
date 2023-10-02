
use std::{mem::swap, sync::{mpsc::Sender, Arc}};

use ccdi_common::{
    ExposureCommand, ClientMessage, RawImage, ProcessMessage, ConvertRawImage, log_err,
    CameraParams, StorageMessage
};
use ccdi_imager_interface::{BasicProperties, ImagerDevice, ExposureParams, ExposureArea};
use log::debug;
use nanocv::ImgSize;

// ============================================ PUBLIC =============================================

pub struct ExposureController {
    properties: BasicProperties,
    camera_params: CameraParams,
    current_exposure: Option<ExposureParams>,
    process_tx: Sender<ProcessMessage>,
    storage_tx: Sender<StorageMessage>,
    trigger_active: bool,
    save_active: bool,
}

impl ExposureController {
    pub fn new(
        render_size: ImgSize,
        properties: BasicProperties,
        process_tx: Sender<ProcessMessage>,
        storage_tx: Sender<StorageMessage>,
    ) -> Self {
        Self {
            properties,
            camera_params: CameraParams::new(render_size),
            current_exposure: None,
            process_tx,
            storage_tx,
            trigger_active: false,
            save_active: false,
        }
    }

    pub fn periodic(
        &mut self,
        device: &mut dyn ImagerDevice
    ) -> Result<Vec<ClientMessage>, String> {
        if self.current_exposure.is_some() && device.image_ready()? {
            debug!("Image ready to download");
            let mut exposure = None;
            swap(&mut exposure, &mut self.current_exposure);

            if let Some(mut params) = exposure {
                let data = device.download_image(&mut params)?;
                let raw_image = RawImage { params, data };
                debug!("Image downloaded");
                self.call_process_message(Arc::new(raw_image));
            }
        }

        if !self.exposure_active() && self.camera_params.loop_enabled {
            if self.trigger_active || !self.camera_params.trigger_required  {
                self.start_exposure(device)?;
            }
        }

        Ok(vec![])
    }

    pub fn update_camera_params(&mut self, params: CameraParams) {
        self.camera_params = params;
    }

    pub fn exposure_command(
        &mut self,
        device: &mut dyn ImagerDevice,
        command: ExposureCommand
    ) -> Result<(), String> {
        Ok(match command {
            ExposureCommand::Start => self.start_exposure(device)?,
        })
    }

    pub fn exposure_active(&self) -> bool {
        self.current_exposure.is_some()
    }

    pub fn update_trigger_status(&mut self, value: bool) {
        self.trigger_active = value;
    }
}

// =========================================== PRIVATE =============================================

impl ExposureController {
    fn call_process_message(&self, image: Arc<RawImage>) {
        let rendering = self.camera_params.rendering;
        let size = self.camera_params.render_size;
        let message = StorageMessage::ProcessImage(image.clone());
        log_err("Self process message", self.storage_tx.send(message));
        let message = ProcessMessage::ConvertRawImage(ConvertRawImage{image, size, rendering});
        log_err("Self process message", self.process_tx.send(message));
    }

    fn start_exposure(&mut self, device: &mut dyn ImagerDevice) -> Result<(), String> {
        debug!("Starting exposure");
        if self.current_exposure.is_some() {
            return Err(format!("Exposure already in progress."))
        }

        let params = self.make_exposure_description();
        let result = device.start_exposure(&params);

        if result.is_ok() {
            self.current_exposure = Some(params)
        }

        debug!("Exposure started");
        result
    }

    fn make_exposure_description(&mut self) -> ExposureParams {
        let mut x = self.camera_params.x.min(self.properties.width - 1);
        let mut y = self.camera_params.y.min(self.properties.height - 1);
        let mut w = self.camera_params.w.min(self.properties.width);
        let mut h = self.camera_params.h.min(self.properties.height);
        if w == 0 {
            w = self.properties.width;
        }
        if h == 0 {
            h = self.properties.height;
        }
        if x + w > self.properties.width {
            x = 0;
        }
        if y + h > self.properties.height {
            y = 0;
        }

        self.camera_params.x = x;
        self.camera_params.y = y;
        self.camera_params.w = w;
        self.camera_params.h = h;

        ExposureParams {
            gain: self.camera_params.gain,
            time: self.camera_params.time,
            area: ExposureArea {
                x: x,
                y: y,
                width: w,
                height: h
            },
            autoexp: self.camera_params.autoexp,
            pixel_tgt: self.camera_params.pixel_tgt,
            pixel_tol: self.camera_params.pixel_tol,
            percentile_pix: self.camera_params.percentile_pix,
            save: self.save_active,
        }        
    }
}
