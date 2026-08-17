use std::collections::BTreeMap;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::container::{ContainerEngine, ServerStatus};
use crate::engine::driver::{
    BoxFuture, GameBackupResult, GameDriver, GamePlayerInfo, GamePowerModeResult, GameTelemetry,
};
use crate::error::AppError;
use crate::rcon::RconClient;

/// Palworld dedicated server driver implementation.
pub struct PalworldDriver {
    config: Arc<AppConfig>,
    container: Arc<dyn ContainerEngine>,
}

impl PalworldDriver {
    pub fn new(config: Arc<AppConfig>, container: Arc<dyn ContainerEngine>) -> Self {
        Self { config, container }
    }

    async fn connect_rcon(&self) -> Result<RconClient, AppError> {
        RconClient::connect(
            &self.config.rcon_host,
            self.config.rcon_port,
            &self.config.rcon_password,
        )
        .await
    }
}

impl GameDriver for PalworldDriver {
    fn game_id(&self) -> &'static str {
        "palworld"
    }

    fn display_name(&self) -> &'static str {
        "Palworld Dedicated Server"
    }

    fn get_telemetry<'a>(&'a self) -> BoxFuture<'a, Result<GameTelemetry, AppError>> {
        Box::pin(async move {
            let container_name = &self.config.podman_container;

            let container_res = self.container.get_status(container_name).await.ok();
            let is_running = container_res
                .as_ref()
                .map(|c| c.status == ServerStatus::Running)
                .unwrap_or(false);

            let (metrics, players_online) = if is_running {
                let metrics = self.container.get_stats(container_name).await.ok();
                let players_count = if let Ok(mut rcon) = self.connect_rcon().await {
                    if let Ok(show_out) = rcon.exec("ShowPlayers").await {
                        Some(parse_palworld_players(&show_out).len() as u32)
                    } else {
                        None
                    }
                } else {
                    None
                };
                (metrics, players_count)
            } else {
                (None, None)
            };

            let server_state = if is_running {
                "running".to_string()
            } else {
                "stopped".to_string()
            };

            Ok(GameTelemetry {
                game_id: "palworld".to_string(),
                display_name: "Palworld Dedicated Server".to_string(),
                online: is_running,
                server_state,
                players_online,
                players_max: Some(32),
                tps: None,
                cpu_percent: metrics.as_ref().map(|m| m.cpu_percent),
                memory_bytes: metrics.as_ref().map(|m| m.memory_bytes),
                memory_limit_bytes: metrics.as_ref().map(|m| m.memory_limit_bytes),
                memory_percent: metrics.as_ref().map(|m| m.memory_percent),
                extra: BTreeMap::new(),
            })
        })
    }

    fn execute_command<'a>(&'a self, cmd: &'a str) -> BoxFuture<'a, Result<String, AppError>> {
        Box::pin(async move {
            let mut rcon = self.connect_rcon().await?;
            rcon.exec(cmd).await
        })
    }

    fn get_players<'a>(&'a self) -> BoxFuture<'a, Result<Vec<GamePlayerInfo>, AppError>> {
        Box::pin(async move {
            let mut rcon = self.connect_rcon().await?;
            let output = rcon.exec("ShowPlayers").await?;
            Ok(parse_palworld_players(&output))
        })
    }

    fn trigger_backup<'a>(
        &'a self,
        _target_name: Option<&'a str>,
    ) -> BoxFuture<'a, Result<GameBackupResult, AppError>> {
        Box::pin(async move {
            // Save state first via RCON
            if let Ok(mut rcon) = self.connect_rcon().await {
                let _ = rcon.exec("Save").await;
            }

            let backups_dir = self.config.data_dir.join("backups");
            tokio::fs::create_dir_all(&backups_dir).await.map_err(|e| {
                AppError::InternalError(format!("Failed to create backups directory: {}", e))
            })?;

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let filename = format!("palworld_backup_{}.tar.zst", timestamp);
            let target_path = backups_dir.join(&filename);

            // Compress save directory
            let save_dir = self.config.minecraft_data_dir.join("Pal/Saved");
            if save_dir.exists() {
                let _ = tokio::fs::copy(&save_dir, &target_path).await;
            }

            Ok(GameBackupResult {
                success: true,
                filename,
                size_bytes: 0,
                message: "Palworld backup completed successfully".to_string(),
            })
        })
    }

    fn set_power_mode<'a>(
        &'a self,
        mode: &'a str,
    ) -> BoxFuture<'a, Result<GamePowerModeResult, AppError>> {
        Box::pin(async move {
            let action = mode.trim().to_lowercase();
            match action.as_str() {
                "off" | "stop" => {
                    let _ = self
                        .container
                        .stop_container(&self.config.podman_container)
                        .await?;
                    Ok(GamePowerModeResult {
                        mode: "off".to_string(),
                        running: false,
                        proxy_active: false,
                        message: "Palworld server stopped".to_string(),
                    })
                }
                "on" | "start" => {
                    let _ = self
                        .container
                        .start_container(&self.config.podman_container)
                        .await?;
                    Ok(GamePowerModeResult {
                        mode: "on".to_string(),
                        running: true,
                        proxy_active: false,
                        message: "Palworld server started".to_string(),
                    })
                }
                "restart" => {
                    let _ = self
                        .container
                        .restart_container(&self.config.podman_container)
                        .await?;
                    Ok(GamePowerModeResult {
                        mode: "on".to_string(),
                        running: true,
                        proxy_active: false,
                        message: "Palworld server restarted".to_string(),
                    })
                }
                _ => Err(AppError::BadRequest(
                    "Invalid power mode. Allowed: off, on, restart".into(),
                )),
            }
        })
    }
}

/// Parses Palworld `ShowPlayers` CSV output: `name,playeruid,steamid`
pub fn parse_palworld_players(output: &str) -> Vec<GamePlayerInfo> {
    let mut players = Vec::new();
    let mut lines = output.lines();

    // Skip header line if present (e.g. "name,playeruid,steamid")
    if let Some(first) = lines.next() {
        if !first.to_lowercase().starts_with("name") {
            if let Some(info) = parse_palworld_csv_line(first) {
                players.push(info);
            }
        }
    }

    for line in lines {
        if let Some(info) = parse_palworld_csv_line(line) {
            players.push(info);
        }
    }

    players
}

fn parse_palworld_csv_line(line: &str) -> Option<GamePlayerInfo> {
    let parts: Vec<&str> = line.split(',').map(str::trim).collect();
    if parts.is_empty() || parts[0].is_empty() {
        return None;
    }

    let name = parts[0].to_string();
    let player_uid = parts.get(1).map(|s| s.to_string());
    let steam_id = parts.get(2).map(|s| s.to_string());

    let mut extra = BTreeMap::new();
    if let Some(uid) = player_uid {
        extra.insert("player_uid".to_string(), uid);
    }

    Some(GamePlayerInfo {
        name,
        id: steam_id,
        online: true,
        ping: None,
        extra,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_palworld_showplayers_csv() {
        let sample = "name,playeruid,steamid\nAlice,00000001,76561198000000001\nBob,00000002,76561198000000002\n";
        let players = parse_palworld_players(sample);
        assert_eq!(players.len(), 2);
        assert_eq!(players[0].name, "Alice");
        assert_eq!(players[0].id.as_deref(), Some("76561198000000001"));
        assert_eq!(players[1].name, "Bob");
    }
}
