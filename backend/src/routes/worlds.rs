use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path},
    http::header,
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, sync::LazyLock, time::Duration};
use tokio::sync::Mutex;
use tracing::info;

use crate::{
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
    minecraft::{
        server_properties::set_properties,
        version_meta::version_meta,
        worlds::{
            create_backup, delete_backup, delete_world_dir, export_world_to_temp,
            extract_zip_world, get_active_world_name, get_world_detail, is_valid_backup_filename,
            is_valid_world_name, list_backups, parse_chunky_status, restore_backup,
            sanitize_backup_filename, sanitize_name, scan_worlds, validate_difficulty,
            validate_gamemode, validate_gamerule_rule, validate_gamerule_value,
            validate_level_type, ChunkyStatus, WorldBackupInfo, WorldBorderInfo, WorldInfo,
        },
    },
    models::podman::ServerStatus,
    podman::PodmanClient,
    rcon::RconClient,
    routes::server::{current_quadlet_version, restart_lazymc_if_up},
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

#[derive(Debug, Serialize)]
pub struct WorldListResponse {
    pub worlds: Vec<WorldInfo>,
    pub active_world: Option<String>,
    /// Data version of the currently-configured server version (from the quadlet),
    /// used by the UI to badge each world as compatible/incompatible.
    pub server_data_version: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SwitchWorldRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorldRequest {
    pub name: String,
    pub seed: Option<String>,
    pub level_type: Option<String>,
    pub gamemode: Option<String>,
    pub difficulty: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImportBackupRequest {
    pub filename: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigureWorldRequest {
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct GameruleEntry {
    pub rule: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct SpawnPoint {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Deserialize)]
pub struct GameruleRequest {
    pub gamerules: Vec<GameruleEntry>,
    pub set_spawn: Option<SpawnPoint>,
}

#[derive(Debug, Serialize)]
pub struct WorldMutationResponse {
    pub success: bool,
    pub message: String,
    /// Whether the server (lazymc) was restarted to apply the change, or deferred.
    pub restarted: bool,
    /// Non-blocking advisory (e.g. data-version mismatch).
    pub warning: Option<String>,
    /// For delete: the filename of the auto-created safety backup.
    pub backup_filename: Option<String>,
}

/// Max accepted upload size for world import (512 MB). Larger worlds should be
/// imported from an existing server-side backup instead.
const MAX_WORLD_ZIP_BYTES: usize = 512 * 1024 * 1024;

/// Serializes world mutations (switch/create/delete/import/configure) so a second
/// concurrent request fails fast (409) instead of racing on `server.properties`.
static WORLD_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

async fn acquire_world_lock() -> Result<tokio::sync::MutexGuard<'static, ()>, AppError> {
    WORLD_MUTEX
        .try_lock()
        .map_err(|_| AppError::Conflict("Another world operation is already in progress".into()))
}

/// Serializes a gamerule value into the RCON `gamerule <rule> <value>` argument.
fn gamerule_value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        _ => String::new(),
    }
}

/// Compares a world's data version against the current quadlet `VERSION`, returning
/// a French advisory when the two are known and differ (downgrade risk).
async fn world_version_warning(config: &AppConfig, name: &str) -> Option<String> {
    let world_dv = crate::minecraft::worlds::read_world_data_version(
        &config.minecraft_data_dir.join(name).join("level.dat"),
    )?;
    let current_version = current_quadlet_version(&config.systemd_config_dir).await?;
    let target_dv = version_meta(&current_version).await?.data_version;
    if world_dv > 0 && target_dv > 0 && world_dv != target_dv {
        Some(format!(
            "Le monde a été créé en data version {} alors que {} attend {} — un downgrade peut corrompre le monde.",
            world_dv, current_version, target_dv
        ))
    } else {
        None
    }
}

/// Polls the Minecraft container until it has fully stopped (so a graceful world
/// save finishes before the folder is removed), or the timeout elapses.
async fn wait_for_server_stopped(config: &AppConfig, timeout: Duration) -> Result<(), AppError> {
    let client = PodmanClient::default();
    let start = std::time::Instant::now();
    loop {
        match client.inspect_container(&config.podman_container).await {
            Ok(status) if status.status == ServerStatus::Stopped => return Ok(()),
            Ok(_) => {}
            Err(_) => return Ok(()), // container gone → stopped
        }
        if start.elapsed() > timeout {
            return Err(AppError::InternalError(
                "Timed out waiting for the server to stop".into(),
            ));
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

pub fn worlds_router() -> Router {
    Router::new()
        .route("/", get(list_worlds_handler))
        .route("/switch", post(switch_world_handler))
        .route("/create", post(create_world_handler))
        .route(
            "/import",
            post(import_world_upload_handler).layer(DefaultBodyLimit::max(MAX_WORLD_ZIP_BYTES)),
        )
        .route("/import/backup", post(import_world_backup_handler))
        .route("/configure", post(configure_world_handler))
        .route("/gamerules", post(gamerules_handler))
        .route("/chunky/status", get(chunky_status_handler))
        .route("/chunky/control", post(chunky_control_handler))
        .route("/backups", get(list_backups_handler))
        .route("/backups/create", post(create_backup_handler))
        .route("/backups/restore", post(restore_backup_handler))
        .route("/backups/:filename", delete(delete_backup_handler))
        .route("/:name", get(get_world_detail_handler).delete(delete_world_handler))
        .route("/:name/download", get(download_world_handler))
        .route("/:name/worldborder", post(update_worldborder_handler))
}

/// GET /api/worlds
/// Returns the list of detected worlds plus the active world name (from
/// server.properties `level-name`).
pub async fn list_worlds_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<WorldListResponse>, AppError> {
    let worlds = scan_worlds(&config.minecraft_data_dir).await?;
    let active_world = get_active_world_name(&config.minecraft_data_dir).await;
    let server_data_version = match current_quadlet_version(&config.systemd_config_dir).await {
        Some(version) => version_meta(&version).await.map(|m| m.data_version),
        None => None,
    };
    Ok(Json(WorldListResponse {
        worlds,
        active_world,
        server_data_version,
    }))
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

/// POST /api/worlds/switch
/// Instant pointer change: sets `level-name` and restarts the Minecraft container
/// directly in the background (the systemd restart is queued, not awaited), so the
/// new world loads/generates asynchronously. The target world does NOT need to
/// exist yet — a pending world is generated when the server starts on it.
pub async fn switch_world_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<SwitchWorldRequest>,
) -> Result<Json<WorldMutationResponse>, AppError> {
    let _lock = acquire_world_lock().await?;
    let name = payload.name.trim().to_string();
    if !is_valid_world_name(&name) {
        return Err(AppError::BadRequest("Invalid world name".into()));
    }
    sanitize_name(&name)?;

    let active = get_active_world_name(&config.minecraft_data_dir).await;
    if active.as_deref() == Some(name.as_str()) {
        return Ok(Json(WorldMutationResponse {
            success: true,
            message: format!("World '{}' is already active", name),
            restarted: false,
            warning: None,
            backup_filename: None,
        }));
    }

    let props_path = config.minecraft_data_dir.join("server.properties");
    set_properties(&props_path, &[("level-name".to_string(), name.clone())]).await?;

    // Restart the container directly (bypassing lazymc's wake wrapper, which blocks
    // a player-triggered start when the world is missing). This queues the restart
    // and returns immediately — the new world generates/loads in the background.
    crate::podman::try_systemd_action("minecraft", "restart").await?;

    // Only meaningful for an already-generated world; a pending world has no
    // level.dat yet and simply generates against the current server version.
    let warning = world_version_warning(&config, &name).await;

    Ok(Json(WorldMutationResponse {
        success: true,
        message: format!("Active world switched to '{}'", name),
        restarted: true,
        warning,
        backup_filename: None,
    }))
}

/// POST /api/worlds/create
/// Points `level-name` at a new folder (plus seed/generator/gamemode/difficulty) and
/// restarts lazymc (when up) so the server generates the world.
pub async fn create_world_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<CreateWorldRequest>,
) -> Result<Json<WorldMutationResponse>, AppError> {
    let _lock = acquire_world_lock().await?;
    let name = payload.name.trim().to_string();
    if !is_valid_world_name(&name) {
        return Err(AppError::BadRequest("Invalid world name".into()));
    }
    sanitize_name(&name)?;

    if config.minecraft_data_dir.join(&name).exists() {
        return Err(AppError::Conflict(format!(
            "World '{}' already exists",
            name
        )));
    }

    let mut updates: Vec<(String, String)> = vec![("level-name".to_string(), name.clone())];

    // level-type and level-seed only apply at world generation, and are global in
    // server.properties — a previous flat/amplified world would otherwise leak into
    // this new one. Always write them explicitly (defaulting when unspecified).
    let level_type = payload
        .level_type
        .as_deref()
        .map(str::trim)
        .map(str::to_lowercase)
        .unwrap_or_else(|| "default".to_string());
    if !validate_level_type(&level_type) {
        return Err(AppError::BadRequest(format!(
            "Invalid level_type '{}'",
            level_type
        )));
    }
    updates.push(("level-type".to_string(), level_type));

    let seed = payload
        .seed
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    updates.push(("level-seed".to_string(), seed));

    if let Some(gm) = &payload.gamemode {
        let gm = gm.trim().to_lowercase();
        if !validate_gamemode(&gm) {
            return Err(AppError::BadRequest(format!("Invalid gamemode '{}'", gm)));
        }
        updates.push(("gamemode".to_string(), gm));
    }
    if let Some(diff) = &payload.difficulty {
        let diff = diff.trim().to_lowercase();
        if !validate_difficulty(&diff) {
            return Err(AppError::BadRequest(format!("Invalid difficulty '{}'", diff)));
        }
        updates.push(("difficulty".to_string(), diff));
    }

    let props_path = config.minecraft_data_dir.join("server.properties");
    set_properties(&props_path, &updates).await?;

    // Generate the world immediately by restarting the Minecraft container
    // directly. This bypasses lazymc's wake wrapper, which deliberately refuses to
    // start a server whose world is missing (see minecraft-wake.sh) so that a
    // player connecting can never auto-generate a world.
    crate::podman::try_systemd_action("minecraft", "restart").await?;

    Ok(Json(WorldMutationResponse {
        success: true,
        message: format!("World '{}' created and generated", name),
        restarted: true,
        warning: None,
        backup_filename: None,
    }))
}

/// DELETE /api/worlds/:name
/// Auto-backups then deletes a world folder. Deleting the *active* world is allowed:
/// the server is stopped first (so files aren't removed out from under a running
/// Minecraft process) and a warning is returned telling the operator to create or
/// switch to another world before players can join.
pub async fn delete_world_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(name): Path<String>,
) -> Result<Json<WorldMutationResponse>, AppError> {
    let _lock = acquire_world_lock().await?;
    if !is_valid_world_name(&name) {
        return Err(AppError::BadRequest("Invalid world name".into()));
    }
    sanitize_name(&name)?;

    let active = get_active_world_name(&config.minecraft_data_dir).await;
    let is_active = active.as_deref() == Some(name.as_str());

    // Stop the server before removing the active world's files, and wait for it to
    // finish its graceful save so it cannot re-create the folder mid-delete.
    if is_active {
        crate::podman::try_systemd_action("minecraft", "stop").await?;
        wait_for_server_stopped(&config, Duration::from_secs(30)).await?;
    }

    let backups_dir = config.data_dir.join("backups");
    let backup = create_backup(&config.minecraft_data_dir, &backups_dir, &name).await?;

    delete_world_dir(&config.minecraft_data_dir, &name).await?;

    let warning = if is_active {
        Some(format!(
            "Vous avez supprimé le monde actif '{}' — le serveur n'a plus de monde à charger. Créez ou changez de monde avant qu'un joueur puisse se connecter.",
            name
        ))
    } else {
        None
    };

    Ok(Json(WorldMutationResponse {
        success: true,
        message: format!("World '{}' deleted (backup '{}' kept)", name, backup.filename),
        restarted: false,
        warning,
        backup_filename: Some(backup.filename),
    }))
}

/// POST /api/worlds/import (multipart: `file` = .zip, optional `name` = target folder)
pub async fn import_world_upload_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    mut multipart: Multipart,
) -> Result<Json<WorldMutationResponse>, AppError> {
    let _lock = acquire_world_lock().await?;

    let tmp_dir = config.minecraft_data_dir.join("tmp");
    tokio::fs::create_dir_all(&tmp_dir)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to create tmp dir: {}", e)))?;

    let mut target_name: Option<String> = None;
    let mut zip_path: Option<std::path::PathBuf> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name == "name" {
            target_name = Some(
                field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read name field: {}", e)))?,
            );
        } else if field_name == "file" {
            let filename = field.file_name().unwrap_or("world.zip").to_string();
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("Failed to read upload: {}", e)))?;
            if bytes.is_empty() {
                return Err(AppError::BadRequest("Empty upload".into()));
            }
            let tmp_path = tmp_dir.join(format!("upload_{}.zip", std::process::id()));
            tokio::fs::write(&tmp_path, &bytes)
                .await
                .map_err(|e| AppError::InternalError(format!("Failed to write upload temp file: {}", e)))?;
            zip_path = Some(tmp_path);
            if target_name.is_none() {
                target_name = filename
                    .strip_suffix(".zip")
                    .map(str::to_string)
                    .filter(|s| !s.is_empty());
            }
        }
    }

    let target_name = target_name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .ok_or_else(|| AppError::BadRequest("Missing world name".into()))?;
    if !is_valid_world_name(&target_name) {
        return Err(AppError::BadRequest("Invalid world name".into()));
    }
    sanitize_name(&target_name)?;

    let zip_path = zip_path.ok_or_else(|| AppError::BadRequest("Missing 'file' field".into()))?;

    let result = extract_zip_world(&zip_path, &config.minecraft_data_dir, &target_name).await;
    let _ = tokio::fs::remove_file(&zip_path).await;
    result?;

    Ok(Json(WorldMutationResponse {
        success: true,
        message: format!("World '{}' imported", target_name),
        restarted: false,
        warning: world_version_warning(&config, &target_name).await,
        backup_filename: None,
    }))
}

