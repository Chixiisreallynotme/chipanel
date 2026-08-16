use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{
    auth::password::{hash_password, verify_password},
    error::AppError,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub role: String, // "admin" | "operator" | "viewer" | "custom"
    #[serde(default)]
    pub permissions: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccountPublic {
    pub id: String,
    pub username: String,
    pub role: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserAccountRequest {
    pub username: String,
    pub password: String,
    pub role: String,
    #[serde(default)]
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct UserStore {
    storage_path: PathBuf,
    users: Arc<RwLock<HashMap<String, UserAccount>>>,
}

impl UserStore {
    pub async fn load_or_create(data_dir: &Path, admin_username: &str, admin_hash: &str) -> Self {
        let storage_path = data_dir.join("users.json");
        let mut users_map = HashMap::new();

        if storage_path.exists() {
            match tokio::fs::read_to_string(&storage_path).await {
                Ok(content) => {
                    if let Ok(parsed) = serde_json::from_str::<Vec<UserAccount>>(&content) {
                        for u in parsed {
                            users_map.insert(u.username.to_lowercase(), u);
                        }
                        info!("Loaded {} user accounts from disk", users_map.len());
                    }
                }
                Err(e) => {
                    warn!("Could not read users storage file: {}", e);
                }
            }
        }

        // Ensure default admin account exists
        let admin_key = admin_username.to_lowercase();
        users_map.entry(admin_key).or_insert_with(|| UserAccount {
            id: "usr_admin_1".to_string(),
            username: admin_username.to_string(),
            password_hash: admin_hash.to_string(),
            role: "admin".to_string(),
            permissions: vec!["*".to_string()],
            created_at: "System Default".to_string(),
        });

        let store = Self {
            storage_path,
            users: Arc::new(RwLock::new(users_map)),
        };

        let _ = store.persist().await;
        store
    }

    async fn persist(&self) -> Result<(), AppError> {
        let map = self.users.read().await;
        let user_list: Vec<UserAccount> = map.values().cloned().collect();

        if let Some(parent) = self.storage_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        let json_data = serde_json::to_string_pretty(&user_list)
            .map_err(|e| AppError::InternalError(format!("Failed to serialize users: {}", e)))?;

        tokio::fs::write(&self.storage_path, json_data)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to write users file: {}", e)))?;

        Ok(())
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> Option<UserAccount> {
        let map = self.users.read().await;
        let key = username.trim().to_lowercase();

        if let Some(user) = map.get(&key) {
            let pass = password.to_string();
            let hash = user.password_hash.clone();
            
            let is_valid = match tokio::task::spawn_blocking(move || verify_password(&pass, &hash)).await {
                Ok(Ok(valid)) => valid,
                _ => false,
            };

            if is_valid {
                return Some(user.clone());
            }
        }

        None
    }

    pub async fn create_user(
        &self,
        username: &str,
        password: &str,
        role: &str,
        custom_perms: Option<Vec<String>>,
    ) -> Result<UserAccountPublic, AppError> {
        let clean_username = username.trim();
        if clean_username.len() < 3 {
            return Err(AppError::BadRequest("Username must be at least 3 characters long".to_string()));
        }

        if password.len() < 4 {
            return Err(AppError::BadRequest("Password must be at least 4 characters long".to_string()));
        }

        let key = clean_username.to_lowercase();

        {
            let map = self.users.read().await;
            if map.contains_key(&key) {
                return Err(AppError::BadRequest(format!("User '{}' already exists", clean_username)));
            }
        }

        let raw_pass = password.to_string();
        let pass_hash = tokio::task::spawn_blocking(move || hash_password(&raw_pass))
            .await
            .map_err(|e| AppError::InternalError(format!("Task error: {}", e)))??;

        let id = format!("usr_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
        let now_str = crate::auth::now_iso_like();

        let valid_role = match role.to_lowercase().as_str() {
            "operator" | "op" => "operator",
            "viewer" | "read" => "viewer",
            "custom" => "custom",
            _ => "admin",
        };

        let perms = custom_perms.unwrap_or_default();

        let new_account = UserAccount {
            id: id.clone(),
            username: clean_username.to_string(),
            password_hash: pass_hash,
            role: valid_role.to_string(),
            permissions: perms.clone(),
            created_at: now_str.clone(),
        };

        {
            let mut map = self.users.write().await;
            map.insert(key, new_account.clone());
        }

        self.persist().await?;

        Ok(UserAccountPublic {
            id,
            username: clean_username.to_string(),
            role: valid_role.to_string(),
            permissions: perms,
            created_at: now_str,
        })
    }

    pub async fn list_users(&self) -> Vec<UserAccountPublic> {
        let map = self.users.read().await;
        let mut list: Vec<UserAccountPublic> = map
            .values()
            .map(|u| UserAccountPublic {
                id: u.id.clone(),
                username: u.username.clone(),
                role: u.role.clone(),
                permissions: u.permissions.clone(),
                created_at: u.created_at.clone(),
            })
            .collect();

        list.sort_by_key(|a| a.username.to_lowercase());
        list
    }

    pub async fn delete_user(&self, username: &str, requester: &str) -> Result<bool, AppError> {
        let clean = username.trim().to_lowercase();
        if clean == requester.trim().to_lowercase() {
            return Err(AppError::BadRequest("Cannot delete your own active account".to_string()));
        }

        let existed = {
            let mut map = self.users.write().await;
            map.remove(&clean).is_some()
        };

        if existed {
            self.persist().await?;
        }

        Ok(existed)
    }
}
