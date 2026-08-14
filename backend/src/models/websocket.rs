use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    #[serde(alias = "Subscribe")]
    Subscribe { topic: String },
    #[serde(alias = "Command")]
    Command { command: String },
    #[serde(alias = "Ping")]
    Ping,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsServerMessage {
    /// Live telemetry frame. Every numeric field is `Option` and serializes to `null` when the
    /// underlying source did not answer — never substitute a plausible-looking value (a fabricated
    /// "20.0 TPS" is indistinguishable from a healthy server). `podman_available` /
    /// `rcon_available` tell the UI *why* a value is missing.
    Telemetry {
        /// The Podman API answered the container inspect call.
        podman_available: bool,
        /// `None` when `podman_available` is false; otherwise whether the container is running.
        container_running: Option<bool>,
        cpu_percent: Option<f32>,
        memory_bytes: Option<u64>,
        memory_limit_bytes: Option<u64>,
        memory_percent: Option<f32>,
        /// An RCON connection to the Minecraft server was established.
        rcon_available: bool,
        /// `None` if RCON is down or the server has no `tps` command (vanilla).
        tps: Option<f32>,
        online_players: Option<u32>,
        max_players: Option<u32>,
    },
    ConsoleLog {
        timestamp: i64,
        level: String,
        message: String,
    },
    CommandResult {
        command: String,
        output: String,
        success: bool,
    },
    Error {
        message: String,
    },
    Pong,
}
