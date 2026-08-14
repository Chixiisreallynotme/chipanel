use axum::{
    extract::Path,
    response::Json,
    routing::{delete, get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::{
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
    minecraft::worlds::{
        create_backup, delete_backup, get_world_detail, is_valid_backup_filename,
        is_valid_world_name, list_backups, parse_chunky_status, restore_backup,
        sanitize_backup_filename, sanitize_name, scan_worlds, ChunkyStatus, WorldBackupInfo,
        WorldBorderInfo, WorldInfo,
    },
    rcon::RconClient,
};

#[derive(Debug, Deserialize)]
pub struct WorldBorderRequest {
    pub size: f64,
    pub center_x: Option<f64>,
    pub center_z: Option<f64>,
    pub time_seconds: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkyControlRequest {
    pub action: String,
    pub world: Option<String>,
    pub radius: Option<u32>,
    pub shape: Option<String>,
    pub center_x: Option<i32>,
    pub center_z: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBackupRequest {
    pub world_name: String,
}

#[derive(Debug, Deserialize)]
pub struct RestoreBackupRequest {
    pub filename: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionResponse {
    pub success: bool,
    pub message: String,
}

pub fn worlds_router() -> Router {
    Router::new()
        .route("/", get(list_worlds_handler))
        .route("/chunky/status", get(chunky_status_handler))
        .route("/chunky/control", post(chunky_control_handler))
        .route("/backups", get(list_backups_handler))
        .route("/backups/create", post(create_backup_handler))
        .route("/backups/restore", post(restore_backup_handler))
        .route("/backups/:filename", delete(delete_backup_handler))
        .route("/:name", get(get_world_detail_handler))
        .route("/:name/worldborder", post(update_worldborder_handler))
}

/// GET /api/worlds
/// Returns list of all detected worlds (Vec<WorldInfo>).
pub async fn list_worlds_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<Vec<WorldInfo>>, AppError> {
    let worlds = scan_worlds(&config.minecraft_data_dir).await?;
    Ok(Json(worlds))
}

/// GET /api/worlds/:name
/// Returns (WorldInfo, WorldBorderInfo) for specified world name.
pub async fn get_world_detail_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(name): Path<String>,
) -> Result<Json<(WorldInfo, WorldBorderInfo)>, AppError> {
    if !is_valid_world_name(&name) {
        return Err(AppError::BadRequest("Invalid world name format".into()));
    }
    sanitize_name(&name)?;
    let detail = get_world_detail(&config.minecraft_data_dir, &name).await?;
    Ok(Json(detail))
}

/// POST /api/worlds/:name/worldborder
/// Body { "size": f64, "center_x": Option<f64>, "center_z": Option<f64>, "time_seconds": Option<u32> }
/// Executes RCON `worldborder set <size> [time]` and `worldborder center <x> <z>`.
pub async fn update_worldborder_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(name): Path<String>,
    Json(payload): Json<WorldBorderRequest>,
) -> Result<Json<ActionResponse>, AppError> {
    if !is_valid_world_name(&name) {
        return Err(AppError::BadRequest("Invalid world name format".into()));
    }
    sanitize_name(&name)?;

    if payload.size < 1.0 || payload.size > 60000000.0 {
        return Err(AppError::BadRequest(
            "worldborder size must be between 1 and 60,000,000".into(),
        ));
    }

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let set_cmd = if let Some(time) = payload.time_seconds {
        format!("worldborder set {} {}", payload.size, time)
    } else {
        format!("worldborder set {}", payload.size)
    };

    info!("Executing worldborder set command via RCON: '{}'", set_cmd);
    rcon_client
        .exec(&set_cmd)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON set worldborder failed: {}", err)))?;

    if let (Some(cx), Some(cz)) = (payload.center_x, payload.center_z) {
        let center_cmd = format!("worldborder center {} {}", cx, cz);
        info!("Executing worldborder center command via RCON: '{}'", center_cmd);
        rcon_client
            .exec(&center_cmd)
            .await
            .map_err(|err| AppError::InternalError(format!("RCON center worldborder failed: {}", err)))?;
    }

    Ok(Json(ActionResponse {
        success: true,
        message: format!("Worldborder updated for world '{}'", name),
    }))
}

/// GET /api/worlds/chunky/status
/// Executes RCON `chunky progress` and returns ChunkyStatus.
pub async fn chunky_status_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<ChunkyStatus>, AppError> {
    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let output = rcon_client
        .exec("chunky progress")
        .await
        .map_err(|err| AppError::InternalError(format!("RCON command failed: {}", err)))?;

    let status = parse_chunky_status(&output);
    Ok(Json(status))
}

/// POST /api/worlds/chunky/control
/// Body { "action": "start" | "pause" | "cancel", "world": Option<String>, "radius": Option<u32>, "shape": Option<String>, "center_x": Option<i32>, "center_z": Option<i32> }
/// Executes Chunky RCON commands.
pub async fn chunky_control_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<ChunkyControlRequest>,
) -> Result<Json<ActionResponse>, AppError> {
    let action_clean = payload.action.trim().to_lowercase();
    if action_clean != "start" && action_clean != "pause" && action_clean != "cancel" {
        return Err(AppError::BadRequest(
            "Invalid chunky action. Allowed: start, pause, cancel".into(),
        ));
    }

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    if let Some(ref world) = payload.world {
        if world.contains('\n')
            || world.contains('\r')
            || world.contains('\0')
            || world.is_empty()
            || !world.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(AppError::BadRequest("Invalid chunky world name".into()));
        }
        if !is_valid_world_name(world) {
            return Err(AppError::BadRequest("Invalid chunky world name".into()));
        }
        sanitize_name(world)?;
        let cmd = format!("chunky world {}", world);
        rcon_client
            .exec(&cmd)
            .await
            .map_err(|err| AppError::InternalError(format!("RCON chunky world failed: {}", err)))?;
    }

    if let Some(ref shape) = payload.shape {
        let shape_clean = shape.trim().to_lowercase();
        if shape_clean.contains('\n')
            || shape_clean.contains('\r')
            || shape_clean.contains('\0')
            || shape_clean.contains(' ')
            || !shape_clean.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(AppError::BadRequest("Invalid chunky shape format".into()));
        }
        let cmd = format!("chunky shape {}", shape_clean);
        rcon_client
            .exec(&cmd)
            .await
            .map_err(|err| AppError::InternalError(format!("RCON chunky shape failed: {}", err)))?;
    }

    if let Some(radius) = payload.radius {
        let cmd = format!("chunky radius {}", radius);
        rcon_client
            .exec(&cmd)
            .await
            .map_err(|err| AppError::InternalError(format!("RCON chunky radius failed: {}", err)))?;
    }

    if let (Some(cx), Some(cz)) = (payload.center_x, payload.center_z) {
        let cmd = format!("chunky center {} {}", cx, cz);
        rcon_client
            .exec(&cmd)
            .await
            .map_err(|err| AppError::InternalError(format!("RCON chunky center failed: {}", err)))?;
    }

    let action_cmd = format!("chunky {}", action_clean);
    info!("Executing Chunky control command via RCON: '{}'", action_cmd);
    let output = rcon_client
        .exec(&action_cmd)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON chunky action failed: {}", err)))?;

    Ok(Json(ActionResponse {
        success: true,
        message: format!("Chunky action '{}' executed: {}", action_clean, output),
    }))
}

