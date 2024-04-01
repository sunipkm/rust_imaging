use std::{path::Path, fs::{File, OpenOptions}, io::{BufReader, Read, BufWriter, Write}};

use crate::to_string;

// ============================================ PUBLIC =============================================

pub fn append_to_file(
    data: &str, path: &Path
) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .map_err(to_string)?;

    file.write_all(data.as_bytes()).map_err(to_string)?;
    Ok(())
}

pub fn save_text_file(
    data: &str, path: &Path
) -> Result<(), String> {
    let prefix = path.parent().ok_or("Invalid path parent".to_string())?;
    std::fs::create_dir_all(prefix).map_err(to_string)?;
    let file = File::create(path).map_err(to_string)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(data.as_bytes()).map_err(to_string)?;
    Ok(())
}

pub fn read_text_file(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(to_string)?;
    let mut reader = BufReader::new(file);
    let mut data = String::new();
    reader.read_to_string(&mut data).map_err(to_string)?;
    Ok(data)
}