/// POST /api/worlds/import/backup
/// Extracts an existing backup zip as a new world folder.
pub async fn import_world_backup_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<ImportBackupRequest>,
) -> Result<Json<WorldMutationResponse>, AppError> {
    let _lock = acquire_world_lock().await?;
    if !is_valid_backup_filename(&payload.filename) {
        return Err(AppError::BadRequest("Invalid backup filename".into()));
    }
    sanitize_backup_filename(&payload.filename)?;

    let target_name = payload
        .name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            payload
                .filename
                .rsplit_once("_backup_")
                .map(|(p, _)| p.to_string())
                .or_else(|| payload.filename.strip_suffix(".zip").map(str::to_string))
                .unwrap_or_else(|| "imported-world".to_string())
        });

    if !is_valid_world_name(&target_name) {
        return Err(AppError::BadRequest("Invalid world name".into()));
    }
    sanitize_name(&target_name)?;

    let backups_dir = config.data_dir.join("backups");
    let zip_path = backups_dir.join(&payload.filename);
    if !zip_path.exists() {
        return Err(AppError::NotFound(format!(
            "Backup '{}' not found",
            payload.filename
        )));
    }

    extract_zip_world(&zip_path, &config.minecraft_data_dir, &target_name).await?;

    Ok(Json(WorldMutationResponse {
        success: true,
        message: format!("World '{}' imported from backup", target_name),
        restarted: false,
        warning: world_version_warning(&config, &target_name).await,
        backup_filename: None,
    }))
}

