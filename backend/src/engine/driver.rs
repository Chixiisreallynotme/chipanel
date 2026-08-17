use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use crate::error::AppError;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Unified game telemetry across all supported game engines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTelemetry {
    pub game_id: String,
    pub display_name: String,
    pub online: bool,
    /// "running" | "hibernating" | "stopped" | "unknown"
    pub server_state: String,
    pub players_online: Option<u32>,
    pub players_max: Option<u32>,
    pub tps: Option<f32>,
    pub cpu_percent: Option<f32>,
    pub memory_bytes: Option<u64>,
    pub memory_limit_bytes: Option<u64>,
    pub memory_percent: Option<f32>,
    #[serde(default)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Unified player info across all supported game engines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePlayerInfo {
    pub name: String,
    /// Identifier such as Minecraft UUID or SteamID64
    pub id: Option<String>,
    pub online: bool,
    pub ping: Option<u32>,
    #[serde(default)]
    pub extra: BTreeMap<String, String>,
}

/// Result of triggering a game world backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBackupResult {
    pub success: bool,
    pub filename: String,
    pub size_bytes: u64,
    pub message: String,
}

/// Result of modifying the game power / hibernation mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePowerModeResult {
    pub mode: String,
    pub running: bool,
    pub proxy_active: bool,
    pub message: String,
}

/// Extensible Game Driver abstraction for dedicated game servers.
///
/// Designed to future-proof ChiPanel for Minecraft, Palworld, Valheim, and beyond.
pub trait GameDriver: Send + Sync {
    /// Unique identifier for this game (e.g. "minecraft", "palworld", "valheim")
    fn game_id(&self) -> &'static str;

    /// Human-friendly display name (e.g. "Minecraft: Java Edition")
    fn display_name(&self) -> &'static str;

    /// Queries live game server telemetry (container status, CPU/RAM, tick rate, players).
    fn get_telemetry<'a>(&'a self) -> BoxFuture<'a, Result<GameTelemetry, AppError>>;

    /// Executes an in-game administrative or console command.
    fn execute_command<'a>(&'a self, cmd: &'a str) -> BoxFuture<'a, Result<String, AppError>>;

    /// Retrieves current online players.
    fn get_players<'a>(&'a self) -> BoxFuture<'a, Result<Vec<GamePlayerInfo>, AppError>>;

    /// Triggers a world save and backup creation.
    fn trigger_backup<'a>(
        &'a self,
        target_name: Option<&'a str>,
    ) -> BoxFuture<'a, Result<GameBackupResult, AppError>>;

    /// Sets the operational power mode ("off", "on", "hibernate").
    fn set_power_mode<'a>(
        &'a self,
        mode: &'a str,
    ) -> BoxFuture<'a, Result<GamePowerModeResult, AppError>>;
}
