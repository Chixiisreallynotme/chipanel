use axum::{
    extract::{Multipart, Query},
    response::Json,
    routing::{delete, get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

use crate::{
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
    minecraft::{
        plugins::{self, InstalledPlugin},
        tools::detect_engine,
    },
    modrinth::client::{ModrinthClient, ModrinthProject, ModrinthVersion},
};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub loader: String,
    #[serde(default)]
    pub game_version: String,
    #[serde(default)]
    pub project_type: String,
    #[serde(default)]
    pub sort: String,
    #[serde(default)]
    pub auto_filter: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct VersionsQuery {
    pub project_id: String,
}

#[derive(Debug, Deserialize)]
pub struct InstallPluginRequest {
    pub project_id: String,
    pub version_id: Option<String>,
    pub target_dir: String, // "plugins" | "mods" | "resourcepacks" | "datapacks"
}

#[derive(Debug, Serialize)]
pub struct InstallPluginResponse {
    pub success: bool,
    pub filename: String,
    pub file_size_bytes: u64,
    pub target_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct TogglePluginRequest {
    pub filename: String,
    pub target_dir: String, // "plugins" | "mods" | "resourcepacks" | "datapacks"
}

#[derive(Debug, Deserialize)]
pub struct DeletePluginRequest {
    pub filename: String,
    pub target_dir: String, // "plugins" | "mods" | "resourcepacks" | "datapacks"
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CheckUpdatesQuery {
    #[serde(default)]
    pub target_engine: Option<String>,
    #[serde(default)]
    pub target_version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AddonUpdateItem {
    pub filename: String,
    pub name: String,
    pub current_version: String,
    pub target_dir: String,
    pub loader_type: String,
    pub hash: String,
    pub project_id: Option<String>,
    pub project_title: Option<String>,
    pub status: String, // "up_to_date" | "update_available" | "incompatible" | "unknown"
    pub latest_version_id: Option<String>,
    pub latest_version_number: Option<String>,
    pub latest_filename: Option<String>,
    pub latest_download_url: Option<String>,
    pub changelog: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CheckUpdatesResponse {
    pub engine: String,
    pub game_version: String,
    pub total_scanned: usize,
    pub updates_available_count: usize,
    pub up_to_date_count: usize,
    pub incompatible_count: usize,
    pub items: Vec<AddonUpdateItem>,
}

#[derive(Debug, Deserialize)]
pub struct ApplySingleUpdate {
    pub target_dir: String,
    pub old_filename: String,
    pub new_filename: String,
    pub download_url: String,
    pub version_number: String,
}

#[derive(Debug, Deserialize)]
pub struct ApplyUpdatesRequest {
    pub updates: Vec<ApplySingleUpdate>,
}

#[derive(Debug, Serialize)]
pub struct ApplyUpdatesResponse {
    pub success: bool,
    pub updated_count: usize,
    pub errors: Vec<String>,
}

pub fn plugins_router() -> Router {
    Router::new()
        .route("/installed", get(installed_handler))
        .route("/search", get(search_handler))
        .route("/versions", get(versions_handler))
        .route("/install", post(install_handler))
        .route("/upload", post(upload_handler))
        .route("/toggle", post(toggle_handler))
        .route("/delete", delete(delete_handler))
        .route("/updates/check", get(check_updates_handler).post(check_updates_handler))
        .route("/updates/apply", post(apply_updates_handler))
}

pub async fn installed_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<Vec<InstalledPlugin>>, AppError> {
    let installed = plugins::scan_all_installed(&config.minecraft_data_dir).await?;
    Ok(Json(installed))
}

pub async fn search_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Extension(client): Extension<Arc<ModrinthClient>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<ModrinthProject>>, AppError> {
    let auto_filter = params.auto_filter.unwrap_or(true);
    let detected = detect_engine(&config).await;

    let (effective_loader, effective_version) = if auto_filter {
        let ldr = if params.loader.is_empty() || params.loader == "auto" {
            engine_to_default_loader(&detected.engine)
        } else {
            params.loader.as_str()
        };

        let ver = if params.game_version.is_empty() || params.game_version == "auto" {
            if !detected.version.is_empty() {
                detected.version.as_str()
            } else {
                ""
            }
        } else {
            params.game_version.as_str()
        };

        (ldr, ver)
    } else {
        (params.loader.as_str(), params.game_version.as_str())
    };

    let project_type = if !params.project_type.is_empty() {
        params.project_type.as_str()
    } else if effective_loader.eq_ignore_ascii_case("mod")
        || effective_loader.eq_ignore_ascii_case("fabric")
        || effective_loader.eq_ignore_ascii_case("forge")
        || effective_loader.eq_ignore_ascii_case("neoforge")
        || effective_loader.eq_ignore_ascii_case("quilt")
    {
        "mod"
    } else if !effective_loader.is_empty() && !effective_loader.eq_ignore_ascii_case("all") {
        "plugin"
    } else {
        ""
    };

    let projects = client
        .search_projects(
            &params.query,
            project_type,
            effective_version,
            effective_loader,
            &params.sort,
        )
        .await?;

    Ok(Json(projects))
}

pub async fn versions_handler(
    _auth: AuthUser,
    Extension(client): Extension<Arc<ModrinthClient>>,
    Query(params): Query<VersionsQuery>,
) -> Result<Json<Vec<ModrinthVersion>>, AppError> {
    let versions = client.get_project_versions(&params.project_id).await?;
    Ok(Json(versions))
}

pub async fn install_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Extension(client): Extension<Arc<ModrinthClient>>,
    Json(payload): Json<InstallPluginRequest>,
) -> Result<Json<InstallPluginResponse>, AppError> {
    let target_dir = payload.target_dir.trim().to_lowercase();
    if target_dir != "plugins"
        && target_dir != "mods"
        && target_dir != "resourcepacks"
        && target_dir != "datapacks"
    {
        return Err(AppError::BadRequest(
            "target_dir must be 'plugins', 'mods', 'resourcepacks', or 'datapacks'".to_string(),
        ));
    }

    let target_path = config.minecraft_data_dir.join(&target_dir);

    let (filename, bytes_len) = client
        .download_and_install_version(
            &payload.project_id,
            payload.version_id.as_deref(),
            &target_path,
        )
        .await?;

    Ok(Json(InstallPluginResponse {
        success: true,
        filename,
        file_size_bytes: bytes_len,
        target_dir,
    }))
}

pub async fn upload_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    mut multipart: Multipart,
) -> Result<Json<InstalledPlugin>, AppError> {
    let mut target_dir = String::new();
    let mut filename = String::new();
    let mut file_bytes = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart parse error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "target_dir" {
            target_dir = field
                .text()
                .await
                .map_err(|e| AppError::BadRequest(format!("Error reading target_dir: {}", e)))?
                .trim()
                .to_lowercase();
        } else if name == "file" {
            let original_name = field.file_name().unwrap_or("addon.jar").to_string();
            filename = original_name;
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("Error reading file bytes: {}", e)))?;
            file_bytes = data.to_vec();
        }
    }

    if target_dir.is_empty() {
        target_dir = "plugins".to_string();
    }

    plugins::sanitize_target_dir_and_filename(&target_dir, &filename)?;

    let is_valid_ext = filename.ends_with(".jar")
        || filename.ends_with(".zip")
        || filename.ends_with(".plugin");
    if !is_valid_ext {
        return Err(AppError::BadRequest(
            "Only .jar and .zip addon files are accepted".to_string(),
        ));
    }

    let target_dir_path = config.minecraft_data_dir.join(&target_dir);
    if !target_dir_path.exists() {
        tokio::fs::create_dir_all(&target_dir_path)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create folder: {}", e)))?;
    }

    let final_dest = target_dir_path.join(&filename);
    tokio::fs::write(&final_dest, &file_bytes)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to write uploaded file: {}", e)))?;

    info!("Direct upload saved to {:?}", final_dest);

    let installed = plugins::scan_directory(&target_dir_path, &target_dir).await?;
    let found = installed
        .into_iter()
        .find(|p| p.filename == filename)
        .ok_or_else(|| AppError::InternalError("Failed to parse uploaded addon metadata".to_string()))?;

    Ok(Json(found))
}

pub async fn toggle_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<TogglePluginRequest>,
) -> Result<Json<InstalledPlugin>, AppError> {
    let toggled = plugins::toggle_plugin(
        &config.minecraft_data_dir,
        &payload.target_dir,
        &payload.filename,
    )
    .await?;

    Ok(Json(toggled))
}

