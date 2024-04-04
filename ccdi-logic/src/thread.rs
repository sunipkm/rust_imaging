use std::sync::mpsc::{Receiver, Sender};
use std::{
    io::Cursor,
    sync::{mpsc::RecvTimeoutError, Arc},
    thread::{self, JoinHandle},
    time::Duration,
};

use ccdi_common::{
    log_err, ClientMessage, IoMessage, ProcessMessage, StateMessage, StorageMessage,
};
use image::{DynamicImage, ImageOutputFormat};
use log::{debug, error};

use crate::{io::IoManager, state::BackendState, storage::Storage, ServiceConfig};

// ============================================ PUBLIC =============================================

pub struct LogicParams {
    pub demo_mode: bool,
}

pub fn start_logic_thread(
    params: LogicParams,
    config: Arc<ServiceConfig>,
    server_rx: Receiver<StateMessage>,
    clients_tx: Sender<ClientMessage>,
    io_tx: Sender<IoMessage>,
    process_tx: Sender<ProcessMessage>,
    storage_tx: Sender<StorageMessage>,
) -> Result<JoinHandle<()>, String> {
    thread::Builder::new()
        .name("logic".to_string())
        .spawn(move || {
            let mut state =
                BackendState::new(params.demo_mode, process_tx, storage_tx.clone(), config);

            loop {
                match server_rx.recv_timeout(Duration::from_millis(50)) {
                    // Process the received message
                    Ok(message) => {
                        receive_message(&mut state, message, &clients_tx, &storage_tx, &io_tx)
                    }
                    // Last sender disconnected - exit thread
                    Err(RecvTimeoutError::Disconnected) => return,
                    // No messages received within timeout - perform periodic tasks
                    Err(RecvTimeoutError::Timeout) => {
                        periodic_tasks(&mut state, &clients_tx, &storage_tx, &io_tx)
                    }
                }
            }
        })
        .map_err(|err| format!("{:?}", err))
}

pub fn start_process_thread(
    process_rx: Receiver<ProcessMessage>,
    clients_tx: Sender<ClientMessage>,
    server_tx: Sender<StateMessage>,
) -> Result<JoinHandle<()>, String> {
    thread::Builder::new()
        .name("logic".to_string())
        .spawn(move || {
            loop {
                match process_rx.recv() {
                    // Process the received message
                    Ok(message) => {
                        debug!("Handling image process request");

                        // let reply = handle_process_message(message);
                        let reply = match message {
                            ProcessMessage::ConvertRawImage(message) => {
                                let img = message.image;
                                let size = message.size;
                                let img = DynamicImage::from(&img.data);
                                let img = img.resize(
                                    size.x as u32,
                                    size.y as u32,
                                    image::imageops::FilterType::Nearest,
                                );
                                let mut buf = Cursor::new(Vec::new());
                                let img = img
                                    .write_to(&mut buf, ImageOutputFormat::Png)
                                    .map_err(|err| format!("Error converting to PNG: {:?}", err));
                                match img {
                                    Ok(_) => {
                                        let img = buf.into_inner();
                                        vec![ClientMessage::PngImage(Arc::new(img))]
                                    }
                                    Err(err) => {
                                        error!("Error converting to PNG: {:?}", err);
                                        return;
                                    }
                                }
                            }
                        };

                        debug!("Image process finished");

                        for message in reply.into_iter() {
                            if let ClientMessage::PngImage(ref image) = message {
                                log_err(
                                    "Send process message to server",
                                    server_tx.send(StateMessage::ImageDisplayed(image.clone())),
                                );
                            }

                            log_err("Send process message to client", clients_tx.send(message));
                        }
                    }
                    // Last sender disconnected - exit thread
                    Err(_) => return,
                }
            }
        })
        .map_err(|err| format!("{:?}", err))
}

