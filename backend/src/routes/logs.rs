use axum::{
    extract::Query,
    response::Json,
    routing::{get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};

use crate::{
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
};

pub fn logs_router() -> Router {
    Router::new()
        .route("/mclogs", post(upload_mclogs_handler))
        .route("/history", get(get_log_history_handler))
}

#[derive(Deserialize)]
pub struct McLogsUploadRequest {
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct McLogsResponse {
    pub success: bool,
    pub id: Option<String>,
    pub url: Option<String>,
    pub raw: Option<String>,
    pub error: Option<String>,
}

/// Uploads log content to mclo.gs API v1 with automatic redaction of IPs and sensitive tokens.
pub async fn upload_mclogs_handler(
    _admin: RequireAdmin,
    Json(payload): Json<McLogsUploadRequest>,
) -> Result<Json<McLogsResponse>, AppError> {
    if payload.content.trim().is_empty() {
        return Err(AppError::BadRequest("Log content cannot be empty".to_string()));
    }

    // Limit log payload to 10MB as required by mclo.gs API
    let content_to_send = if payload.content.len() > 10 * 1024 * 1024 {
        &payload.content[..10 * 1024 * 1024]
    } else {
        &payload.content
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("ChiPanel/1.0")
        .build()
        .map_err(|e| AppError::InternalError(format!("Failed to build HTTP client: {}", e)))?;

    let form_params = [("content", content_to_send)];

    let resp = client
        .post("https://api.mclo.gs/1/log")
        .form(&form_params)
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to reach mclo.gs API: {}", e)))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read mclo.gs response: {}", e)))?;

    if !status.is_success() {
        error!("mclo.gs API returned error status {}: {}", status, text);
        return Ok(Json(McLogsResponse {
            success: false,
            id: None,
            url: None,
            raw: None,
            error: Some(format!("mclo.gs API error ({}): {}", status, text)),
        }));
    }

    match serde_json::from_str::<McLogsResponse>(&text) {
        Ok(parsed) => {
            info!("Log successfully exported to mclo.gs: {:?}", parsed.url);
            Ok(Json(parsed))
        }
        Err(err) => {
            error!("Failed to parse mclo.gs response JSON: {}", err);
            Err(AppError::InternalError(format!(
                "Invalid response from mclo.gs: {}",
                err
            )))
        }
    }
}

#[derive(Deserialize)]
pub struct LogHistoryQuery {
    pub lines: Option<usize>,
    pub filter: Option<String>,
}

#[derive(Serialize)]
pub struct LogHistoryResponse {
    pub lines: Vec<String>,
    pub total_lines: usize,
    pub filename: String,
}

/// Reads recent lines from `latest.log`.
pub async fn get_log_history_handler(
    _auth: AuthUser,
    Query(params): Query<LogHistoryQuery>,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<LogHistoryResponse>, AppError> {
    let log_path = PathBuf::from(&config.minecraft_data_dir).join("logs").join("latest.log");

    if !log_path.exists() {
        return Ok(Json(LogHistoryResponse {
            lines: vec!["No latest.log file found on disk yet.".to_string()],
            total_lines: 0,
            filename: "latest.log".to_string(),
        }));
    }

    let content = tokio::fs::read_to_string(&log_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read latest.log: {}", e)))?;

    let max_lines = params.lines.unwrap_or(500).min(5000);
    let mut all_lines: Vec<&str> = content.lines().collect();

    if let Some(ref filter_str) = params.filter {
        let filter_lower = filter_str.to_lowercase();
        all_lines.retain(|line| line.to_lowercase().contains(&filter_lower));
    }

    let total_lines = all_lines.len();
    let start_idx = total_lines.saturating_sub(max_lines);
    let slice = &all_lines[start_idx..];

    let result_lines: Vec<String> = slice.iter().map(|s| s.to_string()).collect();

    Ok(Json(LogHistoryResponse {
        lines: result_lines,
        total_lines,
        filename: "latest.log".to_string(),
    }))
}
