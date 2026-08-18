use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::Path,
};
use flate2::read::GzDecoder;

use crate::{
    error::AppError,
    minecraft::player::{format_uuid, is_valid_uuid, normalize_uuid},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusEffectInfo {
    pub id: String,
    pub name: String,
    pub amplifier: u8,
    pub duration_ticks: u32,
    pub duration_formatted: String,
    pub icon_name: String,
    pub category: String, // "beneficial" | "harmful" | "neutral"
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawPlayerEffectsData {
    #[serde(default, rename = "active_effects", alias = "ActiveEffects")]
    pub active_effects: Vec<RawStatusEffectTag>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawStatusEffectTag {
    #[serde(default, rename = "id", alias = "Id")]
    pub id: Option<String>,
    #[serde(default, rename = "id_byte", alias = "IdByte")]
    pub id_byte: Option<i8>,
    #[serde(default, rename = "amplifier", alias = "Amplifier")]
    pub amplifier: Option<i8>,
    #[serde(default, rename = "duration", alias = "Duration")]
    pub duration: Option<i32>,
    #[serde(default, rename = "ambient", alias = "Ambient")]
    #[allow(dead_code)]
    pub ambient: Option<u8>,
    #[serde(default, rename = "show_particles", alias = "ShowParticles")]
    #[allow(dead_code)]
    pub show_particles: Option<u8>,
    #[serde(default, rename = "show_icon", alias = "ShowIcon")]
    #[allow(dead_code)]
    pub show_icon: Option<u8>,
}

/// Reads `/app/minecraft-data/world/playerdata/{uuid}.dat` file
/// (or from default MINECRAFT_DATA_DIR) and extracts active status effects.
#[allow(dead_code)]
pub fn parse_player_effects(uuid: &str) -> Result<Vec<StatusEffectInfo>, AppError> {
    let base_dir_str = std::env::var("MINECRAFT_DATA_DIR")
        .unwrap_or_else(|_| "/app/minecraft-data".to_string());
    parse_player_effects_with_dir(Path::new(&base_dir_str), uuid)
}

/// Parses active status effects for a player from a specific base minecraft data directory.
pub fn parse_player_effects_with_dir(base_dir: &Path, uuid: &str) -> Result<Vec<StatusEffectInfo>, AppError> {
    let norm_uuid = normalize_uuid(uuid);
    let formatted_uuid = format_uuid(uuid);
    if norm_uuid.is_empty() || formatted_uuid.is_empty() || !is_valid_uuid(uuid) {
        return Err(AppError::BadRequest("Invalid player UUID format".into()));
    }

    let file_path = crate::minecraft::player::find_player_dat_file(base_dir, uuid).ok_or_else(|| {
        AppError::NotFound(format!("Player data file for UUID '{}' not found", uuid))
    })?;

    let bytes = fs::read(&file_path).map_err(|e| {
        AppError::InternalError(format!("Failed to read player data file {:?}: {}", file_path, e))
    })?;

    let mut decompressed = Vec::with_capacity(8192);
    let decoder = GzDecoder::new(&bytes[..]);
    let raw_bytes = if decoder.take(5 * 1024 * 1024).read_to_end(&mut decompressed).is_ok() && !decompressed.is_empty() {
        &decompressed[..]
    } else {
        &bytes[..]
    };

    let raw_data: RawPlayerEffectsData = fastnbt::from_bytes(raw_bytes).map_err(|e| {
        AppError::InternalError(format!("Failed to parse NBT data from {:?}: {}", file_path, e))
    })?;

    Ok(parse_player_effects_from_nbt(&raw_data))
}

#[allow(dead_code)]
fn matches_uuid_in_place(stem: &str, target_norm_uuid: &str) -> bool {
    let mut target_bytes = target_norm_uuid.bytes();
    let mut count = 0;

    for b in stem.bytes() {
        if b == b'-' {
            continue;
        }
        let lower_b = b.to_ascii_lowercase();
        if let Some(target_b) = target_bytes.next() {
            if lower_b != target_b {
                return false;
            }
            count += 1;
        } else {
            return false;
        }
    }

    count == 32 && target_bytes.next().is_none()
}

pub fn parse_player_effects_from_nbt(raw_data: &RawPlayerEffectsData) -> Vec<StatusEffectInfo> {
    let mut results = Vec::new();
    for tag in &raw_data.active_effects {
        let effect_id = match extract_effect_id(tag) {
            Some(id) => id,
            None => continue,
        };

        let raw_amp = tag.amplifier.unwrap_or(0);
        let amplifier = raw_amp as u8;

        let raw_dur = tag.duration.unwrap_or(0);
        let (duration_ticks, duration_formatted) = if !(0..1_000_000_000).contains(&raw_dur) {
            (u32::MAX, "Infinite".to_string())
        } else {
            let ticks = raw_dur as u32;
            let total_secs = ticks / 20;
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            let formatted = format!("{}:{:02}", mins, secs);
            (ticks, formatted)
        };

        let (name, category, icon_name) = get_effect_metadata(&effect_id);

        results.push(StatusEffectInfo {
            id: effect_id,
            name: name.to_string(),
            amplifier,
            duration_ticks,
            duration_formatted,
            icon_name: icon_name.to_string(),
            category: category.to_string(),
        });
    }
    results
}

fn extract_effect_id(tag: &RawStatusEffectTag) -> Option<String> {
    if let Some(ref s) = tag.id {
        if s.is_empty() {
            None
        } else if !s.contains(':') {
            Some(format!("minecraft:{}", s))
        } else {
            Some(s.clone())
        }
    } else {
        tag.id_byte.map(|b| map_numeric_effect_id(b as u8 as u32))
    }
}

pub fn map_numeric_effect_id(id: u32) -> String {
    let name = match id {
        1 => "speed",
        2 => "slowness",
        3 => "haste",
        4 => "mining_fatigue",
        5 => "strength",
        6 => "instant_health",
        7 => "instant_damage",
        8 => "jump_boost",
        9 => "nausea",
        10 => "regeneration",
        11 => "resistance",
        12 => "fire_resistance",
        13 => "water_breathing",
        14 => "invisibility",
        15 => "blindness",
        16 => "night_vision",
        17 => "hunger",
        18 => "weakness",
        19 => "poison",
        20 => "wither",
        21 => "health_boost",
        22 => "absorption",
        23 => "saturation",
        24 => "glowing",
        25 => "levitation",
        26 => "luck",
        27 => "unluck",
        28 => "slow_falling",
        29 => "conduit_power",
        30 => "dolphins_grace",
        31 => "bad_omen",
        32 => "hero_of_the_village",
        33 => "darkness",
        _ => return format!("minecraft:effect_{}", id),
    };
    format!("minecraft:{}", name)
}

pub fn get_effect_metadata(effect_id: &str) -> (&'static str, &'static str, &'static str) {
    let clean_id = effect_id.strip_prefix("minecraft:").unwrap_or(effect_id);

    match clean_id {
        "speed" => ("Speed", "beneficial", "speed"),
        "slowness" => ("Slowness", "harmful", "slowness"),
        "haste" => ("Haste", "beneficial", "haste"),
        "mining_fatigue" => ("Mining Fatigue", "harmful", "mining_fatigue"),
        "strength" => ("Strength", "beneficial", "strength"),
        "instant_health" => ("Instant Health", "beneficial", "instant_health"),
        "instant_damage" => ("Instant Damage", "harmful", "instant_damage"),
        "jump_boost" => ("Jump Boost", "beneficial", "jump_boost"),
        "nausea" => ("Nausea", "harmful", "nausea"),
        "regeneration" => ("Regeneration", "beneficial", "regeneration"),
        "resistance" => ("Resistance", "beneficial", "resistance"),
        "fire_resistance" => ("Fire Resistance", "beneficial", "fire_resistance"),
        "water_breathing" => ("Water Breathing", "beneficial", "water_breathing"),
        "invisibility" => ("Invisibility", "beneficial", "invisibility"),
        "blindness" => ("Blindness", "harmful", "blindness"),
        "night_vision" => ("Night Vision", "beneficial", "night_vision"),
        "hunger" => ("Hunger", "harmful", "hunger"),
        "weakness" => ("Weakness", "harmful", "weakness"),
        "poison" => ("Poison", "harmful", "poison"),
        "wither" => ("Wither", "harmful", "wither"),
        "health_boost" => ("Health Boost", "beneficial", "health_boost"),
        "absorption" => ("Absorption", "beneficial", "absorption"),
        "saturation" => ("Saturation", "beneficial", "saturation"),
        "glowing" => ("Glowing", "neutral", "glowing"),
        "levitation" => ("Levitation", "harmful", "levitation"),
        "luck" => ("Luck", "beneficial", "luck"),
        "unluck" | "bad_luck" => ("Bad Luck", "harmful", "bad_luck"),
        "slow_falling" => ("Slow Falling", "beneficial", "slow_falling"),
        "conduit_power" => ("Conduit Power", "beneficial", "conduit_power"),
        "dolphins_grace" => ("Dolphin's Grace", "beneficial", "dolphins_grace"),
        "bad_omen" => ("Bad Omen", "harmful", "bad_omen"),
        "hero_of_the_village" => ("Hero of the Village", "beneficial", "hero_of_the_village"),
        "darkness" => ("Darkness", "harmful", "darkness"),
        "trial_omen" => ("Trial Omen", "harmful", "trial_omen"),
        "raid_omen" => ("Raid Omen", "harmful", "raid_omen"),
        "wind_charged" => ("Wind Charged", "neutral", "wind_charged"),
        "weaving" => ("Weaving", "neutral", "weaving"),
        "oozing" => ("Oozing", "neutral", "oozing"),
        "infested" => ("Infested", "harmful", "infested"),
        _ => ("Unknown Effect", "neutral", "unknown"),
    }
}

#[allow(dead_code)]
pub fn format_effect_name(clean_id: &str) -> String {
    let mut res = String::with_capacity(clean_id.len());
    for (i, word) in clean_id.split('_').enumerate() {
        if i > 0 {
            res.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            for c in first.to_uppercase() {
                res.push(c);
            }
            res.push_str(chars.as_str());
        }
    }
    if res.is_empty() {
        clean_id.to_string()
    } else {
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_numeric_effect_id() {
        assert_eq!(map_numeric_effect_id(1), "minecraft:speed");
        assert_eq!(map_numeric_effect_id(19), "minecraft:poison");
        assert_eq!(map_numeric_effect_id(999), "minecraft:effect_999");
    }

    #[test]
    fn test_get_effect_metadata() {
        let (name, category, icon) = get_effect_metadata("minecraft:speed");
        assert_eq!(name, "Speed");
        assert_eq!(category, "beneficial");
        assert_eq!(icon, "speed");

        let (name2, category2, icon2) = get_effect_metadata("minecraft:poison");
        assert_eq!(name2, "Poison");
        assert_eq!(category2, "harmful");
        assert_eq!(icon2, "poison");

        let (name3, category3, icon3) = get_effect_metadata("custom_mod:magic_shield");
        assert_eq!(name3, "Unknown Effect");
        assert_eq!(category3, "neutral");
        assert_eq!(icon3, "unknown");
    }

    #[test]
    fn test_parse_player_effects_from_nbt() {
        let raw = RawPlayerEffectsData {
            active_effects: vec![
                RawStatusEffectTag {
                    id: Some("minecraft:speed".to_string()),
                    amplifier: Some(1),
                    duration: Some(600),
                    ..Default::default()
                },
                RawStatusEffectTag {
                    id: Some("minecraft:poison".to_string()),
                    amplifier: Some(0),
                    duration: Some(-1),
                    ..Default::default()
                },
            ],
        };

        let parsed = parse_player_effects_from_nbt(&raw);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].id, "minecraft:speed");
        assert_eq!(parsed[0].name, "Speed");
        assert_eq!(parsed[0].amplifier, 1);
        assert_eq!(parsed[0].duration_ticks, 600);
        assert_eq!(parsed[0].duration_formatted, "0:30");
        assert_eq!(parsed[0].category, "beneficial");

        assert_eq!(parsed[1].id, "minecraft:poison");
        assert_eq!(parsed[1].name, "Poison");
        assert_eq!(parsed[1].duration_formatted, "Infinite");
        assert_eq!(parsed[1].category, "harmful");
    }

    #[test]
    fn test_matches_uuid_in_place() {
        let norm = "4a3b2c1d8e9f0a1b2c3d4e5f6a7b8c9d";
        assert!(matches_uuid_in_place("4a3b2c1d-8e9f-0a1b-2c3d-4e5f6a7b8c9d", norm));
        assert!(matches_uuid_in_place("4a3b2c1d8e9f0a1b2c3d4e5f6a7b8c9d", norm));
        assert!(matches_uuid_in_place("4A3B2C1D-8E9F-0A1B-2C3D-4E5F6A7B8C9D", norm));
        assert!(!matches_uuid_in_place("4a3b2c1d-8e9f-0a1b-2c3d-4e5f6a7b8c9e", norm));
        assert!(!matches_uuid_in_place("short", norm));
    }
}
