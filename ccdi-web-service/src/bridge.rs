use ccdi_common::{log_err, to_string};
use std::fmt::Debug;
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc::UnboundedReceiver;

// ============================================ PUBLIC =============================================

pub fn start_std_to_tokio_channel_bridge<T: Debug + Send + 'static>(
    clients_rx: std::sync::mpsc::Receiver<T>,
    async_clients_tx: tokio::sync::mpsc::UnboundedSender<T>,
) -> Result<JoinHandle<()>, String> {
    thread::Builder::new()
        .name("server_to_tokio_transmitter".to_string())
        .spawn(move || {
            loop {
                match clients_rx.recv() {
                    Ok(message) => {
                        log_err(
                            "Clients sync to async transmitter",
                            async_clients_tx.send(message),
                        );
                    }
                    Err(_) => return, // Channel closed, exit
                }
            }
        })
        .map_err(to_string)
}

pub fn start_tokio_to_std_channel_bridge<T: Debug + Send + 'static>(
    mut async_clients_rx: UnboundedReceiver<T>,
    sync_server_tx: std::sync::mpsc::Sender<T>,
) {
    tokio::spawn(async move {
        while let Some(message) = async_clients_rx.recv().await {
            log_err(
                "Sending message to server worker thread",
                sync_server_tx.send(message),
            );
        }
    });
}
