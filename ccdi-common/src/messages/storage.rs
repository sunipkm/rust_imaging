use std::{sync::Arc, time::Duration};

use serde_derive::{Serialize, Deserialize};

use crate::{RawImage, StorageState};

// ============================================ PUBLIC =============================================

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum StorageMessage {
    EnableStore,
    DisableStore,
    UpdateCadence(Duration),
    ProcessImage(Arc<RawImage>),
    SetDirectory(String),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StorageDetail {
    pub storage_name: String,
    pub cadence: Duration,
    pub counter: usize,
    pub storage_log: Vec<StorageLogRecord>,
    pub storage_enabled: bool,
    pub state: StorageState,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StorageLogRecord {
    pub name: String,
    pub status: StorageLogStatus,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum StorageLogStatus {
    Success,
    Error(String),
}

impl Default for StorageDetail {
    fn default() -> Self {
        Self {
            storage_name: String::from("?"),
            cadence: Duration::from_secs(60),
            counter: 0,
            storage_log: Vec::new(),
            storage_enabled: false,
            state: StorageState::Unknown,
        }
    }
}