pub fn start_storage_thread(
    config: Arc<ServiceConfig>,
    storage_rx: Receiver<StorageMessage>,
    server_tx: Sender<StateMessage>,
) -> Result<JoinHandle<()>, String> {
    thread::Builder::new()
        .name("logic".to_string())
        .spawn(move || {
            let mut storage = Storage::new(config);

            let send_results = |result: Result<Vec<StateMessage>, String>| match result {
                Ok(messages) => {
                    for message in messages {
                        log_err(
                            "Send message from storage to server",
                            server_tx.send(message),
                        );
                    }
                }
                Err(error) => error!(
                    "Processing storage messages or periodic task failed: {}",
                    error
                ),
            };

            loop {
                match storage_rx.recv_timeout(Duration::from_millis(1000)) {
                    // Process the received message
                    Ok(message) => send_results(storage.process(message)),
                    // Last sender disconnected - exit thread
                    Err(RecvTimeoutError::Disconnected) => return,
                    // No messages received within timeout - perform periodic tasks
                    Err(RecvTimeoutError::Timeout) => send_results(storage.periodic_tasks()),
                }
            }
        })
        .map_err(|err| format!("{:?}", err))
}

pub fn start_io_thread(
    config: Arc<ServiceConfig>,
    storage_rx: Receiver<IoMessage>,
    server_tx: Sender<StateMessage>,
) -> Result<JoinHandle<()>, String> {
    thread::Builder::new()
        .name("logic".to_string())
        .spawn(move || {
            let mut io = IoManager::new(&config.io);

            let send_results = |result: Result<Vec<StateMessage>, String>| match result {
                Ok(messages) => {
                    for message in messages {
                        log_err(
                            "Send message from storage to server",
                            server_tx.send(message),
                        );
                    }
                }
                Err(error) => error!(
                    "Processing storage messages or periodic task failed: {}",
                    error
                ),
            };

            loop {
                match storage_rx.recv_timeout(Duration::from_millis(20)) {
                    // Process the received message
                    Ok(message) => send_results(io.process(message)),
                    // Last sender disconnected - exit thread
                    Err(RecvTimeoutError::Disconnected) => return,
                    // No messages received within timeout - perform periodic tasks
                    Err(RecvTimeoutError::Timeout) => send_results(io.periodic_tasks()),
                }
            }
        })
        .map_err(|err| format!("{:?}", err))
}

// =========================================== PRIVATE =============================================

fn receive_message(
    state: &mut BackendState,
    message: StateMessage,
    clients_tx: &Sender<ClientMessage>,
    storage_tx: &Sender<StorageMessage>,
    io_tx: &Sender<IoMessage>,
) {
    if let Some(responses) = log_err("Process state message", state.process(message)) {
        send_client_messages(responses.client_messages, clients_tx);
        send_storage_messages(responses.storage_messages, storage_tx);
        send_io_messages(responses.io_messages, io_tx);
    }
}

fn periodic_tasks(
    state: &mut BackendState,
    clients_tx: &Sender<ClientMessage>,
    storage_tx: &Sender<StorageMessage>,
    io_tx: &Sender<IoMessage>,
) {
    if let Some(responses) = log_err("Perform periodic tasks", state.periodic()) {
        send_client_messages(responses.client_messages, clients_tx);
        send_storage_messages(responses.storage_messages, storage_tx);
        send_io_messages(responses.io_messages, io_tx);
    }
}

fn send_client_messages(messages: Vec<ClientMessage>, clients_tx: &Sender<ClientMessage>) {
    for message in messages {
        log_err("Send client response", clients_tx.send(message));
    }
}

fn send_io_messages(messages: Vec<IoMessage>, io_tx: &Sender<IoMessage>) {
    for message in messages {
        log_err("Send io response", io_tx.send(message));
    }
}

fn send_storage_messages(messages: Vec<StorageMessage>, storage_tx: &Sender<StorageMessage>) {
    for message in messages {
        log_err("Send storage response", storage_tx.send(message));
    }
}
