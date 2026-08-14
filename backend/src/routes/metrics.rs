use axum::{
    extract::{Extension, Query},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
    minecraft::metrics::{
        send_test_discord_webhook, trigger_spark_profiler, validate_discord_webhook_url,
        AlertConfig, MetricSample, MetricsHistoryResponse, MetricsStore, SparkProfilerRequest,
        SparkProfilerResponse,
    },
};

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub range: Option<String>,
}

pub fn metrics_router() -> Router {
    Router::new()
        .route("/history", get(get_metrics_history))
        .route("/current", get(get_current_metrics))
        .route("/spark/trigger", post(trigger_spark_handler))
        .route("/alerts", get(get_alert_config).post(update_alert_config))
}

pub async fn get_metrics_history(
    Extension(store): Extension<Arc<MetricsStore>>,
    Query(query): Query<HistoryQuery>,
    _auth: AuthUser,
) -> Json<MetricsHistoryResponse> {
    let range = query.range.as_deref().unwrap_or("1h");
    Json(store.get_history(range))
}

pub async fn get_current_metrics(
    Extension(store): Extension<Arc<MetricsStore>>,
    _auth: AuthUser,
) -> Result<Json<MetricSample>, AppError> {
    let sample = store
        .get_current()
        .ok_or_else(|| AppError::NotFound("No telemetry metric samples recorded yet".to_string()))?;
    Ok(Json(sample))
}

/// Admin-only: starts a Spark profiler on the live server and holds an RCON connection
/// for up to 300s, so an unprivileged caller could stall the game server on demand.
pub async fn trigger_spark_handler(
    Extension(config): Extension<Arc<AppConfig>>,
    _auth: RequireAdmin,
    Json(req): Json<SparkProfilerRequest>,
) -> Result<Json<SparkProfilerResponse>, AppError> {
    let res = trigger_spark_profiler(req.duration_seconds, &config).await?;
    Ok(Json(res))
}

pub async fn get_alert_config(
    Extension(alert_config): Extension<Arc<RwLock<AlertConfig>>>,
    _auth: AuthUser,
) -> Json<AlertConfig> {
    let config = alert_config.read().await.clone();
    Json(config)
}

pub async fn update_alert_config(
    Extension(alert_config): Extension<Arc<RwLock<AlertConfig>>>,
    Extension(config): Extension<Arc<AppConfig>>,
    _auth: RequireAdmin,
    Json(new_config): Json<AlertConfig>,
) -> Result<Json<AlertConfig>, AppError> {
    // Reject a non-Discord webhook host up front, before it is ever stored or contacted.
    if let Some(ref url) = new_config.discord_webhook_url {
        if !url.trim().is_empty() {
            validate_discord_webhook_url(url)?;
        }
    }

    // Hold the write lock only long enough to swap the config in. Sending the test webhook
    // while still holding it would block the telemetry sampler (which reads this lock every
    // 2s) for the entire duration of an outbound HTTP call.
    let test_webhook = {
        let mut lock = alert_config.write().await;

        let should_test = new_config.alerts_enabled
            && new_config.discord_webhook_url.is_some()
            && (lock.discord_webhook_url != new_config.discord_webhook_url || !lock.alerts_enabled);

        *lock = new_config.clone();
        should_test
    };

    new_config.save(&config.data_dir).await?;

    if test_webhook {
        if let Some(ref url) = new_config.discord_webhook_url {
            if let Err(e) = send_test_discord_webhook(url).await {
                tracing::warn!("Test Discord webhook failed: {}", e);
            }
        }
    }

    Ok(Json(new_config))
}
