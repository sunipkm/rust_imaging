use std::fmt::Debug;

use ccdi_common::to_string;
use ccdi_driver_moravian::{get_any_camera_id, CameraDriver, connect_usb_camera, CameraError};
use ccdi_imager_interface::{
    ImagerDriver, ImagerDevice, ImagerProperties, DeviceDescriptor, DeviceProperty
};

// ============================================ PUBLIC =============================================

pub struct MoravianImagerDriver {

}

impl MoravianImagerDriver {
    pub fn new() -> Self {
        Self { }
    }
}

impl ImagerDriver for MoravianImagerDriver {
    fn list_devices(&mut self) -> Result<Vec<DeviceDescriptor>, String> {
        Ok(match get_any_camera_id() {
            Some(id) => vec![
                DeviceDescriptor { id, name: String::from("Camera #0") }
            ],
            None => vec![],
        })
    }

    fn connect_device(
        &mut self,
        descriptor: &DeviceDescriptor
    ) ->  Result<Box<dyn ImagerDevice>, String> {
        Ok(Box::new(
            MoravianImagerDevice {
                device: connect_usb_camera(descriptor.id).map_err(to_string)?
            }
        ))
    }
}

pub struct MoravianImagerDevice {
    device: CameraDriver
}

impl ImagerDevice for MoravianImagerDevice {
    fn read_properties(&mut self) -> Result<ImagerProperties, String> {
        Ok(ImagerProperties {
            other: read_all_properties(&self.device).map_err(to_string)?
        })
    }
}

fn read_all_properties(device: &CameraDriver) -> Result<Vec<DeviceProperty>, CameraError> {
    Ok(vec![
        prop("Chip Temperature", device.read_chip_temperature()?),
        prop("Hot Temperature", device.read_hot_temperature()?),
        prop("Camera Temperature", device.read_camera_temperature()?),
        prop("Env Temperature", device.read_environment_temperature()?),
        prop("Supply Voltage", device.read_supply_voltage()?),
        prop("Power Utilization", device.read_power_utilization()?),
        prop("ADC Gain", device.read_adc_gain()?),
        prop("Camera ID", device.read_camera_id()?),
        prop("Camera Chip Width", device.read_chip_width()?),
        prop("Camera Chip Height", device.read_chip_height()?),
        prop("Min Exposure Time", device.read_min_exposure()?),
        prop("Max Exposure Time", device.read_max_exposure()?),
        prop("Max Gain", device.read_max_gain()?),
    ])
}

fn prop<T: Debug>(name: &str, value: T) -> DeviceProperty {
    DeviceProperty {
        name: name.to_owned(),
        value: format!("{:?}", value)
    }
}