use std::{env::args, path::{PathBuf, Path}, thread, time::Duration};

use ccdi_common::{log_err, append_to_file};

// ============================================ PUBLIC =============================================

fn main() {
    let path = match args().nth(1) {
        Some(path) => path,
        None => {
            return;
            //  println!("Go argument passed, exiting.");
       }
    };

    println!("Testing GPIO on: {}", path);
    let path = PathBuf::from(path);

    loop {
        println!("Writing True");
        log_err("write_output", write_output(&path, true));
        thread::sleep(Duration::from_secs(1));

        println!("Writing False");
        log_err("write_output", write_output(&path, false));
        thread::sleep(Duration::from_secs(1));
    }
}

pub fn write_output(path: &Path, value: bool) -> Result<(), String> {
    append_to_file(
        match value {
            false => "0\n",
            true => "1\n",
        },
        path
    )
}