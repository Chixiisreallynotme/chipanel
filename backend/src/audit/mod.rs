use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::config::AppConfig;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditCategory {
    Auth,
    Server,
    Rcon,
    Backups,
    Maintenance,
    Permissions,
    Addons,
    Config,
    System,
}

impl std::fmt::Display for AuditCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auth => write!(f, "auth"),
            Self::Server => write!(f, "server"),
            Self::Rcon => write!(f, "rcon"),
            Self::Backups => write!(f, "backups"),
            Self::Maintenance => write!(f, "maintenance"),
            Self::Permissions => write!(f, "permissions"),
            Self::Addons => write!(f, "addons"),
            Self::Config => write!(f, "config"),
            Self::System => write!(f, "system"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp_secs: u64,
    pub username: String,
    pub action: String,
    pub category: AuditCategory,
    pub status: String, // "SUCCESS" | "FAILED"
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
}

pub fn get_audit_log_path(config: &AppConfig) -> PathBuf {
    config.data_dir.join("audit_log.jsonl")
}

/// Appends an immutable audit event to the audit log
pub async fn record_audit_event(
    config: &AppConfig,
    username: &str,
    action: &str,
    category: AuditCategory,
    status: &str,
    details: serde_json::Value,
    ip_address: Option<String>,
) {
    let timestamp_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Generate a unique 16-char ID
    use argon2::password_hash::rand_core::{OsRng, RngCore};
    let mut random_bytes = [0u8; 8];
    OsRng.fill_bytes(&mut random_bytes);
    let id = format!("{:x}-{}", timestamp_secs, hex::encode(random_bytes));

    let event = AuditEvent {
        id,
        timestamp_secs,
        username: username.to_string(),
        action: action.to_string(),
        category,
        status: status.to_string(),
        details,
        ip_address,
    };

    let log_path = get_audit_log_path(config);
    if let Some(parent) = log_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }

    if let Ok(line) = serde_json::to_string(&event) {
        if let Ok(mut file) = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await
        {
            let mut data = line.into_bytes();
            data.push(b'\n');
            let _ = file.write_all(&data).await;
        }
    }

    info!(
        "AUDIT: [{}] User '{}' performed '{}' ({}) -> {}",
        event.category, event.username, event.action, event.status, event.details
    );
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub search: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub username: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct AuditQueryResult {
    pub total: usize,
    pub events: Vec<AuditEvent>,
}

/// Queries audit log events with search filters and pagination
pub async fn query_audit_logs(
    config: &AppConfig,
    query: AuditQuery,
) -> Result<AuditQueryResult, AppError> {
    let log_path = get_audit_log_path(config);
    if !log_path.exists() {
        return Ok(AuditQueryResult {
            total: 0,
            events: Vec::new(),
        });
    }

    let content = tokio::fs::read_to_string(&log_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read audit log: {}", e)))?;

    let search_lower = query.search.as_ref().map(|s| s.to_lowercase());
    let category_filter = query.category.as_ref().map(|c| c.to_lowercase());
    let status_filter = query.status.as_ref().map(|s| s.to_uppercase());
    let username_filter = query.username.as_ref().map(|u| u.to_lowercase());

    let mut matched_events = Vec::new();

    for line in content.lines().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Ok(event) = serde_json::from_str::<AuditEvent>(trimmed) {
            // Apply category filter
            if let Some(ref cat) = category_filter {
                if cat != "all" && event.category.to_string().to_lowercase() != *cat {
                    continue;
                }
            }

            // Apply status filter
            if let Some(ref st) = status_filter {
                if st != "ALL" && event.status != *st {
                    continue;
                }
            }

            // Apply username filter
            if let Some(ref un) = username_filter {
                if !event.username.to_lowercase().contains(un) {
                    continue;
                }
            }

            // Apply search filter (searches action, username, IP, or details JSON)
            if let Some(ref term) = search_lower {
                let action_match = event.action.to_lowercase().contains(term);
                let user_match = event.username.to_lowercase().contains(term);
                let ip_match = event.ip_address.as_ref().map(|ip| ip.contains(term)).unwrap_or(false);
                let details_match = event.details.to_string().to_lowercase().contains(term);

                if !action_match && !user_match && !ip_match && !details_match {
                    continue;
                }
            }

            matched_events.push(event);
        }
    }

    let total = matched_events.len();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);

    let paginated = matched_events
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    Ok(AuditQueryResult {
        total,
        events: paginated,
    })
}

/// Exports audit logs as formatted CSV
pub async fn export_audit_logs_csv(config: &AppConfig) -> Result<String, AppError> {
    let log_path = get_audit_log_path(config);
    if !log_path.exists() {
        return Ok("ID,Timestamp,Date,Username,Category,Action,Status,IP,Details\n".to_string());
    }

    let content = tokio::fs::read_to_string(&log_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read audit log: {}", e)))?;

    let mut csv = String::from("ID,Timestamp,Date,Username,Category,Action,Status,IP,Details\n");

    for line in content.lines().rev() {
        if let Ok(event) = serde_json::from_str::<AuditEvent>(line) {
            let details_escaped = event.details.to_string().replace('"', "\"\"");
            let date_str = match std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(event.timestamp_secs)) {
                Some(_) => format!("{}", event.timestamp_secs),
                None => "0".to_string(),
            };

            csv.push_str(&format!(
                "\"{}\",{},\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                event.id,
                event.timestamp_secs,
                date_str,
                event.username,
                event.category,
                event.action,
                event.status,
                event.ip_address.unwrap_or_default(),
                details_escaped
            ));
        }
    }

    Ok(csv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_category_display() {
        assert_eq!(AuditCategory::Server.to_string(), "server");
        assert_eq!(AuditCategory::Auth.to_string(), "auth");
        assert_eq!(AuditCategory::Backups.to_string(), "backups");
    }
}
