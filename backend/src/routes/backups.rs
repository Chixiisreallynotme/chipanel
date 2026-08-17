use axum::{
    extract::Path,
    response::Json,
    routing::{delete, get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::{
    audit::{record_audit_event, AuditCategory},
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
    minecraft::server_backup::{
        create_server_backup, delete_server_backup, get_backups_dir, list_server_backups,
        load_backup_settings, restore_server_backup, save_backup_settings, CreateBackupOptions,
        ServerBackupMetadata, ServerBackupSettings,
    },
};

pub fn backups_router() -> Router {
    Router::new()
        .route("/", get(list_backups_handler))
        .route("/create", post(create_backup_handler))
        .route("/restore", post(restore_backup_handler))
        .route("/:filename", delete(delete_backup_handler))
        .route("/settings", get(get_settings_handler).post(update_settings_handler))
        .route("/export/s3", post(export_s3_handler))
}

/// GET /api/backups — lists all available server backups
pub async fn list_backups_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<Vec<ServerBackupMetadata>>, AppError> {
    let backups = list_server_backups(&config).await?;
    Ok(Json(backups))
}

/// POST /api/backups/create — triggers a new backup creation
pub async fn create_backup_handler(
    admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<CreateBackupOptions>,
) -> Result<Json<ServerBackupMetadata>, AppError> {
    let scope = payload.scope.clone().unwrap_or_else(|| "full".to_string());
    match create_server_backup(&config, payload).await {
        Ok(metadata) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "BACKUP_CREATE",
                AuditCategory::Backups,
                "SUCCESS",
                serde_json::json!({ "filename": metadata.filename, "scope": scope, "size_bytes": metadata.file_size_bytes }),
                None,
            ).await;
            Ok(Json(metadata))
        }
        Err(e) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "BACKUP_CREATE",
                AuditCategory::Backups,
                "FAILED",
                serde_json::json!({ "scope": scope, "error": e.to_string() }),
                None,
            ).await;
            Err(e)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RestoreBackupRequest {
    pub filename: String,
}

#[derive(Debug, Serialize)]
pub struct RestoreBackupResponse {
    pub success: bool,
    pub restored_files_count: usize,
    pub message: String,
}

/// POST /api/backups/restore — restores the server from a backup archive
pub async fn restore_backup_handler(
    admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<RestoreBackupRequest>,
) -> Result<Json<RestoreBackupResponse>, AppError> {
    match restore_server_backup(&config, &payload.filename).await {
        Ok(count) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "BACKUP_RESTORE",
                AuditCategory::Backups,
                "SUCCESS",
                serde_json::json!({ "filename": payload.filename, "restored_files_count": count }),
                None,
            ).await;
            Ok(Json(RestoreBackupResponse {
                success: true,
                restored_files_count: count,
                message: format!("Sauvegarde '{}' restaurée avec succès ({} fichiers).", payload.filename, count),
            }))
        }
        Err(e) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "BACKUP_RESTORE",
                AuditCategory::Backups,
                "FAILED",
                serde_json::json!({ "filename": payload.filename, "error": e.to_string() }),
                None,
            ).await;
            Err(e)
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DeleteBackupResponse {
    pub success: bool,
    pub message: String,
}

/// DELETE /api/backups/:filename — removes a backup file from storage
pub async fn delete_backup_handler(
    admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(filename): Path<String>,
) -> Result<Json<DeleteBackupResponse>, AppError> {
    match delete_server_backup(&config, &filename).await {
        Ok(_) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "BACKUP_DELETE",
                AuditCategory::Backups,
                "SUCCESS",
                serde_json::json!({ "filename": filename }),
                None,
            ).await;
            Ok(Json(DeleteBackupResponse {
                success: true,
                message: format!("Sauvegarde '{}' supprimée avec succès.", filename),
            }))
        }
        Err(e) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "BACKUP_DELETE",
                AuditCategory::Backups,
                "FAILED",
                serde_json::json!({ "filename": filename, "error": e.to_string() }),
                None,
            ).await;
            Err(e)
        }
    }
}

/// GET /api/backups/settings — retrieves current retention and exclusion rules
pub async fn get_settings_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<ServerBackupSettings>, AppError> {
    let settings = load_backup_settings(&config).await;
    Ok(Json(settings))
}

/// POST /api/backups/settings — updates backup configuration
pub async fn update_settings_handler(
    _admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(settings): Json<ServerBackupSettings>,
) -> Result<Json<ServerBackupSettings>, AppError> {
    save_backup_settings(&config, &settings).await?;
    info!("Server backup settings updated");
    Ok(Json(settings))
}

#[derive(Debug, Deserialize)]
pub struct ExportS3Request {
    pub filename: String,
}

#[derive(Debug, Serialize)]
pub struct ExportS3Response {
    pub success: bool,
    pub message: String,
    pub s3_url: Option<String>,
}

/// POST /api/backups/export/s3 — exports an archive to an S3/MinIO bucket
pub async fn export_s3_handler(
    _admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<ExportS3Request>,
) -> Result<Json<ExportS3Response>, AppError> {
    let settings = load_backup_settings(&config).await;
    let s3 = match settings.s3_config {
        Some(ref cfg) if cfg.enabled => cfg,
        _ => return Err(AppError::BadRequest("Stockage distant S3/MinIO non configuré ou désactivé".to_string())),
    };

    let backups_dir = get_backups_dir(&config);
    let target_path = backups_dir.join(&payload.filename);

    if !target_path.exists() {
        return Err(AppError::NotFound(format!("Fichier de sauvegarde '{}' introuvable", payload.filename)));
    }

    let file_bytes = tokio::fs::read(&target_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Erreur lors de la lecture de l'archive: {}", e)))?;

    let clean_endpoint = s3.endpoint.trim_end_matches('/');
    let target_key = if s3.path_prefix.is_empty() {
        payload.filename.clone()
    } else {
        format!("{}/{}", s3.path_prefix.trim_matches('/'), payload.filename)
    };

    let upload_url = format!("{}/{}/{}", clean_endpoint, s3.bucket, target_key);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| AppError::InternalError(format!("Reqwest client error: {}", e)))?;

    // Perform HTTP PUT request to S3/MinIO endpoint
    let res = client
        .put(&upload_url)
        .header("Content-Type", "application/zip")
        .body(file_bytes)
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("Échec de l'envoi HTTP vers S3/MinIO: {}", e)))?;

    if !res.status().is_success() {
        let status = res.status();
        let error_body = res.text().await.unwrap_or_default();
        return Err(AppError::InternalError(format!("S3/MinIO a retourné une erreur {}: {}", status, error_body)));
    }

    info!("Backup '{}' successfully exported to S3: {}", payload.filename, upload_url);

    Ok(Json(ExportS3Response {
        success: true,
        message: format!("Sauvegarde '{}' exportée vers S3/MinIO avec succès.", payload.filename),
        s3_url: Some(upload_url),
    }))
}
