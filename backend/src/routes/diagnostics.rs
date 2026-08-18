use axum::{
    extract::Path,
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
    minecraft::diagnostics::{
        compute_server_health, execute_remediation, scan_and_diagnose_crashes,
        CrashDiagnosticReport, ServerHealthSummary,
    },
};

pub fn diagnostics_router() -> Router {
    Router::new()
        .route("/crashes", get(list_crashes_handler))
        .route("/crashes/:id", get(get_crash_detail_handler))
        .route("/crashes/:id/remediate", post(remediate_crash_handler))
        .route("/health", get(get_server_health_handler))
}

/// GET /api/diagnostics/crashes — lists all diagnosed crash reports
pub async fn list_crashes_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<Vec<CrashDiagnosticReport>>, AppError> {
    let reports = scan_and_diagnose_crashes(&config).await?;
    Ok(Json(reports))
}

/// GET /api/diagnostics/crashes/:id — gets detail for a specific crash
pub async fn get_crash_detail_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(id): Path<String>,
) -> Result<Json<CrashDiagnosticReport>, AppError> {
    let reports = scan_and_diagnose_crashes(&config).await?;
    let report = reports.into_iter().find(|r| r.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Crash report '{}' not found", id)))?;
    Ok(Json(report))
}

#[derive(Serialize)]
pub struct RemediationResponse {
    pub success: bool,
    pub message: String,
    pub report_id: String,
}

/// POST /api/diagnostics/crashes/:id/remediate — triggers automated fix
pub async fn remediate_crash_handler(
    admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(id): Path<String>,
) -> Result<Json<RemediationResponse>, AppError> {
    match execute_remediation(&config, &id).await {
        Ok(message) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "DIAGNOSTIC_REMEDIATE",
                AuditCategory::Server,
                "SUCCESS",
                serde_json::json!({ "report_id": id, "action": message }),
                None,
            ).await;
            Ok(Json(RemediationResponse {
                success: true,
                message,
                report_id: id,
            }))
        }
        Err(e) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "DIAGNOSTIC_REMEDIATE",
                AuditCategory::Server,
                "FAILED",
                serde_json::json!({ "report_id": id, "error": e.to_string() }),
                None,
            ).await;
            Err(e)
        }
    }
}

/// GET /api/diagnostics/health — returns overall server health score and issue summary
pub async fn get_server_health_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<ServerHealthSummary>, AppError> {
    let health = compute_server_health(&config).await?;
    Ok(Json(health))
}
