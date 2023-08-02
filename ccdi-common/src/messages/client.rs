use std::sync::Arc;

use ccdi_imager_interface::{ImagerProperties, ExposureParams};
use nanocv::ImgSize;
use serde_derive::{Serialize, Deserialize};

use crate::{RgbImage, RenderingType, StorageState, StorageDetail};

use super::gui_config::GuiConfig;

// ============================================ PUBLIC =============================================

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Reconnect,
    View(ViewState),
    RgbImage(Arc<RgbImage<u16>>),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RawImage {
    pub params: ExposureParams,
    pub data: Vec<u16>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ViewState {
    pub detail: String,
    pub status: LogicStatus,
    pub camera_properties: Option<Arc<ImagerProperties>>,
    pub camera_params: CameraParams,
    pub storage_detail: StorageDetail,
    pub config: GuiConfig,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            detail: String::new(),
            status: Default::default(),
            camera_properties: None,
            camera_params: Default::default(),
            storage_detail: Default::default(),
            config: GuiConfig::default(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CameraParams {
    pub loop_enabled: bool,
    pub gain: u16,
    pub time: f64,
    pub rendering: RenderingType,
    pub render_size: ImgSize,
    pub temperature: f64,
    pub trigger_required: bool,
    pub heating_pwm: f64,
}

impl CameraParams {
    pub fn new(render_size: ImgSize) -> Self {
        Self {
            loop_enabled: false,
            gain: 0,
            time: 1.0,
            rendering: RenderingType::FullImage,
            render_size,
            temperature: 25.0,
            trigger_required: false,
            heating_pwm: 0.0,
        }
    }
}

impl Default for CameraParams {
    fn default() -> Self {
        CameraParams::new(ImgSize::new(900, 600))
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LogicStatus {
    pub camera: ConnectionState,
    pub exposure: ConnectionState,
    pub storage: StorageState,
    pub trigger: ConnectionState,
    pub required: ConnectionState,
    pub loop_enabled: ConnectionState,
    pub save: ConnectionState,
}

impl Default for LogicStatus {
    fn default() -> Self {
        Self {
            camera: ConnectionState::Disconnected,
            exposure: ConnectionState::Disconnected,
            trigger: ConnectionState::Disconnected,
            required: ConnectionState::Disconnected,
            storage: StorageState::Unknown,
            save: ConnectionState::Disconnected,
            loop_enabled: ConnectionState::Disconnected,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Established
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}