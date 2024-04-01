use std::{path::{PathBuf, Path}, fs};

use ccdi_common::to_string;
use chrono::{Utc, TimeZone, Duration};
use fitsio::FitsFile;
use itertools::Itertools;

// ============================================ PUBLIC =============================================

fn main() -> Result<(), String> {
    let src = "/home/vchvatal/astro/2023-08-17-mae/C2020-V2-ZTF-g4k-60s-m15c/workspace/lights-time";

    let start_time = match Utc.with_ymd_and_hms(2023, 8, 18, 1, 45, 0) {
        chrono::LocalResult::None => return Err("Invalid date time".to_string()),
        chrono::LocalResult::Single(result) => result,
        chrono::LocalResult::Ambiguous(_, _) => return Err("Ambigous date time".to_string()),
    };

    let duration = 65i64;

    let inputs = find_files(src)?;

    for (index, file) in inputs.iter().enumerate() {
        let ts = format!("{}", (start_time + Duration::seconds(duration*index as i64)).format("%+"));
        println!("File: {:?}", file);
        println!("Desired TS: {:?}", ts);
        add_timestamp(file, &ts)?;
    }

    Ok(())
}

fn add_timestamp(file: &Path, ts: &str) -> Result<(), String> {
    let mut fits = FitsFile::edit(file).map_err(to_string)?;

    fits.hdu(0)
        .map_err(to_string)?
        .write_key(&mut fits, "DATE-OBS", ts)
        .map_err(to_string)
}

fn find_files(src: &str) -> Result<Vec<PathBuf>, String> {
    let paths = fs::read_dir(src).map_err(to_string)?;

    paths
        .map(|path| path.map(|path| path.path()).map_err(to_string))
        .collect::<Result<Vec<PathBuf>, String>>()
        .map(|files| files.into_iter().sorted_by(|a, b| a.cmp(b)).collect::<Vec<PathBuf>>())
}