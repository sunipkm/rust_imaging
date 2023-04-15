use std::{thread::{self, JoinHandle}, time::Duration, sync::mpsc::RecvTimeoutError};
use std::sync::mpsc::{Sender, Receiver};

use ccdi_common::{StateMessage, ClientMessage, log_err};
use log::{debug};

use crate::state::State;

// ============================================ PUBLIC =============================================

pub fn start_logic_thread(
    server_rx: Receiver<StateMessage>,
    clients_tx: Sender<ClientMessage>,
) -> Result<JoinHandle<()>, String> {
    thread::Builder::new()
        .name("server".to_string())
        .spawn(move || {
            loop {
                let mut state = State::new();

                match server_rx.recv_timeout(Duration::from_millis(500)) {
                    // Process the received message
                    Ok(message) => receive_message(&mut state, message, &clients_tx),
                    // Last sender disconnected - exit thread
                    Err(RecvTimeoutError::Disconnected) => return,
                    // No messages received within timeout - perform periodic tasks
                    Err(RecvTimeoutError::Timeout) => periodic_tasks(&mut state, &clients_tx),
                }
            }
        })
        .map_err(|err| format!("{:?}", err))
}

// =========================================== PRIVATE =============================================

fn receive_message(
    state: &mut State,
    message: StateMessage,
    clients_tx: &Sender<ClientMessage>,
) {
    debug!("State message received: {:?}", &message);

    if let Some(responses) = log_err("Process state message", state.process(message)) {
        send_client_messages(responses, clients_tx);
    }
}

fn periodic_tasks(
    state: &mut State,
    clients_tx: &Sender<ClientMessage>,
) {
    if let Some(responses) = log_err("Perform periodic tasks", state.periodic()) {
        send_client_messages(responses, clients_tx);
    }
}

fn send_client_messages(
    messages: Vec<ClientMessage>,
    clients_tx: &Sender<ClientMessage>,
) {
    for message in messages {
        log_err("Send client response", clients_tx.send(message));
    }
}