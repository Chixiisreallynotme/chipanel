use axum::{
    extract::Query,
    response::Json,
    routing::{delete, get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
    minecraft::plugins::{self, InstalledPlugin},
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
}

#[derive(Debug, Deserialize)]
pub struct VersionsQuery {
    pub project_id: String,
}

#[derive(Debug, Deserialize)]
pub struct InstallPluginRequest {
    pub project_id: String,
    pub version_id: Option<String>,
    pub target_dir: String, // "plugins" | "mods"
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
    pub target_dir: String, // "plugins" | "mods"
}

#[derive(Debug, Deserialize)]
pub struct DeletePluginRequest {
    pub filename: String,
    pub target_dir: String, // "plugins" | "mods"
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub success: bool,
    pub message: String,
}

pub fn plugins_router() -> Router {
    Router::new()
        .route("/installed", get(installed_handler))
        .route("/search", get(search_handler))
        .route("/versions", get(versions_handler))
        .route("/install", post(install_handler))
        .route("/toggle", post(toggle_handler))
        .route("/delete", delete(delete_handler))
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
    Extension(client): Extension<Arc<ModrinthClient>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<ModrinthProject>>, AppError> {
    let project_type = if !params.project_type.is_empty() {
        params.project_type.as_str()
    } else if params.loader.eq_ignore_ascii_case("mod")
        || params.loader.eq_ignore_ascii_case("fabric")
        || params.loader.eq_ignore_ascii_case("forge")
        || params.loader.eq_ignore_ascii_case("neoforge")
        || params.loader.eq_ignore_ascii_case("quilt")
    {
        "mod"
    } else if !params.loader.is_empty() && !params.loader.eq_ignore_ascii_case("all") {
        "plugin"
    } else {
        ""
    };

    let projects = client
        .search_projects(
            &params.query,
            project_type,
            &params.game_version,
            &params.loader,
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
    if target_dir != "plugins" && target_dir != "mods" && target_dir != "resourcepacks" && target_dir != "datapacks" {
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
