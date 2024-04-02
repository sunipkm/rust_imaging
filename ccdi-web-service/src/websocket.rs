use ccdi_common::{log_err, rgb_image_to_bytes, to_string, ClientMessage, StateMessage};
use log::*;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender, UnboundedSender};
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

use futures::{FutureExt, StreamExt};
use warp::ws::{Message, WebSocket, Ws};
use warp::{Error, Filter, Rejection, Reply};

// ============================================ PUBLIC =============================================

pub type Clients = Arc<RwLock<ClientSharedState>>;

pub struct ClientSharedState {
    counter: usize,
    server_tx: UnboundedSender<StateMessage>,
    transmitters: HashMap<usize, Sender<Result<Message, Error>>>,
}

pub fn start_single_async_to_multiple_clients_sender(
    clients: Clients,
    mut async_clients_rx: tokio::sync::mpsc::UnboundedReceiver<ClientMessage>,
) {
    tokio::spawn(async move {
        let reconnect = serde_json::to_string(&ClientMessage::Reconnect)
            .expect("Could not prepare reconnect message");

        loop {
            if let Some(message) = async_clients_rx.recv().await {
                if let Ok(payload) = serialize(&message) {
                    for (index, transmitter) in clients.read().await.transmitters.iter() {
                        if transmitter.capacity() < MIN_CAPACITY {
                            log_err(
                                "Send reconnect message",
                                transmitter.try_send(Ok(Message::text(&reconnect))),
                            );
                            warn!(
                                "Client {} queue full (cap {}), instructing to reconnect.",
                                index,
                                transmitter.capacity()
                            );
                        } else if let Err(_error) = transmitter.try_send(Ok(payload.clone())) {
                            warn!("Error sending message to client {}", index);
                        }
                    }
                }
            }
        }
    });
}

pub fn create_clients(ws_from_client_tx: UnboundedSender<StateMessage>) -> Clients {
    Arc::new(RwLock::new(ClientSharedState::new(ws_from_client_tx)))
}

pub fn create_websocket_service(
    path: &str,
    clients: Clients,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + '_ {
    warp::path(path)
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and_then(ws_handler)
}

// =========================================== PRIVATE =============================================

fn serialize(message: &ClientMessage) -> Result<Message, String> {
    match message {
        ClientMessage::RgbImage(image) => Ok(Message::binary(rgb_image_to_bytes(image))),
        _other => Ok(Message::text(
            serde_json::to_string(&message).map_err(to_string)?,
        )),
    }
}

const CAPACITY: usize = 20;
const MIN_CAPACITY: usize = 5;

async fn ws_handler(ws: Ws, clients: Clients) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(move |socket| handle_client_connection(socket, clients)))
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

async fn handle_client_connection(websocket: WebSocket, clients: Clients) {
    let server_tx = clients.read().await.server_tx.clone();
    let (ws_tx, mut ws_rx) = websocket.split();

    let (client_sender, client_rcv) = mpsc::channel::<Result<Message, Error>>(CAPACITY);

    let client_rcv = ReceiverStream::new(client_rcv);

    tokio::task::spawn(client_rcv.forward(ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    let id = clients.write().await.register_client(client_sender);

    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(message) => log_err(
                "Processing server message from client",
                process_message(message, &server_tx).await,
            ),
            Err(e) => {
                eprintln!("error receiving ws message: {}", e);
                break;
            }
        };
    }
    log_err(
        "Unregister client",
        clients.write().await.unregister_client(id),
    );
}

async fn process_message(
    message: Message,
    server_tx: &UnboundedSender<StateMessage>,
) -> Result<(), String> {
    server_tx
        .send(convert_state_message(message)?)
        .map_err(to_string)
}

fn convert_state_message(message: Message) -> Result<StateMessage, String> {
    if message.is_text() {
        let json_string = String::from_utf8(message.into_bytes()).map_err(to_string)?;
        serde_json::from_str::<StateMessage>(&json_string).map_err(to_string)
    } else {
        Err(format!("Msg {:?} is not text", message))
    }
}

impl ClientSharedState {
    fn new(server_tx: UnboundedSender<StateMessage>) -> Self {
        Self {
            counter: 0,
            server_tx,
            transmitters: HashMap::new(),
        }
    }

    fn register_client(&mut self, transmitter: Sender<Result<Message, Error>>) -> usize {
        let id = self.counter;
        self.counter += 1;
        self.transmitters.insert(id, transmitter);
        info!(
            "Client {} registered ({} clients total)",
            id,
            self.transmitters.len()
        );
        id
    }

    fn unregister_client(&mut self, id: usize) -> Result<(), String> {
        self.transmitters.remove(&id);
        info!(
            "Client {} unregistered ({} clients total)",
            id,
            self.transmitters.len()
        );
        Ok(())
    }
}
