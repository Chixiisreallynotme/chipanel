use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::config::AppConfig;
use crate::error::AppError;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub id: String,
    pub name: String,
    pub url: String,
    pub secret: Option<String>,
    pub channel_type: String, // "discord" | "telegram" | "generic_json"
    pub subscribed_events: Vec<String>, // ["SERVER_START", "SERVER_STOP", "PLAYER_JOIN", "PLAYER_LEAVE", "CRASH_DETECTED", "BACKUP_COMPLETED"]
    pub enabled: bool,
    pub last_dispatched_secs: Option<u64>,
    pub last_status_code: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct WebhookDispatcher {
    config: Arc<AppConfig>,
    endpoints: Arc<RwLock<Vec<WebhookEndpoint>>>,
    client: reqwest::Client,
}

impl WebhookDispatcher {
    pub async fn new(config: Arc<AppConfig>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        let dispatcher = Self {
            config,
            endpoints: Arc::new(RwLock::new(Vec::new())),
            client,
        };
        dispatcher.load_persisted_endpoints().await;
        dispatcher
    }

    pub fn get_storage_path(&self) -> PathBuf {
        self.config.data_dir.join("webhooks.json")
    }

    pub async fn load_persisted_endpoints(&self) {
        let path = self.get_storage_path();
        if path.exists() {
            if let Ok(data) = tokio::fs::read_to_string(&path).await {
                if let Ok(endpoints) = serde_json::from_str::<Vec<WebhookEndpoint>>(&data) {
                    let mut lock = self.endpoints.write().await;
                    *lock = endpoints;
                    info!("Loaded {} configured webhook endpoints", lock.len());
                }
            }
        }
    }

    async fn save_endpoints_internal(&self, endpoints: &[WebhookEndpoint]) -> Result<(), AppError> {
        let path = self.get_storage_path();
        if let Some(parent) = path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        let json = serde_json::to_string_pretty(endpoints)
            .map_err(|e| AppError::InternalError(format!("Failed serializing webhooks: {}", e)))?;
        tokio::fs::write(&path, json)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed saving webhooks: {}", e)))?;
        Ok(())
    }

    pub async fn list_endpoints(&self) -> Vec<WebhookEndpoint> {
        self.endpoints.read().await.clone()
    }

    pub async fn add_or_update_endpoint(&self, mut ep: WebhookEndpoint) -> Result<WebhookEndpoint, AppError> {
        if ep.name.trim().is_empty() {
            return Err(AppError::BadRequest("Le nom du webhook est obligatoire".to_string()));
        }
        if ep.url.trim().is_empty() || !ep.url.starts_with("http") {
            return Err(AppError::BadRequest("URL de webhook HTTP/HTTPS valide requise".to_string()));
        }

        let mut lock = self.endpoints.write().await;
        if let Some(pos) = lock.iter().position(|e| e.id == ep.id) {
            lock[pos] = ep.clone();
        } else {
            if ep.id.is_empty() {
                ep.id = format!("wh_{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis());
            }
            lock.push(ep.clone());
        }

        self.save_endpoints_internal(&lock).await?;
        info!("Saved webhook endpoint: {} ({})", ep.name, ep.id);
        Ok(ep)
    }

    pub async fn delete_endpoint(&self, id: &str) -> Result<(), AppError> {
        let mut lock = self.endpoints.write().await;
        let initial_len = lock.len();
        lock.retain(|e| e.id != id);

        if lock.len() == initial_len {
            return Err(AppError::NotFound(format!("Webhook '{}' introuvable", id)));
        }

        self.save_endpoints_internal(&lock).await?;
        info!("Deleted webhook: {}", id);
        Ok(())
    }

    /// Dispatches an event payload to all subscribed webhook endpoints
    pub async fn dispatch_event(&self, event_name: &str, title: &str, description: &str, details: serde_json::Value) {
        let active_endpoints: Vec<WebhookEndpoint> = {
            let lock = self.endpoints.read().await;
            lock.iter()
                .filter(|e| e.enabled)
                .filter(|e| e.subscribed_events.is_empty() || e.subscribed_events.contains(&event_name.to_string()))
                .cloned()
                .collect()
        };

        for ep in active_endpoints {
            let client = self.client.clone();
            let event_name_owned = event_name.to_string();
            let title_owned = title.to_string();
            let desc_owned = description.to_string();
            let details_clone = details.clone();
            let dispatcher = self.clone();

            tokio::spawn(async move {
                let res = match ep.channel_type.as_str() {
                    "discord" => {
                        let discord_payload = serde_json::json!({
                            "username": "ChiPanel Bot",
                            "embeds": [{
                                "title": format!("🔔 [{}] {}", event_name_owned, title_owned),
                                "description": desc_owned,
                                "color": match event_name_owned.as_str() {
                                    "SERVER_START" | "BACKUP_COMPLETED" => 0x10B981, // Green
                                    "SERVER_STOP" | "CRASH_DETECTED" => 0xEF4444,    // Red
                                    _ => 0x3B82F6,                                   // Blue
                                },
                                "fields": [
                                    { "name": "Événement", "value": event_name_owned, "inline": true },
                                    { "name": "Serveur", "value": "ChiServ Host-007", "inline": true }
                                ],
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            }]
                        });
                        client.post(&ep.url).json(&discord_payload).send().await
                    }
                    _ => {
                        // Generic JSON with HMAC SHA256 Signature
                        let raw_body = serde_json::json!({
                            "event": event_name_owned,
                            "title": title_owned,
                            "description": desc_owned,
                            "data": details_clone,
                            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
                        });

                        let body_str = raw_body.to_string();
                        let mut req = client.post(&ep.url)
                            .header("Content-Type", "application/json");

                        if let Some(ref secret) = ep.secret {
                            if !secret.is_empty() {
                                if let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) {
                                    mac.update(body_str.as_bytes());
                                    let signature_hex = hex::encode(mac.finalize().into_bytes());
                                    req = req.header("X-ChiPanel-Signature", format!("sha256={}", signature_hex));
                                }
                            }
                        }

                        req.body(body_str).send().await
                    }
                };

                let status_code = match res {
                    Ok(r) => r.status().as_u16(),
                    Err(e) => {
                        warn!("Webhook dispatch failed for '{}': {}", ep.name, e);
                        500
                    }
                };

                // Update endpoint status
                let mut lock = dispatcher.endpoints.write().await;
                if let Some(e) = lock.iter_mut().find(|x| x.id == ep.id) {
                    e.last_dispatched_secs = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs());
                    e.last_status_code = Some(status_code);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_signature_calculation() {
        let secret = "secret123";
        let body = r#"{"event":"TEST"}"#;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body.as_bytes());
        let hex_sig = hex::encode(mac.finalize().into_bytes());
        assert!(!hex_sig.is_empty());
        assert_eq!(hex_sig.len(), 64);
    }
}
