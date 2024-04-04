use std::sync::{mpsc::Sender, Arc};

use ccdi_common::{ClientMessage, IoMessage, ProcessMessage, StateMessage, StorageMessage};

use crate::{camera::CameraController, ServiceConfig};
use log::info;

// ============================================ PUBLIC =============================================

pub struct BackendState {
    camera: CameraController,
    /// Last image sent to clients
    image: Option<Arc<Vec<u8>>>,
}

impl BackendState {
    pub fn new(
        demo_mode: &str,
        process_tx: Sender<ProcessMessage>,
        storage_tx: Sender<StorageMessage>,
        config: Arc<ServiceConfig>,
    ) -> Self {
        Self {
            camera: CameraController::new(
                match demo_mode {
                    "asi" => {
                        let mut drv = ccdi_imager_asicam::ASICameraDriver::new();
                        drv.update_opt_config(config.exp.get_optimum_exp_config().unwrap());
                        Box::new(drv)
                    },
                    "fli" => {
                        let mut drv = ccdi_imager_fli::FLICameraDriver::new();
                        drv.update_opt_config(config.exp.get_optimum_exp_config().unwrap());
                        Box::new(drv)
                    }
                    _ => Box::new(ccdi_imager_demo::DemoImagerDriver::new()),
                },
                process_tx,
                storage_tx,
                config,
            ),
            image: None,
        }
    }

    // This is where messages are processed by the server. ~Mit
    /// Process incoming message and return messages to be sent to clients
    pub fn process(&mut self, message: StateMessage) -> Result<BackendResult, String> {
        use StateMessage::*;

        Ok(match message {
            ClientInformation(info) => {
                // Do something with the information.
                info!("Client information received: {:?}", info);
                self.return_view() // ?
            }
            ImageDisplayed(image) => {
                self.image = Some(image);
                BackendResult::empty()
            }
            // TODO: ImageParam?
            CameraParam(message) => {
                let heating = match message {
                    ccdi_common::CameraParamMessage::SetHeatingPwm(value) => Some(value),
                    _ => None,
                };

                self.camera.update_camera_params(message);

                match heating {
                    None => self.return_view(),
                    Some(heating) => BackendResult {
                        client_messages: vec![ClientMessage::View(Box::new(
                            self.camera.get_view(),
                        ))],
                        storage_messages: vec![],
                        io_messages: vec![IoMessage::SetHeating(heating as f32)],
                    },
                }
            }
            ImageParam(message) => {
                log::debug!("ImageParam message: {:?}", message);

                self.camera.update_image_params(message);
                // TODO: Do something with the exact type of image param message

                BackendResult::empty()
            }
            ExposureMessage(command) => {
                self.camera.exposure_command(command);
                self.return_view()
            }
            ClientConnected => {
                let view_msg = ClientMessage::View(Box::new(self.camera.get_view()));

                BackendResult::client(match self.image.as_ref() {
                    None => vec![view_msg],
                    Some(image) => vec![view_msg, ClientMessage::PngImage(image.clone())],
                })
            }
            UpdateStorageState(storage_state) => {
                self.camera.update_storage_status(storage_state);
                self.return_view()
            }
            TriggerValueChanged(value) => {
                self.camera.update_trigger_status(value);
                // Trigger might be switched on, perform idle tasks immediately
                let (client, io) = self.camera.periodic();
                BackendResult::client_io(client, io)
            }
            StorageMessage(message) => BackendResult {
                client_messages: Vec::new(),
                storage_messages: vec![message],
                io_messages: Vec::new(),
            },
            UpdateStorageDetail(detail) => {
                self.camera.update_storage_detail(detail);
                self.return_view()
            }
            PowerOff => {
                self.camera.turn_off();
                BackendResult::empty()
            }
        })
    }

    /// Called periodically to perform any tasks needed and return messages for clients
    pub fn periodic(&mut self) -> Result<BackendResult, String> {
        let (client, io) = self.camera.periodic();
        Ok(BackendResult::client_io(client, io))
    }
}

pub struct BackendResult {
    pub client_messages: Vec<ClientMessage>,
    pub storage_messages: Vec<StorageMessage>,
    pub io_messages: Vec<IoMessage>,
}

impl BackendResult {
    pub fn empty() -> Self {
        BackendResult {
            client_messages: Vec::new(),
            storage_messages: Vec::new(),
            io_messages: Vec::new(),
        }
    }

    pub fn client(client_messages: Vec<ClientMessage>) -> Self {
        Self {
            client_messages,
            storage_messages: Vec::new(),
            io_messages: Vec::new(),
        }
    }

    pub fn client_io(client: Vec<ClientMessage>, io: Vec<IoMessage>) -> Self {
        Self {
            client_messages: client,
            io_messages: io,
            storage_messages: Vec::new(),
        }
    }
}

// =========================================== PRIVATE =============================================

impl BackendState {
    fn return_view(&self) -> BackendResult {
        BackendResult {
            client_messages: vec![ClientMessage::View(Box::new(self.camera.get_view()))],
            storage_messages: Vec::new(),
            io_messages: Vec::new(),
        }
    }
}
