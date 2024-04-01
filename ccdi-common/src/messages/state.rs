use std::sync::Arc;

use serde_derive::{Serialize, Deserialize};

use crate::{RgbImage, StorageState, StorageMessage, StorageDetail};

// ============================================ PUBLIC =============================================

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum StateMessage {
    ExposureMessage(ExposureCommand),
    CameraParam(CameraParamMessage),
    ClientConnected,
    ImageDisplayed(Arc<RgbImage<u16>>),
    UpdateStorageState(StorageState),
    TriggerValueChanged(bool),
    StorageMessage(StorageMessage),
    UpdateStorageDetail(StorageDetail),
    PowerOff,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ExposureCommand {
    Start,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum CameraParamMessage {
    EnableLoop(bool),
    SetGain(u16),
    SetTime(f64),
    SetTemp(f64),
    SetHeatingPwm(f64),
    SetRenderingType(RenderingType),
    SetTriggerRequired(bool),
    SetAutoExp(bool),
    SetPercentilePix(f32),
    SetPixelTgt(f32),
    SetPixelTol(f32),
    SetRoi((u16, u16, u16, u16)),
    SetFlipX(bool),
    SetFlipY(bool),
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum RenderingType {
    FullImage,
    Center1x,
    Corners1x,
}