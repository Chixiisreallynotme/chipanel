use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::{
    error::AppError,
    models::profiles::{
        CreateProfileRequest, JvmConfig, ServerEngineConfig, ServerProfile, ServerPropertiesConfig,
    },
};

#[derive(Debug, Clone)]
pub struct ProfileStore {
    storage_path: PathBuf,
    active_profile_path: PathBuf,
    profiles: Arc<RwLock<HashMap<String, ServerProfile>>>,
    active_profile_id: Arc<RwLock<String>>,
}

impl ProfileStore {
    pub async fn load_or_create(data_dir: &Path) -> Self {
        let storage_path = data_dir.join("profiles.json");
        let active_profile_path = data_dir.join("active_profile.txt");

        let mut map = HashMap::new();

        // Load Built-in Preset Profiles first
        for preset in get_builtin_presets() {
            map.insert(preset.id.clone(), preset);
        }

        // Load custom user profiles from disk if file exists
        if storage_path.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&storage_path).await {
                if let Ok(parsed) = serde_json::from_str::<Vec<ServerProfile>>(&content) {
                    for prof in parsed {
                        map.insert(prof.id.clone(), prof);
                    }
                    info!("Loaded custom server profiles from disk");
                }
            }
        }

        let mut active_id = "prof_paper_pvp".to_string();
        if active_profile_path.exists() {
            if let Ok(id) = tokio::fs::read_to_string(&active_profile_path).await {
                let clean = id.trim();
                if !clean.is_empty() && map.contains_key(clean) {
                    active_id = clean.to_string();
                }
            }
        }

        let store = Self {
            storage_path,
            active_profile_path,
            profiles: Arc::new(RwLock::new(map)),
            active_profile_id: Arc::new(RwLock::new(active_id)),
        };

        let _ = store.persist().await;
        store
    }

    async fn persist(&self) -> Result<(), AppError> {
        let map = self.profiles.read().await;
        let custom_list: Vec<ServerProfile> = map
            .values()
            .filter(|p| !p.is_preset)
            .cloned()
            .collect();

        if let Some(parent) = self.storage_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        let json = serde_json::to_string_pretty(&custom_list)
            .map_err(|e| AppError::InternalError(format!("Failed to serialize profiles: {}", e)))?;

        tokio::fs::write(&self.storage_path, json)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to write profiles file: {}", e)))?;

        let active = self.active_profile_id.read().await;
        let _ = tokio::fs::write(&self.active_profile_path, active.as_bytes()).await;

        Ok(())
    }

    pub async fn list_profiles(&self) -> Vec<ServerProfile> {
        let map = self.profiles.read().await;
        let mut list: Vec<ServerProfile> = map.values().cloned().collect();
        list.sort_by(|a, b| b.is_preset.cmp(&a.is_preset).then_with(|| a.name.cmp(&b.name)));
        list
    }

    pub async fn get_profile(&self, id: &str) -> Option<ServerProfile> {
        let map = self.profiles.read().await;
        map.get(id).cloned()
    }

    pub async fn get_active_profile(&self) -> Option<ServerProfile> {
        let active_id = self.active_profile_id.read().await;
        self.get_profile(&active_id).await
    }

    pub async fn create_profile(&self, req: CreateProfileRequest) -> Result<ServerProfile, AppError> {
        if req.name.trim().is_empty() {
            return Err(AppError::BadRequest("Profile name cannot be empty".to_string()));
        }

        let id = format!("prof_custom_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
        let category = req.category.unwrap_or_else(|| "custom".to_string());

        let new_profile = ServerProfile {
            id: id.clone(),
            name: req.name.trim().to_string(),
            description: req.description,
            icon: "Sliders".to_string(),
            category,
            is_preset: false,
            engine: req.engine,
            jvm: req.jvm,
            properties: req.properties,
            plugins: req.plugins.unwrap_or_default(),
            mods: req.mods.unwrap_or_default(),
            created_at: crate::auth::now_iso_like(),
        };

        {
            let mut map = self.profiles.write().await;
            map.insert(id.clone(), new_profile.clone());
        }

        self.persist().await?;
        Ok(new_profile)
    }

    pub async fn set_active_profile(&self, id: &str) -> Result<ServerProfile, AppError> {
        let profile = self.get_profile(id).await.ok_or_else(|| AppError::NotFound(format!("Profile ID '{}' not found", id)))?;

        {
            let mut active = self.active_profile_id.write().await;
            *active = id.to_string();
        }

        self.persist().await?;
        Ok(profile)
    }

    pub async fn delete_profile(&self, id: &str) -> Result<bool, AppError> {
        {
            let map = self.profiles.read().await;
            if let Some(p) = map.get(id) {
                if p.is_preset {
                    return Err(AppError::BadRequest("Built-in preset profiles cannot be deleted".to_string()));
                }
            } else {
                return Ok(false);
            }
        }

        let existed = {
            let mut map = self.profiles.write().await;
            map.remove(id).is_some()
        };

        if existed {
            self.persist().await?;
        }

        Ok(existed)
    }
}

