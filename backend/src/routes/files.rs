use axum::{
    extract::Query,
    response::Json,
    routing::{delete, get, post},
    Extension, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    auth::middleware::RequireAdmin,
    config::AppConfig,
    error::AppError,
    minecraft::files::{
        create_file_or_dir, delete_file_or_dir, get_file_tree_with_base,
        read_file_content_with_base, save_file_content_with_base, FileActionResponse,
        FileContentResponse, FileCreateRequest, FileSaveRequest, FileTreeNode,
    },
};

#[derive(Debug, Deserialize)]
pub struct TreeQuery {
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReadQuery {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    pub path: String,
}

pub fn files_router() -> Router {
    Router::new()
        .route("/tree", get(tree_handler))
        .route("/read", get(read_handler))
        .route("/save", post(save_handler))
        .route("/create", post(create_handler))
        .route("/delete", delete(delete_handler))
}

/// GET /api/files/tree?path=...
/// Returns directory tree starting from path (or root if omitted).
pub async fn tree_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Query(params): Query<TreeQuery>,
) -> Result<Json<FileTreeNode>, AppError> {
    let subpath = params.path.as_deref().unwrap_or("");
    let tree = get_file_tree_with_base(subpath, 0, &config.minecraft_data_dir).await?;
    Ok(Json(tree))
}

/// GET /api/files/read?path=...
/// Returns file content and inferred syntax mode.
pub async fn read_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Query(params): Query<ReadQuery>,
) -> Result<Json<FileContentResponse>, AppError> {
    if params.path.trim().is_empty() {
        return Err(AppError::BadRequest("path query parameter is required".to_string()));
    }
    let content = read_file_content_with_base(&params.path, &config.minecraft_data_dir).await?;
    Ok(Json(content))
}

/// POST /api/files/save
/// Body `FileSaveRequest` saving file content with optional live RCON reload trigger.
pub async fn save_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<FileSaveRequest>,
) -> Result<Json<FileActionResponse>, AppError> {
    if payload.path.trim().is_empty() {
        return Err(AppError::BadRequest("path parameter is required".to_string()));
    }
    let trigger_reload = payload.trigger_reload.unwrap_or(false);
    let message = save_file_content_with_base(
        &payload.path,
        &payload.content,
        trigger_reload,
        Some(&config),
        &config.minecraft_data_dir,
    )
    .await?;

    Ok(Json(FileActionResponse {
        success: true,
        message,
    }))
}

/// POST /api/files/create
/// Body `FileCreateRequest` creating file or directory.
pub async fn create_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<FileCreateRequest>,
) -> Result<Json<FileActionResponse>, AppError> {
    if payload.path.trim().is_empty() {
        return Err(AppError::BadRequest("path parameter is required".to_string()));
    }
    let message =
        create_file_or_dir(&payload.path, payload.is_dir, &config.minecraft_data_dir).await?;

    Ok(Json(FileActionResponse {
        success: true,
        message,
    }))
}

/// DELETE /api/files/delete?path=...
/// Deletes file or empty directory.
pub async fn delete_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Query(params): Query<DeleteQuery>,
) -> Result<Json<FileActionResponse>, AppError> {
    if params.path.trim().is_empty() {
        return Err(AppError::BadRequest("path query parameter is required".to_string()));
    }
    let message = delete_file_or_dir(&params.path, &config.minecraft_data_dir).await?;

    Ok(Json(FileActionResponse {
        success: true,
        message,
    }))
}