/// GET /api/worlds/:name/download
/// Streams a world as a .zip attachment.
pub async fn download_world_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(name): Path<String>,
) -> Result<Response, AppError> {
    if !is_valid_world_name(&name) {
        return Err(AppError::BadRequest("Invalid world name".into()));
    }
    sanitize_name(&name)?;

    let tmp_dir = config.data_dir.join("tmp");
    let zip_path = export_world_to_temp(&config.minecraft_data_dir, &name, &tmp_dir).await?;

    let file = tokio::fs::File::open(&zip_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to open export zip: {}", e)))?;
    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let safe_name = name.replace('"', "_");
    let headers = [
        (header::CONTENT_TYPE, "application/zip".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}.zip\"", safe_name),
        ),
    ];

    Ok((headers, body).into_response())
}

/// POST /api/worlds/configure
/// Whitelisted server.properties edits (gamemode, difficulty, pvp, spawn-protection,
/// max-players, view-distance, level-seed, level-type) + restart when lazymc is up.
pub async fn configure_world_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<ConfigureWorldRequest>,
) -> Result<Json<WorldMutationResponse>, AppError> {
    let _lock = acquire_world_lock().await?;

    if payload.properties.is_empty() {
        return Err(AppError::BadRequest("No properties provided".into()));
    }

    let mut updates: Vec<(String, String)> = Vec::new();
    for (key, raw) in &payload.properties {
        let value = raw.trim().to_string();
        match key.as_str() {
            "gamemode" => {
                let v = value.to_lowercase();
                if !validate_gamemode(&v) {
                    return Err(AppError::BadRequest(format!("Invalid gamemode '{}'", value)));
                }
                updates.push(("gamemode".to_string(), v));
            }
            "difficulty" => {
                let v = value.to_lowercase();
                if !validate_difficulty(&v) {
                    return Err(AppError::BadRequest(format!("Invalid difficulty '{}'", value)));
                }
                updates.push(("difficulty".to_string(), v));
            }
            "pvp" => {
                if value != "true" && value != "false" {
                    return Err(AppError::BadRequest("pvp must be true or false".into()));
                }
                updates.push(("pvp".to_string(), value));
            }
            "spawn-protection" => {
                let n: i64 = value
                    .parse()
                    .map_err(|_| AppError::BadRequest("spawn-protection must be an integer".into()))?;
                if !(0..=64).contains(&n) {
                    return Err(AppError::BadRequest("spawn-protection must be 0..64".into()));
                }
                updates.push(("spawn-protection".to_string(), value));
            }
            "max-players" => {
                let n: i64 = value
                    .parse()
                    .map_err(|_| AppError::BadRequest("max-players must be an integer".into()))?;
                if !(1..=1000).contains(&n) {
                    return Err(AppError::BadRequest("max-players must be 1..1000".into()));
                }
                updates.push(("max-players".to_string(), value));
            }
            "view-distance" => {
                let n: i64 = value
                    .parse()
                    .map_err(|_| AppError::BadRequest("view-distance must be an integer".into()))?;
                if !(3..=32).contains(&n) {
                    return Err(AppError::BadRequest("view-distance must be 3..32".into()));
                }
                updates.push(("view-distance".to_string(), value));
            }
            "level-seed" => {
                if value.is_empty() {
                    return Err(AppError::BadRequest("level-seed must not be empty".into()));
                }
                updates.push(("level-seed".to_string(), value));
            }
            "level-type" => {
                let v = value.to_lowercase();
                if !validate_level_type(&v) {
                    return Err(AppError::BadRequest(format!("Invalid level-type '{}'", value)));
                }
                updates.push(("level-type".to_string(), v));
            }
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Property '{}' is not configurable",
                    key
                )))
            }
        }
    }

    let props_path = config.minecraft_data_dir.join("server.properties");
    set_properties(&props_path, &updates).await?;

    let restarted = restart_lazymc_if_up(&config).await?;

    Ok(Json(WorldMutationResponse {
        success: true,
        message: format!("Configured {} property(s)", updates.len()),
        restarted,
        warning: None,
        backup_filename: None,
    }))
}

