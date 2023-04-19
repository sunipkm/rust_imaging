use ccdi_common::{ClientMessage, StateMessage};

use crate::camera::CameraController;

// ============================================ PUBLIC =============================================

pub struct BackendState {
    camera: CameraController
}

impl BackendState {
    pub fn new(demo_mode: bool) -> Self {
        Self {
            camera: CameraController::new(
                match demo_mode {
                    false => Box::new(ccdi_imager_moravian::MoravianImagerDriver::new()),
                    true => Box::new(ccdi_imager_demo::DemoImagerDriver::new()),
                }
            )
        }
    }

    /// Process incoming message and return messages to be sent to clients
    pub fn process(&mut self, message: StateMessage) -> Result<Vec<ClientMessage>, String> {
        use StateMessage::*;

        Ok(match message {
            ExposureMessage(command) => {
                self.camera.exposure_command(command);
                vec![]
            },
            ClientConnected => vec![
                ClientMessage::View(self.camera.get_view()),
                ClientMessage::JpegImage(TEST_IMAGE.to_vec()),
            ]
        })
    }

    /// Called periodically to perform any tasks needed and return messages for clients
    pub fn periodic(&mut self) -> Result<Vec<ClientMessage>, String> {
        Ok(self.camera.periodic())
    }
}

// =========================================== PRIVATE =============================================

const TEST_IMAGE: &[u8] = include_bytes!("test-image.jpg");