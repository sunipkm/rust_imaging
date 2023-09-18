use std::{fmt::Debug, time::Duration};

use ccdi_imager_interface::{
    BasicProperties, DeviceDescriptor, DeviceProperty, ExposureParams, ImagerDevice, ImagerDriver,
    ImagerProperties, TemperatureRequest,
};

use cameraunit::{CameraInfo, CameraUnit, ROI};
use cameraunit_asi::{get_camera_ids, open_camera, CameraUnit_ASI};

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
    ) -> Result<Box<dyn ImagerDevice>, String> {
        let (cam, _) = open_camera(descriptor.id).map_err(|x| x.to_string())?;
        Ok(Box::new(ASICameraImager { device: cam }))
    }
}

pub struct ASICameraImager {
    device: CameraUnit_ASI,
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
        self.device
            .set_gain_raw(params.gain as i64)
            .map_err(|x| x.to_string())?;
        self.device
            .set_exposure(Duration::from_secs_f64(params.time))
            .map_err(|x| x.to_string())?;
        let roi = ROI {
            x_min: params.area.x as i32,
            x_max: (params.area.width + params.area.x) as i32,
            y_min: params.area.y as i32,
            y_max: (params.area.height + params.area.y) as i32,
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

    fn download_image(&mut self, params: &ExposureParams) -> Result<Vec<u16>, String> {
        let img = self.device.download_image().map_err(|x| x.to_string())?;
        if let Some(img) = img.get_image().as_luma16() {
            let val = img.clone().into_vec();
            if val.len() != params.area.height * params.area.width {
                return Err(format!(
                    "Length of image: {}, Requested size: {} x {}",
                    val.len(),
                    params.area.width,
                    params.area.height
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

fn read_basic_props(device: &CameraUnit_ASI) -> BasicProperties {
    BasicProperties {
        width: device.get_ccd_width() as usize,
        height: device.get_ccd_height() as usize,
        temperature: device.get_temperature().unwrap_or(-273.0),
    }
}

fn read_all_props(device: &CameraUnit_ASI) -> Vec<DeviceProperty> {
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
