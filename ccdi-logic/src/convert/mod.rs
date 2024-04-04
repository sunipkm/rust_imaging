use std::sync::Arc;

use ccdi_common::{ClientMessage, ProcessMessage, ConvertRawImage, debayer_scale_fast};
use log::debug;

// ============================================ PUBLIC =============================================

pub fn handle_process_message(message: ProcessMessage) -> Vec<ClientMessage> {
    match message {
        ProcessMessage::ConvertRawImage(message) => vec![
            convert_raw_image(message)
        ],
    }
}

// =========================================== PRIVATE =============================================

fn convert_raw_image(message: ConvertRawImage) -> ClientMessage {
    debug!(
        "Processing image {} x {} -> {} x {}",
        message.image.params.area.width, message.image.params.area.height,
        message.size.x, message.size.y
    );
    let rgb_image = Arc::new(debayer_scale_fast(&message.image, message.size));
    ClientMessage::PngImage(rgb_image)
}