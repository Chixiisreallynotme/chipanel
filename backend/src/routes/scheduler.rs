use axum::{
    extract::Path,
    response::Json,
    routing::{delete, get, post},
    Extension, Router,
};
use serde::Serialize;
use std::sync::Arc;

use crate::{
    audit::{record_audit_event, AuditCategory},
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
    minecraft::scheduler::{CronJob, SchedulerEngine},
};

pub fn scheduler_router() -> Router {
    Router::new()
        .route("/jobs", get(list_jobs_handler).post(save_job_handler))
        .route("/jobs/:id", delete(delete_job_handler))
        .route("/jobs/:id/run", post(run_job_handler))
        .route("/jobs/:id/toggle", post(toggle_job_handler))
}

/// GET /api/scheduler/jobs — lists all scheduled cron jobs
pub async fn list_jobs_handler(
    _auth: AuthUser,
    Extension(scheduler): Extension<Arc<SchedulerEngine>>,
) -> Result<Json<Vec<CronJob>>, AppError> {
    let jobs = scheduler.list_jobs().await;
    Ok(Json(jobs))
}

/// POST /api/scheduler/jobs — creates or updates a cron job
pub async fn save_job_handler(
    admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Extension(scheduler): Extension<Arc<SchedulerEngine>>,
    Json(job): Json<CronJob>,
) -> Result<Json<CronJob>, AppError> {
    match scheduler.add_or_update_job(job.clone()).await {
        Ok(saved) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "SCHEDULER_JOB_SAVE",
                AuditCategory::Server,
                "SUCCESS",
                serde_json::json!({ "id": saved.id, "name": saved.name, "cron": saved.cron_expression, "action": saved.action_type }),
                None,
            ).await;
            Ok(Json(saved))
        }
        Err(e) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "SCHEDULER_JOB_SAVE",
                AuditCategory::Server,
                "FAILED",
                serde_json::json!({ "name": job.name, "error": e.to_string() }),
                None,
            ).await;
            Err(e)
        }
    }
}

#[derive(Serialize)]
pub struct ActionResponse {
    pub success: bool,
    pub message: String,
}

/// DELETE /api/scheduler/jobs/:id — deletes a scheduled job
pub async fn delete_job_handler(
    admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Extension(scheduler): Extension<Arc<SchedulerEngine>>,
    Path(id): Path<String>,
) -> Result<Json<ActionResponse>, AppError> {
    match scheduler.delete_job(&id).await {
        Ok(_) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "SCHEDULER_JOB_DELETE",
                AuditCategory::Server,
                "SUCCESS",
                serde_json::json!({ "id": id }),
                None,
            ).await;
            Ok(Json(ActionResponse {
                success: true,
                message: format!("Tâche planifiée '{}' supprimée avec succès.", id),
            }))
        }
        Err(e) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "SCHEDULER_JOB_DELETE",
                AuditCategory::Server,
                "FAILED",
                serde_json::json!({ "id": id, "error": e.to_string() }),
                None,
            ).await;
            Err(e)
        }
    }
}

/// POST /api/scheduler/jobs/:id/run — runs a job immediately
pub async fn run_job_handler(
    admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Extension(scheduler): Extension<Arc<SchedulerEngine>>,
    Path(id): Path<String>,
) -> Result<Json<ActionResponse>, AppError> {
    match scheduler.execute_job(&id).await {
        Ok(msg) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "SCHEDULER_JOB_TRIGGER",
                AuditCategory::Server,
                "SUCCESS",
                serde_json::json!({ "id": id, "result": msg }),
                None,
            ).await;
            Ok(Json(ActionResponse {
                success: true,
                message: format!("Exécution réussie : {}", msg),
            }))
        }
        Err(e) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "SCHEDULER_JOB_TRIGGER",
                AuditCategory::Server,
                "FAILED",
                serde_json::json!({ "id": id, "error": e.to_string() }),
                None,
            ).await;
            Err(e)
        }
    }
}

#[derive(Serialize)]
pub struct ToggleResponse {
    pub success: bool,
    pub enabled: bool,
    pub message: String,
}

/// POST /api/scheduler/jobs/:id/toggle — toggles enable/disable state
pub async fn toggle_job_handler(
    _admin: RequireAdmin,
    Extension(scheduler): Extension<Arc<SchedulerEngine>>,
    Path(id): Path<String>,
) -> Result<Json<ToggleResponse>, AppError> {
    let enabled = scheduler.toggle_job(&id).await?;
    Ok(Json(ToggleResponse {
        success: true,
        enabled,
        message: if enabled { "Tâche activée.".to_string() } else { "Tâche désactivée.".to_string() },
    }))
}
