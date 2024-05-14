use std::sync::Arc;

use ccdi_imager_interface::{ExposureArea, ExposureParams, ImagerProperties};
use crate::ImgSize;
use serde_derive::{Deserialize, Serialize};
use serialimage::DynamicSerialImage;

use crate::{OptExposureConfig, StorageDetail, StorageState};

use super::gui_config::GuiConfig;

// ============================================ PUBLIC =============================================

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Reconnect,
    View(Box<ViewState>),
    PngImage(Arc<Vec<u8>>),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RawImage {
    pub params: ExposureParams,
    pub data: DynamicSerialImage,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct ViewState {
    pub detail: String,
    pub status: LogicStatus,
    pub camera_properties: Option<Arc<ImagerProperties>>,
    pub image_params: ImageParams,
    pub camera_params: CameraParams,
    pub storage_detail: StorageDetail,
    pub config: GuiConfig,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ImageParams {
    // pub loop_enabled: bool,
    pub render_size: ImgSize,
    pub percentile_pix: f32,
    pub pixel_tgt: f32,
    pub pixel_tol: f32,
    pub max_exp: f32,
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
    pub flipx: bool,
    pub flipy: bool,
}

impl ImageParams {
    pub fn new(render_size: ImgSize, roi: ExposureArea, opt: OptExposureConfig) -> Self {
        Self {
            render_size,
            percentile_pix: opt.percentile_pix,
            pixel_tgt: opt.pixel_tgt,
            pixel_tol: opt.pixel_tol,
            max_exp: opt.max_exposure.as_secs_f32(),
            x: roi.x as u16,
            y: roi.y as u16,
            w: roi.width as u16,
            h: roi.height as u16,
            flipx: false,
            flipy: false,
        }
    }
}

impl Default for ImageParams {
    fn default() -> Self {
        ImageParams::new(
            ImgSize::new(900, 600),
            ExposureArea {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            OptExposureConfig::default(),
        )
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CameraParams {
    pub loop_enabled: bool,
    pub gain: u16,
    pub time: f64,
    pub temperature: f64,
    pub trigger_required: bool,
    pub heating_pwm: f64,
    pub autoexp: bool,
}

impl CameraParams {
    pub fn new() -> Self {
        Self {
            loop_enabled: false,
            gain: 0,
            time: 1.0,
            temperature: -10.0,
            trigger_required: false,
            heating_pwm: 0.0,
            autoexp: true,
        }
    }
}

impl Default for CameraParams {
    fn default() -> Self {
        CameraParams::new()
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
    Established,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}
