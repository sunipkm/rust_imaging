use std::fs::File;

use simplelog::{CombinedLogger, Config, LevelFilter, SharedLogger, SimpleLogger, WriteLogger};

// ============================================ PUBLIC =============================================

pub fn init_logger(debug: bool, log_file: Option<&String>) {
    let log_level = match debug {
        true => LevelFilter::Debug,
        false => LevelFilter::Info,
    };

    let mut loggers: Vec<Box<dyn SharedLogger>> =
        vec![SimpleLogger::new(log_level, Config::default())];

    if let Some(file) = log_file {
        if let Ok(created_file) = File::create(file) {
            loggers.push(WriteLogger::new(log_level, Config::default(), created_file));
        }
    }

    if let Err(error) = CombinedLogger::init(loggers) {
        eprintln!("Could not initialize logger: {:?}", error)
    }
}
