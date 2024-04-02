mod bridge;
mod config;
mod logger;
mod static_files;
mod websocket;

use ccdi_common::ClientMessage;
use ccdi_common::IoMessage;
use ccdi_common::ProcessMessage;
use ccdi_common::StateMessage;
use ccdi_common::StorageMessage;
use ccdi_logic::create_default_config_file;
use ccdi_logic::load_config_file;
use ccdi_logic::start_io_thread;
use ccdi_logic::start_logic_thread;
use ccdi_logic::start_process_thread;
use ccdi_logic::start_storage_thread;
use ccdi_logic::LogicParams;
use config::ServiceConfig;
use log::debug;
use log::{error, info};
use logger::init_logger;
use static_files::static_files_rules;
use tokio::sync::mpsc;

use bridge::start_std_to_tokio_channel_bridge;
use bridge::start_tokio_to_std_channel_bridge;
use warp::Filter;
use websocket::create_clients;
use websocket::create_websocket_service;
use websocket::start_single_async_to_multiple_clients_sender;

// ============================================ PUBLIC =============================================

fn main() {
    let config: ServiceConfig = argh::from_env();
    init_logger(config.debug, config.log.as_ref());

    let params = LogicParams {
        demo_mode: config.demo,
    };

    match create_default_config_file() {
        Ok(path) => info!(
            "Created default config in '{}'. Rename it to config.yaml to use it.",
            path
        ),
        Err(error) => error!("Could not create default config file: {}", error),
    }

    let config = match load_config_file() {
        Ok(config) => config,
        Err(error) => {
            error!("Config file could not be loaded: {}", error);
            return;
        }
    };

    debug!("Current config: {:?}", config);

    let (server_tx, server_rx) = std::sync::mpsc::channel::<StateMessage>();
    let (clients_tx, clients_rx) = std::sync::mpsc::channel::<ClientMessage>();
    let (process_tx, process_rx) = std::sync::mpsc::channel::<ProcessMessage>();
    let (storage_tx, storage_rx) = std::sync::mpsc::channel::<StorageMessage>();
    let (io_tx, io_rx) = std::sync::mpsc::channel::<IoMessage>();

    let _storage_thread = start_storage_thread(config.clone(), storage_rx, server_tx.clone());
    let _process_thread = start_process_thread(process_rx, clients_tx.clone(), server_tx.clone());
    let _io_thread = start_io_thread(config.clone(), io_rx, server_tx.clone());

    let _server_thread = start_logic_thread(
        params,
        config.clone(),
        server_rx,
        clients_tx,
        io_tx,
        process_tx,
        storage_tx,
    );

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Tokio failed to start")
        .block_on(tokio_main(server_tx, clients_rx))
}

async fn tokio_main(
    sync_server_tx: std::sync::mpsc::Sender<StateMessage>,
    sync_clients_rx: std::sync::mpsc::Receiver<ClientMessage>,
) {
    let (ws_from_client_tx, ws_from_client_rx) = mpsc::unbounded_channel::<StateMessage>();
    let (async_clients_tx, async_clients_rx) = mpsc::unbounded_channel::<ClientMessage>();
    // let server_tx = Arc::new(server_tx);

    let clients = create_clients(ws_from_client_tx);

    start_tokio_to_std_channel_bridge(ws_from_client_rx, sync_server_tx);
    start_single_async_to_multiple_clients_sender(clients.clone(), async_clients_rx);
    let _thread = start_std_to_tokio_channel_bridge(sync_clients_rx, async_clients_tx);

    let websocket_service = create_websocket_service("ccdi", clients);

    let routes = warp::get().and(websocket_service.or(static_files_rules()));

    warp::serve(routes).run(([0, 0, 0, 0], 8081)).await;
}
