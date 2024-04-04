use std::sync::Arc;

use nanocv::ImgSize;
use serde_derive::{Deserialize, Serialize};

use crate::RawImage;

// ============================================ PUBLIC =============================================

/// Message for image processing thread
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ProcessMessage {
    ConvertRawImage(ConvertRawImage),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ConvertRawImage {
    pub image: Arc<RawImage>,
    pub size: ImgSize,
}
