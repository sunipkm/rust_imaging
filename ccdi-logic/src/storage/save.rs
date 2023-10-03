use std::path::PathBuf;

use ccdi_common::{to_string, RawImage};
use cameraunit::ImageData;

// ============================================ PUBLIC =============================================

pub fn save_fits_file(image: &RawImage, file_name: &str) -> Result<(), String> {
    let path = PathBuf::from(file_name.clone());
    let prefix = path.parent().ok_or(format!("Invalid path parent"))?;
    std::fs::create_dir_all(prefix).map_err(to_string)?;

    let img = ImageData::from(image.data.clone());
    img.save_fits(prefix, "ccdi", "CCDI ASI", true, true)
        .map_err(to_string)?;

    Ok(())
}