pub async fn delete_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<DeletePluginRequest>,
) -> Result<Json<ActionResponse>, AppError> {
    plugins::delete_plugin(
        &config.minecraft_data_dir,
        &payload.target_dir,
        &payload.filename,
    )
    .await?;

    Ok(Json(ActionResponse {
        success: true,
        message: format!("Successfully deleted '{}'", payload.filename),
    }))
}

/// Checks all installed plugins/mods against Modrinth for available updates
pub async fn check_updates_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Extension(client): Extension<Arc<ModrinthClient>>,
    Query(params): Query<CheckUpdatesQuery>,
) -> Result<Json<CheckUpdatesResponse>, AppError> {
    let detected = detect_engine(&config).await;
    let target_engine = params.target_engine.unwrap_or(detected.engine);
    let target_version = params.target_version.unwrap_or(detected.version);

    let plugins_dir = config.minecraft_data_dir.join("plugins");
    let mods_dir = config.minecraft_data_dir.join("mods");

    let mut installed_files = Vec::new();
    let installed_plugins = plugins::scan_directory(&plugins_dir, "plugins").await.unwrap_or_default();
    let installed_mods = plugins::scan_directory(&mods_dir, "mods").await.unwrap_or_default();

    installed_files.extend(installed_plugins);
    installed_files.extend(installed_mods);

    let mut hashes = Vec::new();
    let mut file_hash_map = HashMap::new();

    for item in &installed_files {
        let dir = config.minecraft_data_dir.join(&item.target_dir);
        let path = dir.join(&item.filename);
        if let Ok(hash) = plugins::compute_file_sha512(&path).await {
            hashes.push(hash.clone());
            file_hash_map.insert(hash, item.clone());
        }
    }

    let loaders_slice = engine_to_loaders(&target_engine);
    let version_slice = if !target_version.is_empty() {
        vec![target_version.as_str()]
    } else {
        vec![]
    };

    // 1. Identify which projects these hashes belong to
    let current_version_info_map = client.get_version_files(&hashes).await.unwrap_or_default();

    // 2. Fetch updates matching target loaders and target game version
    let updates_map = client
        .check_version_files_update(&hashes, &loaders_slice, &version_slice)
        .await
        .unwrap_or_default();

    let mut update_items = Vec::new();
    let mut updates_available_count = 0;
    let mut up_to_date_count = 0;
    let mut incompatible_count = 0;

    for hash in hashes {
        if let Some(installed_item) = file_hash_map.get(&hash) {
            let current_ver_info = current_version_info_map.get(&hash);
            let update_ver_info = updates_map.get(&hash);

            let (project_id, project_title) = if let Some(v) = current_ver_info {
                (Some(v.project_id.clone()), Some(v.name.clone()))
            } else if let Some(u) = update_ver_info {
                (Some(u.project_id.clone()), Some(u.name.clone()))
            } else {
                (None, None)
            };

            if let Some(update_ver) = update_ver_info {
                let latest_primary_file = update_ver
                    .files
                    .iter()
                    .find(|f| f.primary && f.filename.ends_with(".jar"))
                    .or_else(|| update_ver.files.first());

                let is_same_version = current_ver_info
                    .map(|c| c.id == update_ver.id || c.version_number == update_ver.version_number)
                    .unwrap_or(false);

                if is_same_version {
                    up_to_date_count += 1;
                    update_items.push(AddonUpdateItem {
                        filename: installed_item.filename.clone(),
                        name: installed_item.name.clone(),
                        current_version: installed_item.version.clone(),
                        target_dir: installed_item.target_dir.clone(),
                        loader_type: installed_item.loader_type.clone(),
                        hash: hash.clone(),
                        project_id,
                        project_title,
                        status: "up_to_date".to_string(),
                        latest_version_id: Some(update_ver.id.clone()),
                        latest_version_number: Some(update_ver.version_number.clone()),
                        latest_filename: latest_primary_file.map(|f| f.filename.clone()),
                        latest_download_url: latest_primary_file.map(|f| f.url.clone()),
                        changelog: update_ver.changelog.clone(),
                    });
                } else {
                    updates_available_count += 1;
                    update_items.push(AddonUpdateItem {
                        filename: installed_item.filename.clone(),
                        name: installed_item.name.clone(),
                        current_version: installed_item.version.clone(),
                        target_dir: installed_item.target_dir.clone(),
                        loader_type: installed_item.loader_type.clone(),
                        hash: hash.clone(),
                        project_id,
                        project_title,
                        status: "update_available".to_string(),
                        latest_version_id: Some(update_ver.id.clone()),
                        latest_version_number: Some(update_ver.version_number.clone()),
                        latest_filename: latest_primary_file.map(|f| f.filename.clone()),
                        latest_download_url: latest_primary_file.map(|f| f.url.clone()),
                        changelog: update_ver.changelog.clone(),
                    });
                }
            } else if current_ver_info.is_some() {
                // Known Modrinth mod, but no compatible version for this engine/version!
                incompatible_count += 1;
                update_items.push(AddonUpdateItem {
                    filename: installed_item.filename.clone(),
                    name: installed_item.name.clone(),
                    current_version: installed_item.version.clone(),
                    target_dir: installed_item.target_dir.clone(),
                    loader_type: installed_item.loader_type.clone(),
                    hash: hash.clone(),
                    project_id,
                    project_title,
                    status: "incompatible".to_string(),
                    latest_version_id: None,
                    latest_version_number: None,
                    latest_filename: None,
                    latest_download_url: None,
                    changelog: None,
                });
            } else {
                // Not found on Modrinth by hash (custom jar or downloaded elsewhere)
                update_items.push(AddonUpdateItem {
                    filename: installed_item.filename.clone(),
                    name: installed_item.name.clone(),
                    current_version: installed_item.version.clone(),
                    target_dir: installed_item.target_dir.clone(),
                    loader_type: installed_item.loader_type.clone(),
                    hash: hash.clone(),
                    project_id: None,
                    project_title: None,
                    status: "unknown".to_string(),
                    latest_version_id: None,
                    latest_version_number: None,
                    latest_filename: None,
                    latest_download_url: None,
                    changelog: None,
                });
            }
        }
    }

    Ok(Json(CheckUpdatesResponse {
        engine: target_engine,
        game_version: target_version,
        total_scanned: update_items.len(),
        updates_available_count,
        up_to_date_count,
        incompatible_count,
        items: update_items,
    }))
}

