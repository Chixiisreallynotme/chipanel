use axum::{
    response::Json,
    routing::{get, post},
    Extension, Json as AxumJson, Router,
};
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};
use tracing::{info, warn};

use crate::{
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
    minecraft::version_meta::version_meta,
    minecraft::version_watch::{get_version_watch_info, VersionWatchInfo},
    minecraft::worlds::read_world_data_version,
    models::{
        podman::{ContainerMetrics, ContainerStatusResponse, ServerStatus},
        rcon::{RconCommandRequest, RconCommandResponse},
    },
    podman::PodmanClient,
    rcon::RconClient,
    routes::engine_catalog::{
        fetch_version_catalog, get_available_engines, resolve_minecraft_version, EngineTypeInfo,
    },
};

pub fn server_router() -> Router {
    Router::new()
        .route("/rcon", post(rcon_handler))
        .route("/status", get(status_handler))
        .route("/power", post(power_handler))
        .route("/mode", get(mode_status_handler).post(mode_handler))
        .route("/engine", get(get_engine_handler).post(update_engine_handler))
        .route("/resource-pack", get(get_resource_pack_handler))
        .route("/resource-pack/activate", post(activate_resource_pack_handler))
        .route("/resource-pack/disable", post(disable_resource_pack_handler))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStatusCombinedResponse {
    pub container: ContainerStatusResponse,
    pub container_name: String,
    pub metrics: Option<ContainerMetrics>,
    pub rcon_online: bool,
    /// Operator-selected power mode: "off" | "on" | "hibernate".
    pub power_mode: String,
    /// Whether something (lazymc or the server) is listening on the public 25565.
    pub lazymc_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct PowerActionRequest {
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PowerActionResponse {
    pub success: bool,
    pub message: String,
}

/// Handler for POST /api/server/rcon
/// Executes an RCON command against the server.
/// Admin-only: an arbitrary RCON command is `op <attacker>`, i.e. full server takeover.
pub async fn rcon_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    AxumJson(payload): AxumJson<RconCommandRequest>,
) -> Result<Json<RconCommandResponse>, AppError> {
    if payload.command.contains('\n') || payload.command.contains('\r') || payload.command.contains('\0') {
        return Err(AppError::BadRequest(
            "Invalid RCON command: control characters forbidden".into(),
        ));
    }

    info!("RCON command request: {}", payload.command);

    // RconClient already returns typed AppError variants (AuthError for a bad RCON
    // password, InternalError for genuine I/O failures) - re-wrapping turned every
    // one of them into a 500.
    let mut client =
        RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password).await?;

    let output = client.exec(&payload.command).await?;

    Ok(Json(RconCommandResponse {
        output,
        success: true,
    }))
}

/// Handler for GET /api/server/status
/// Returns container status, CPU/RAM metrics, and RCON online state concurrently using tokio::join!.
pub async fn status_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<ServerStatusCombinedResponse>, AppError> {
    let podman_client = PodmanClient::default();

    let container = match podman_client.inspect_container(&config.podman_container).await {
        Ok(status) => status,
        Err(_) => ContainerStatusResponse {
            status: ServerStatus::Stopped,
            uptime_seconds: 0,
            container_id: String::new(),
        },
    };

    let (metrics, rcon_online) = if container.status == ServerStatus::Running {
        let metrics_fut = podman_client.get_container_metrics(&config.podman_container);
        let rcon_fut = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password);
        let (metrics_res, rcon_res) = tokio::join!(metrics_fut, rcon_fut);
        (metrics_res.ok(), rcon_res.is_ok())
    } else {
        (None, false)
    };

    let power_mode = read_power_mode(&config.data_dir).await;
    let lazymc_active = probe_tcp(&config.rcon_host, 25565).await;

    Ok(Json(ServerStatusCombinedResponse {
        container,
        container_name: config.podman_container.clone(),
        metrics,
        rcon_online,
        power_mode,
        lazymc_active,
    }))
}

