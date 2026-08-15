use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Write as _,
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use flate2::read::GzDecoder;

use crate::{
    error::AppError,
    minecraft::player::{format_uuid, is_valid_uuid, normalize_uuid},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventorySlot {
    pub slot_index: u8,
    pub item_id: String,
    pub count: u8,
    pub damage: u32,
    pub max_damage: u32,
    pub display_name: Option<String>,
    pub enchantments: Vec<EnchantmentInfo>,
    pub is_trimmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnchantmentInfo {
    pub id: String,
    pub level: u8,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerInventory {
    pub main: Vec<Option<InventorySlot>>,
    pub armor: Vec<Option<InventorySlot>>,
    pub offhand: Option<InventorySlot>,
    pub ender_chest: Vec<Option<InventorySlot>>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case)]
pub struct RawPlayerNbtData {
    #[serde(default, alias = "inventory")]
    pub Inventory: Vec<RawItemTag>,
    #[serde(default, alias = "ender_items")]
    pub EnderItems: Vec<RawItemTag>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawItemTag {
    #[serde(default, rename = "Slot", alias = "slot")]
    pub slot: Option<i16>,
    #[serde(default, rename = "id", alias = "Id")]
    pub id: Option<String>,
    #[serde(default, rename = "Count", alias = "count")]
    pub count: Option<i64>,
    #[serde(default, rename = "Damage", alias = "damage")]
    pub damage: Option<i64>,
    #[serde(default, rename = "tag", alias = "Tag")]
    pub tag: Option<HashMap<String, fastnbt::Value>>,
    #[serde(default, rename = "components", alias = "Components")]
    pub components: Option<HashMap<String, fastnbt::Value>>,
    #[serde(default, rename = "Trim", alias = "trim")]
    pub trim: Option<fastnbt::Value>,
}

/// Reads `/app/minecraft-data/world/playerdata/{uuid}.dat` file (or from default MINECRAFT_DATA_DIR)
/// and parses the player's inventory and ender chest contents.
pub fn parse_player_inventory(uuid: &str) -> Result<PlayerInventory, AppError> {
    let base_dir_str = std::env::var("MINECRAFT_DATA_DIR")
        .unwrap_or_else(|_| "/app/minecraft-data".to_string());
    parse_player_inventory_with_dir(Path::new(&base_dir_str), uuid)
}

/// Parses the player's inventory from a specific base minecraft data directory.
pub fn parse_player_inventory_with_dir(base_dir: &Path, uuid: &str) -> Result<PlayerInventory, AppError> {
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

    let mut decompressed = Vec::new();
    let mut decoder = GzDecoder::new(&bytes[..]);
    let raw_bytes = if decoder.take(5 * 1024 * 1024).read_to_end(&mut decompressed).is_ok() && !decompressed.is_empty() {
        &decompressed[..]
    } else {
        &bytes[..]
    };

    let nbt_data: RawPlayerNbtData = fastnbt::from_bytes(raw_bytes).map_err(|e| {
        AppError::InternalError(format!("Failed to parse NBT data from {:?}: {}", file_path, e))
    })?;

    parse_inventory_nbt(&nbt_data)
}

pub fn parse_inventory_nbt(nbt_data: &RawPlayerNbtData) -> Result<PlayerInventory, AppError> {
    let mut main = vec![None; 36];
    let mut armor = vec![None; 4];
    let mut offhand: Option<InventorySlot> = None;
    let mut ender_chest = vec![None; 27];

    for item_tag in &nbt_data.Inventory {
        if let Some(slot_info) = parse_item_slot(item_tag) {
            let slot_raw = item_tag.slot.unwrap_or(-999);
            match slot_raw {
                // Hotbar (0..=8) & Main inventory grid (9..=35)
                0..=35 => {
                    let idx = slot_raw as usize;
                    if idx < main.len() {
                        main[idx] = Some(slot_info);
                    }
                }
                // Armor slots (100=boots, 101=leggings, 102=chestplate, 103=helmet)
                100 => armor[0] = Some(slot_info),
                101 => armor[1] = Some(slot_info),
                102 => armor[2] = Some(slot_info),
                103 => armor[3] = Some(slot_info),
                // Offhand (-106 or 150)
                -106 | 150 => offhand = Some(slot_info),
                _ => {}
            }
        }
    }

    for item_tag in &nbt_data.EnderItems {
        if let Some(slot_info) = parse_item_slot(item_tag) {
            let slot_raw = item_tag.slot.unwrap_or(-999);
            if (0..=26).contains(&slot_raw) {
                let idx = slot_raw as usize;
                if idx < ender_chest.len() {
                    ender_chest[idx] = Some(slot_info);
                }
            }
        }
    }

    Ok(PlayerInventory {
        main,
        armor,
        offhand,
        ender_chest,
    })
}

fn parse_item_slot(item_tag: &RawItemTag) -> Option<InventorySlot> {
    let raw_id = item_tag.id.as_deref().unwrap_or("minecraft:air");

    let item_id = if raw_id.is_empty() {
        "minecraft:air".to_string()
    } else if !raw_id.contains(':') {
        format!("minecraft:{}", raw_id)
    } else {
        raw_id.to_string()
    };

    if item_id == "minecraft:air" || item_id == "air" {
        return None;
    }

    let count = item_tag
        .count
        .map(|c| c as u8)
        .unwrap_or(1);

    if count == 0 {
        return None;
    }

    let raw_slot = item_tag.slot.unwrap_or(-999);
    let slot_index = match raw_slot {
        -106 => 150,
        s if s >= 0 => (s & 0xFF) as u8,
        _ => 0,
    };

    let damage = extract_damage(item_tag);
    let custom_max_damage = extract_custom_max_damage(item_tag);
    let max_damage = custom_max_damage.unwrap_or_else(|| get_item_max_damage(&item_id));

    let display_name = extract_display_name(item_tag);
    let enchantments = extract_enchantments(item_tag);
    let is_trimmed = check_is_trimmed(item_tag);

    Some(InventorySlot {
        slot_index,
        item_id,
        count,
        damage,
        max_damage,
        display_name,
        enchantments,
        is_trimmed,
    })
}

fn extract_damage(item_tag: &RawItemTag) -> u32 {
    if let Some(tag) = &item_tag.tag {
        if let Some(d) = tag.get("Damage").and_then(get_i64) {
            return d.max(0) as u32;
        }
    }
    if let Some(components) = &item_tag.components {
        if let Some(d) = components.get("minecraft:damage").or_else(|| components.get("damage")).and_then(get_i64) {
            return d.max(0) as u32;
        }
    }
    if let Some(d) = item_tag.damage {
        return d.max(0) as u32;
    }
    0
}

fn extract_custom_max_damage(item_tag: &RawItemTag) -> Option<u32> {
    if let Some(tag) = &item_tag.tag {
        if let Some(m) = tag.get("max_damage").or_else(|| tag.get("MaxDamage")).and_then(get_i64) {
            return Some(m.max(0) as u32);
        }
    }
    if let Some(components) = &item_tag.components {
        if let Some(m) = components.get("minecraft:max_damage").or_else(|| components.get("max_damage")).and_then(get_i64) {
            return Some(m.max(0) as u32);
        }
    }
    None
}

fn extract_display_name(item_tag: &RawItemTag) -> Option<String> {
    if let Some(tag) = &item_tag.tag {
        if let Some(display) = tag.get("display").and_then(get_compound) {
            if let Some(name_val) = display.get("Name") {
                if let Some(name_str) = get_string(name_val) {
                    return parse_custom_name_json(&name_str);
                }
            }
        }
        if let Some(name_val) = tag.get("custom_name").or_else(|| tag.get("CustomName")) {
            if let Some(name_str) = get_string(name_val) {
                return parse_custom_name_json(&name_str);
            }
        }
    }
    if let Some(components) = &item_tag.components {
        if let Some(name_val) = components.get("minecraft:custom_name").or_else(|| components.get("custom_name")) {
            if let Some(name_str) = get_string(name_val) {
                return parse_custom_name_json(&name_str);
            }
        }
    }
    None
}

pub fn parse_custom_name_json(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(trimmed) {
        match json_val {
            serde_json::Value::Object(map) => {
                let mut result = String::new();
                if let Some(text) = map.get("text").and_then(|v| v.as_str()) {
                    result.push_str(text);
                }
                if let Some(extra) = map.get("extra").and_then(|v| v.as_array()) {
                    for item in extra {
                        if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                            result.push_str(t);
                        } else if let Some(s) = item.as_str() {
                            result.push_str(s);
                        }
                    }
                }
                if !result.is_empty() {
                    return Some(result);
                }
            }
            serde_json::Value::Array(arr) => {
                let mut result = String::new();
                for item in arr {
                    if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                        result.push_str(t);
                    } else if let Some(s) = item.as_str() {
                        result.push_str(s);
                    }
                }
                if !result.is_empty() {
                    return Some(result);
                }
            }
            serde_json::Value::String(s) => {
                if !s.is_empty() {
                    return Some(s);
                }
            }
            _ => {}
        }
    }

    let unquoted = trimmed.trim_matches('"');
    if !unquoted.is_empty() {
        Some(unquoted.to_string())
    } else {
        None
    }
}

fn extract_enchantments(item_tag: &RawItemTag) -> Vec<EnchantmentInfo> {
    let mut results = Vec::new();

    let mut process_ench_list = |list: &[fastnbt::Value]| {
        for entry in list {
            if let Some(ench_map) = get_compound(entry) {
                let raw_id = ench_map.get("id").and_then(get_string).unwrap_or_default();
                let level = ench_map
                    .get("lvl")
                    .or_else(|| ench_map.get("level"))
                    .or_else(|| ench_map.get("Level"))
                    .and_then(get_i64)
                    .map(|l| l.clamp(0, 255) as u8)
                    .unwrap_or(1);

                let id = if let Ok(num_id) = raw_id.parse::<u32>() {
                    map_numeric_enchantment_id(num_id)
                } else if !raw_id.is_empty() && !raw_id.contains(':') {
                    format!("minecraft:{}", raw_id)
                } else {
                    raw_id
                };

                if !id.is_empty() {
                    let name = format_enchantment_name(&id);
                    results.push(EnchantmentInfo { id, level, name });
                }
            }
        }
    };

    if let Some(tag) = &item_tag.tag {
        if let Some(ench_val) = tag.get("Enchantments").or_else(|| tag.get("ench")).or_else(|| tag.get("StoredEnchantments")) {
            if let Some(list) = get_list(ench_val) {
                process_ench_list(list);
            }
        }
    }

    if let Some(components) = &item_tag.components {
        if let Some(ench_val) = components.get("minecraft:enchantments").or_else(|| components.get("enchantments")) {
            if let Some(list) = get_list(ench_val) {
                process_ench_list(list);
            } else if let Some(levels_map) = get_compound(ench_val) {
                if let Some(levels) = levels_map.get("levels").and_then(get_compound) {
                    for (ench_id, level_val) in levels {
                        let level = get_i64(level_val).map(|l| l.clamp(0, 255) as u8).unwrap_or(1);
                        let id = if !ench_id.contains(':') {
                            format!("minecraft:{}", ench_id)
                        } else {
                            ench_id.clone()
                        };
                        let name = format_enchantment_name(&id);
                        results.push(EnchantmentInfo { id, level, name });
                    }
                }
            }
        }
    }

    results
}

fn map_numeric_enchantment_id(id: u32) -> String {
    let name = match id {
        0 => "protection",
        1 => "fire_protection",
        2 => "feather_falling",
        3 => "blast_protection",
        4 => "projectile_protection",
        5 => "respiration",
        6 => "aqua_affinity",
        7 => "thorns",
        8 => "depth_strider",
        9 => "frost_walker",
        10 => "binding_curse",
        16 => "sharpness",
        17 => "smite",
        18 => "bane_of_arthropods",
        19 => "knockback",
        20 => "fire_aspect",
        21 => "looting",
        22 => "sweeping",
        32 => "efficiency",
        33 => "silk_touch",
        34 => "unbreaking",
        35 => "fortune",
        48 => "power",
        49 => "punch",
        50 => "flame",
        51 => "infinity",
        61 => "luck_of_the_sea",
        62 => "lure",
        70 => "mending",
        71 => "vanishing_curse",
        _ => return format!("minecraft:enchantment_{}", id),
    };
    format!("minecraft:{}", name)
}

pub fn format_enchantment_name(id: &str) -> String {
    let clean_id = id.strip_prefix("minecraft:").unwrap_or(id);
    let mapped = match clean_id {
        "protection" => "Protection",
        "fire_protection" => "Fire Protection",
        "feather_falling" => "Feather Falling",
        "blast_protection" => "Blast Protection",
        "projectile_protection" => "Projectile Protection",
        "respiration" => "Respiration",
        "aqua_affinity" => "Aqua Affinity",
        "thorns" => "Thorns",
        "depth_strider" => "Depth Strider",
        "frost_walker" => "Frost Walker",
        "binding_curse" => "Curse of Binding",
        "soul_speed" => "Soul Speed",
        "swift_sneak" => "Swift Sneak",
        "sharpness" => "Sharpness",
        "smite" => "Smite",
        "bane_of_arthropods" => "Bane of Arthropods",
        "knockback" => "Knockback",
        "fire_aspect" => "Fire Aspect",
        "looting" => "Looting",
        "sweeping" => "Sweeping Edge",
        "efficiency" => "Efficiency",
        "silk_touch" => "Silk Touch",
        "unbreaking" => "Unbreaking",
        "fortune" => "Fortune",
        "power" => "Power",
        "punch" => "Punch",
        "flame" => "Flame",
        "infinity" => "Infinity",
        "luck_of_the_sea" => "Luck of the Sea",
        "lure" => "Lure",
        "loyalty" => "Loyalty",
        "impaling" => "Impaling",
        "riptide" => "Riptide",
        "channeling" => "Channeling",
        "multishot" => "Multishot",
        "quick_charge" => "Quick Charge",
        "piercing" => "Piercing",
        "density" => "Density",
        "breach" => "Breach",
        "wind_burst" => "Wind Burst",
        "mending" => "Mending",
        "vanishing_curse" => "Curse of Vanishing",
        _ => "",
    };

    if !mapped.is_empty() {
        return mapped.to_string();
    }

    let mut res = String::with_capacity(clean_id.len());
    for (i, word) in clean_id.split('_').enumerate() {
        if i > 0 {
            let _ = write!(res, " ");
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            for c in first.to_uppercase() {
                let _ = write!(res, "{}", c);
            }
            let _ = write!(res, "{}", chars.as_str());
        }
    }
    res
}

fn check_is_trimmed(item_tag: &RawItemTag) -> bool {
    if let Some(tag) = &item_tag.tag {
        if tag.contains_key("Trim") || tag.contains_key("trim") {
            return true;
        }
    }
    if let Some(components) = &item_tag.components {
        if components.contains_key("minecraft:trim") || components.contains_key("trim") {
            return true;
        }
    }
    if item_tag.trim.is_some() {
        return true;
    }
    false
}

pub fn get_item_max_damage(item_id: &str) -> u32 {
    let clean = item_id.strip_prefix("minecraft:").unwrap_or(item_id);
    match clean {
        "netherite_sword" | "netherite_pickaxe" | "netherite_axe" | "netherite_shovel" | "netherite_hoe" => 2031,
        "diamond_sword" | "diamond_pickaxe" | "diamond_axe" | "diamond_shovel" | "diamond_hoe" => 1561,
        "iron_sword" | "iron_pickaxe" | "iron_axe" | "iron_shovel" | "iron_hoe" => 250,
        "stone_sword" | "stone_pickaxe" | "stone_axe" | "stone_shovel" | "stone_hoe" => 131,
        "wooden_sword" | "wooden_pickaxe" | "wooden_axe" | "wooden_shovel" | "wooden_hoe" => 59,
        "golden_sword" | "golden_pickaxe" | "golden_axe" | "golden_shovel" | "golden_hoe" => 32,

        "netherite_helmet" => 407,
        "netherite_chestplate" => 592,
        "netherite_leggings" => 555,
        "netherite_boots" => 481,

        "diamond_helmet" => 363,
        "diamond_chestplate" => 528,
        "diamond_leggings" => 495,
        "diamond_boots" => 429,

        "iron_helmet" => 165,
        "iron_chestplate" => 240,
        "iron_leggings" => 225,
        "iron_boots" => 195,

        "chainmail_helmet" => 165,
        "chainmail_chestplate" => 240,
        "chainmail_leggings" => 225,
        "chainmail_boots" => 195,

        "golden_helmet" => 77,
        "golden_chestplate" => 112,
        "golden_leggings" => 105,
        "golden_boots" => 91,

        "leather_helmet" => 55,
        "leather_chestplate" => 80,
        "leather_leggings" => 75,
        "leather_boots" => 65,

        "turtle_helmet" => 275,

        "elytra" => 432,
        "shield" => 336,
        "bow" => 384,
        "crossbow" => 465,
        "trident" => 250,
        "fishing_rod" => 64,
        "flint_and_steel" => 64,
        "shears" => 238,
        "carrot_on_a_stick" => 25,
        "warped_fungus_on_a_stick" => 100,
        "brush" => 64,
        "mace" => 500,

        _ => 0,
    }
}

fn get_string(value: &fastnbt::Value) -> Option<String> {
    match value {
        fastnbt::Value::String(s) => Some(s.clone()),
        fastnbt::Value::Byte(b) => Some(b.to_string()),
        fastnbt::Value::Short(s) => Some(s.to_string()),
        fastnbt::Value::Int(i) => Some(i.to_string()),
        fastnbt::Value::Long(l) => Some(l.to_string()),
        _ => None,
    }
}

fn get_i64(value: &fastnbt::Value) -> Option<i64> {
    match value {
        fastnbt::Value::Byte(b) => Some(*b as i64),
        fastnbt::Value::Short(s) => Some(*s as i64),
        fastnbt::Value::Int(i) => Some(*i as i64),
        fastnbt::Value::Long(l) => Some(*l),
        _ => None,
    }
}

fn get_compound(value: &fastnbt::Value) -> Option<&HashMap<String, fastnbt::Value>> {
    match value {
        fastnbt::Value::Compound(map) => Some(map),
        _ => None,
    }
}

fn get_list(value: &fastnbt::Value) -> Option<&Vec<fastnbt::Value>> {
    match value {
        fastnbt::Value::List(list) => Some(list),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_enchantment_name() {
        assert_eq!(format_enchantment_name("minecraft:sharpness"), "Sharpness");
        assert_eq!(format_enchantment_name("fire_aspect"), "Fire Aspect");
        assert_eq!(format_enchantment_name("sweeping"), "Sweeping Edge");
        assert_eq!(format_enchantment_name("binding_curse"), "Curse of Binding");
        assert_eq!(format_enchantment_name("some_custom_enchantment"), "Some Custom Enchantment");
    }

    #[test]
    fn test_parse_custom_name_json() {
        let json_obj = r#"{"text":"Excalibur","color":"gold"}"#;
        assert_eq!(parse_custom_name_json(json_obj), Some("Excalibur".to_string()));

        let json_arr = r#"[{"text":"Super "},{"text":"Sword"}]"#;
        assert_eq!(parse_custom_name_json(json_arr), Some("Super Sword".to_string()));

        let plain_str = "Custom Item";
        assert_eq!(parse_custom_name_json(plain_str), Some("Custom Item".to_string()));
    }

    #[test]
    fn test_get_item_max_damage() {
        assert_eq!(get_item_max_damage("minecraft:diamond_sword"), 1561);
        assert_eq!(get_item_max_damage("netherite_chestplate"), 592);
        assert_eq!(get_item_max_damage("minecraft:apple"), 0);
    }

    #[test]
    fn test_parse_inventory_nbt_struct() {
        let raw_data = RawPlayerNbtData {
            Inventory: vec![RawItemTag {
                slot: Some(0),
                id: Some("minecraft:diamond_sword".to_string()),
                count: Some(1),
                damage: Some(10),
                tag: None,
                components: None,
                trim: None,
            }],
            EnderItems: vec![],
        };
        let inv = parse_inventory_nbt(&raw_data).unwrap();
        assert!(inv.main[0].is_some());
        let item = inv.main[0].as_ref().unwrap();
        assert_eq!(item.item_id, "minecraft:diamond_sword");
        assert_eq!(item.damage, 10);
        assert_eq!(item.max_damage, 1561);
    }
}