/// GET /api/worlds/backups
/// Returns list of .zip backups (Vec<WorldBackupInfo>).
pub async fn list_backups_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<Vec<WorldBackupInfo>>, AppError> {
    let backups_dir = config.data_dir.join("backups");
    let backups = list_backups(&backups_dir).await?;
    Ok(Json(backups))
}

/// POST /api/worlds/backups/create
/// Body { "world_name": String } creating zip backup.
pub async fn create_backup_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<CreateBackupRequest>,
) -> Result<Json<WorldBackupInfo>, AppError> {
    if !is_valid_world_name(&payload.world_name) {
        return Err(AppError::BadRequest("Invalid world name format".into()));
    }
    sanitize_name(&payload.world_name)?;

    let backups_dir = config.data_dir.join("backups");
    let info = create_backup(&config.minecraft_data_dir, &backups_dir, &payload.world_name).await?;
    Ok(Json(info))
}

/// POST /api/worlds/backups/restore
/// Body { "filename": String } restoring zip backup.
pub async fn restore_backup_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<RestoreBackupRequest>,
) -> Result<Json<ActionResponse>, AppError> {
    if !is_valid_backup_filename(&payload.filename) {
        return Err(AppError::BadRequest("Invalid backup filename".into()));
    }
    sanitize_backup_filename(&payload.filename)?;

    let backups_dir = config.data_dir.join("backups");
    restore_backup(&config.minecraft_data_dir, &backups_dir, &payload.filename).await?;

    Ok(Json(ActionResponse {
        success: true,
        message: format!("Backup '{}' restored successfully", payload.filename),
    }))
}

/// DELETE /api/worlds/backups/:filename
/// Deletes specified zip backup.
pub async fn delete_backup_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(filename): Path<String>,
) -> Result<Json<ActionResponse>, AppError> {
    if !is_valid_backup_filename(&filename) {
        return Err(AppError::BadRequest("Invalid backup filename".into()));
    }
    sanitize_backup_filename(&filename)?;

    let backups_dir = config.data_dir.join("backups");
    delete_backup(&backups_dir, &filename).await?;

    Ok(Json(ActionResponse {
        success: true,
        message: format!("Backup '{}' deleted successfully", filename),
    }))
}
