use std::path::PathBuf;

use ccdi_common::{to_string, RawImage};

use simple_expand_tilde::*;

// ============================================ PUBLIC =============================================

pub fn save_fits_file(image: &RawImage, file_name: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(file_name);
    let prefix = path.parent().ok_or("Invalid path parent".to_string())?;
    println!("Prefix: {:?}", prefix);
    let prefix = if prefix.starts_with("~") {
        expand_tilde(prefix).ok_or("Could not un-tilde".to_string())?
    } else {
        prefix.to_owned()
    };
    println!("Prefix with tilde expansion: {:?}", prefix);
    // let prefix = prefix
    //     .canonicalize()
    //     .map_err(|err| format!("Invalid path {:?} could not canonicalize", err))?;
    // println!("After canonicalize: Prefix: {:?}", prefix);
    std::fs::create_dir_all(&prefix).map_err(to_string)?;
    println!(
        "Saving to: {:?}", prefix
    );
    let img = image.data.clone();
    let path = img.savefits(&prefix, "ccdi", Some("CCDI ASI"), true, true)
        .map_err(to_string)?;

    Ok(path)
}
