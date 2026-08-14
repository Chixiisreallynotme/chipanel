use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    Extension,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use crate::{
    auth::middleware::AuthUser,
    config::AppConfig,
    models::websocket::{WsClientMessage, WsServerMessage},
    rcon::RconClient,
    websocket::WsHub,
};

/// Axum GET /ws WebSocket upgrade handler.
/// Rejects unauthenticated requests before upgrade using `AuthUser` extractor
/// (validates Bearer Authorization header or ?token= query parameter).
pub async fn websocket_handler(
    _auth: AuthUser,
    ws: WebSocketUpgrade,
    Extension(hub): Extension<WsHub>,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Response {
    ws.max_frame_size(65536)
        .max_message_size(65536)
        .on_upgrade(move |socket| handle_socket(socket, hub, config))
}

async fn handle_socket(socket: WebSocket, hub: WsHub, config: Arc<AppConfig>) {
    info!("WebSocket client connected");

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Send initial Telemetry payload upon connection
    let initial_telemetry = hub.get_latest_telemetry().await;
    if let Ok(json_str) = serde_json::to_string(&initial_telemetry) {
        if ws_sender.send(Message::Text(json_str)).await.is_err() {
            info!("Failed to send initial telemetry; terminating WebSocket connection");
            return;
        }
    }

    let mut broadcast_rx = hub.subscribe();

    // Bounded MPSC channel for outbound socket messages
    let (tx_out, mut rx_out) = tokio::sync::mpsc::channel::<Message>(64);

    // Writer task: forwards messages from tx_out channel to the WebSocket sink
    let writer_handle = tokio::spawn(async move {
        while let Some(msg) = rx_out.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Broadcast task: forwards broadcast messages from WsHub to tx_out channel
    let tx_out_broadcast = tx_out.clone();
    let broadcast_handle = tokio::spawn(async move {
        loop {
            match broadcast_rx.recv().await {
                Ok(server_msg) => {
                    if let Ok(json_str) = serde_json::to_string(&server_msg) {
                        if tx_out_broadcast.send(Message::Text(json_str)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!("WebSocket client lagged behind broadcast channel by {} messages", skipped);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });

    let mut last_command_time: Option<Instant> = None;

    // Reader loop: handles incoming client messages
    while let Some(msg_result) = ws_receiver.next().await {
        let msg = match msg_result {
            Ok(msg) => msg,
            Err(e) => {
                debug!("WebSocket connection error: {}", e);
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                match serde_json::from_str::<WsClientMessage>(&text) {
                    Ok(client_msg) => match client_msg {
                        WsClientMessage::Subscribe { topic } => {
                            debug!("Client subscribed to topic: {}", topic);
                        }
                        WsClientMessage::Command { command } => {
                            if let Some(last_time) = last_command_time {
                                if last_time.elapsed() < Duration::from_millis(200) {
                                    let err_msg = WsServerMessage::Error {
                                        message: "Rate limit exceeded: please wait between commands".into(),
                                    };
                                    send_server_message(&tx_out, &err_msg).await;
                                    continue;
                                }
                            }
                            last_command_time = Some(Instant::now());

                            if command.trim().is_empty() || command.len() > 1024 {
                                let err_msg = WsServerMessage::Error {
                                    message: "Invalid command: must be non-empty and under 1024 chars".into(),
                                };
                                send_server_message(&tx_out, &err_msg).await;
                                continue;
                            }

                            if command.contains('\n') || command.contains('\r') || command.contains('\0') {
                                let err_msg = WsServerMessage::CommandResult {
                                    command: command.clone(),
                                    output: "Control characters forbidden in command".to_string(),
                                    success: false,
                                };
                                send_server_message(&tx_out, &err_msg).await;
                                continue;
                            }

                            info!("Executing RCON command over WS: {}", command);

                            let result_msg = match RconClient::connect(
                                &config.rcon_host,
                                config.rcon_port,
                                &config.rcon_password,
                            )
                            .await
                            {
                                Ok(mut rcon) => match rcon.exec(&command).await {
                                    Ok(output) => WsServerMessage::CommandResult {
                                        command,
                                        output,
                                        success: true,
                                    },
                                    Err(err) => WsServerMessage::CommandResult {
                                        command,
                                        output: format!("Command execution failed: {}", err),
                                        success: false,
                                    },
                                },
                                Err(err) => WsServerMessage::CommandResult {
                                    command,
                                    output: format!("RCON connection failed: {}", err),
                                    success: false,
                                },
                            };

                            send_server_message(&tx_out, &result_msg).await;
                        }
                        WsClientMessage::Ping => {
                            send_server_message(&tx_out, &WsServerMessage::Pong).await;
                        }
                    },
                    Err(err) => {
                        let err_msg = WsServerMessage::Error {
                            message: format!("Invalid message JSON: {}", err),
                        };
                        send_server_message(&tx_out, &err_msg).await;
                    }
                }
            }
            Message::Close(_) => {
                info!("Client sent Close frame");
                break;
            }
            Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => {}
        }
    }

    // Abort helper tasks on client disconnect to prevent leaks
    broadcast_handle.abort();
    writer_handle.abort();

    info!("WebSocket connection closed");
}

async fn send_server_message(
    tx_out: &tokio::sync::mpsc::Sender<Message>,
    msg: &WsServerMessage,
) {
    if let Ok(json_str) = serde_json::to_string(msg) {
        let _ = tx_out.send(Message::Text(json_str)).await;
    }
}
