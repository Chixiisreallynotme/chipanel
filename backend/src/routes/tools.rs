use axum::{response::Json, routing::{get, post}, Extension, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
    minecraft::luckperms::{backup_luckperms, list_luckperms_backups, restore_luckperms, LuckPermsBackup},
    minecraft::tools::{detect_engine, ensure_tools, tools_status, ToolSyncReport},
    rcon::RconActorHandle,
};

pub fn tools_router() -> Router {
    Router::new()
        .route("/status", get(status_handler))
        .route("/sync", post(sync_handler))
        .route("/luckperms/backups", get(luckperms_backups_handler).post(luckperms_backup_handler))
        .route("/luckperms/restore", post(luckperms_restore_handler))
        .route("/spark/sampler", post(spark_sampler_handler))
        .route("/spark/health", get(spark_health_handler))
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

#[derive(Debug, Deserialize)]
pub struct SparkSamplerRequest {
    pub action: String, // "start" | "stop" | "viewer"
    pub timeout_secs: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SparkSamplerResponse {
    pub success: bool,
    pub action: String,
    pub url: Option<String>,
    pub output: String,
}

/// POST /api/tools/spark/sampler — controls the Spark profiler sampling and retrieves viewer URLs.
pub async fn spark_sampler_handler(
    _admin: RequireAdmin,
    Extension(rcon): Extension<RconActorHandle>,
    Json(payload): Json<SparkSamplerRequest>,
) -> Result<Json<SparkSamplerResponse>, AppError> {
    let command = match payload.action.as_str() {
        "start" => {
            if let Some(t) = payload.timeout_secs {
                format!("spark sampler --timeout {}", t)
            } else {
                "spark sampler".to_string()
            }
        }
        "stop" => "spark sampler --stop".to_string(),
        "viewer" => "spark sampler --viewer".to_string(),
        _ => return Err(AppError::BadRequest(format!("Unknown spark action: {}", payload.action))),
    };

    match rcon.exec(&command).await {
        Ok(output) => {
            // Extract Spark flamegraph viewer URL if present
            let mut spark_url = None;
            for line in output.lines() {
                if let Some(pos) = line.find("https://spark.lucko.me/") {
                    let candidate = &line[pos..];
                    let end_pos = candidate.find(|c: char| c.is_whitespace() || c == ']' || c == ')')
                        .unwrap_or(candidate.len());
                    spark_url = Some(candidate[..end_pos].to_string());
                    break;
                }
            }

            Ok(Json(SparkSamplerResponse {
                success: true,
                action: payload.action,
                url: spark_url,
                output,
            }))
        }
        Err(e) => Err(AppError::InternalError(format!("Erreur lors de l'exécution de spark: {}", e))),
    }
}

#[derive(Debug, Serialize)]
pub struct SparkHealthResponse {
    pub success: bool,
    pub report: String,
}

/// GET /api/tools/spark/health — runs `/spark health --memory` for instant JVM heap & CPU diagnostics.
pub async fn spark_health_handler(
    _auth: AuthUser,
    Extension(rcon): Extension<RconActorHandle>,
) -> Result<Json<SparkHealthResponse>, AppError> {
    match rcon.exec("spark health --memory").await {
        Ok(report) => Ok(Json(SparkHealthResponse {
            success: true,
            report,
        })),
        Err(e) => Err(AppError::InternalError(format!("Erreur spark health: {}", e))),
    }
}

