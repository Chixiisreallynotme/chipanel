use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::SystemTime,
};
use tracing::{info, warn};

use crate::{
    config::AppConfig,
    error::AppError,
    rcon::RconClient,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSummary {
    pub uuid: String,
    pub username: String,
    pub is_online: bool,
    pub ping: u32,
    pub health: f32,
    pub food: i32,
    pub dimension: String,
    pub last_seen: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDetail {
    pub uuid: String,
    pub username: String,
    pub is_online: bool,
    pub health: f32,
    pub max_health: f32,
    pub food: i32,
    pub exp_level: i32,
    pub exp_progress: f32,
    pub position_x: f64,
    pub position_y: f64,
    pub position_z: f64,
    pub dimension: String,
    pub playtime_seconds: u64,
    pub first_joined_timestamp: u64,
    pub last_joined_timestamp: u64,
    pub is_op: bool,
    pub is_banned: bool,
    pub ban_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct NbtAttribute {
    #[serde(rename = "Name", default)]
    pub name: String,
    #[serde(rename = "Base", default)]
    pub base: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DimensionValue {
    String(String),
    Int(i32),
}

impl Default for DimensionValue {
    fn default() -> Self {
        DimensionValue::String("minecraft:overworld".to_string())
    }
}

impl DimensionValue {
    pub fn to_dimension_string(&self) -> String {
        match self {
            DimensionValue::String(s) => s.clone(),
            DimensionValue::Int(0) => "minecraft:overworld".to_string(),
            DimensionValue::Int(-1) => "minecraft:the_nether".to_string(),
            DimensionValue::Int(1) => "minecraft:the_end".to_string(),
            DimensionValue::Int(n) => format!("dimension_{}", n),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawNbtPlayerData {
    #[serde(rename = "Pos", default)]
    pub pos: Option<Vec<f64>>,
    #[serde(rename = "Dimension", default)]
    pub dimension: Option<DimensionValue>,
    #[serde(rename = "Health", default)]
    pub health: Option<f32>,
    #[serde(rename = "foodLevel", default)]
    pub food_level: Option<i32>,
    #[serde(rename = "XpLevel", default)]
    pub xp_level: Option<i32>,
    #[serde(rename = "XpP", default)]
    pub xp_p: Option<f32>,
    #[serde(rename = "Score", default)]
    pub score: Option<i32>,
    #[serde(rename = "Attributes", default)]
    pub attributes: Option<Vec<NbtAttribute>>,
}

#[derive(Debug, Deserialize)]
struct UserCacheEntry {
    pub name: String,
    pub uuid: String,
}

#[derive(Debug, Deserialize)]
struct OpEntry {
    pub uuid: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BannedPlayerEntry {
    pub uuid: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

pub fn is_valid_uuid(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.len() == 32 {
        trimmed.chars().all(|c| c.is_ascii_hexdigit())
    } else if trimmed.len() == 36 {
        let b = trimmed.as_bytes();
        b[8] == b'-'
            && b[13] == b'-'
            && b[18] == b'-'
            && b[23] == b'-'
            && trimmed.chars().enumerate().all(|(i, c)| match i {
                8 | 13 | 18 | 23 => c == '-',
                _ => c.is_ascii_hexdigit(),
            })
    } else {
        false
    }
}

pub fn normalize_uuid(s: &str) -> String {
    let trimmed = s.trim();
    if !is_valid_uuid(trimmed) {
        return String::new();
    }
    trimmed.replace('-', "").to_lowercase()
}

pub fn format_uuid(s: &str) -> String {
    let clean = normalize_uuid(s);
    if clean.len() == 32 {
        format!(
            "{}-{}-{}-{}-{}",
            &clean[0..8],
            &clean[8..12],
            &clean[12..16],
            &clean[16..20],
            &clean[20..32]
        )
    } else {
        String::new()
    }
}

pub fn load_usercache(base_dir: &Path) -> (HashMap<String, String>, HashMap<String, String>) {
    let cache_path = base_dir.join("usercache.json");
    let mut uuid_to_name = HashMap::new();
    let mut name_to_uuid = HashMap::new();

    info!("Attempting to load usercache at {:?}", cache_path);
    match fs::read_to_string(&cache_path) {
        Ok(content) => {
            info!("Read usercache.json ({} bytes)", content.len());
            match serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                Ok(json_arr) => {
                    info!("Parsed usercache.json array with {} elements", json_arr.len());
                    for entry in json_arr {
                        let name = entry.get("name").and_then(|v| v.as_str());
                        let uuid = entry.get("uuid").and_then(|v| v.as_str());
                        if let (Some(name_str), Some(uuid_str)) = (name, uuid) {
                            let norm = normalize_uuid(uuid_str);
                            if !norm.is_empty() {
                                uuid_to_name.insert(norm.clone(), name_str.to_string());
                                name_to_uuid.insert(name_str.to_lowercase(), norm);
                            }
                        }
                    }
                    info!("Loaded {} UUID mappings from usercache", uuid_to_name.len());
                }
                Err(err) => warn!("Failed to parse usercache.json JSON: {}", err),
            }
        }
        Err(err) => warn!("Failed to read usercache.json at {:?}: {}", cache_path, err),
    }

    (uuid_to_name, name_to_uuid)
}

pub fn load_ops(base_dir: &Path) -> HashSet<String> {
    let ops_path = base_dir.join("ops.json");
    let mut ops_set = HashSet::new();

    if let Ok(content) = fs::read_to_string(&ops_path) {
        if let Ok(json_arr) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            for entry in json_arr {
                if let Some(uuid) = entry.get("uuid").and_then(|v| v.as_str()) {
                    let norm = normalize_uuid(uuid);
                    if !norm.is_empty() {
                        ops_set.insert(norm);
                    }
                }
                if let Some(name) = entry.get("name").and_then(|v| v.as_str()) {
                    ops_set.insert(name.to_lowercase());
                }
            }
        }
    }

    ops_set
}

pub fn load_banned_players(base_dir: &Path) -> HashMap<String, Option<String>> {
    let banned_path = base_dir.join("banned-players.json");
    let mut banned_map = HashMap::new();

    if let Ok(content) = fs::read_to_string(&banned_path) {
        if let Ok(json_arr) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            for entry in json_arr {
                let reason = entry.get("reason").and_then(|v| v.as_str()).map(|s| s.to_string());
                if let Some(uuid) = entry.get("uuid").and_then(|v| v.as_str()) {
                    let norm = normalize_uuid(uuid);
                    if !norm.is_empty() {
                        banned_map.insert(norm.clone(), reason.clone());
                    }
                }
                if let Some(name) = entry.get("name").and_then(|v| v.as_str()) {
                    banned_map.insert(name.to_lowercase(), reason);
                }
            }
        }
    }

    banned_map
}

pub fn read_nbt_player_file(file_path: &Path) -> Result<(RawNbtPlayerData, u64, u64), AppError> {
    let bytes = fs::read(file_path).map_err(|e| {
        AppError::InternalError(format!("Failed to read player data file {:?}: {}", file_path, e))
    })?;

    let mut decompressed = Vec::new();
    let mut decoder = GzDecoder::new(&bytes[..]);
    let raw_bytes = if decoder.read_to_end(&mut decompressed).is_ok() && !decompressed.is_empty() {
        &decompressed[..]
    } else {
        &bytes[..]
    };

    let nbt_data: RawNbtPlayerData = fastnbt::from_bytes(raw_bytes).map_err(|e| {
        AppError::InternalError(format!("Failed to parse NBT data from {:?}: {}", file_path, e))
    })?;

    let metadata = fs::metadata(file_path).map_err(|e| {
        AppError::InternalError(format!("Failed to read metadata for {:?}: {}", file_path, e))
    })?;

    let last_joined = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let first_joined = metadata
        .created()
        .or_else(|_| metadata.modified())
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(last_joined);

    Ok((nbt_data, first_joined, last_joined))
}

pub fn get_playtime_seconds(base_dir: &Path, uuid_str: &str) -> u64 {
    let formatted_uuid = format_uuid(uuid_str);
    let norm_uuid = normalize_uuid(uuid_str);

    if formatted_uuid.is_empty() || norm_uuid.is_empty() {
        return 0;
    }

    let candidate_paths = [
        base_dir.join("world").join("stats").join(format!("{}.json", formatted_uuid)),
        base_dir.join("world").join("stats").join(format!("{}.json", norm_uuid)),
    ];

    for path in &candidate_paths {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(stats) = val.get("stats") {
                    if let Some(custom) = stats.get("minecraft:custom") {
                        if let Some(ticks) = custom.get("minecraft:play_time").and_then(|v| v.as_u64()) {
                            return ticks / 20;
                        }
                        if let Some(ticks) = custom.get("minecraft:time_since_rest").and_then(|v| v.as_u64()) {
                            return ticks / 20;
                        }
                    }
                }
            }
        }
    }
    0
}

pub fn parse_online_players_from_rcon(output: &str) -> HashSet<String> {
    let mut online_set = HashSet::new();
    if output.is_empty() {
        return online_set;
    }

    let players_part = match output.find(':') {
        Some(idx) => &output[idx + 1..],
        None => output,
    };

    for item in players_part.split(',') {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(open_paren) = trimmed.find('(') {
            if let Some(close_paren) = trimmed.find(')') {
                let name = trimmed[..open_paren].trim();
                let uuid = trimmed[open_paren + 1..close_paren].trim();
                if !name.is_empty() {
                    online_set.insert(name.to_lowercase());
                }
                if !uuid.is_empty() {
                    let norm = normalize_uuid(uuid);
                    if !norm.is_empty() {
                        online_set.insert(norm);
                    }
                }
                continue;
            }
        }

        online_set.insert(trimmed.to_lowercase());
    }

    online_set
}

pub async fn get_all_players(
    config: &AppConfig,
    query: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<PlayerSummary>, AppError> {
    let base_dir = config.minecraft_data_dir.clone();
    let query_owned = query.map(|s| s.to_string());
    let status_owned = status.map(|s| s.to_string());

    let mut online_set = HashSet::new();
    if let Ok(mut rcon) = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password).await {
        if let Ok(list_out) = rcon.exec("list uuids").await {
            online_set.extend(parse_online_players_from_rcon(&list_out));
        } else if let Ok(list_out) = rcon.exec("list").await {
            online_set.extend(parse_online_players_from_rcon(&list_out));
        }
    }

    tokio::task::spawn_blocking(move || {
        let playerdata_dir = base_dir.join("world").join("playerdata");
        let (uuid_to_name, _name_to_uuid) = load_usercache(&base_dir);
        let ops_set = load_ops(&base_dir);
        let banned_map = load_banned_players(&base_dir);

        let query_lower = query_owned
            .as_deref()
            .map(|q| q.trim().to_lowercase())
            .filter(|q| !q.is_empty());
        let status_filter = status_owned
            .as_deref()
            .map(|s| s.trim().to_lowercase())
            .unwrap_or_else(|| "all".to_string());

        let mut all_uuids = HashSet::new();

        for u in uuid_to_name.keys() {
            all_uuids.insert(u.clone());
        }
        for u in banned_map.keys() {
            if is_valid_uuid(u) {
                all_uuids.insert(normalize_uuid(u));
            }
        }
        for u in &ops_set {
            if is_valid_uuid(u) {
                all_uuids.insert(normalize_uuid(u));
            }
        }
        for u in &online_set {
            if is_valid_uuid(u) {
                all_uuids.insert(normalize_uuid(u));
            }
        }

        if playerdata_dir.exists() && playerdata_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&playerdata_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        if file_name.ends_with(".dat") && !file_name.ends_with(".dat_old") {
                            let uuid_raw = file_name.trim_end_matches(".dat");
                            let norm_uuid = normalize_uuid(uuid_raw);
                            if !norm_uuid.is_empty() {
                                all_uuids.insert(norm_uuid);
                            }
                        }
                    }
                }
            }
        }

        info!("get_all_players: base_dir = {:?}, total unique UUIDs collected = {}", base_dir, all_uuids.len());

        let mut players = Vec::new();

        for norm_uuid in all_uuids {
            let formatted_uuid = format_uuid(&norm_uuid);
            if formatted_uuid.is_empty() {
                continue;
            }

            let username = uuid_to_name
                .get(&norm_uuid)
                .cloned()
                .unwrap_or_else(|| formatted_uuid.clone());

            let is_online = online_set.contains(&norm_uuid) || online_set.contains(&username.to_lowercase());
            let is_banned = banned_map.contains_key(&norm_uuid) || banned_map.contains_key(&username.to_lowercase());

            if let Some(ref q) = query_lower {
                if !username.to_lowercase().contains(q) && !formatted_uuid.to_lowercase().contains(q) {
                    continue;
                }
            }

            match status_filter.as_str() {
                "online" => {
                    if !is_online {
                        continue;
                    }
                }
                "banned" => {
                    if !is_banned {
                        continue;
                    }
                }
                _ => {}
            }

            let dat_file = playerdata_dir.join(format!("{}.dat", formatted_uuid));
            let alt_dat_file = playerdata_dir.join(format!("{}.dat", norm_uuid));
            let dat_path = if dat_file.exists() {
                Some(dat_file)
            } else if alt_dat_file.exists() {
                Some(alt_dat_file)
            } else {
                None
            };

            let (health, food, dimension, last_joined) = if let Some(path) = dat_path {
                match read_nbt_player_file(&path) {
                    Ok((nbt_data, _first, last)) => (
                        nbt_data.health.unwrap_or(20.0),
                        nbt_data.food_level.unwrap_or(20),
                        nbt_data
                            .dimension
                            .map(|d| d.to_dimension_string())
                            .unwrap_or_else(|| "minecraft:overworld".to_string()),
                        last,
                    ),
                    Err(_) => (20.0, 20, "minecraft:overworld".to_string(), 0),
                }
            } else {
                (20.0, 20, "minecraft:overworld".to_string(), 0)
            };

            let summary = PlayerSummary {
                uuid: formatted_uuid,
                username,
                is_online,
                ping: if is_online { 20 } else { 0 },
                health,
                food,
                dimension,
                last_seen: last_joined,
            };

            players.push(summary);
        }

        players.sort_by(|a, b| b.is_online.cmp(&a.is_online).then_with(|| a.username.to_lowercase().cmp(&b.username.to_lowercase())));

        Ok(players)
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
}

pub async fn get_player_detail(config: &AppConfig, uuid_str: &str) -> Result<PlayerDetail, AppError> {
    let norm_uuid = normalize_uuid(uuid_str);
    let formatted_uuid = format_uuid(uuid_str);
    if norm_uuid.is_empty() || formatted_uuid.is_empty() {
        return Err(AppError::BadRequest("Invalid player UUID format".into()));
    }

    let base_dir = config.minecraft_data_dir.clone();
    let uuid_owned = uuid_str.to_string();

    let mut online_set = HashSet::new();
    if let Ok(mut rcon) = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password).await {
        if let Ok(list_out) = rcon.exec("list uuids").await {
            online_set.extend(parse_online_players_from_rcon(&list_out));
        } else if let Ok(list_out) = rcon.exec("list").await {
            online_set.extend(parse_online_players_from_rcon(&list_out));
        }
    }

    tokio::task::spawn_blocking(move || {
        let playerdata_dir = base_dir.join("world").join("playerdata");

        let (uuid_to_name, _name_to_uuid) = load_usercache(&base_dir);
        let ops_set = load_ops(&base_dir);
        let banned_map = load_banned_players(&base_dir);

        let username = uuid_to_name
            .get(&norm_uuid)
            .cloned()
            .unwrap_or_else(|| formatted_uuid.clone());

        let is_op = ops_set.contains(&norm_uuid) || ops_set.contains(&username.to_lowercase());
        let ban_reason = banned_map
            .get(&norm_uuid)
            .or_else(|| banned_map.get(&username.to_lowercase()))
            .cloned()
            .flatten();
        let is_banned = banned_map.contains_key(&norm_uuid) || banned_map.contains_key(&username.to_lowercase());
        let is_online = online_set.contains(&norm_uuid) || online_set.contains(&username.to_lowercase());

        let candidate_files = [
            playerdata_dir.join(format!("{}.dat", formatted_uuid)),
            playerdata_dir.join(format!("{}.dat", norm_uuid)),
        ];

        let mut found_path: Option<PathBuf> = None;
        for path in &candidate_files {
            if path.exists() && path.is_file() {
                found_path = Some(path.clone());
                break;
            }
        }

        if found_path.is_none() && playerdata_dir.exists() && playerdata_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&playerdata_dir) {
                let mut processed_files = 0;
                for entry in entries.flatten() {
                    if processed_files >= 500 {
                        break;
                    }
                    processed_files += 1;

                    let path = entry.path();
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        let stem = file_name.trim_end_matches(".dat");
                        if normalize_uuid(stem) == norm_uuid {
                            found_path = Some(path);
                            break;
                        }
                    }
                }
            }
        }

        if found_path.is_none() && !uuid_to_name.contains_key(&norm_uuid) && !is_op && !is_banned && !is_online {
            return Err(AppError::NotFound(format!("Player UUID '{}' not found", uuid_owned)));
        }

        let (nbt_data, first_joined, last_joined) = if let Some(ref file_path) = found_path {
            read_nbt_player_file(file_path).unwrap_or((RawNbtPlayerData::default(), 0, 0))
        } else {
            (RawNbtPlayerData::default(), 0, 0)
        };

        let playtime_seconds = get_playtime_seconds(&base_dir, &norm_uuid);

        let pos = nbt_data.pos.unwrap_or_default();
        let pos_x = pos.get(0).copied().unwrap_or(0.0);
        let pos_y = pos.get(1).copied().unwrap_or(64.0);
        let pos_z = pos.get(2).copied().unwrap_or(0.0);

        let health = nbt_data.health.unwrap_or(20.0);
        let mut max_health = 20.0;
        if let Some(attrs) = nbt_data.attributes {
            for attr in attrs {
                if attr.name == "generic.max_health" || attr.name == "generic.maxHealth" {
                    if let Some(b) = attr.base {
                        max_health = b as f32;
                    }
                }
            }
        }

        let food = nbt_data.food_level.unwrap_or(20);
        let exp_level = nbt_data.xp_level.unwrap_or(0);
        let exp_progress = nbt_data.xp_p.unwrap_or(0.0);
        let dimension = nbt_data
            .dimension
            .map(|d| d.to_dimension_string())
            .unwrap_or_else(|| "minecraft:overworld".to_string());

        Ok(PlayerDetail {
            uuid: formatted_uuid,
            username,
            is_online,
            health,
            max_health,
            food,
            exp_level,
            exp_progress,
            position_x: pos_x,
            position_y: pos_y,
            position_z: pos_z,
            dimension,
            playtime_seconds,
            first_joined_timestamp: first_joined,
            last_joined_timestamp: last_joined,
            is_op,
            is_banned,
            ban_reason,
        })
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
}

