use axum::{response::Json, routing::get, Extension, Router};
use std::sync::Arc;

use crate::{
    auth::middleware::AuthUser,
    config::AppConfig,
    error::AppError,
    minecraft::tools::{ensure_tools, tools_status, ToolSyncReport},
};

pub fn tools_router() -> Router {
    Router::new()
        .route("/status", get(status_handler))
        .route("/sync", axum::routing::post(sync_handler))
}

/// GET /api/tools/status — read-only snapshot of the detected engine/version and
/// which tools would be managed (no download).
pub async fn status_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<ToolSyncReport>, AppError> {
    Ok(Json(tools_status(&config).await))
}

/// POST /api/tools/sync — installs/updates spark + chunky for the current engine.
pub async fn sync_handler(
    _auth: crate::auth::middleware::RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<ToolSyncReport>, AppError> {
    Ok(Json(ensure_tools(&config).await?))
}
