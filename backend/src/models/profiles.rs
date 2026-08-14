use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEngineConfig {
    pub engine_type: String, // "paper" | "fabric" | "neoforge" | "purpur" | "forge"
    pub version: String,     // "26.2" | "26.3 (Snapshot 7)" | "1.20.4"
    pub java_version: String, // "21" | "17"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JvmConfig {
    pub ram_max: String,     // "8GB" | "4GB"
    pub ram_min: String,     // "4GB" | "2GB"
    pub aikars_flags: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPropertiesConfig {
    pub difficulty: String,      // "hard" | "normal" | "easy"
    pub pvp: bool,
    pub gamemode: String,         // "survival" | "creative"
    pub max_players: u32,
    pub view_distance: u32,
    pub simulation_distance: u32,
    pub motd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: String, // "pvp" | "survival" | "rpg" | "modded" | "custom"
    pub is_preset: bool,
    pub engine: ServerEngineConfig,
    pub jvm: JvmConfig,
    pub properties: ServerPropertiesConfig,
    #[serde(default)]
    pub plugins: Vec<String>,
    #[serde(default)]
    pub mods: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProfileRequest {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub engine: ServerEngineConfig,
    pub jvm: JvmConfig,
    pub properties: ServerPropertiesConfig,
    pub plugins: Option<Vec<String>>,
    pub mods: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyProfileRequest {
    pub restart_server: Option<bool>,
}
