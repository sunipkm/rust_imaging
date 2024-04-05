use ccdi_common::to_string;
use image::DynamicImage;
use std::{fmt::Debug, time::Duration};

use ccdi_imager_interface::{
    BasicProperties, DeviceDescriptor, DeviceProperty, ExposureArea, ExposureParams, ImagerDevice,
    ImagerDriver, ImagerProperties, OptConfigCmd, TemperatureRequest,
};

use log::{info, warn};

use cameraunit_fli::{
    get_camera_ids, open_camera, CameraInfo, CameraUnit, CameraUnitFLI, DynamicSerialImage,
    OptimumExposure, ROI,
};

pub struct FLICameraDriver {
    opt: Option<OptimumExposure>,
}

impl FLICameraDriver {
    pub fn new() -> Self {
        Self { opt: None }
    }

    pub fn update_opt_config(&mut self, config: OptimumExposure) {
        self.opt = Some(config);
    }
}

impl Default for FLICameraDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl ImagerDriver for FLICameraDriver {
    fn list_devices(&mut self) -> Result<Vec<DeviceDescriptor>, String> {
        let ids = get_camera_ids().map_err(to_string)?;
        let mut out: Vec<DeviceDescriptor> = Vec::new();
        for (id, name) in ids.iter().enumerate() {
            out.push(DeviceDescriptor {
                id: id as i32,
                name: name.clone(),
            });
        }
        Ok(out)
    }

    fn connect_device(
        &mut self,
        descriptor: &DeviceDescriptor,
        roi_request: &ExposureArea,
    ) -> Result<(Box<dyn ImagerDevice>, ExposureArea), String> {
        let (mut cam, _) = open_camera(&descriptor.name).map_err(to_string)?;
        let mut roi = *roi_request;
        if roi.height == 0 || roi.width == 0 {
            let croi = *cam.get_roi();
            info!(
                "Firstcall ROI: ({}, {}), {} x {}",
                croi.x_min, croi.y_min, croi.width, croi.height,
            );
            roi = ExposureArea {
                x: croi.x_min as usize,
                y: croi.y_min as usize,
                width: croi.width as usize,
                height: croi.height as usize,
            };
        } else {
            let roi = ROI {
                x_min: roi.x as u32,
                y_min: roi.y as u32,
                width: roi.width as u32,
                height: roi.height as u32,
                bin_x: 1,
                bin_y: 1,
            };
            cam.set_roi(&roi).map_err(|x| x.to_string())?;
        }
        let cam = Box::new(FLICameraImager {
            device: cam,
            opt: self.opt,
        });

        Ok((cam, roi))
    }
}

pub struct FLICameraImager {
    device: CameraUnitFLI,
    opt: Option<OptimumExposure>,
}

impl ImagerDevice for FLICameraImager {
    fn update_opt_config(&mut self, config: OptConfigCmd) {
        if let Some(opt) = self.opt {
            if let Ok(config) = opt
                .get_builder()
                .percentile_pix(config.percentile_pix)
                .pixel_tgt(config.pixel_tgt)
                .pixel_uncertainty(config.pixel_tol)
                .max_allowed_exp(Duration::from_secs_f32(config.max_exp))
                .build()
            {
                self.opt = Some(config);
            }
        }
    }

    fn read_properties(&mut self) -> Result<ImagerProperties, String> {
        Ok(ImagerProperties {
            basic: read_basic_props(&self.device),
            other: read_all_props(&self.device),
        })
    }

    fn close(&mut self) {
        self.device
            .cancel_capture()
            .map_err(|x| println!("ImagerDeviceClose: CancelCapture: {}", x))
            .expect("May fail");
    }

