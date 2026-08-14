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
    curseforge::client::{CurseForgeClient, CurseForgeMod},
    error::AppError,
    minecraft::modpacks::{
        self, DeployModpackRequest, DeployModpackResponse, ProfileInfo,
    },
};

use crate::modrinth::client::ModrinthClient;

#[derive(Debug, Deserialize)]
pub struct SearchModpacksQuery {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub loader: String,
    #[serde(default)]
    pub game_version: String,
}

#[derive(Debug, Deserialize)]
pub struct ModpackDetailsQuery {
    pub mod_id: u32,
}

#[derive(Debug, Deserialize)]
pub struct SwitchProfileRequest {
    pub profile_name: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteProfileRequest {
    pub profile_name: String,
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub success: bool,
    pub message: String,
}

pub fn modpacks_router() -> Router {
    Router::new()
        .route("/search", get(search_modpacks_handler))
        .route("/details", get(modpack_details_handler))
        .route("/deploy", post(deploy_modpack_handler))
        .route("/profiles", get(list_profiles_handler))
        .route("/profiles/switch", post(switch_profile_handler))
        .route(
            "/profiles/delete",
            delete(delete_profile_handler).post(delete_profile_handler),
        )
}

pub async fn search_modpacks_handler(
    _auth: AuthUser,
    Extension(cf_client): Extension<Arc<CurseForgeClient>>,
    Extension(mr_client): Extension<Arc<ModrinthClient>>,
    Query(params): Query<SearchModpacksQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    if params.provider.eq_ignore_ascii_case("modrinth") {
        let projects = mr_client
            .search_projects(
                &params.query,
                "modpack",
                &params.game_version,
                &params.loader,
                "downloads",
            )
            .await?;
        let mapped: Vec<serde_json::Value> = projects
            .into_iter()
            .map(|p| {
                serde_json::json!({
                    "id": p.project_id,
                    "title": p.title,
                    "author": p.author,
                    "provider": "modrinth",
                    "loader": if !params.loader.is_empty() && params.loader != "all" { params.loader.clone() } else { "Fabric".to_string() },
                    "downloads": p.downloads,
                    "thumbnail": p.icon_url,
                    "summary": p.description,
                    "versions": [
                        {
                            "id": p.latest_version.clone().unwrap_or_else(|| p.project_id.clone()),
                            "version_number": "1.0.0",
                            "game_version": if !params.game_version.is_empty() { params.game_version.clone() } else { "1.20.4".to_string() }
                        }
                    ]
                })
            })
            .collect();
        return Ok(Json(serde_json::json!(mapped)));
    }

    let mods = cf_client.search_modpacks(&params.query).await?;
    let mapped: Vec<serde_json::Value> = mods
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id.to_string(),
                "title": m.name,
                "author": m.authors.first().map(|a| a.name.clone()).unwrap_or_else(|| "CurseForge".to_string()),
                "provider": "curseforge",
                "loader": "Forge",
                "downloads": m.download_count,
                "thumbnail": m.logo.map(|l| l.thumbnail_url),
                "summary": m.summary,
                "versions": [
                    {
                        "id": m.id.to_string(),
                        "version_number": "1.0.0",
                        "game_version": "1.20.1"
                    }
                ]
            })
        })
        .collect();
    Ok(Json(serde_json::json!(mapped)))
}

pub async fn modpack_details_handler(
    _auth: AuthUser,
    Extension(cf_client): Extension<Arc<CurseForgeClient>>,
    Query(params): Query<ModpackDetailsQuery>,
) -> Result<Json<CurseForgeMod>, AppError> {
    let details = cf_client.get_modpack_details(params.mod_id).await?;
    Ok(Json(details))
}

pub async fn deploy_modpack_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Extension(cf_client): Extension<Arc<CurseForgeClient>>,
    Json(payload): Json<DeployModpackRequest>,
) -> Result<Json<DeployModpackResponse>, AppError> {
    let profiles_dir = config.data_dir.join("profiles");
    let resp = modpacks::deploy_modpack(
        &cf_client,
        &profiles_dir,
        &config.minecraft_data_dir,
        payload,
    )
    .await?;
    Ok(Json(resp))
}

pub async fn list_profiles_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<Vec<ProfileInfo>>, AppError> {
    let profiles_dir = config.data_dir.join("profiles");
    let profiles = modpacks::list_profiles(&profiles_dir).await?;
    Ok(Json(profiles))
}

pub async fn switch_profile_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<SwitchProfileRequest>,
) -> Result<Json<ProfileInfo>, AppError> {
    let profiles_dir = config.data_dir.join("profiles");
    let profile = modpacks::switch_profile(
        &profiles_dir,
        &config.minecraft_data_dir,
        &payload.profile_name,
    )
    .await?;
    Ok(Json(profile))
}

pub async fn delete_profile_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<DeleteProfileRequest>,
) -> Result<Json<ActionResponse>, AppError> {
    let profiles_dir = config.data_dir.join("profiles");
    modpacks::delete_profile(&profiles_dir, &payload.profile_name).await?;
    Ok(Json(ActionResponse {
        success: true,
        message: format!("Successfully deleted profile '{}'", payload.profile_name),
    }))
}
