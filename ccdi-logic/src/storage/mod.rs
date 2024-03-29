use std::{sync::Arc, process::Command, path::PathBuf, collections::VecDeque};

use ccdi_common::{
    StorageMessage, StateMessage, StorageState, StorageCapacity, to_string, StorageLogRecord,
    RawImage, StorageLogStatus, StorageDetail
};
use log::debug;

use crate::ServiceConfig;

use self::save::save_fits_file;

mod save;

// ============================================ PUBLIC =============================================

pub struct Storage {
    config: Arc<ServiceConfig>,
    last_storage_state: StorageState,
    counter: usize,
    storage_name: String,
    storage_active: bool,
    details: VecDeque<StorageLogRecord>,
}

impl Storage {
    pub fn new(config: Arc<ServiceConfig>) -> Self {
        Self {
            config,
            last_storage_state: StorageState::Unknown,
            counter: 0,
            storage_name: String::from("default"),
            storage_active: false,
            details: VecDeque::new(),
        }
    }

    pub fn process(&mut self, message: StorageMessage) -> Result<Vec<StateMessage>, String> {
        match message {
            StorageMessage::SetDirectory(name) => {
                self.storage_name = name;
                self.counter = 0;
            },
            StorageMessage::DisableStore => {
                debug!("Storage disabled");
                self.storage_active = false;
            },
            StorageMessage::EnableStore => {
                debug!("Storage enabled");
                self.storage_active = true;
            },
            StorageMessage::ProcessImage(image) => {
                if self.storage_active {
                    self.handle_image(image);
                }
            }
        }

        Ok(vec![StateMessage::UpdateStorageDetail(self.get_details())])
    }

    pub fn periodic_tasks(&mut self) -> Result<Vec<StateMessage>, String> {
        let storage_state = check_storage(&self.config.storage);

        if storage_state == self.last_storage_state {
            Ok(vec![])
        } else {
            self.last_storage_state = storage_state.clone();
            Ok(vec![StateMessage::UpdateStorageState(storage_state)])
        }
    }
}

// =========================================== PRIVATE =============================================

impl Storage {
    fn get_details(&self) -> StorageDetail {
        StorageDetail {
            storage_name: self.storage_name.clone(),
            counter: self.counter,
            storage_log: self.details.iter().cloned().collect(),
            storage_enabled: self.storage_active,
            state: self.last_storage_state.clone(),
        }
    }

    fn current_dir(&self) -> Option<String> {
        PathBuf::from(&self.config.storage)
            .join(PathBuf::from(self.storage_name.clone()))
            .to_str()
            .map(|path| path.to_owned())
    }

    fn current_file_name(&self) -> Option<String> {
        self.current_dir().map(|dir| format!("{}/{:05}.fits", dir, self.counter))
    }

    fn handle_image(&mut self, image: Arc<RawImage>) {
        let result = match self.current_file_name() {
            None => file_name_err(),
            Some(file_name) => match save_fits_file(&image, &file_name) {
                Ok(_) => ok_record(file_name),
                Err(error) => StorageLogRecord {
                    name: file_name,
                    status: StorageLogStatus::Error(error)
                }
            }
        };

        self.counter += 1;
        self.details.push_back(result);

        while self.details.len() > 20 {
            self.details.pop_front();
        }
    }
}

fn ok_record(name: String) -> StorageLogRecord {
    StorageLogRecord { name, status: StorageLogStatus::Success }
}

fn file_name_err() -> StorageLogRecord {
    StorageLogRecord {
        name: String::from("Could not assemble file name"),
        status: StorageLogStatus::Error(String::new())
    }
}

fn check_storage(path: &str) -> StorageState {
    match Command::new("df").args([path]).output() {
        Ok(output) => match output.status.code() {
            Some(0) => match String::from_utf8(output.stdout) {
                Ok(stdout) => match parse_free_space(&stdout) {
                    Ok(details) => StorageState::Available(details),
                    Err(error) => StorageState::Error(error),
                },
                Err(error) => StorageState::Error(
                    format!("Could not parse stdout as utf8: {:?}", error)
                ),
            },
            Some(code) => StorageState::Error(
                format!(
                    "Storage check returned error code: {:?} {:?}",
                    code, String::from_utf8_lossy(&output.stderr)
                )
            ),
            status => StorageState::Error(
                format!("Storage check did not return successfully: {:?}", status)
            )
        },
        Err(error) => StorageState::Error(
            format!("Storage check call failed: {:?}", error)
        ),
    }
}

fn parse_free_space(stdout: &str) -> Result<StorageCapacity, String> {
    let line = stdout.lines().nth(1).ok_or("df output second line missing")?;
    let total_gigabytes = kb_to_gb(parse_nth_token(line, 1)?);
    let free_gigabytes = kb_to_gb(parse_nth_token(line, 3)?);
    Ok(StorageCapacity{total_gigabytes, free_gigabytes})
}

fn parse_nth_token(line: &str, index: usize) -> Result<f64, String> {
    let token = line.split_whitespace()
        .nth(index)
        .ok_or(format!("{}th token not present in '{}'", index, line))?;

    token.parse::<f64>().map_err(to_string)
}

fn kb_to_gb(kilobytes: f64) -> f64 {
    kilobytes/1024.0/1024.0
}

// ============================================= TEST ==============================================

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    const TEST_DF_OUTPUT: &str = indoc!{"
        Filesystem           1K-blocks  Used      Available  Use% Mounted on
        /dev/mapper/luks-a6e 1967861712 111750632 1756075448   6% /media/x/759
    "};

    #[test]
    fn parse_df_output() {
        let details = parse_free_space(TEST_DF_OUTPUT).expect("Parse details failed");
        assert_eq!(details.total_gigabytes, 1967861712.0/1024.0/1024.0);
        assert_eq!(details.free_gigabytes, 1756075448.0/1024.0/1024.0);
    }
}