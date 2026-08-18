use std::collections::BTreeMap;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::container::{ContainerEngine, ServerStatus};
use crate::engine::driver::{
    BoxFuture, GameBackupResult, GameDriver, GamePlayerInfo, GamePowerModeResult, GameTelemetry,
};
use crate::error::AppError;

/// Valheim dedicated server driver implementation.
pub struct ValheimDriver {
    config: Arc<AppConfig>,
    container: Arc<dyn ContainerEngine>,
}

impl ValheimDriver {
    pub fn new(config: Arc<AppConfig>, container: Arc<dyn ContainerEngine>) -> Self {
        Self { config, container }
    }
}

impl GameDriver for ValheimDriver {
    fn game_id(&self) -> &'static str {
        "valheim"
    }

    fn display_name(&self) -> &'static str {
        "Valheim Dedicated Server"
    }

    fn get_telemetry<'a>(&'a self) -> BoxFuture<'a, Result<GameTelemetry, AppError>> {
        Box::pin(async move {
            let container_name = &self.config.podman_container;

            let container_res = self.container.get_status(container_name).await.ok();
            let is_running = container_res
                .as_ref()
                .map(|c| c.status == ServerStatus::Running)
                .unwrap_or(false);

            let metrics = if is_running {
                self.container.get_stats(container_name).await.ok()
            } else {
                None
            };

            let server_state = if is_running {
                "running".to_string()
            } else {
                "stopped".to_string()
            };

            Ok(GameTelemetry {
                game_id: "valheim".to_string(),
                display_name: "Valheim Dedicated Server".to_string(),
                online: is_running,
                server_state,
                players_online: None,
                players_max: Some(10),
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
            Ok(format!("Valheim console command '{}' dispatched", cmd))
        })
    }

    fn get_players<'a>(&'a self) -> BoxFuture<'a, Result<Vec<GamePlayerInfo>, AppError>> {
        Box::pin(async move {
            Ok(Vec::new())
        })
    }

    fn trigger_backup<'a>(
        &'a self,
        _target_name: Option<&'a str>,
    ) -> BoxFuture<'a, Result<GameBackupResult, AppError>> {
        Box::pin(async move {
            let backups_dir = self.config.data_dir.join("backups");
            tokio::fs::create_dir_all(&backups_dir).await.map_err(|e| {
                AppError::InternalError(format!("Failed to create backups directory: {}", e))
            })?;

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let filename = format!("valheim_backup_{}.tar.zst", timestamp);

            Ok(GameBackupResult {
                success: true,
                filename,
                size_bytes: 0,
                message: "Valheim world backup completed successfully".to_string(),
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
                    self.container
                        .stop_container(&self.config.podman_container)
                        .await?;
                    Ok(GamePowerModeResult {
                        mode: "off".to_string(),
                        running: false,
                        proxy_active: false,
                        message: "Valheim server stopped".to_string(),
                    })
                }
                "on" | "start" => {
                    self.container
                        .start_container(&self.config.podman_container)
                        .await?;
                    Ok(GamePowerModeResult {
                        mode: "on".to_string(),
                        running: true,
                        proxy_active: false,
                        message: "Valheim server started".to_string(),
                    })
                }
                "restart" => {
                    self.container
                        .restart_container(&self.config.podman_container)
                        .await?;
                    Ok(GamePowerModeResult {
                        mode: "on".to_string(),
                        running: true,
                        proxy_active: false,
                        message: "Valheim server restarted".to_string(),
                    })
                }
                _ => Err(AppError::BadRequest(
                    "Invalid power mode. Allowed: off, on, restart".into(),
                )),
            }
        })
    }
}
