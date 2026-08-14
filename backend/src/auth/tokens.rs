use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{
    error::AppError,
    models::auth::{ApiTokenInfo, CreateTokenResponse, UserClaims},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToken {
    pub id: String,
    pub name: String,
    pub token_hash: String,
    pub token_prefix: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TokenStore {
    storage_path: PathBuf,
    tokens: Arc<RwLock<HashMap<String, StoredToken>>>,
}

impl TokenStore {
    pub async fn load_or_create(data_dir: &Path) -> Self {
        let storage_path = data_dir.join("api_tokens.json");
        let mut tokens_map = HashMap::new();

        if storage_path.exists() {
            match tokio::fs::read_to_string(&storage_path).await {
                Ok(content) => {
                    if let Ok(parsed) = serde_json::from_str::<Vec<StoredToken>>(&content) {
                        for token in parsed {
                            tokens_map.insert(token.id.clone(), token);
                        }
                        info!("Loaded {} active API tokens from disk", tokens_map.len());
                    }
                }
                Err(e) => {
                    warn!("Could not read tokens storage file: {}", e);
                }
            }
        }

        Self {
            storage_path,
            tokens: Arc::new(RwLock::new(tokens_map)),
        }
    }

    async fn persist(&self) -> Result<(), AppError> {
        let map = self.tokens.read().await;
        let token_list: Vec<StoredToken> = map.values().cloned().collect();

        if let Some(parent) = self.storage_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        let json_data = serde_json::to_string_pretty(&token_list)
            .map_err(|e| AppError::InternalError(format!("Failed to serialize API tokens: {}", e)))?;

        tokio::fs::write(&self.storage_path, json_data)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to write API tokens file: {}", e)))?;

        Ok(())
    }

    pub async fn create_token(
        &self,
        name: &str,
        expires_in_days: Option<u32>,
    ) -> Result<CreateTokenResponse, AppError> {
        let id = format!("tok_{}", generate_random_hex(8));
        let raw_entropy = generate_random_hex(24);
        let raw_token = format!("chipanel_sec_{}", raw_entropy);
        let token_hash = hash_raw_token(&raw_token);
        
        let token_prefix = format!("chipanel_sec_{}...", &raw_entropy[..6]);

        let now = crate::auth::now_iso_like();
        let expires_at = expires_in_days.map(|days| {
            let now_ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let exp_ts = now_ts + (days as u64 * 86_400);
            crate::auth::format_timestamp(exp_ts)
        });

        let stored = StoredToken {
            id: id.clone(),
            name: name.to_string(),
            token_hash,
            token_prefix: token_prefix.clone(),
            created_at: now.clone(),
            expires_at: expires_at.clone(),
            last_used_at: None,
        };

        {
            let mut map = self.tokens.write().await;
            map.insert(id.clone(), stored.clone());
        }

        self.persist().await?;

        let token_info = ApiTokenInfo {
            id,
            name: name.to_string(),
            token_prefix,
            created_at: now,
            expires_at,
            last_used_at: None,
        };

        Ok(CreateTokenResponse {
            token_info,
            raw_token,
        })
    }

    pub async fn list_tokens(&self) -> Vec<ApiTokenInfo> {
        let map = self.tokens.read().await;
        let mut list: Vec<ApiTokenInfo> = map
            .values()
            .map(|t| ApiTokenInfo {
                id: t.id.clone(),
                name: t.name.clone(),
                token_prefix: t.token_prefix.clone(),
                created_at: t.created_at.clone(),
                expires_at: t.expires_at.clone(),
                last_used_at: t.last_used_at.clone(),
            })
            .collect();

        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list
    }

    pub async fn revoke_token(&self, id: &str) -> Result<bool, AppError> {
        let existed = {
            let mut map = self.tokens.write().await;
            map.remove(id).is_some()
        };

        if existed {
            self.persist().await?;
        }

        Ok(existed)
    }

    pub async fn validate_token(&self, raw_token: &str) -> Option<UserClaims> {
        let input_hash = hash_raw_token(raw_token);
        let mut map = self.tokens.write().await;

        for stored in map.values_mut() {
            if stored.token_hash == input_hash {
                if let Some(exp_str) = &stored.expires_at {
                    if is_expired(exp_str) {
                        return None;
                    }
                }

                stored.last_used_at = Some(crate::auth::now_iso_like());

                return Some(UserClaims {
                    sub: "admin".to_string(),
                    exp: (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() + 86400) as usize,
                    iat: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as usize,
                });
            }
        }

        None
    }
}

fn hash_raw_token(raw: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    raw.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn generate_random_hex(len: usize) -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    let seed = format!("{}-{}-{}", now, pid, len);
    
    let mut hash_val: u64 = 14695981039346656037;
    for b in seed.bytes() {
        hash_val ^= b as u64;
        hash_val = hash_val.wrapping_mul(1099511628211);
    }

    let mut result = String::with_capacity(len * 2);
    for i in 0..len {
        let byte = ((hash_val >> ((i % 8) * 8)) & 0xFF) as u8;
        result.push_str(&format!("{:02x}", byte));
        hash_val = hash_val.wrapping_add(now as u64);
    }
    result.truncate(len * 2);
    result
}

fn is_expired(exp_str: &str) -> bool {
    let now_str = crate::auth::now_iso_like();
    now_str.as_str() > exp_str
}
