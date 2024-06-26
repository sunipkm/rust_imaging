use serde_derive::{Deserialize, Serialize};
use serialimage::DynamicSerialImage;

// ============================================ PUBLIC =============================================

pub trait ImagerDriver {
    fn list_devices(&mut self) -> Result<Vec<DeviceDescriptor>, String>;
    fn connect_device(
        &mut self,
        descriptor: &DeviceDescriptor,
        roi_request: &ExposureArea,
    ) -> Result<(Box<dyn ImagerDevice>, ExposureArea), String>;
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DeviceDescriptor {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct OptConfigCmd {
    pub percentile_pix: f32,
    pub pixel_tgt: f32,
    pub pixel_tol: f32,
    pub max_exp: f32,
}

pub trait ImagerDevice {
    fn read_properties(&mut self) -> Result<ImagerProperties, String>;
    fn close(&mut self);
    fn start_exposure(&mut self, params: &ExposureParams) -> Result<(), String>;
    fn image_ready(&mut self) -> Result<bool, String>;
    fn download_image(&mut self, params: &mut ExposureParams)
        -> Result<DynamicSerialImage, String>;
    fn set_temperature(&mut self, request: TemperatureRequest) -> Result<(), String>;
    fn update_opt_config(&mut self, config: OptConfigCmd);
    fn cancel_capture(&mut self) -> Result<(), String>;
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TemperatureRequest {
    /// Desired temperature in degrees celsius
    pub temperature: f32,
    /// Desired change speed in degrees celsius per minute
    pub speed: f32,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ImagerProperties {
    pub basic: BasicProperties,
    pub other: Vec<DeviceProperty>,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct BasicProperties {
    pub width: usize,
    pub height: usize,
    pub temperature: f32,
    pub exposure: f32,
    pub roi: ExposureArea,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DeviceProperty {
    pub name: String,
    pub value: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ExposureParams {
    pub gain: u16,
    pub time: f64,
    pub area: ExposureArea,
    pub autoexp: bool,
    pub flipx: bool,
    pub flipy: bool,
    pub percentile_pix: f32,
    pub pixel_tgt: f32,
    pub pixel_tol: f32,
    pub save: bool,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct ExposureArea {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl ExposureArea {
    pub fn pixel_count(&self) -> usize {
        self.width * self.height
    }

    pub fn into_tuple(&self) -> (usize, usize, usize, usize) {
        (self.x, self.y, self.width, self.height)
    }
}
