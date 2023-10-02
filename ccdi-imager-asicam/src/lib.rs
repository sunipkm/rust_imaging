use once_cell::sync::Lazy;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
    time::Duration,
};

use ccdi_imager_interface::{
    BasicProperties, DeviceDescriptor, DeviceProperty, ExposureArea, ExposureParams, ImagerDevice,
    ImagerDriver, ImagerProperties, TemperatureRequest,
};

use log::info;

use cameraunit::{CameraInfo, CameraUnit, ROI};
use cameraunit_asi::{get_camera_ids, open_camera, CameraUnitASI};

pub struct ASICameraDriver {}

impl ASICameraDriver {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImagerDriver for ASICameraDriver {
    fn list_devices(&mut self) -> Result<Vec<DeviceDescriptor>, String> {
        let ids = get_camera_ids();
        match ids {
            Some(ids) => {
                let mut out: Vec<DeviceDescriptor> = Vec::new();
                for id in ids {
                    out.push(DeviceDescriptor {
                        id: id.0,
                        name: id.1,
                    });
                }
                return Ok(out);
            }
            None => Ok(Vec::<DeviceDescriptor>::new()),
        }
    }

    fn connect_device(
        &mut self,
        descriptor: &DeviceDescriptor,
        roi_request: &ExposureArea,
    ) -> Result<Box<dyn ImagerDevice>, String> {
        let (cam, _) = open_camera(descriptor.id).map_err(|x| x.to_string())?;
        Ok(Box::new(ASICameraImager { device: cam, roi: *roi_request }))
    }
}

pub struct ASICameraImager {
    device: CameraUnitASI,
    roi: ExposureArea,
}

impl ImagerDevice for ASICameraImager {
    fn close(&mut self) {
        self.device
            .cancel_capture()
            .map_err(|x| println!("ImagerDeviceClose: CancelCapture: {}", x))
            .expect("May fail");
        self.device
            .set_cooler(false)
            .map_err(|x| println!("ImagerDeviceClose: SetCooler: {}", x))
            .expect("May fail");
    }

    fn read_properties(&mut self) -> Result<ImagerProperties, String> {
        Ok(ImagerProperties {
            basic: read_basic_props(&self.device),
            other: read_all_props(&self.device),
        })
    }

    fn start_exposure(&mut self, params: &ExposureParams) -> Result<(), String> {
        static FIRST_CALL: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(true)));
        self.device
            .set_gain_raw(params.gain as i64)
            .map_err(|x| x.to_string())?;
        if !params.autoexp {
            self.device
                .set_exposure(Duration::from_secs_f64(params.time))
                .map_err(|x| x.to_string())?;
        }

        let roi = {
            let mut val = FIRST_CALL.lock().unwrap();
            if *val {
                let roi = ROI {
                    x_min: self.roi.x as i32,
                    x_max: (self.roi.width + self.roi.x) as i32,
                    y_min: self.roi.y as i32,
                    y_max: (self.roi.height + self.roi.y) as i32,
                    bin_x: 1,
                    bin_y: 1,
                };
                info!(
                    "Firstcall ROI: ({}, {}), {} x {}",
                    roi.x_min,
                    roi.x_max,
                    roi.x_max - roi.x_min,
                    roi.y_max - roi.y_min
                );
                *val = false;
                roi
            }
            else
            {
                ROI {
                    x_min: params.area.x as i32,
                    x_max: (params.area.width + params.area.x) as i32,
                    y_min: params.area.y as i32,
                    y_max: (params.area.height + params.area.y) as i32,
                    bin_x: 1,
                    bin_y: 1,
                }
            }
        };

        self.device.set_roi(&roi).map_err(|x| x.to_string())?;
        self.device.start_exposure().map_err(|x| x.to_string())?;
        Ok(())
    }

    fn image_ready(&mut self) -> Result<bool, String> {
        self.device.image_ready().map_err(|x| x.to_string())
    }

    fn download_image(&mut self, params: &mut ExposureParams) -> Result<Vec<u16>, String> {
        let mut img = self.device.download_image().map_err(|x| x.to_string())?;
        if params.autoexp {
            if let Ok((exposure, _)) = img.find_optimum_exposure(
                params.percentile_pix,
                params.pixel_tgt,
                params.pixel_tol,
                self.device
                    .get_min_exposure()
                    .unwrap_or(Duration::from_millis(1)),
                Duration::from_secs(60),
                1,
                100,
            ) {
                self.device
                    .set_exposure(exposure)
                    .map_err(|x| x.to_string())?;
                params.time = exposure.as_secs_f64();
            }
        }
        if params.flipx {
            let bimg = img.get_image_mut();
            bimg.fliph();
        }
        if params.flipy {
            let bimg = img.get_image_mut();
            bimg.flipv();
        }
        let roi = self.device.get_roi();
        let width = roi.x_max - roi.x_min;
        let height = roi.y_max - roi.y_min;
        params.area.x = roi.x_min as usize;
        params.area.y = roi.y_min as usize;
        params.area.width = width as usize;
        params.area.height = height as usize;
        if let Some(img) = img.get_image().as_luma16() {
            let val = img.clone().into_vec();
            if val.len() != params.area.height * params.area.width {
                return Err(format!(
                    "Length of image: {}, Requested size: {} x {}",
                    val.len(),
                    width,
                    height
                ));
            }
            return Ok(val);
        }
        Err("Could not get 16-bit image".to_string())
    }

    fn set_temperature(&mut self, request: TemperatureRequest) -> Result<(), String> {
        self.device
            .set_temperature(request.temperature)
            .map_err(|x| x.to_string())?;
        Ok(())
    }
}

fn read_basic_props(device: &CameraUnitASI) -> BasicProperties {
    let roi = device.get_roi();
    BasicProperties {
        width: device.get_ccd_width() as usize,
        height: device.get_ccd_height() as usize,
        temperature: device.get_temperature().unwrap_or(-273.0),
        exposure: device.get_exposure().as_secs_f32(),
        roi: ExposureArea {
            x: roi.x_min as usize,
            y: roi.y_min as usize,
            width: (roi.x_max - roi.x_min) as usize,
            height: (roi.y_max - roi.y_min) as usize,
        },
    }
}

fn read_all_props(device: &CameraUnitASI) -> Vec<DeviceProperty> {
    vec![
        prop_f32(
            "Chip Temperature",
            device.get_temperature().unwrap_or(-273.0) as f32,
            1,
        ),
        prop("ADC Gain", device.get_gain_raw()),
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
        prop("Max ADC Gain", device.get_max_gain().unwrap_or(0)),
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
