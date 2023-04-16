use std::{thread, time::Duration};

use ccdi_driver_moravian::{get_any_camera_id, connect_usb_camera, CameraError, CameraDriver};


fn main() -> Result<(), String> {
    let camera_id = get_any_camera_id().ok_or("No camera connected")?;

    if let Ok(camera) = connect_usb_camera(camera_id) {
        print_camera_info(&camera).map_err(|err| format!("{:?}", err))?;
    }

    dbg!(camera_id);
    Ok(())
}

fn print_camera_info(camera: &CameraDriver) -> Result<(), CameraError> {
    println!("Chip temperature: {}", camera.read_chip_temperature()?);
    println!("Supply voltage: {}", camera.read_supply_voltage()?);
    println!("Resolution: {} x {}", camera.read_chip_width()?, camera.read_chip_height()?);

    for (index, mode) in camera.enumerate_read_modes()?.iter().enumerate() {
        println!("Read mode {}: {}", index, mode)
    }

    let width = camera.read_chip_width()?;
    let height = camera.read_chip_height()?;

    camera.start_exposure(1.0, true, 0, 0, width, height)?;

    while !(camera.image_ready()?) {
        println!("Image not ready, waiting ...");
        thread::sleep(Duration::from_millis(100));
    }

    println!("Starting image download");
    let image_data = camera.read_image((width*height) as usize)?;
    println!("Image downloaded, pixels: {}", image_data.len());
    println!("Data: {:?}", image_data);
    Ok(())
}