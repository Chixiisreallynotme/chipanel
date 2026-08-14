use std::{env, path::PathBuf, sync::Arc};
use tracing::{info, warn};

use crate::{
    auth::password::hash_password,
    curseforge::client::CurseForgeClient,
    error::AppError,
    modrinth::client::ModrinthClient,
};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub admin_username: String,
    pub admin_password_hash: String,
    pub data_dir: PathBuf,
    pub port: u16,
    pub host: String,
    pub allowed_origins: Vec<String>,
    pub rcon_host: String,
    pub rcon_port: u16,
    pub rcon_password: String,
    pub podman_container: String,
    pub minecraft_data_dir: PathBuf,
    pub systemd_config_dir: PathBuf,
    pub lazymc_config_file: PathBuf,
    pub modrinth_client: Arc<ModrinthClient>,
    pub curseforge_client: Arc<CurseForgeClient>,
}

impl AppConfig {
    pub fn load() -> Result<Self, AppError> {
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            info!("JWT_SECRET environment variable not found; generating random 32-byte secret");
            use argon2::password_hash::rand_core::{OsRng, RngCore};
            let mut key = [0u8; 32];
            OsRng.fill_bytes(&mut key);
            hex::encode(key)
        });

        let admin_username = env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());

        let admin_password_hash = if let Ok(hash) = env::var("ADMIN_PASSWORD_HASH") {
            hash
        } else if let Ok(password) = env::var("ADMIN_PASSWORD") {
            hash_password(&password)?
        } else {
            use argon2::password_hash::rand_core::{OsRng, RngCore};
            let mut random_bytes = [0u8; 16];
            OsRng.fill_bytes(&mut random_bytes);
            let generated_password = hex::encode(random_bytes);

            warn!("============================================================");
            warn!("SECURITY WARNING: Neither ADMIN_PASSWORD nor ADMIN_PASSWORD_HASH was set.");
            warn!("A secure random default admin password has been generated for initial startup:");
            warn!("  Username: {}", admin_username);
            warn!("  Password: {}", generated_password);
            warn!("Please set ADMIN_PASSWORD or ADMIN_PASSWORD_HASH in environment variables.");
            warn!("============================================================");

            hash_password(&generated_password)?
        };

        let data_dir = env::var("DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/app/data"));

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let port: u16 = match env::var("PORT") {
            Ok(val) => val
                .parse()
                .map_err(|e| AppError::InternalError(format!("Invalid PORT configuration: {}", e)))?,
            Err(_) => 3000,
        };

        let allowed_origins_env = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://127.0.0.1:3000,http://localhost:3000".to_string());

        let allowed_origins: Vec<String> = allowed_origins_env
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let rcon_host = env::var("RCON_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let rcon_port: u16 = match env::var("RCON_PORT") {
            Ok(val) => val
                .parse()
                .map_err(|e| AppError::InternalError(format!("Invalid RCON_PORT configuration: {}", e)))?,
            Err(_) => 25575,
        };

        let rcon_password = env::var("RCON_PASSWORD").unwrap_or_default();

        let podman_container = env::var("PODMAN_CONTAINER_NAME")
            .or_else(|_| env::var("MINECRAFT_CONTAINER_NAME"))
            .unwrap_or_else(|_| "minecraft-server".to_string());

        let minecraft_data_dir = env::var("MINECRAFT_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/app/minecraft-data"));

        let systemd_config_dir = env::var("SYSTEMD_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/app/systemd-config"));

        let lazymc_config_file = env::var("LAZYMC_CONFIG_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/app/lazymc-config/lazymc.toml"));

        let modrinth_client = Arc::new(ModrinthClient::new());
        let curseforge_client = Arc::new(CurseForgeClient::new());

        Ok(AppConfig {
            jwt_secret,
            admin_username,
            admin_password_hash,
            data_dir,
            port,
            host,
            allowed_origins,
            rcon_host,
            rcon_port,
            rcon_password,
            podman_container,
            minecraft_data_dir,
            systemd_config_dir,
            lazymc_config_file,
            modrinth_client,
            curseforge_client,
        })
    }
}

