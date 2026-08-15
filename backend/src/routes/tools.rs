use axum::{response::Json, routing::get, Extension, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
    minecraft::luckperms::{backup_luckperms, list_luckperms_backups, restore_luckperms, LuckPermsBackup},
    minecraft::tools::{detect_engine, ensure_tools, tools_status, ToolSyncReport},
};

pub fn tools_router() -> Router {
    Router::new()
        .route("/status", get(status_handler))
        .route("/sync", axum::routing::post(sync_handler))
        .route("/luckperms/backups", get(luckperms_backups_handler).post(luckperms_backup_handler))
        .route("/luckperms/restore", axum::routing::post(luckperms_restore_handler))
}

/// GET /api/tools/status — read-only snapshot of the detected engine/version and
/// which tools would be managed (no download).
pub async fn status_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<ToolSyncReport>, AppError> {
    Ok(Json(tools_status(&config).await))
}

/// POST /api/tools/sync — installs/updates spark + chunky + luckperms for the
/// current engine.
pub async fn sync_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<ToolSyncReport>, AppError> {
    Ok(Json(ensure_tools(&config).await?))
}

/// GET /api/tools/luckperms/backups — lists available LuckPerms data backups.
pub async fn luckperms_backups_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<Vec<LuckPermsBackup>>, AppError> {
    Ok(Json(list_luckperms_backups(&config).await?))
}

#[derive(Debug, Serialize)]
pub struct BackupResponse {
    pub backed_up: bool,
    pub path: Option<String>,
}

/// POST /api/tools/luckperms/backup — manual snapshot of the current LuckPerms data.
pub async fn luckperms_backup_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<BackupResponse>, AppError> {
    let engine = detect_engine(&config).await.engine;
    let path = backup_luckperms(&config, &engine).await?;
    Ok(Json(BackupResponse {
        backed_up: path.is_some(),
        path: path.map(|p| p.display().to_string()),
    }))
}

#[derive(Debug, Deserialize)]
pub struct RestoreRequest {
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RestoreResponse {
    pub restored: bool,
    pub message: String,
}

/// POST /api/tools/luckperms/restore — restores LuckPerms data into the current
/// engine's expected location. Body `{ "name": "<backup>" }` selects a specific
/// snapshot; omitted = latest.
pub async fn luckperms_restore_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<RestoreRequest>,
) -> Result<Json<RestoreResponse>, AppError> {
    let engine = detect_engine(&config).await.engine;
    let restored = restore_luckperms(&config, &engine, payload.name.as_deref()).await?;
    Ok(Json(RestoreResponse {
        restored,
        message: if restored {
            "LuckPerms data restored".to_string()
        } else {
            "No restore performed (no backup, unsupported engine, or target already populated)".to_string()
        },
    }))
}
