use axum::{
    response::Json,
    routing::{get, post},
    Extension, Router,
};
use serde::Serialize;
use std::sync::Arc;

use crate::{
    audit::{record_audit_event, AuditCategory},
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
    minecraft::autotune::{
        apply_autotune_tweaks, compute_autotune_recommendations, ApplyAutoTunePayload,
        AutoTuneRecommendations,
    },
};

pub fn autotune_router() -> Router {
    Router::new()
        .route("/recommendations", get(get_recommendations_handler))
        .route("/apply", post(apply_autotune_handler))
}

/// GET /api/autotune/recommendations — calculates hardware-optimal settings
pub async fn get_recommendations_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<AutoTuneRecommendations>, AppError> {
    let recs = compute_autotune_recommendations(&config).await?;
    Ok(Json(recs))
}

#[derive(Serialize)]
pub struct ApplyAutoTuneResponse {
    pub success: bool,
    pub modified_count: usize,
    pub message: String,
}

/// POST /api/autotune/apply — applies recommended server.properties tweaks
pub async fn apply_autotune_handler(
    admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<ApplyAutoTunePayload>,
) -> Result<Json<ApplyAutoTuneResponse>, AppError> {
    match apply_autotune_tweaks(&config, payload).await {
        Ok(count) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "AUTOTUNE_APPLY",
                AuditCategory::Server,
                "SUCCESS",
                serde_json::json!({ "modified_properties": count }),
                None,
            ).await;
            Ok(Json(ApplyAutoTuneResponse {
                success: true,
                modified_count: count,
                message: format!("{} paramètres optimisés appliqués avec succès à server.properties.", count),
            }))
        }
        Err(e) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "AUTOTUNE_APPLY",
                AuditCategory::Server,
                "FAILED",
                serde_json::json!({ "error": e.to_string() }),
                None,
            ).await;
            Err(e)
        }
    }
}