/// POST /api/worlds/gamerules
/// Live gamerules + world spawn via RCON (requires the server to be awake).
pub async fn gamerules_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<GameruleRequest>,
) -> Result<Json<WorldMutationResponse>, AppError> {
    let _lock = acquire_world_lock().await?;

    if payload.gamerules.is_empty() && payload.set_spawn.is_none() {
        return Err(AppError::BadRequest("No gamerules or spawn provided".into()));
    }

    for entry in &payload.gamerules {
        if !validate_gamerule_rule(&entry.rule) {
            return Err(AppError::BadRequest(format!(
                "Invalid gamerule name '{}'",
                entry.rule
            )));
        }
        if !validate_gamerule_value(&entry.value) {
            return Err(AppError::BadRequest(format!(
                "Invalid value for gamerule '{}'",
                entry.rule
            )));
        }
    }

    let mut rcon_client = RconClient::connect(
        &config.rcon_host,
        config.rcon_port,
        &config.rcon_password,
    )
    .await
    .map_err(|_| {
        AppError::PartialFailure("Gamerules require the server to be awake — turn it On first".into())
    })?;

    for entry in &payload.gamerules {
        let value_str = gamerule_value_to_string(&entry.value);
        let cmd = format!("gamerule {} {}", entry.rule, value_str);
        rcon_client
            .exec(&cmd)
            .await
            .map_err(|_| AppError::PartialFailure(format!("Failed to set gamerule '{}'", entry.rule)))?;
    }

    if let Some(spawn) = &payload.set_spawn {
        let cmd = format!("setworldspawn {} {} {}", spawn.x, spawn.y, spawn.z);
        rcon_client
            .exec(&cmd)
            .await
            .map_err(|_| AppError::PartialFailure("Failed to set world spawn".to_string()))?;
    }

    Ok(Json(WorldMutationResponse {
        success: true,
        message: format!("Applied {} gamerule(s)", payload.gamerules.len()),
        restarted: false,
        warning: None,
        backup_filename: None,
    }))
}
