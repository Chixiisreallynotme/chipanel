use axum::{
    response::Json,
    routing::{get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

use crate::{
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
    rcon::RconActorHandle,
};

pub fn geyser_router() -> Router {
    Router::new()
        .route("/status", get(get_geyser_status_handler))
        .route("/setup", post(setup_geyser_handler))
}

#[derive(Serialize)]
pub struct GeyserStatusResponse {
    pub is_geyser_installed: bool,
    pub is_floodgate_installed: bool,
    pub geyser_jar_name: Option<String>,
    pub floodgate_jar_name: Option<String>,
    pub bedrock_port: u16,
    pub auth_type: String,
    pub has_encryption_key: bool,
    pub is_running: bool,
    pub recommended_engine: String,
}

/// Detects Geyser & Floodgate plugins/mods, config, and encryption keys.
pub async fn get_geyser_status_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Extension(rcon): Extension<RconActorHandle>,
) -> Result<Json<GeyserStatusResponse>, AppError> {
    let data_dir = PathBuf::from(&config.minecraft_data_dir);
    let plugins_dir = data_dir.join("plugins");
    let mods_dir = data_dir.join("mods");

    let mut geyser_jar = None;
    let mut floodgate_jar = None;

    // Scan plugins and mods folders
    for dir in [&plugins_dir, &mods_dir] {
        if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("geyser") && name.ends_with(".jar") {
                    geyser_jar = Some(entry.file_name().to_string_lossy().to_string());
                }
                if name.contains("floodgate") && name.ends_with(".jar") {
                    floodgate_jar = Some(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
    }

    // Check Geyser config
    let geyser_config_path = plugins_dir.join("Geyser-Spigot").join("config.yml");
    let geyser_fabric_config = data_dir.join("config").join("Geyser-Fabric").join("config.yml");
    let target_config = if geyser_config_path.exists() {
        Some(geyser_config_path)
    } else if geyser_fabric_config.exists() {
        Some(geyser_fabric_config)
    } else {
        None
    };

    let mut bedrock_port = 19132;
    let mut auth_type = "floodgate".to_string();

    if let Some(cfg_path) = target_config {
        if let Ok(content) = tokio::fs::read_to_string(cfg_path).await {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("port:") {
                    if let Some(val) = trimmed.split(':').nth(1) {
                        if let Ok(p) = val.trim().parse::<u16>() {
                            bedrock_port = p;
                        }
                    }
                }
                if trimmed.starts_with("auth-type:") {
                    if let Some(val) = trimmed.split(':').nth(1) {
                        auth_type = val.trim().trim_matches('"').trim_matches('\'').to_string();
                    }
                }
            }
        }
    }

    // Check Floodgate encryption key
    let key_paper = plugins_dir.join("Floodgate").join("key.pem");
    let key_fabric = data_dir.join("config").join("floodgate").join("key.pem");
    let has_encryption_key = key_paper.exists() || key_fabric.exists();

    // Check if Geyser responds to RCON command
    let is_running = match rcon.exec("geyser version").await {
        Ok(out) => !out.to_lowercase().contains("unknown command") && !out.is_empty(),
        Err(_) => false,
    };

    Ok(Json(GeyserStatusResponse {
        is_geyser_installed: geyser_jar.is_some(),
        is_floodgate_installed: floodgate_jar.is_some(),
        geyser_jar_name: geyser_jar,
        floodgate_jar_name: floodgate_jar,
        bedrock_port,
        auth_type,
        has_encryption_key,
        is_running,
        recommended_engine: "Paper / Purpur (Recommandé pour Geyser)".to_string(),
    }))
}

#[derive(Deserialize)]
pub struct GeyserSetupRequest {
    pub bedrock_port: Option<u16>,
    pub auth_type: Option<String>, // "floodgate" | "online" | "offline"
}

#[derive(Serialize)]
pub struct GeyserSetupResponse {
    pub success: bool,
    pub message: String,
    pub bedrock_port: u16,
    pub auth_type: String,
    pub instructions: Vec<String>,
}

/// Generates or updates Geyser configuration for seamless Bedrock cross-play.
pub async fn setup_geyser_handler(
    _admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<GeyserSetupRequest>,
) -> Result<Json<GeyserSetupResponse>, AppError> {
    let port = payload.bedrock_port.unwrap_or(19132);
    let auth = payload.auth_type.unwrap_or_else(|| "floodgate".to_string());

    let data_dir = PathBuf::from(&config.minecraft_data_dir);
    let plugins_geyser_dir = data_dir.join("plugins").join("Geyser-Spigot");

    if !plugins_geyser_dir.exists() {
        tokio::fs::create_dir_all(&plugins_geyser_dir)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create Geyser directory: {}", e)))?;
    }

    let config_file = plugins_geyser_dir.join("config.yml");

    // Standard optimized Geyser config for Bedrock cross-play behind lazymc / Host-007
    let default_config = format!(
        r#"# Geyser Configuration generated by ChiPanel
bedrock:
  address: 0.0.0.0
  port: {port}
  clone-remote-port: false
  motd1: "ChiServ Minecraft Server"
  motd2: "Cross-Play Bedrock & Java"
  server-name: "ChiServ Bedrock"
  max-players: 20

remote:
  address: 127.0.0.1
  port: 25566
  auth-type: {auth}
  use-proxy-protocol: false

passthrough-motd: true
passthrough-player-counts: true
allow-third-party-capes: true
show-cooldown: title
"#
    );

    tokio::fs::write(&config_file, default_config)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to write Geyser config: {}", e)))?;

    info!("Geyser configuration initialized on UDP port {} with auth-type {}", port, auth);

    let instructions = vec![
        format!("1. Assurez-vous que le plugin Geyser-Spigot.jar est placé dans le dossier 'plugins/'."),
        format!("2. Si vous utilisez Floodgate (authentification sans compte Java), ajoutez floodgate-spigot.jar dans 'plugins/'."),
        format!("3. Le port UDP {} doit être autorisé sur votre pare-feu / box Internet pour les joueurs mobiles/consoles.", port),
        format!("4. Redémarrez le serveur Minecraft depuis ChiPanel pour charger les plugins."),
    ];

    Ok(Json(GeyserSetupResponse {
        success: true,
        message: "Configuration Geyser générée avec succès.".to_string(),
        bedrock_port: port,
        auth_type: auth,
        instructions,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_geyser_port() {
        let req = GeyserSetupRequest {
            bedrock_port: None,
            auth_type: None,
        };
        assert_eq!(req.bedrock_port.unwrap_or(19132), 19132);
        assert_eq!(req.auth_type.unwrap_or_else(|| "floodgate".to_string()), "floodgate");
    }
}