/// Handler for POST /api/server/power
/// Controls container power state (start, stop, restart).
pub async fn power_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    AxumJson(payload): AxumJson<PowerActionRequest>,
) -> Result<Json<PowerActionResponse>, AppError> {
    let container_name = &config.podman_container;
    let podman_client = PodmanClient::default();

    let action = payload.action.trim().to_lowercase();
    info!("Power action requested: '{}' for container '{}'", action, container_name);

    match action.as_str() {
        "start" => {
            podman_client.start_container(container_name).await?;
            Ok(Json(PowerActionResponse {
                success: true,
                message: format!("Container '{}' start initiated", container_name),
            }))
        }
        "stop" => {
            podman_client.stop_container(container_name).await?;
            Ok(Json(PowerActionResponse {
                success: true,
                message: format!("Container '{}' stop initiated", container_name),
            }))
        }
        "restart" => {
            podman_client.restart_container(container_name).await?;
            Ok(Json(PowerActionResponse {
                success: true,
                message: format!("Container '{}' restart initiated", container_name),
            }))
        }
        _ => Err(AppError::BadRequest(
            "Invalid power action. Allowed: start, stop, restart".into(),
        )),
    }
}

const POWER_MODE_FILE: &str = "power_mode.json";

/// Valid power modes. "off" = everything stopped (port 25565 closed); "on" = lazymc
/// front + server woken; "hibernate" = lazymc front, server asleep until a join.
const VALID_MODES: [&str; 3] = ["off", "on", "hibernate"];

#[derive(Debug, Deserialize)]
pub struct ModeActionRequest {
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModeResponse {
    pub mode: String,
    pub lazymc_active: bool,
    pub minecraft_running: bool,
}

/// Handler for GET /api/server/mode
/// Returns the persisted operator mode plus the live state of the two units.
pub async fn mode_status_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<ModeResponse>, AppError> {
    Ok(Json(build_mode_response(&config).await))
}

/// Shared snapshot of the live power state, used by both mode handlers.
async fn build_mode_response(config: &AppConfig) -> ModeResponse {
    let podman_client = PodmanClient::default();
    let minecraft_running = matches!(
        podman_client.inspect_container(&config.podman_container).await,
        Ok(status) if status.status == ServerStatus::Running
    );
    let lazymc_active = probe_tcp(&config.rcon_host, 25565).await;

    ModeResponse {
        mode: read_power_mode(&config.data_dir).await,
        lazymc_active,
        minecraft_running,
    }
}

/// Handler for POST /api/server/mode
/// Switches between the three power modes by driving the `lazymc` and `minecraft`
/// systemd units and toggling lazymc's `wake_on_start`.
pub async fn mode_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    AxumJson(payload): AxumJson<ModeActionRequest>,
) -> Result<Json<ModeResponse>, AppError> {
    let action = payload.action.trim().to_lowercase();
    if !VALID_MODES.contains(&action.as_str()) {
        return Err(AppError::BadRequest(
            "Invalid mode. Allowed: off, on, hibernate".into(),
        ));
    }

    info!("Mode change requested: '{}'", action);

    match action.as_str() {
        "off" => {
            // Close the public port and free the server entirely. Stop order matters
            // (lazymc first so it cannot re-wake the server mid-shutdown).
            crate::podman::try_systemd_action("lazymc", "stop").await?;
            crate::podman::try_systemd_action("minecraft", "stop").await?;
            write_power_mode(&config.data_dir, "off").await?;
        }
        "hibernate" => {
            set_lazymc_key(&config.lazymc_config_file, "server", "wake_on_start", "false").await?;
            // Restart re-reads lazymc.toml and re-enters the sleeping lobby.
            crate::podman::try_systemd_action("lazymc", "restart").await?;
            write_power_mode(&config.data_dir, "hibernate").await?;
        }
        "on" => {
            set_lazymc_key(&config.lazymc_config_file, "server", "wake_on_start", "true").await?;
            // Restart wakes the server immediately, then lazymc sleeps it again
            // after the idle timeout.
            crate::podman::try_systemd_action("lazymc", "restart").await?;
            write_power_mode(&config.data_dir, "on").await?;
        }
        _ => unreachable!(),
    }

    // Return a fresh, live snapshot of the resulting state.
    Ok(Json(build_mode_response(&config).await))
}