    fn start_exposure(&mut self, params: &ExposureParams) -> Result<(), String> {
        if !params.autoexp {
            self.device
                .set_exposure(Duration::from_secs_f64(params.time))
                .map_err(|x| x.to_string())?;
        }

        let roi = ROI {
            x_min: params.area.x as u32,
            y_min: params.area.y as u32,
            width: params.area.width as u32,
            height: params.area.height as u32,
            bin_x: 1,
            bin_y: 1,
        };
        self.device.set_roi(&roi).map_err(|x| x.to_string())?;
        self.device.start_exposure().map_err(|x| x.to_string())?;
        Ok(())
    }

    fn image_ready(&mut self) -> Result<bool, String> {
        self.device.image_ready().map_err(|x| x.to_string())
    }

    fn download_image(
        &mut self,
        params: &mut ExposureParams,
    ) -> Result<DynamicSerialImage, String> {
        let mut img = self.device.download_image().map_err(|x| x.to_string())?;
        if params.autoexp & self.opt.is_some() {
            match self.opt.unwrap().calculate(
                img.into_luma().into_vec(),
                self.device.get_exposure(),
                1,
            ) {
                Ok((exposure, _)) => {
                    let res = self
                        .device
                        .set_exposure(exposure)
                        .map_err(|x| x.to_string());
                    if res.is_err() {
                        warn!("Set exposure failed: {}", res.unwrap_err());
                        return Err("Autoexposure set exposure failed".to_owned());
                    }
                    params.time = exposure.as_secs_f64();
                }
                Err(e) => {
                    warn!("Autoexposure failed: {}", e);
                }
            }
        }
        if params.flipx || params.flipy {
            let meta = img.get_metadata().clone();
            let mut bimg = DynamicImage::from(img);
            if params.flipx {
                bimg = bimg.fliph();
            }
            if params.flipy {
                bimg = bimg.flipv();
            }
            img = DynamicSerialImage::from(bimg);
            if let Some(meta) = meta {
                img.set_metadata(meta);
            }
        }
        if let Some(meta) = img.get_metadata() {
            params.area.x = meta.img_left as usize;
            params.area.y = meta.img_top as usize;
            params.area.width = img.width();
            params.area.height = img.height();
        }

        Ok(img)
    }

    fn set_temperature(&mut self, request: TemperatureRequest) -> Result<(), String> {
        self.device
            .set_temperature(request.temperature)
            .map_err(|x| x.to_string())?;
        Ok(())
    }

    fn cancel_capture(&mut self) -> Result<(), String> {
        self.device.cancel_capture().map_err(|x| x.to_string())
    }
}

fn read_basic_props(device: &CameraUnitFLI) -> BasicProperties {
    let roi = device.get_roi();
    BasicProperties {
        width: device.get_ccd_width() as usize,
        height: device.get_ccd_height() as usize,
        temperature: device.get_temperature().unwrap_or(-273.0),
        exposure: device.get_exposure().as_secs_f32(),
        roi: ExposureArea {
            x: roi.x_min as usize,
            y: roi.y_min as usize,
            width: roi.width as usize,
            height: roi.height as usize,
        },
    }
}

fn read_all_props(device: &CameraUnitFLI) -> Vec<DeviceProperty> {
    vec![
        prop_f32(
            "Chip Temperature",
            device.get_temperature().unwrap_or(-273.0),
            1,
        ),
        prop("Camera ID", device.get_uuid()),
        prop_f32(
            "Min Exposure Time",
            device
                .get_min_exposure()
                .unwrap_or(Duration::from_millis(1))
                .as_secs_f32(),
            3,
        ),
        prop_f32(
            "Max Exposure Time",
            device
                .get_max_exposure()
                .unwrap_or(Duration::from_millis(1))
                .as_secs_f32(),
            3,
        ),
    ]
}

fn prop<T: Debug>(name: &str, value: T) -> DeviceProperty {
    DeviceProperty {
        name: name.to_owned(),
        value: format!("{:?}", value),
    }
}

fn prop_f32(name: &str, value: f32, precision: usize) -> DeviceProperty {
    DeviceProperty {
        name: name.to_owned(),
        value: format!("{:0.prec$}", value, prec = precision),
    }
}
