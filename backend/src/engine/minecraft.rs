use std::collections::BTreeMap;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::container::{ContainerEngine, ServerStatus};
use crate::engine::driver::{
    BoxFuture, GameBackupResult, GameDriver, GamePlayerInfo, GamePowerModeResult, GameTelemetry,
};
use crate::error::AppError;
use crate::rcon::RconActorHandle;

/// 1st class Minecraft server driver implementation encapsulating RCON, container runtime, and worlds.
pub struct MinecraftDriver {
    config: Arc<AppConfig>,
    container: Arc<dyn ContainerEngine>,
    rcon: RconActorHandle,
}

impl MinecraftDriver {
    pub fn new(
        config: Arc<AppConfig>,
        container: Arc<dyn ContainerEngine>,
        rcon: RconActorHandle,
    ) -> Self {
        Self {
            config,
            container,
            rcon,
        }
    }

    async fn probe_public_port(&self) -> bool {
        crate::routes::server::probe_tcp(&self.config.rcon_host, 25565).await
    }
}

impl GameDriver for MinecraftDriver {
    fn game_id(&self) -> &'static str {
        "minecraft"
    }

    fn display_name(&self) -> &'static str {
        "Minecraft: Java Edition"
    }

    fn get_telemetry<'a>(&'a self) -> BoxFuture<'a, Result<GameTelemetry, AppError>> {
        Box::pin(async move {
            let container_name = &self.config.podman_container;

            let container_res = self.container.get_status(container_name).await.ok();
            let is_running = container_res
                .as_ref()
                .map(|c| c.status == ServerStatus::Running)
                .unwrap_or(false);

            let (metrics, rcon_list, rcon_tps) = if is_running {
                let stats_fut = self.container.get_stats(container_name);
                let list_fut = self.rcon.exec("list");
                let tps_fut = self.rcon.exec("tps");
                let (stats, list, tps) = tokio::join!(stats_fut, list_fut, tps_fut);
                (stats.ok(), list.ok(), tps.ok())
            } else {
                (None, None, None)
            };

            let (players_online, players_max) = match rcon_list {
                Some(ref output) => parse_minecraft_player_list(output),
                None => (None, None),
            };

            let tps = rcon_tps.as_deref().and_then(parse_minecraft_tps);

            let public_port_active = self.probe_public_port().await;
            let server_state = if is_running {
                "running".to_string()
            } else if public_port_active {
                "hibernating".to_string()
            } else {
                "stopped".to_string()
            };

            let mut extra = BTreeMap::new();
            if let Some(active_world) =
                crate::minecraft::worlds::get_active_world_name(&self.config.minecraft_data_dir)
                    .await
            {
                extra.insert(
                    "active_world".to_string(),
                    serde_json::Value::String(active_world),
                );
            }

            Ok(GameTelemetry {
                game_id: "minecraft".to_string(),
                display_name: "Minecraft: Java Edition".to_string(),
                online: is_running,
                server_state,
                players_online,
                players_max,
                tps,
                cpu_percent: metrics.as_ref().map(|m| m.cpu_percent),
                memory_bytes: metrics.as_ref().map(|m| m.memory_bytes),
                memory_limit_bytes: metrics.as_ref().map(|m| m.memory_limit_bytes),
                memory_percent: metrics.as_ref().map(|m| m.memory_percent),
                extra,
            })
        })
    }

    fn execute_command<'a>(&'a self, cmd: &'a str) -> BoxFuture<'a, Result<String, AppError>> {
        Box::pin(async move {
            if cmd.contains('\n') || cmd.contains('\r') || cmd.contains('\0') {
                return Err(AppError::BadRequest(
                    "Invalid command: control characters forbidden".into(),
                ));
            }
            self.rcon.exec(cmd).await
        })
    }

    fn get_players<'a>(&'a self) -> BoxFuture<'a, Result<Vec<GamePlayerInfo>, AppError>> {
        Box::pin(async move {
            let list_out = match self.rcon.exec("list").await {
                Ok(out) => out,
                Err(_) => return Ok(Vec::new()),
            };

            let online_names =
                crate::minecraft::player::parse_online_players_from_rcon(&list_out);
            let mut players = Vec::with_capacity(online_names.len());

            for name in online_names {
                players.push(GamePlayerInfo {
                    name,
                    id: None,
                    online: true,
                    ping: None,
                    extra: BTreeMap::new(),
                });
            }

            Ok(players)
        })
    }

    fn trigger_backup<'a>(
        &'a self,
        target_name: Option<&'a str>,
    ) -> BoxFuture<'a, Result<GameBackupResult, AppError>> {
        Box::pin(async move {
            let backups_dir = self.config.data_dir.join("backups");
            tokio::fs::create_dir_all(&backups_dir).await.map_err(|e| {
                AppError::InternalError(format!("Failed to create backups directory: {}", e))
            })?;

            if let Some(world_name) = target_name {
                let info = crate::minecraft::worlds::create_backup(
                    &self.config.minecraft_data_dir,
                    &backups_dir,
                    world_name,
                )
                .await?;

                Ok(GameBackupResult {
                    success: true,
                    filename: info.filename,
                    size_bytes: info.size_bytes,
                    message: format!("World '{}' backup created successfully", world_name),
                })
            } else {
                let active_world = crate::minecraft::worlds::get_active_world_name(
                    &self.config.minecraft_data_dir,
                )
                .await
                .unwrap_or_else(|| "world".to_string());

                let info = crate::minecraft::worlds::create_backup(
                    &self.config.minecraft_data_dir,
                    &backups_dir,
                    &active_world,
                )
                .await?;

                Ok(GameBackupResult {
                    success: true,
                    filename: info.filename,
                    size_bytes: info.size_bytes,
                    message: format!("Active world '{}' backup created successfully", active_world),
                })
            }
        })
    }

    fn set_power_mode<'a>(
        &'a self,
        mode: &'a str,
    ) -> BoxFuture<'a, Result<GamePowerModeResult, AppError>> {
        Box::pin(async move {
            let action = mode.trim().to_lowercase();
            match action.as_str() {
                "off" => {
                    let _ = crate::container::try_systemd_action("lazymc", "stop").await;
                    let _ = crate::container::try_systemd_action("minecraft", "stop").await;
                    let _ = self
                        .container
                        .stop_container(&self.config.podman_container)
                        .await;

                    Ok(GamePowerModeResult {
                        mode: "off".to_string(),
                        running: false,
                        proxy_active: false,
                        message: "Minecraft server powered off completely".to_string(),
                    })
                }
                "hibernate" => {
                    let _ = crate::container::try_systemd_action("lazymc", "restart").await;
                    Ok(GamePowerModeResult {
                        mode: "hibernate".to_string(),
                        running: false,
                        proxy_active: true,
                        message: "Minecraft server placed in hibernation mode".to_string(),
                    })
                }
                "on" => {
                    let _ = crate::container::try_systemd_action("lazymc", "restart").await;
                    let _ = self
                        .container
                        .start_container(&self.config.podman_container)
                        .await;

                    Ok(GamePowerModeResult {
                        mode: "on".to_string(),
                        running: true,
                        proxy_active: true,
                        message: "Minecraft server started and active".to_string(),
                    })
                }
                _ => Err(AppError::BadRequest(
                    "Invalid power mode. Allowed: off, on, hibernate".into(),
                )),
            }
        })
    }
}