/// Reads the persisted operator mode, defaulting to "off" when unknown/missing.
async fn read_power_mode(data_dir: &Path) -> String {
    let path = data_dir.join(POWER_MODE_FILE);
    match tokio::fs::read_to_string(&path).await {
        Ok(data) => {
            let mode = data.trim().to_string();
            if VALID_MODES.contains(&mode.as_str()) {
                mode
            } else {
                "off".to_string()
            }
        }
        Err(_) => "off".to_string(),
    }
}

async fn write_power_mode(data_dir: &Path, mode: &str) -> Result<(), AppError> {
    let path = data_dir.join(POWER_MODE_FILE);
    tokio::fs::write(&path, format!("{}\n", mode))
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to write {:?}: {}", path, e)))
}

/// Rewrites a single `key = value` line in lazymc.toml, scoped to a `[section]`
/// (several keys exist in multiple sections — e.g. `version` under both `[public]`
/// and `[config]` — so a bare line match would corrupt the wrong one). The line is
/// replaced in place; if the key is absent from the section it is inserted right
/// after the section header.
async fn set_lazymc_key(
    path: &Path,
    section: &str,
    key: &str,
    value: &str,
) -> Result<(), AppError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read {:?}: {}", path, e)))?;

    let new_line = format!("{} = {}", key, value);
    let mut out = Vec::new();
    let mut current_section = String::new();
    let mut replaced = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].to_string();
            out.push(line.to_string());
            continue;
        }
        if current_section == section {
            if let Some(eq) = trimmed.find('=') {
                if trimmed[..eq].trim() == key {
                    let indent = &line[..line.len() - line.trim_start().len()];
                    out.push(format!("{}{}", indent, new_line));
                    replaced = true;
                    continue;
                }
            }
        }
        out.push(line.to_string());
    }

    if !replaced {
        let mut inserted = Vec::new();
        let mut done = false;
        let mut current_section = String::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len() - 1].to_string();
                inserted.push(line.to_string());
                if current_section == section {
                    inserted.push(new_line.clone());
                    done = true;
                }
                continue;
            }
            inserted.push(line.to_string());
        }
        if !done {
            inserted.push(format!("[{}]", section));
            inserted.push(new_line.clone());
        }
        out = inserted;
    }

    let new_content = out.join("\n") + "\n";
    tokio::fs::write(path, new_content)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to update {:?}: {}", path, e)))
}

/// Best-effort TCP reachability probe — returns whether a listener answered on `port`.
pub(crate) async fn probe_tcp(host: &str, port: u16) -> bool {
    tokio::time::timeout(
        std::time::Duration::from_millis(500),
        tokio::net::TcpStream::connect((host, port)),
    )
    .await
    .map(|r| r.is_ok())
    .unwrap_or(false)
}

