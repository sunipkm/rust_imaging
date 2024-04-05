use std::sync::Arc;

use serde_derive::{Deserialize, Serialize};

use crate::{OptExposureConfig, StorageDetail, StorageMessage, StorageState};

// ============================================ PUBLIC =============================================

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum StateMessage {
    ClientInformation((String, String)), // String for testing.
    ExposureMessage(ExposureCommand),
    ImageParam(ImageParamMessage),
    CameraParam(CameraParamMessage),
    ClientConnected,
    ImageDisplayed(Arc<Vec<u8>>),
    UpdateStorageState(StorageState),
    TriggerValueChanged(bool),
    StorageMessage(StorageMessage),
    UpdateStorageDetail(StorageDetail),
    PowerOff,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ExposureCommand {
    Start,
    Update(OptExposureConfig),
    Cancel,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ImageParamMessage {
    // SetGain(u16),
    // SetTime(f64),
    // SetTemp(f64),
    // SetHeatingPwm(f64),
    // SetTriggerRequired(bool),
    // SetAutoExp(bool),
    SetPercentilePix(f32),
    SetPixelTgt(f32),
    SetPixelTol(f32),
    SetRoi((u16, u16, u16, u16)),
    SetFlipX(bool),
    SetFlipY(bool),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum CameraParamMessage {
    // EnableLoop(bool),
    EnableLoop(bool),
    SetGain(u16),
    SetTime(f64),
    SetTemp(f64),
    SetHeatingPwm(f64),
    // SetRenderingType(RenderingType),
    SetTriggerRequired(bool),
    SetAutoExp(bool),
    // SetPercentilePix(f32),
    // SetPixelTgt(f32),
    // SetPixelTol(f32),
    // SetRoi((u16, u16, u16, u16)),
    // SetFlipX(bool),
    // SetFlipY(bool),
}
