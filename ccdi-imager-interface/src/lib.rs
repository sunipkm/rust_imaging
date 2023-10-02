use serde_derive::{Serialize, Deserialize};

// ============================================ PUBLIC =============================================

pub trait ImagerDriver {
    fn list_devices(&mut self) -> Result<Vec<DeviceDescriptor>, String>;
    fn connect_device(&mut self, descriptor: &DeviceDescriptor) -> Result<Box<dyn ImagerDevice>, String>;
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DeviceDescriptor {
    pub id: i32,
    pub name: String,
}

pub trait ImagerDevice {
    fn read_properties(&mut self) -> Result<ImagerProperties, String>;
    fn close(&mut self);
    fn start_exposure(&mut self, params: &ExposureParams) -> Result<(), String>;
    fn image_ready(&mut self, ) -> Result<bool, String>;
    fn download_image(&mut self, params: &mut ExposureParams) -> Result<Vec<u16>, String>;
    fn set_temperature(&mut self, request: TemperatureRequest) -> Result<(), String>;
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
    pub percentile_pix: f32,
    pub pixel_tgt: f32,
    pub pixel_tol: f32,
    pub save: bool,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ExposureArea {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize
}

impl ExposureArea {
    pub fn pixel_count(&self) -> usize {
        self.width*self.height
    }

    pub fn into_tuple(&self) -> (usize, usize, usize, usize) {
        (self.x, self.y, self.width, self.height)
    }
}