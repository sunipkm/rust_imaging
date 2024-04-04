use std::{
    mem::swap,
    sync::{mpsc::Sender, Arc},
};

use ccdi_common::{
    log_err, ImageParams, CameraParams, ClientMessage, ConvertRawImage, ExposureCommand, ProcessMessage,
    RawImage, StorageMessage,
};
use ccdi_imager_interface::{BasicProperties, ExposureArea, ExposureParams, ImagerDevice};
use log::debug;
use nanocv::ImgSize;

// ============================================ PUBLIC =============================================

pub struct ExposureController {
    properties: BasicProperties,
    image_params: ImageParams,
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
            image_params: ImageParams::new(
                render_size,
                ExposureArea {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
            ),
            camera_params: CameraParams::new(),
            current_exposure: None,
            process_tx,
            storage_tx,
            trigger_active: false,
            save_active: false,
        }
    }

    pub fn periodic(
        &mut self,
        device: &mut dyn ImagerDevice,
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

        if !self.exposure_active() && self.camera_params.loop_enabled && (self.trigger_active || !self.camera_params.trigger_required) {
            self.start_exposure(device)?;
        }

        Ok(vec![])
    }

    pub fn update_image_params(&mut self, params: ImageParams) {
        self.image_params = params;
    }

    pub fn update_camera_params(&mut self, params: CameraParams) {
        self.camera_params = params;
    }

    pub fn exposure_command(
        &mut self,
        device: &mut dyn ImagerDevice,
        command: ExposureCommand,
    ) -> Result<(), String> {
        match command {
            ExposureCommand::Start => self.start_exposure(device)?,
        };
        Ok(())
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
        let size = self.image_params.render_size;

        // Package and send a message instructing the system to save the Raw image. ~Mit
        let message = StorageMessage::ProcessImage(image.clone());
        log_err("Self process message", self.storage_tx.send(message));
        
        // Package and send a message to convert the RawImage into something stupid. ~Mit
        let message = ProcessMessage::ConvertRawImage(ConvertRawImage {
            image,
            size,
        });
        log_err("Self process message", self.process_tx.send(message));
    }

    fn start_exposure(&mut self, device: &mut dyn ImagerDevice) -> Result<(), String> {
        debug!("Starting exposure");
        if self.current_exposure.is_some() {
            return Err("Exposure already in progress.".to_string());
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
        let mut x = self.image_params.x.min((self.properties.width - 1) as u16) as usize;
        let mut y = self.image_params.y.min((self.properties.height - 1) as u16) as usize;
        let mut w = self.image_params.w.min(self.properties.width as u16) as usize;
        let mut h = self.image_params.h.min(self.properties.height as u16) as usize;
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

        self.image_params.x = x as u16;
        self.image_params.y = y as u16;
        self.image_params.w = w    as u16;
        self.image_params.h = h   as u16;

        ExposureParams {
            gain: self.camera_params.gain,
            time: self.camera_params.time,
            area: ExposureArea {
                x,
                y,
                width: w,
                height: h,
            },
            autoexp: self.camera_params.autoexp,
            flipx: self.image_params.flipx,
            flipy: self.image_params.flipy,
            pixel_tgt: self.image_params.pixel_tgt,
            pixel_tol: self.image_params.pixel_tol,
            percentile_pix: self.image_params.percentile_pix,
            save: self.save_active,
        }
    }
}
