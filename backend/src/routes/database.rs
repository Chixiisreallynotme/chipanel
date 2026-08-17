use axum::{
    response::Json,
    routing::{get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

use crate::{
    auth::middleware::RequireAdmin,
    config::AppConfig,
    error::AppError,
    rcon::RconActorHandle,
};

pub fn database_router() -> Router {
    Router::new()
        .route("/db-stats", get(get_db_stats_handler))
        .route("/purge-db", post(purge_db_handler))
}

#[derive(Serialize)]
pub struct TableStat {
    pub name: String,
    pub plugin: String,
    pub description: String,
    pub size_bytes: u64,
    pub formatted_size: String,
}

#[derive(Serialize)]
pub struct DatabaseStatsResponse {
    pub total_storage_bytes: u64,
    pub formatted_total: String,
    pub coreprotect_db_bytes: u64,
    pub luckperms_bytes: u64,
    pub logs_bytes: u64,
    pub nether_bytes: u64,
    pub end_bytes: u64,
    pub tables: Vec<TableStat>,
    pub db_type: String,
}

/// Recursively calculates directory size in bytes.
async fn calculate_dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut total_size = 0;
    if let Ok(mut entries) = tokio::fs::read_dir(path).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(meta) = entry.metadata().await {
                if meta.is_dir() {
                    total_size += Box::pin(calculate_dir_size(&entry.path())).await;
                } else {
                    total_size += meta.len();
                }
            }
        }
    }
    total_size
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Provides detailed disk footprint diagnostics for CoreProtect, LuckPerms, logs, and dimensions.
pub async fn get_db_stats_handler(
    _admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<DatabaseStatsResponse>, AppError> {
    let data_dir = PathBuf::from(&config.minecraft_data_dir);

    // 1. CoreProtect database check
    let cp_db_path = data_dir.join("plugins").join("CoreProtect").join("database.db");
    let cp_db_bytes = if cp_db_path.exists() {
        tokio::fs::metadata(&cp_db_path)
            .await
            .map(|m| m.len())
            .unwrap_or(0)
    } else {
        0
    };

    // 2. LuckPerms storage check
    let lp_path_paper = data_dir.join("plugins").join("LuckPerms");
    let lp_path_fabric = data_dir.join("mods").join("luckperms");
    let lp_path_forge = data_dir.join("config").join("luckperms");

    let luckperms_bytes = calculate_dir_size(&lp_path_paper).await
        + calculate_dir_size(&lp_path_fabric).await
        + calculate_dir_size(&lp_path_forge).await;

    // 3. Logs directory
    let logs_path = data_dir.join("logs");
    let logs_bytes = calculate_dir_size(&logs_path).await;

    // 4. Dimensions (Nether DIM-1 and End DIM1)
    let nether_path = data_dir.join("world_nether").join("DIM-1");
    let end_path = data_dir.join("world_the_end").join("DIM1");
    let nether_fallback = data_dir.join("world").join("DIM-1");
    let end_fallback = data_dir.join("world").join("DIM1");

    let nether_bytes = calculate_dir_size(&nether_path).await + calculate_dir_size(&nether_fallback).await;
    let end_bytes = calculate_dir_size(&end_path).await + calculate_dir_size(&end_fallback).await;

    let total_storage_bytes = cp_db_bytes + luckperms_bytes + logs_bytes + nether_bytes + end_bytes;

    let tables = vec![
        TableStat {
            name: "CoreProtect database.db".to_string(),
            plugin: "CoreProtect".to_string(),
            description: "Blocs placés/détruits, interactions coffres, transactions d'inventaire".to_string(),
            size_bytes: cp_db_bytes,
            formatted_size: format_bytes(cp_db_bytes),
        },
        TableStat {
            name: "LuckPerms Storage".to_string(),
            plugin: "LuckPerms".to_string(),
            description: "Permissions, groupes, contextes et historiques de modifications".to_string(),
            size_bytes: luckperms_bytes,
            formatted_size: format_bytes(luckperms_bytes),
        },
        TableStat {
            name: "Logs et Archives (.log.gz)".to_string(),
            plugin: "Système".to_string(),
            description: "Journaux d'activité serveur et archives de logs compressées".to_string(),
            size_bytes: logs_bytes,
            formatted_size: format_bytes(logs_bytes),
        },
        TableStat {
            name: "Dimension Nether (DIM-1)".to_string(),
            plugin: "Monde".to_string(),
            description: "Fichiers de région .mca générés dans le Nether".to_string(),
            size_bytes: nether_bytes,
            formatted_size: format_bytes(nether_bytes),
        },
        TableStat {
            name: "Dimension End (DIM1)".to_string(),
            plugin: "Monde".to_string(),
            description: "Fichiers de région .mca générés dans l'Ender".to_string(),
            size_bytes: end_bytes,
            formatted_size: format_bytes(end_bytes),
        },
    ];

    Ok(Json(DatabaseStatsResponse {
        total_storage_bytes,
        formatted_total: format_bytes(total_storage_bytes),
        coreprotect_db_bytes: cp_db_bytes,
        luckperms_bytes,
        logs_bytes,
        nether_bytes,
        end_bytes,
        tables,
        db_type: if cp_db_bytes > 0 { "SQLite".to_string() } else { "Fichiers / MySQL".to_string() },
    }))
}

#[derive(Deserialize)]
pub struct PurgeDbRequest {
    pub target: String,      // "coreprotect" | "logs" | "dimension"
    pub days: Option<u32>,   // ex: 30
    pub dimension: Option<String>, // "DIM-1" | "DIM1"
}

#[derive(Serialize)]
pub struct PurgeDbResponse {
    pub success: bool,
    pub action: String,
    pub output: String,
}

/// Executes safe purge operations via persistent RCON.
pub async fn purge_db_handler(
    admin: RequireAdmin,
    Extension(rcon): Extension<RconActorHandle>,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<PurgeDbRequest>,
) -> Result<Json<PurgeDbResponse>, AppError> {
    match payload.target.as_str() {
        "coreprotect" => {
            let days = payload.days.unwrap_or(30);
            let command = format!("co purge t:{}d", days);
            info!("Running CoreProtect purge: {}", command);

            match rcon.exec(&command).await {
                Ok(output) => {
                    crate::audit::record_audit_event(
                        &config,
                        &admin.0.sub,
                        "DATABASE_PURGE_COREPROTECT",
                        crate::audit::AuditCategory::Maintenance,
                        "SUCCESS",
                        serde_json::json!({ "days": days, "output": output }),
                        None,
                    ).await;
                    Ok(Json(PurgeDbResponse {
                        success: true,
                        action: format!("Purge CoreProtect ({} jours)", days),
                        output,
                    }))
                }
                Err(e) => {
                    crate::audit::record_audit_event(
                        &config,
                        &admin.0.sub,
                        "DATABASE_PURGE_COREPROTECT",
                        crate::audit::AuditCategory::Maintenance,
                        "FAILED",
                        serde_json::json!({ "days": days, "error": e.to_string() }),
                        None,
                    ).await;
                    Err(AppError::InternalError(format!("Erreur lors de la purge CoreProtect: {}", e)))
                }
            }
        }
        "logs" => {
            let logs_dir = PathBuf::from(&config.minecraft_data_dir).join("logs");
            let mut removed_count = 0;
            if let Ok(mut entries) = tokio::fs::read_dir(&logs_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.ends_with(".log.gz")
                        && tokio::fs::remove_file(entry.path()).await.is_ok()
                    {
                        removed_count += 1;
                    }
                }
            }
            Ok(Json(PurgeDbResponse {
                success: true,
                action: "Nettoyage des archives de logs".to_string(),
                output: format!("{} archives .log.gz supprimées avec succès.", removed_count),
            }))
        }
        "dimension" => {
            let dim_name = payload.dimension.as_deref().unwrap_or("DIM-1");
            let target_folder = if dim_name == "DIM1" {
                "world_the_end/DIM1"
            } else {
                "world_nether/DIM-1"
            };

            let dim_path = PathBuf::from(&config.minecraft_data_dir).join(target_folder);
            let fallback_path = PathBuf::from(&config.minecraft_data_dir).join("world").join(dim_name);

            let mut deleted = false;
            if dim_path.exists() && tokio::fs::remove_dir_all(&dim_path).await.is_ok() {
                deleted = true;
            }
            if fallback_path.exists() && tokio::fs::remove_dir_all(&fallback_path).await.is_ok() {
                deleted = true;
            }

            if deleted {
                Ok(Json(PurgeDbResponse {
                    success: true,
                    action: format!("Régénération de la dimension {}", dim_name),
                    output: format!("Dossier de dimension '{}' purgé avec succès. Les chunks seront régénérés.", dim_name),
                }))
            } else {
                Ok(Json(PurgeDbResponse {
                    success: true,
                    action: format!("Régénération de la dimension {}", dim_name),
                    output: format!("Aucune donnée de dimension '{}' trouvée sur le disque.", dim_name),
                }))
            }
        }
        _ => Err(AppError::BadRequest(format!("Cible de purge inconnue: {}", payload.target))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024 * 5), "5.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 2), "2.00 GB");
    }
}