/// Applies a list of addon updates in batch
pub async fn apply_updates_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Extension(client): Extension<Arc<ModrinthClient>>,
    Json(payload): Json<ApplyUpdatesRequest>,
) -> Result<Json<ApplyUpdatesResponse>, AppError> {
    let mut updated_count = 0;
    let mut errors = Vec::new();

    for item in payload.updates {
        if let Err(e) = plugins::sanitize_target_dir_and_filename(&item.target_dir, &item.new_filename) {
            errors.push(format!("Skipping invalid update '{}': {}", item.new_filename, e));
            continue;
        }

        let target_dir_path = config.minecraft_data_dir.join(&item.target_dir);
        let new_file_path = target_dir_path.join(&item.new_filename);

        info!("Applying update: downloading '{}' from {}", item.new_filename, item.download_url);

        match client.download_to(&item.download_url, &new_file_path).await {
            Ok(_) => {
                // If old filename differs from new filename, remove old file
                if item.old_filename != item.new_filename {
                    let old_file_path = target_dir_path.join(&item.old_filename);
                    if old_file_path.exists() {
                        let _ = tokio::fs::remove_file(&old_file_path).await;
                    }
                    let old_file_disabled = target_dir_path.join(format!("{}.disabled", item.old_filename));
                    if old_file_disabled.exists() {
                        let _ = tokio::fs::remove_file(&old_file_disabled).await;
                    }
                }
                updated_count += 1;
            }
            Err(e) => {
                warn!("Failed to download update '{}': {}", item.new_filename, e);
                errors.push(format!("Failed to update '{}': {}", item.old_filename, e));
            }
        }
    }

    Ok(Json(ApplyUpdatesResponse {
        success: errors.is_empty(),
        updated_count,
        errors,
    }))
}

fn engine_to_default_loader(engine: &str) -> &'static str {
    match engine.to_uppercase().as_str() {
        "PURPUR" => "purpur",
        "PAPER" => "paper",
        "SPIGOT" | "BUKKIT" => "spigot",
        "FABRIC" => "fabric",
        "FORGE" => "forge",
        "NEOFORGE" => "neoforge",
        "QUILT" => "quilt",
        _ => "",
    }
}

fn engine_to_loaders(engine: &str) -> Vec<&'static str> {
    match engine.to_uppercase().as_str() {
        "PURPUR" => vec!["purpur", "paper", "spigot", "bukkit"],
        "PAPER" => vec!["paper", "spigot", "bukkit"],
        "SPIGOT" | "BUKKIT" => vec!["spigot", "bukkit"],
        "FABRIC" => vec!["fabric"],
        "FORGE" => vec!["forge"],
        "NEOFORGE" => vec!["neoforge", "forge"],
        "QUILT" => vec!["quilt", "fabric"],
        _ => vec!["paper", "fabric", "spigot"],
    }
}
