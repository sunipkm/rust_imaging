use std::sync::Arc;

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

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Vec2d<T> {
    pub x: T,
    pub y: T,
}

impl<T: Sized> Vec2d<T> {
    pub fn new(x: T, y: T) -> Vec2d<T> {
        Vec2d { x, y }
    }
}

pub type ImgSize = Vec2d<u16>;