fn get_builtin_presets() -> Vec<ServerProfile> {
    vec![
        ServerProfile {
            id: "prof_paper_pvp".to_string(),
            name: "Paper Ultra Performance (PvP & Multijoueur)".to_string(),
            description: "Optimisé pour le PvP réactif avec Paper 26.2, Aikar's Flags (8GB RAM) et plugins essentiels.".to_string(),
            icon: "Zap".to_string(),
            category: "pvp".to_string(),
            is_preset: true,
            engine: ServerEngineConfig {
                engine_type: "paper".to_string(),
                version: "26.2".to_string(),
                java_version: "21".to_string(),
            },
            jvm: JvmConfig {
                ram_max: "8GB".to_string(),
                ram_min: "4GB".to_string(),
                aikars_flags: true,
            },
            properties: ServerPropertiesConfig {
                difficulty: "hard".to_string(),
                pvp: true,
                gamemode: "survival".to_string(),
                max_players: 100,
                view_distance: 8,
                simulation_distance: 6,
                motd: "ChiServ - Paper Ultra Performance PvP".to_string(),
            },
            plugins: vec![
                "GrimAC".to_string(),
                "Chunky".to_string(),
                "EssentialsX".to_string(),
                "LuckPerms".to_string(),
                "Vault".to_string(),
                "WorldEdit".to_string(),
            ],
            mods: vec![],
            created_at: "System Preset".to_string(),
        },
        ServerProfile {
            id: "prof_fabric_vanilla_plus".to_string(),
            name: "Fabric Vanilla+ Performance (Survie Optimisée)".to_string(),
            description: "Expérience Vanilla fluide avec moteur Fabric 26.2 et mods de performance serveur (Lithium, FerriteCore, Krypton).".to_string(),
            icon: "Sparkles".to_string(),
            category: "survival".to_string(),
            is_preset: true,
            engine: ServerEngineConfig {
                engine_type: "fabric".to_string(),
                version: "26.2".to_string(),
                java_version: "21".to_string(),
            },
            jvm: JvmConfig {
                ram_max: "6GB".to_string(),
                ram_min: "2GB".to_string(),
                aikars_flags: true,
            },
            properties: ServerPropertiesConfig {
                difficulty: "normal".to_string(),
                pvp: true,
                gamemode: "survival".to_string(),
                max_players: 50,
                view_distance: 12,
                simulation_distance: 10,
                motd: "ChiServ - Fabric Vanilla+ Performance".to_string(),
            },
            plugins: vec![],
            mods: vec![
                "Lithium".to_string(),
                "FerriteCore".to_string(),
                "Krypton".to_string(),
                "Spark".to_string(),
                "ImmediatelyFast".to_string(),
            ],
            created_at: "System Preset".to_string(),
        },
        ServerProfile {
            id: "prof_purpur_rpg".to_string(),
            name: "RPG & Quests Roleplay (Aventure & NPCs)".to_string(),
            description: "Profil Purpur 26.2 sur-mesure pour serveurs de quêtes avec gestion avancée des entités et NPCs.".to_string(),
            icon: "Crown".to_string(),
            category: "rpg".to_string(),
            is_preset: true,
            engine: ServerEngineConfig {
                engine_type: "purpur".to_string(),
                version: "26.2".to_string(),
                java_version: "21".to_string(),
            },
            jvm: JvmConfig {
                ram_max: "10GB".to_string(),
                ram_min: "4GB".to_string(),
                aikars_flags: true,
            },
            properties: ServerPropertiesConfig {
                difficulty: "hard".to_string(),
                pvp: false,
                gamemode: "survival".to_string(),
                max_players: 60,
                view_distance: 10,
                simulation_distance: 8,
                motd: "ChiServ - RPG & Quests Roleplay Realm".to_string(),
            },
            plugins: vec![
                "Citizens2".to_string(),
                "MythicMobs".to_string(),
                "AureliumSkills".to_string(),
                "Quests".to_string(),
                "EssentialsX".to_string(),
            ],
            mods: vec![],
            created_at: "System Preset".to_string(),
        },
        ServerProfile {
            id: "prof_neoforge_skyblock".to_string(),
            name: "Modded Skyblock / Optimisé (Fabulously Optimized)".to_string(),
            description: "Configuration NeoForge / Fabric pour packs de mods et skyblock haute charge.".to_string(),
            icon: "Boxes".to_string(),
            category: "modded".to_string(),
            is_preset: true,
            engine: ServerEngineConfig {
                engine_type: "neoforge".to_string(),
                version: "26.2".to_string(),
                java_version: "21".to_string(),
            },
            jvm: JvmConfig {
                ram_max: "12GB".to_string(),
                ram_min: "6GB".to_string(),
                aikars_flags: true,
            },
            properties: ServerPropertiesConfig {
                difficulty: "normal".to_string(),
                pvp: true,
                gamemode: "survival".to_string(),
                max_players: 40,
                view_distance: 8,
                simulation_distance: 6,
                motd: "ChiServ - Modded Skyblock World".to_string(),
            },
            plugins: vec![],
            mods: vec![
                "Fabulously Optimized".to_string(),
                "Spark".to_string(),
            ],
            created_at: "System Preset".to_string(),
        },
    ]
}
