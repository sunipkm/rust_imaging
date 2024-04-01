use std::path::{PathBuf, Path};

use ccdi_common::{IoMessage, StateMessage, read_text_file};
use log::{debug, warn, info};

use crate::IoConfig;

use self::led_output::{write_output, ProgrammableOutput, pattern_pwm, status_healthy};

mod led_output;

// ============================================ PUBLIC =============================================

pub struct IoManager {
    #[allow(unused)]
    last_trigger_value: Option<bool>,
    #[allow(unused)]
    trigger_input_path: PathBuf,
    exposure_status_path: PathBuf,
    heating_pwm: ProgrammableOutput,
    main_status: ProgrammableOutput,
}

impl IoManager {
    pub fn new(config: &IoConfig) -> Self {
        let mut main_status = ProgrammableOutput::new(&config.main_status);
        main_status.set_pattern(status_healthy());

        Self {
            last_trigger_value: None,
            trigger_input_path: PathBuf::from(config.trigger_input.clone()),
            exposure_status_path: PathBuf::from(config.exposure_status.clone()),
            heating_pwm: ProgrammableOutput::new(&config.heating_pwm),
            main_status,
        }
    }

    pub fn process(&mut self, message: IoMessage) -> Result<Vec<StateMessage>, String> {
        match message {
            IoMessage::SetHeating(value) => {
                info!("Heating set to {}", value);
                self.heating_pwm.set_pattern(pattern_pwm(value))
            },
            IoMessage::SetExposureActive(value) => {
                let _ = write_output(&self.exposure_status_path, value);
            },
            IoMessage::SetStatus(_) => {
                self.main_status.set_pattern(status_healthy())
            },
        }

        Ok(vec![])
    }

    pub fn periodic_tasks(&mut self) -> Result<Vec<StateMessage>, String> {
        // let _ = log_err("Set PWM", self.heating_pwm.iterate());
        // let _ = log_err("Set Status", self.main_status.iterate());

        // let prev_input = self.last_trigger_value;
        // let actual_input = read_input(&self.trigger_input_path);

        // if actual_input.is_some() {
        //     self.last_trigger_value = actual_input;
        // }

        // let output = match (prev_input, self.last_trigger_value) {
        //     (Some(prev), Some(actual)) if prev != actual => vec![
        //         StateMessage::TriggerValueChanged(actual)
        //     ],
        //     (None, Some(actual)) => vec![
        //         StateMessage::TriggerValueChanged(actual)
        //     ],
        //     _ => vec![],
        // };

        // Ok(output)
        Ok(Vec::<StateMessage>::new())
    }
}

// =========================================== PRIVATE =============================================

#[allow(unused)]
fn read_input(path: &Path) -> Option<bool> {
    let first_char = read_text_file(path)
        .map(|string| string.chars().next().unwrap_or(' '));

    match first_char {
        Err(error) => {
            warn!("Cannot read status from: {:?} {:?}", error, path);
            None
        },
        Ok('0') => Some(true),
        Ok('1') => Some(false),
        Ok(other) => {
            debug!("Invalid status value: {}", other);
            None
        }
    }
}

