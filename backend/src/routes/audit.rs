use axum::{
    extract::Query,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Extension, Router,
};
use hyper::header;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    audit::{export_audit_logs_csv, query_audit_logs, record_audit_event, AuditCategory, AuditQuery, AuditQueryResult},
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
};

pub fn audit_router() -> Router {
    Router::new()
        .route("/", get(get_audit_logs_handler))
        .route("/export/csv", get(export_csv_handler))
        .route("/record", post(record_manual_event_handler))
}

/// GET /api/audit — queries paginated and filtered audit events
pub async fn get_audit_logs_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<AuditQueryResult>, AppError> {
    let result = query_audit_logs(&config, query).await?;
    Ok(Json(result))
}

/// GET /api/audit/export/csv — exports audit logs as downloadable CSV
pub async fn export_csv_handler(
    _admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Response, AppError> {
    let csv_content = export_audit_logs_csv(&config).await?;
    
    let response = (
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"chipanel_audit_log.csv\"",
            ),
        ],
        csv_content,
    )
        .into_response();

    Ok(response)
}

#[derive(Debug, Deserialize)]
pub struct RecordManualEventRequest {
    pub action: String,
    pub category: String,
    pub status: String,
    pub details: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub success: bool,
}

/// POST /api/audit/record — allows logging specific frontend admin actions
pub async fn record_manual_event_handler(
    auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<RecordManualEventRequest>,
) -> Result<Json<ActionResponse>, AppError> {
    let category = match payload.category.to_lowercase().as_str() {
        "auth" => AuditCategory::Auth,
        "server" => AuditCategory::Server,
        "rcon" => AuditCategory::Rcon,
        "backups" => AuditCategory::Backups,
        "maintenance" => AuditCategory::Maintenance,
        "permissions" => AuditCategory::Permissions,
        "addons" => AuditCategory::Addons,
        "config" => AuditCategory::Config,
        _ => AuditCategory::System,
    };

    record_audit_event(
        &config,
        &auth.0.sub,
        &payload.action,
        category,
        &payload.status,
        payload.details,
        None,
    )
    .await;

    Ok(Json(ActionResponse { success: true }))
}