/// Restarts lazymc only when it is currently serving (i.e. not "off" mode). Returns
/// `true` when a restart was issued, `false` when deferred (the change applies on the
/// next start). Shared by the engine switcher and the world lifecycle mutations.
pub(crate) async fn restart_lazymc_if_up(config: &AppConfig) -> Result<bool, AppError> {
    if probe_tcp(&config.rcon_host, 25565).await {
        crate::podman::try_systemd_action("lazymc", "restart").await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Reads `Environment=VERSION=` out of the minecraft.container quadlet (the engine
/// switcher's source of truth) for data-version compatibility checks.
pub(crate) async fn current_quadlet_version(systemd_config_dir: &Path) -> Option<String> {
    let container_file = systemd_config_dir.join("minecraft.container");
    let content = tokio::fs::read_to_string(&container_file).await.ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("Environment=VERSION=") {
            let v = val.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}


/// `current_type` / `current_version` are `None` when the real configuration could not
/// be read - the frontend must render "unknown", never a fabricated default.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerEngineResponse {
    pub current_type: Option<String>,
    pub current_version: Option<String>,
    /// "quadlet" | "state_file" | "unavailable"
    pub config_source: String,
    pub config_error: Option<String>,
    pub available_types: Vec<EngineTypeInfo>,
    /// LATEST/SNAPSHOT keywords + every release + every snapshot (flat list).
    pub all_versions: Vec<String>,
    /// Every official Mojang release, newest first.
    pub release_versions: Vec<String>,
    /// Every official Mojang snapshot, newest first. Empty when the live manifest
    /// could not be fetched (offline fallback only carries releases).
    pub snapshot_versions: Vec<String>,
    /// Background watcher state: latest known release/snapshot and any newly
    /// detected version since the persisted baseline.
    pub version_watch: VersionWatchInfo,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEngineRequest {
    pub engine_type: String,
    pub version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EngineUpdateResponse {
    pub success: bool,
    pub message: String,
    /// The concrete version actually written (LATEST/SNAPSHOT are resolved away).
    pub resolved_version: String,
    /// Non-blocking advisory (e.g. world data-version mismatch), surfaced by the UI.
    pub warning: Option<String>,
}

/// Handler for GET /api/server/engine
pub async fn get_engine_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<ServerEngineResponse>, AppError> {
    let container_file = config.systemd_config_dir.join("minecraft.container");

    let mut current_type = None;
    let mut current_version = None;
    let mut config_source = "unavailable";
    let mut config_error = None;

    if container_file.exists() {
        match tokio::fs::read_to_string(&container_file).await {
            Ok(content) => {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if let Some(val) = trimmed.strip_prefix("Environment=TYPE=") {
                        let t = val.trim();
                        if !t.is_empty() {
                            current_type = Some(t.to_uppercase());
                        }
                    } else if let Some(val) = trimmed.strip_prefix("Environment=VERSION=") {
                        let v = val.trim();
                        if !v.is_empty() {
                            current_version = Some(v.to_uppercase());
                        }
                    }
                }
                if current_type.is_some() || current_version.is_some() {
                    config_source = "quadlet";
                } else {
                    config_error =
                        Some("minecraft.container declares no TYPE/VERSION".to_string());
                }
            }
            Err(e) => {
                warn!("Failed to read {:?}: {}", container_file, e);
                config_error = Some("Could not read minecraft.container".to_string());
            }
        }
    } else {
        let state_file = config.data_dir.join("engine_config.json");
        match tokio::fs::read_to_string(&state_file).await {
            Ok(data) => match serde_json::from_str::<serde_json::Value>(&data) {
                Ok(json) => {
                    current_type = json.get("type").and_then(|v| v.as_str()).map(str::to_uppercase);
                    current_version =
                        json.get("version").and_then(|v| v.as_str()).map(str::to_uppercase);
                    if current_type.is_some() || current_version.is_some() {
                        config_source = "state_file";
                    } else {
                        config_error =
                            Some("engine_config.json declares no type/version".to_string());
                    }
                }
                Err(e) => {
                    warn!("Failed to parse {:?}: {}", state_file, e);
                    config_error = Some("Could not parse engine_config.json".to_string());
                }
            },
            Err(e) => {
                warn!("Failed to read {:?}: {}", state_file, e);
                config_error =
                    Some("Server engine configuration is not readable from ChiPanel".to_string());
            }
        }
    }

    let catalog = fetch_version_catalog().await;

    Ok(Json(ServerEngineResponse {
        current_type,
        current_version,
        config_source: config_source.to_string(),
        config_error,
        available_types: get_available_engines(),
        all_versions: catalog.all_versions(),
        release_versions: catalog.releases.clone(),
        snapshot_versions: catalog.snapshots.clone(),
        version_watch: get_version_watch_info().await,
    }))
}

/// Handler for POST /api/server/engine
///
/// This is the single source of truth for the Minecraft version: it updates
/// `minecraft.container` (TYPE/VERSION) AND `lazymc.toml` (public version +
/// protocol hint) together, so the hibernation proxy never drifts from the real
/// server. `LATEST`/`SNAPSHOT` are resolved to a concrete version at switch time
/// (the config is always pinned, never a moving keyword).
pub async fn update_engine_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    AxumJson(payload): AxumJson<UpdateEngineRequest>,
) -> Result<Json<EngineUpdateResponse>, AppError> {
    let req_type = payload.engine_type.trim().to_uppercase();
    let req_version_raw = payload.version.as_deref().unwrap_or("LATEST").trim();

    // Capture the current engine BEFORE any writes, so LuckPerms data can be backed
    // up from the OLD engine's location and restored into the NEW one below.
    let old_engine = crate::minecraft::tools::detect_engine(&config).await.engine;

    let valid_engines = get_available_engines();
    if !valid_engines.iter().any(|e| e.id == req_type) {
        return Err(AppError::BadRequest(format!("Unsupported engine type '{}'", req_type)));
    }

    // Snapshot ids are case-sensitive on Mojang's side ("24w14a"), so the version is
    // NOT uppercased like the engine type: validate case-insensitively, then write
    // back the canonical manifest id.
    let canonical = resolve_minecraft_version(req_version_raw)
        .await
        .ok_or_else(|| AppError::BadRequest(format!("Unsupported version '{}'", req_version_raw)))?;

    // LATEST/SNAPSHOT are convenience keywords, not concrete versions: resolve them
    // away so the on-disk config is always pinned to an exact version (a literal
    // VERSION=LATEST would auto-update on every container start).
    let catalog = fetch_version_catalog().await;
    let req_version = match canonical.as_str() {
        "LATEST" => catalog
            .releases
            .first()
            .cloned()
            .ok_or_else(|| AppError::BadRequest("No release available to resolve LATEST".into()))?,
        "SNAPSHOT" => catalog
            .snapshots
            .first()
            .cloned()
            .ok_or_else(|| AppError::BadRequest("No snapshot available to resolve SNAPSHOT (offline?)".into()))?,
        _ => canonical.clone(),
    };

    // Protocol + data version for the concrete version, and a world-compatibility
    // advisory (warn only — never block the switch).
    let meta = version_meta(&req_version).await;
    let protocol = meta.map(|m| m.protocol);

    let mut warnings = Vec::new();
    let level_dat = config.minecraft_data_dir.join("world").join("level.dat");
    if let Some(world_dv) = read_world_data_version(&level_dat) {
        if let Some(target_dv) = meta.map(|m| m.data_version) {
            if world_dv > 0 && target_dv > 0 && world_dv != target_dv {
                warnings.push(format!(
                    "Le monde a été créé en data version {} alors que {} attend {} — un downgrade peut corrompre le monde (sauvegarde conseillée).",
                    world_dv, req_version, target_dv
                ));
            }
        }
    }
    if protocol.is_none() {
        warnings.push(format!(
            "Protocole réseau introuvable pour {} : le hint du proxy d'hibernation n'a pas été mis à jour.",
            req_version
        ));
    }

    // 1) minecraft.container — engine type + concrete version.
    let container_file = config.systemd_config_dir.join("minecraft.container");
    if container_file.exists() {
        let content = tokio::fs::read_to_string(&container_file)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to read minecraft.container: {}", e)))?;

        let mut new_lines = Vec::new();
        let mut type_updated = false;
        let mut version_updated = false;

        for line in content.lines() {
            if line.trim().starts_with("Environment=TYPE=") {
                new_lines.push(format!("Environment=TYPE={}", req_type));
                type_updated = true;
            } else if line.trim().starts_with("Environment=VERSION=") {
                new_lines.push(format!("Environment=VERSION={}", req_version));
                version_updated = true;
            } else {
                new_lines.push(line.to_string());
            }
        }

        if !type_updated {
            new_lines.push(format!("Environment=TYPE={}", req_type));
        }
        if !version_updated {
            new_lines.push(format!("Environment=VERSION={}", req_version));
        }

        let new_content = new_lines.join("\n") + "\n";
        tokio::fs::write(&container_file, new_content)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to update minecraft.container: {}", e)))?;
    }

    // 2) lazymc.toml — keep the proxy hint in lockstep with the server version.
    set_lazymc_key(
        &config.lazymc_config_file,
        "public",
        "version",
        &format!("\"{}\"", req_version),
    )
    .await?;
    if let Some(protocol) = protocol {
        set_lazymc_key(&config.lazymc_config_file, "public", "protocol", &protocol.to_string()).await?;
    }

    // 3) Persisted state file.
    let state_file = config.data_dir.join("engine_config.json");
    let state_json = serde_json::json!({
        "type": req_type,
        "version": req_version,
        "updated_at": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
    });
    let state_body = serde_json::to_string_pretty(&state_json)
        .map_err(|e| AppError::InternalError(format!("Failed to serialize engine state: {}", e)))?;
    tokio::fs::write(&state_file, state_body).await.map_err(|e| {
        warn!("Failed to write {:?}: {}", state_file, e);
        AppError::PartialFailure(
            "Saving the engine state file failed - the server was not restarted and the engine change may be incomplete"
                .to_string(),
        )
    })?;

    // 4) Back up LuckPerms data from the old engine's location (best-effort: the
    //    engine switch must proceed even if the snapshot fails).
    match crate::minecraft::luckperms::backup_luckperms(&config, &old_engine).await {
        Ok(Some(dest)) => info!("LuckPerms data backed up to {:?}", dest),
        Ok(None) => info!("No LuckPerms data to back up before engine switch"),
        Err(e) => warn!("LuckPerms backup failed (continuing): {}", e),
    }

    // 5) Restore LuckPerms data into the new engine's location BEFORE the server
    //    starts, so permissions survive a platform change.
    match crate::minecraft::luckperms::restore_luckperms(&config, &req_type, None).await {
        Ok(true) => info!("LuckPerms data restored for engine {}", req_type),
        Ok(false) => info!("LuckPerms restore skipped (no backup or target already populated)"),
        Err(e) => warn!("LuckPerms restore failed (continuing): {}", e),
    }

    // 6) Re-sync the tool jars (spark/chunky/luckperms) for the NEW engine before
    //    the server starts, so they are loaded on the first boot after the switch.
    match crate::minecraft::tools::ensure_tools(&config).await {
        Ok(report) => info!(
            "tools re-synced after engine change: supported={} dir={} managed={:?}",
            report.supported, report.target_dir, report.managed
        ),
        Err(e) => warn!("tools re-sync after engine change failed: {}", e),
    }

    // 7) Apply: restart lazymc so it re-reads its config (new hint) and wakes the
    //    server with the new version only if it is currently in "on" mode. If the
    //    server is fully off, the new version simply applies on the next start.
    let restarted = restart_lazymc_if_up(&config).await?;
    info!(
        "Server engine updated to TYPE={} VERSION={}; {}",
        req_type,
        req_version,
        if restarted { "lazymc restarted to apply" } else { "lazymc is off, change applies on next start" }
    );

    Ok(Json(EngineUpdateResponse {
        success: true,
        message: format!("Server engine changed to {} ({})", req_type, req_version),
        resolved_version: req_version,
        warning: if warnings.is_empty() { None } else { Some(warnings.join("\n")) },
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerResourcePackInfo {
    pub active_filename: Option<String>,
    pub url: Option<String>,
    pub sha1: Option<String>,
    pub required: bool,
    pub prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ActivateResourcePackRequest {
    pub filename: String,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub custom_url: Option<String>,
}

pub async fn get_resource_pack_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<ServerResourcePackInfo>, AppError> {
    let server_props = config.minecraft_data_dir.join("server.properties");
    if !server_props.exists() {
        return Ok(Json(ServerResourcePackInfo {
            active_filename: None,
            url: None,
            sha1: None,
            required: false,
            prompt: None,
        }));
    }

    let url = crate::minecraft::server_properties::get_property(&server_props, "resource-pack")
        .await?
        .filter(|s| !s.is_empty());
    let sha1 = crate::minecraft::server_properties::get_property(&server_props, "resource-pack-sha1")
        .await?
        .filter(|s| !s.is_empty());
    let req_str = crate::minecraft::server_properties::get_property(&server_props, "require-resource-pack")
        .await?
        .unwrap_or_default();
    let required = req_str.eq_ignore_ascii_case("true");
    let prompt = crate::minecraft::server_properties::get_property(&server_props, "resource-pack-prompt")
        .await?
        .filter(|s| !s.is_empty());

    let active_filename = url.as_ref().and_then(|u| {
        if let Some(pos) = u.rfind('/') {
            Some(u[pos + 1..].to_string())
        } else {
            None
        }
    });

    Ok(Json(ServerResourcePackInfo {
        active_filename,
        url,
        sha1,
        required,
        prompt,
    }))
}

pub async fn activate_resource_pack_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    AxumJson(payload): AxumJson<ActivateResourcePackRequest>,
) -> Result<Json<ServerResourcePackInfo>, AppError> {
    crate::minecraft::plugins::sanitize_target_dir_and_filename("resourcepacks", &payload.filename)?;
    let pack_path = config.minecraft_data_dir.join("resourcepacks").join(&payload.filename);
    if !pack_path.exists() {
        return Err(AppError::NotFound(format!(
            "Resource pack '{}' not found in resourcepacks/",
            payload.filename
        )));
    }

    let sha1_hex = crate::minecraft::plugins::compute_file_sha1(&pack_path).await?;

    let download_url = if let Some(custom) = payload.custom_url.filter(|s| !s.trim().is_empty()) {
        custom
    } else {
        let host_ip = if config.host == "0.0.0.0" || config.host == "127.0.0.1" {
            "127.0.0.1".to_string()
        } else {
            config.host.clone()
        };
        format!("http://{}:{}/api/public/resourcepack/{}", host_ip, config.port, payload.filename)
    };

    let prompt_text = payload
        .prompt
        .unwrap_or_else(|| "Pack de textures obligatoire pour rejoindre ce serveur".to_string());
    let prompt_json = format!("{{\"text\":\"{}\"}}", prompt_text.replace('"', "\\\""));

    let server_props = config.minecraft_data_dir.join("server.properties");
    let updates = vec![
        ("resource-pack".to_string(), download_url.clone()),
        ("resource-pack-sha1".to_string(), sha1_hex.clone()),
        ("require-resource-pack".to_string(), "true".to_string()),
        ("resource-pack-prompt".to_string(), prompt_json),
    ];

    crate::minecraft::server_properties::set_properties(&server_props, &updates).await?;

    info!("Activated server resource pack '{}' ({})", payload.filename, sha1_hex);

    Ok(Json(ServerResourcePackInfo {
        active_filename: Some(payload.filename),
        url: Some(download_url),
        sha1: Some(sha1_hex),
        required: true,
        prompt: Some(prompt_text),
    }))
}

pub async fn disable_resource_pack_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<crate::routes::plugins::ActionResponse>, AppError> {
    let server_props = config.minecraft_data_dir.join("server.properties");
    if server_props.exists() {
        let updates = vec![
            ("resource-pack".to_string(), String::new()),
            ("resource-pack-sha1".to_string(), String::new()),
            ("require-resource-pack".to_string(), "false".to_string()),
            ("resource-pack-prompt".to_string(), String::new()),
        ];
        crate::minecraft::server_properties::set_properties(&server_props, &updates).await?;
    }

    info!("Disabled server resource pack in server.properties");

    Ok(Json(crate::routes::plugins::ActionResponse {
        success: true,
        message: "Pack de textures serveur désactivé".to_string(),
    }))
}

pub async fn public_resourcepack_handler(
    Extension(config): Extension<Arc<AppConfig>>,
    axum::extract::Path(filename): axum::extract::Path<String>,
) -> Result<axum::response::Response, AppError> {
    let decoded_filename = percent_encoding::percent_decode_str(&filename)
        .decode_utf8()
        .map_err(|_| AppError::BadRequest("Invalid UTF-8 filename".to_string()))?
        .into_owned();

    crate::minecraft::plugins::sanitize_target_dir_and_filename("resourcepacks", &decoded_filename)?;
    let pack_path = config.minecraft_data_dir.join("resourcepacks").join(&decoded_filename);
    if !pack_path.exists() {
        return Err(AppError::NotFound(format!("Resource pack '{}' not found", decoded_filename)));
    }

    let file = tokio::fs::File::open(&pack_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to open resource pack: {}", e)))?;

    let metadata = file
        .metadata()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read resource pack metadata: {}", e)))?;
    let file_size = metadata.len();

    let stream = tokio_util::io::ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    let response = axum::response::Response::builder()
        .header(axum::http::header::CONTENT_TYPE, "application/zip")
        .header(axum::http::header::CONTENT_LENGTH, file_size)
        .header(
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", decoded_filename),
        )
        .body(body)
        .map_err(|e| AppError::InternalError(format!("Failed to build response: {}", e)))?;

    Ok(response)
}


