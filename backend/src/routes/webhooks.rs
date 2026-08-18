use axum::{
    extract::Path,
    response::Json,
    routing::{delete, get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    audit::{record_audit_event, AuditCategory},
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
    minecraft::webhook::{WebhookDispatcher, WebhookEndpoint},
};

pub fn webhooks_router() -> Router {
    Router::new()
        .route("/", get(list_webhooks_handler).post(save_webhook_handler))
        .route("/:id", delete(delete_webhook_handler))
        .route("/test", post(test_webhook_handler))
}

/// GET /api/webhooks — lists all configured webhook endpoints
pub async fn list_webhooks_handler(
    _auth: AuthUser,
    Extension(dispatcher): Extension<Arc<WebhookDispatcher>>,
) -> Result<Json<Vec<WebhookEndpoint>>, AppError> {
    let list = dispatcher.list_endpoints().await;
    Ok(Json(list))
}

/// POST /api/webhooks — creates or updates a webhook endpoint
pub async fn save_webhook_handler(
    admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Extension(dispatcher): Extension<Arc<WebhookDispatcher>>,
    Json(endpoint): Json<WebhookEndpoint>,
) -> Result<Json<WebhookEndpoint>, AppError> {
    match dispatcher.add_or_update_endpoint(endpoint.clone()).await {
        Ok(saved) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "WEBHOOK_SAVE",
                AuditCategory::Server,
                "SUCCESS",
                serde_json::json!({ "id": saved.id, "name": saved.name, "url": saved.url }),
                None,
            ).await;
            Ok(Json(saved))
        }
        Err(e) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "WEBHOOK_SAVE",
                AuditCategory::Server,
                "FAILED",
                serde_json::json!({ "name": endpoint.name, "error": e.to_string() }),
                None,
            ).await;
            Err(e)
        }
    }
}

#[derive(Serialize)]
pub struct DeleteWebhookResponse {
    pub success: bool,
    pub message: String,
}

/// DELETE /api/webhooks/:id — removes a webhook endpoint
pub async fn delete_webhook_handler(
    admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Extension(dispatcher): Extension<Arc<WebhookDispatcher>>,
    Path(id): Path<String>,
) -> Result<Json<DeleteWebhookResponse>, AppError> {
    match dispatcher.delete_endpoint(&id).await {
        Ok(_) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "WEBHOOK_DELETE",
                AuditCategory::Server,
                "SUCCESS",
                serde_json::json!({ "id": id }),
                None,
            ).await;
            Ok(Json(DeleteWebhookResponse {
                success: true,
                message: format!("Webhook '{}' supprimé avec succès.", id),
            }))
        }
        Err(e) => {
            record_audit_event(
                &config,
                &admin.0.sub,
                "WEBHOOK_DELETE",
                AuditCategory::Server,
                "FAILED",
                serde_json::json!({ "id": id, "error": e.to_string() }),
                None,
            ).await;
            Err(e)
        }
    }
}

#[derive(Deserialize)]
pub struct TestWebhookRequest {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default = "default_channel")]
    pub channel_type: String, // "discord" | "generic_json"
    pub secret: Option<String>,
}

fn default_channel() -> String {
    "discord".to_string()
}

#[derive(Serialize)]
pub struct TestWebhookResponse {
    pub success: bool,
    pub message: String,
}

/// POST /api/webhooks/test — sends an immediate test notification
pub async fn test_webhook_handler(
    _admin: RequireAdmin,
    Extension(dispatcher): Extension<Arc<WebhookDispatcher>>,
    Json(payload): Json<TestWebhookRequest>,
) -> Result<Json<TestWebhookResponse>, AppError> {
    dispatcher.dispatch_event(
        "TEST_PING",
        "Test de Notification ChiPanel",
        "Ceci est un message de test envoyé depuis le tableau de bord ChiPanel.",
        serde_json::json!({
            "ping": "pong",
            "channel": payload.channel_type,
            "target_url": payload.url,
            "has_secret": payload.secret.is_some(),
        }),
    ).await;

    Ok(Json(TestWebhookResponse {
        success: true,
        message: "Événement de test expédié vers les canaux configurés.".to_string(),
    }))
}
