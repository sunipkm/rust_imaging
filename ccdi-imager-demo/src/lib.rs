use std::{cmp::min, fmt::Debug, time::SystemTime};

use cameraunit::DynamicSerialImage;
pub use cameraunit::{ImageData, ImageMetaData, SerialImageBuffer, SerialImagePixel, SerialImageStorageTypes};
use ccdi_imager_interface::{
    BasicProperties, DeviceDescriptor, DeviceProperty, ExposureArea, ExposureParams, ImagerDevice,
    ImagerDriver, ImagerProperties, TemperatureRequest,
};
use image::{DynamicImage, ImageBuffer};

// ============================================ PUBLIC =============================================

pub struct DemoImagerDriver {}

impl DemoImagerDriver {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImagerDriver for DemoImagerDriver {
    fn list_devices(&mut self) -> Result<Vec<DeviceDescriptor>, String> {
        Ok(vec![DeviceDescriptor {
            id: 0,
            name: String::from("Demo Camera #0"),
        }])
    }

    fn connect_device(
        &mut self,
        _descriptor: &DeviceDescriptor,
        _: &ExposureArea,
    ) -> Result<Box<dyn ImagerDevice>, String> {
        Ok(Box::new(DemoImagerDevice {
            offset: 0.0,
            temperature: 30.0,
        }))
    }
}

pub struct DemoImagerDevice {
    offset: f32,
    temperature: f32,
}

impl ImagerDevice for DemoImagerDevice {
    fn read_properties(&mut self) -> Result<ImagerProperties, String> {
        self.offset += 0.001;
        Ok(ImagerProperties {
            basic: BasicProperties {
                width: 6000,
                height: 4000,
                temperature: self.temperature,
                exposure: 0.1,
                roi: ExposureArea {
                    x: 0,
                    y: 0,
                    width: 6000,
                    height: 4000,
                },
            },
            other: list_demo_properties(&self),
        })
    }

    fn close(&mut self) {}

    fn start_exposure(&mut self, _params: &ExposureParams) -> Result<(), String> {
        Ok(())
    }

    fn image_ready(&mut self) -> Result<bool, String> {
        Ok(true)
    }

    fn download_image(&mut self, params: &mut ExposureParams) -> Result<DynamicSerialImage, String> {
        let data = generate_test_image(params.area.width, params.area.height);
        let mut img = DynamicImage::from(ImageBuffer::<image::Luma<u16>, Vec<u16>>::new(
            params.area.width as u32,
            params.area.height as u32,
        ));
        let mut meta: ImageMetaData = Default::default();
        meta.timestamp = SystemTime::now();
        meta.camera_name = "Demo Camera".to_owned();

        let bimg = img.as_mut_luma16().unwrap();
        bimg.copy_from_slice(&data);

        let mut img: DynamicSerialImage = img.into();
        img.set_metadata(meta);

        Ok(img)
    }

    fn set_temperature(&mut self, request: TemperatureRequest) -> Result<(), String> {
        dbg!("Setting temperature: ", request.temperature, request.speed);
        self.temperature = request.temperature;
        Ok(())
    }
}

fn list_demo_properties(device: &DemoImagerDevice) -> Vec<DeviceProperty> {
    vec![
        prop("Chip Temperature", 1.000 + device.offset),
        prop("Hot Temperature", 2.000 + device.offset),
        prop("Camera Temperature", 3.000 + device.offset),
        prop("Env Temperature", 4.000 + device.offset),
        prop("Supply Voltage", 5.000 + device.offset),
        prop("Power Utilization", 6.000 + device.offset),
        prop("ADC Gain", 7.000 + device.offset),
        prop("Camera ID", 8.000 + device.offset),
        prop("Camera Chip Width", 9.000 + device.offset),
        prop("Camera Chip Height", 10.000 + device.offset),
        prop("Min Exposure Time", 11.000 + device.offset),
        prop("Max Exposure Time", 12.000 + device.offset),
        prop("Max Gain", 13.000 + device.offset),
    ]
}

fn prop<T: Debug>(name: &str, value: T) -> DeviceProperty {
    DeviceProperty {
        name: name.to_owned(),
        value: format!("{:?}", value),
    }
}

fn generate_test_image(width: usize, height: usize) -> Vec<u16> {
    let mut buffer = vec![0u16; width * height];
    let len = buffer.len();

    let dx = (XMAX - XMIN) / width as f64;
    let dy = (YMAX - YMIN) / height as f64;

    for y in 0..(height / 2) {
        let lines = &mut buffer[y * width * 2..min(len, (y + 2) * width * 2)];

        for x in 0..(width / 2) {
            let (r, g, b) = generate_pixel(x, y, dx, dy);
            lines[x * 2] = g;
            lines[x * 2 + 1] = r;
            lines[width + x * 2] = b;
            lines[width + x * 2 + 1] = g;
        }
    }

    buffer
}

const XMIN: f64 = 0.27085;
const XMAX: f64 = 0.27100;
const YMIN: f64 = 0.004570;
const YMAX: f64 = 0.004755;
const MAX_ITER: usize = 500;

fn generate_pixel(x: usize, y: usize, dx: f64, dy: f64) -> (u16, u16, u16) {
    let mut u: f64 = 0.0;
    let mut v: f64 = 0.0;

    let mut u2: f64 = u * u;
    let mut v2: f64 = v * v;

    let ry = YMAX - y as f64 * dy;
    let rx = XMIN + x as f64 * dx;

    let mut k = 1;

    while k < MAX_ITER && (u2 + v2 < 4.0) {
        v = 2.0 * u * v + ry;
        u = u2 - v2 + rx;
        u2 = u * u;
        v2 = v * v;
        k += 1;
    }

    let out = ((k as f64 - 50.0) * 100.0).clamp(0.0, 65535.0) as u16;
    (600 * (out % 57), out * 2, 2000 * (out % 17))
}