fn parse_minecraft_player_list(output: &str) -> (Option<u32>, Option<u32>) {
    if output.is_empty() {
        return (None, None);
    }

    if let Some(idx) = output.find("of a max of") {
        let before = &output[..idx];
        let after = &output[idx + "of a max of".len()..];

        let online = before
            .split_whitespace()
            .last()
            .and_then(|s| s.parse::<u32>().ok());

        let max = after
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<u32>().ok());

        if online.is_some() || max.is_some() {
            return (online, max);
        }
    }

    if let Some(slash_idx) = output.find('/') {
        let before = &output[..slash_idx];
        let after = &output[slash_idx + 1..];

        let online = before
            .split_whitespace()
            .last()
            .and_then(|s| s.parse::<u32>().ok());

        let max = after
            .split_whitespace()
            .next()
            .and_then(|s| s.trim_matches(|c: char| !c.is_numeric()).parse::<u32>().ok());

        if online.is_some() && max.is_some() {
            return (online, max);
        }
    }

    (None, None)
}

fn parse_minecraft_tps(output: &str) -> Option<f32> {
    if output.is_empty() {
        return None;
    }

    if let Some(idx) = output.find("1m, 5m, 15m:") {
        let rest = &output[idx + "1m, 5m, 15m:".len()..];
        if let Some(first_tps) = rest.split(',').next() {
            if let Ok(tps) = first_tps.trim().parse::<f32>() {
                return Some(tps.clamp(0.0, 20.0));
            }
        }
    }

    if let Some(idx) = output.find("TPS:") {
        let rest = &output[idx + 4..];
        let num_str: String = rest
            .trim_start()
            .chars()
            .take_while(|c| c.is_numeric() || *c == '.')
            .collect();
        if let Ok(tps) = num_str.parse::<f32>() {
            return Some(tps.clamp(0.0, 20.0));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minecraft_player_lists() {
        let (online, max) =
            parse_minecraft_player_list("There are 5 of a max of 20 players online: p1, p2");
        assert_eq!(online, Some(5));
        assert_eq!(max, Some(20));

        let (online_slash, max_slash) = parse_minecraft_player_list("There are 2/10 players");
        assert_eq!(online_slash, Some(2));
        assert_eq!(max_slash, Some(10));
    }

    #[test]
    fn parses_minecraft_tps_outputs() {
        assert_eq!(
            parse_minecraft_tps("TPS from last 1m, 5m, 15m: 19.8, 20.0, 20.0"),
            Some(19.8)
        );
        assert_eq!(parse_minecraft_tps("TPS: 20.0"), Some(20.0));
        assert_eq!(parse_minecraft_tps(""), None);
    }
}
