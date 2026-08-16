use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, warn};

use crate::config::AppConfig;
use crate::error::AppError;
use crate::rcon::client::RconClient;

enum RconActorMsg {
    Exec {
        command: String,
        responder: oneshot::Sender<Result<String, AppError>>,
    },
}

#[derive(Clone, Debug)]
pub struct RconActorHandle {
    sender: mpsc::Sender<RconActorMsg>,
}

impl RconActorHandle {
    /// Spawns the persistent RCON actor in the background and returns a cloneable handle.
    pub fn spawn(config: Arc<AppConfig>) -> Self {
        let (tx, rx) = mpsc::channel(128);
        let actor = RconActor::new(config, rx);
        tokio::spawn(actor.run());
        Self { sender: tx }
    }

    /// Executes an RCON command through the persistent actor and returns the server response.
    pub async fn exec(&self, command: &str) -> Result<String, AppError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.sender
            .send(RconActorMsg::Exec {
                command: command.to_string(),
                responder: resp_tx,
            })
            .await
            .map_err(|_| AppError::InternalError("RCON background actor is down".to_string()))?;

        match tokio::time::timeout(Duration::from_secs(8), resp_rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(AppError::InternalError("RCON responder dropped before sending output".to_string())),
            Err(_) => Err(AppError::InternalError(format!("RCON command '{}' timed out after 8s", command))),
        }
    }
}

struct RconActor {
    config: Arc<AppConfig>,
    receiver: mpsc::Receiver<RconActorMsg>,
    client: Option<RconClient>,
}

impl RconActor {
    fn new(config: Arc<AppConfig>, receiver: mpsc::Receiver<RconActorMsg>) -> Self {
        Self {
            config,
            receiver,
            client: None,
        }
    }

    async fn run(mut self) {
        debug!("Persistent RCON actor started");

        while let Some(msg) = self.receiver.recv().await {
            match msg {
                RconActorMsg::Exec { command, responder } => {
                    let result = self.handle_exec(&command).await;
                    let _ = responder.send(result);
                }
            }
        }

        debug!("Persistent RCON actor terminated");
    }

    async fn handle_exec(&mut self, command: &str) -> Result<String, AppError> {
        // Attempt execution on existing connection
        if let Some(client) = self.client.as_mut() {
            match client.exec(command).await {
                Ok(output) => return Ok(output),
                Err(err) => {
                    warn!("RCON command on active connection failed ({}); resetting connection...", err);
                    self.client = None;
                }
            }
        }

        // Connect if not currently connected
        let mut new_client = match RconClient::connect(
            &self.config.rcon_host,
            self.config.rcon_port,
            &self.config.rcon_password,
        )
        .await
        {
            Ok(c) => c,
            Err(e) => {
                return Err(AppError::InternalError(format!("Failed to connect to RCON server: {}", e)));
            }
        };

        match new_client.exec(command).await {
            Ok(output) => {
                self.client = Some(new_client);
                Ok(output)
            }
            Err(e) => {
                self.client = None;
                Err(e)
            }
        }
    }
}
