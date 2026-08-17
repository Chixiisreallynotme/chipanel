use std::{env, path::PathBuf, str::FromStr, sync::Arc};
use tracing::{info, warn};

use crate::{
    auth::password::hash_password,
    container::ContainerEngineType,
    curseforge::client::CurseForgeClient,
    error::AppError,
    modrinth::client::ModrinthClient,
};

#[derive(Clone)]
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
    pub container_engine: ContainerEngineType,
    pub container_socket_path: Option<PathBuf>,
    pub game_driver: String,
    pub modrinth_client: Arc<ModrinthClient>,
    pub curseforge_client: Arc<CurseForgeClient>,
}

impl std::fmt::Debug for AppConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppConfig")
            .field("jwt_secret", &"[REDACTED]")
            .field("admin_username", &self.admin_username)
            .field("admin_password_hash", &"[REDACTED]")
            .field("data_dir", &self.data_dir)
            .field("port", &self.port)
            .field("host", &self.host)
            .field("allowed_origins", &self.allowed_origins)
            .field("rcon_host", &self.rcon_host)
            .field("rcon_port", &self.rcon_port)
            .field("rcon_password", &"[REDACTED]")
            .field("podman_container", &self.podman_container)
            .field("minecraft_data_dir", &self.minecraft_data_dir)
            .field("systemd_config_dir", &self.systemd_config_dir)
            .field("lazymc_config_file", &self.lazymc_config_file)
            .field("container_engine", &self.container_engine)
            .field("container_socket_path", &self.container_socket_path)
            .field("game_driver", &self.game_driver)
            .finish()
    }
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
            Err(_) => 25500,
        };

        let allowed_origins_env = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://127.0.0.1:25500,http://localhost:25500".to_string());

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

        let podman_container = env::var("CONTAINER_NAME")
            .or_else(|_| env::var("PODMAN_CONTAINER_NAME"))
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

        let container_engine = env::var("CONTAINER_ENGINE")
            .ok()
            .and_then(|s| ContainerEngineType::from_str(&s).ok())
            .unwrap_or(ContainerEngineType::Auto);

        let container_socket_path = env::var("CONTAINER_SOCKET")
            .or_else(|_| env::var("PODMAN_SOCKET"))
            .or_else(|_| env::var("DOCKER_SOCKET"))
            .ok()
            .map(PathBuf::from);

        let game_driver = env::var("GAME_DRIVER")
            .or_else(|_| env::var("GAME_ENGINE"))
            .unwrap_or_else(|_| "minecraft".to_string());

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
            container_engine,
            container_socket_path,
            game_driver,
            modrinth_client,
            curseforge_client,
        })
    }
}
