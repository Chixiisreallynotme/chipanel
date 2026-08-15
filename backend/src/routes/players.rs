use axum::{
    extract::{Path, Query},
    response::Json,
    Extension, Json as AxumJson,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::{
    auth::middleware::AuthUser,
    config::AppConfig,
    error::AppError,
    minecraft::{
        effects::{parse_player_effects_with_dir, StatusEffectInfo},
        inventory::{parse_player_inventory_with_dir, PlayerInventory},
        player::{
            get_all_players, get_player_detail, is_valid_uuid, load_usercache, normalize_uuid,
            PlayerDetail, PlayerSummary,
        },
    },
    rcon::RconClient,
};

#[derive(Debug, Deserialize)]
pub struct PlayersQuery {
    pub query: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlayerActionRequest {
    pub uuid: String,
    pub action: String,
    pub reason: Option<String>,
    pub target_coords: Option<String>,
    pub gamemode: Option<String>,
    pub item_id: Option<String>,
    pub count: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct PlayerActionResponse {
    pub success: bool,
    pub message: String,
    pub output: String,
}

#[derive(Debug, Deserialize)]
pub struct ApplyEffectRequest {
    pub effect_id: String,
    pub duration_seconds: u32,
    pub amplifier: u8,
}

#[derive(Debug, Deserialize)]
pub struct ClearEffectsRequest {
    pub effect_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerPermissionsResponse {
    pub primary_group: Option<String>,
    pub permissions: Vec<PermissionNodeInfo>,
    pub raw_output: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PermissionNodeInfo {
    pub permission: String,
    pub value: bool,
    pub expiry: Option<String>,
    pub context: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetPlayerGroupRequest {
    pub group_name: String,
}

pub fn is_valid_player_target(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    let is_username = trimmed.len() <= 16 && trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    let is_uuid = is_valid_uuid(trimmed);

    is_username || is_uuid
}

pub fn is_valid_target_coords(coords: &str) -> bool {
    let trimmed = coords.trim();
    if trimmed.is_empty() {
        return false;
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() || parts.len() > 5 {
        return false;
    }
    for part in parts {
        if !is_valid_coord_component(part) {
            return false;
        }
    }
    true
}

fn is_valid_coord_component(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s == "~" || s == "^" {
        return true;
    }
    let s = s.strip_prefix('~').or_else(|| s.strip_prefix('^')).unwrap_or(s);
    if s.is_empty() {
        return true;
    }
    let s = s.strip_prefix('-').or_else(|| s.strip_prefix('+')).unwrap_or(s);
    if s.is_empty() {
        return false;
    }
    s.parse::<f64>().is_ok()
}

/// GET /api/players
/// Returns a list of players (online + offline) with search query and status filter support.
pub async fn list_players_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Query(query_params): Query<PlayersQuery>,
) -> Result<Json<Vec<PlayerSummary>>, AppError> {
    let players = get_all_players(
        &config,
        query_params.query.as_deref(),
        query_params.status.as_deref(),
    )
    .await?;

    Ok(Json(players))
}

/// GET /api/players/:uuid
/// Returns detailed player profile for a specific UUID.
pub async fn get_player_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(uuid): Path<String>,
) -> Result<Json<PlayerDetail>, AppError> {
    let detail = get_player_detail(&config, &uuid).await?;
    Ok(Json(detail))
}

/// POST /api/players/action
/// Executes admin player management actions via RCON (kick, ban, pardon, teleport, op, deop).
pub async fn player_action_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    AxumJson(payload): AxumJson<PlayerActionRequest>,
) -> Result<Json<PlayerActionResponse>, AppError> {
    let action_clean = payload.action.trim().to_lowercase();
    let uuid_raw = payload.uuid.trim();

    if uuid_raw.contains('\n') || uuid_raw.contains('\r') || uuid_raw.contains('\0') || action_clean.contains('\n') || action_clean.contains('\r') || action_clean.contains('\0') {
        return Err(AppError::BadRequest(
            "Invalid request: control characters not allowed".into(),
        ));
    }

    if let Some(ref reason) = payload.reason {
        if reason.contains('\n') || reason.contains('\r') || reason.contains('\0') {
            return Err(AppError::BadRequest("Control characters in reason forbidden".into()));
        }
    }

    if let Some(ref coords) = payload.target_coords {
        if coords.contains('\n') || coords.contains('\r') || coords.contains('\0') {
            return Err(AppError::BadRequest("Control characters in target_coords forbidden".into()));
        }
    }

    let norm_uuid = normalize_uuid(uuid_raw);
    let base_dir = config.minecraft_data_dir.clone();
    let norm_uuid_clone = norm_uuid.clone();
    let uuid_raw_owned = uuid_raw.to_string();

    let target_name = tokio::task::spawn_blocking(move || {
        let (uuid_to_name, _) = load_usercache(&base_dir);
        uuid_to_name
            .get(&norm_uuid_clone)
            .cloned()
            .unwrap_or(uuid_raw_owned)
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?;

    if !is_valid_player_target(&target_name) {
        return Err(AppError::BadRequest(
            "Invalid player username or UUID format".into(),
        ));
    }

    let command = match action_clean.as_str() {
        "kick" => {
            if let Some(reason) = &payload.reason {
                format!("kick {} {}", target_name, reason)
            } else {
                format!("kick {}", target_name)
            }
        }
        "ban" => {
            if let Some(reason) = &payload.reason {
                format!("ban {} {}", target_name, reason)
            } else {
                format!("ban {}", target_name)
            }
        }
        "pardon" | "unban" => {
            format!("pardon {}", target_name)
        }
        "teleport" => {
            let coords = payload
                .target_coords
                .as_deref()
                .ok_or_else(|| AppError::BadRequest("target_coords required for teleport action".into()))?;

            if !is_valid_target_coords(coords) {
                return Err(AppError::BadRequest(
                    "Invalid target_coords format".into(),
                ));
            }

            format!("tp {} {}", target_name, coords)
        }
        "op" => {
            format!("op {}", target_name)
        }
        "deop" => {
            format!("deop {}", target_name)
        }
        "gamemode" => {
            let mode = payload.gamemode.as_deref().unwrap_or("survival").trim().to_lowercase();
            match mode.as_str() {
                "survival" | "creative" | "adventure" | "spectator" => {
                    format!("gamemode {} {}", mode, target_name)
                }
                _ => {
                    return Err(AppError::BadRequest(
                        "Invalid gamemode: allowed values are survival, creative, adventure, spectator".into(),
                    ));
                }
            }
        }
        "heal" => {
            format!("effect give {} minecraft:instant_health 1 255", target_name)
        }
        "feed" => {
            format!("effect give {} minecraft:saturation 1 255", target_name)
        }
        "kill" => {
            format!("kill {}", target_name)
        }
        "clear" => {
            format!("clear {}", target_name)
        }
        "whitelist_add" => {
            format!("whitelist add {}", target_name)
        }
        "whitelist_remove" => {
            format!("whitelist remove {}", target_name)
        }
        "give" => {
            let raw_item = payload.item_id.as_deref().unwrap_or("minecraft:diamond").trim();
            if raw_item.is_empty()
                || raw_item.contains('\n')
                || raw_item.contains('\r')
                || raw_item.contains('\0')
                || raw_item.contains(' ')
            {
                return Err(AppError::BadRequest("Invalid item_id format".into()));
            }
            if !raw_item.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':' || c == '-') {
                return Err(AppError::BadRequest("Invalid item_id format".into()));
            }
            let clean_item = if !raw_item.contains(':') {
                format!("minecraft:{}", raw_item.to_lowercase())
            } else {
                raw_item.to_lowercase()
            };
            let count = payload.count.unwrap_or(1).clamp(1, 64);
            format!("give {} {} {}", target_name, clean_item, count)
        }
        _ => {
            return Err(AppError::BadRequest(
                "Invalid action. Allowed actions: kick, ban, pardon, unban, teleport, op, deop, gamemode, heal, feed, kill, clear, whitelist_add, whitelist_remove, give".into(),
            ));
        }
    };

    info!("Executing player action '{}' via RCON: '{}'", action_clean, command);

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let output = rcon_client
        .exec(&command)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON command execution failed: {}", err)))?;

    Ok(Json(PlayerActionResponse {
        success: true,
        message: format!("Action '{}' executed for player '{}'", action_clean, target_name),
        output,
    }))
}

/// GET /api/players/:uuid/inventory
/// Returns PlayerInventory for specified player UUID.
pub async fn get_player_inventory_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(uuid): Path<String>,
) -> Result<Json<PlayerInventory>, AppError> {
    if !is_valid_uuid(&uuid) {
        return Err(AppError::BadRequest("Invalid player UUID format".into()));
    }

    let base_dir = config.minecraft_data_dir.clone();
    let uuid_owned = uuid.to_string();

    let inventory = tokio::task::spawn_blocking(move || {
        parse_player_inventory_with_dir(&base_dir, &uuid_owned)
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

    Ok(Json(inventory))
}

/// GET /api/players/:uuid/effects
/// Returns active status effects list for player UUID.
pub async fn get_player_effects_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(uuid): Path<String>,
) -> Result<Json<Vec<StatusEffectInfo>>, AppError> {
    if !is_valid_uuid(&uuid) {
        return Err(AppError::BadRequest("Invalid player UUID format".into()));
    }

    let base_dir = config.minecraft_data_dir.clone();
    let uuid_owned = uuid.to_string();

    let effects = tokio::task::spawn_blocking(move || {
        parse_player_effects_with_dir(&base_dir, &uuid_owned)
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

    Ok(Json(effects))
}

/// POST /api/players/:uuid/effects/apply
/// Applies a status effect to a player via RCON (`effect give <player> <effect_id> <seconds> <amplifier>`).
pub async fn apply_player_effect_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(uuid): Path<String>,
    AxumJson(payload): AxumJson<ApplyEffectRequest>,
) -> Result<Json<PlayerActionResponse>, AppError> {
    if !is_valid_uuid(&uuid) {
        return Err(AppError::BadRequest("Invalid player UUID format".into()));
    }

    let raw_effect = payload.effect_id.trim();
    if raw_effect.is_empty()
        || raw_effect.contains('\n')
        || raw_effect.contains('\r')
        || raw_effect.contains('\0')
        || raw_effect.contains(' ')
    {
        return Err(AppError::BadRequest("Invalid effect_id format".into()));
    }

    if !raw_effect.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':' || c == '-') {
        return Err(AppError::BadRequest("Invalid effect_id format".into()));
    }

    let clean_effect = if !raw_effect.contains(':') {
        format!("minecraft:{}", raw_effect.to_lowercase())
    } else {
        raw_effect.to_lowercase()
    };

    let norm_uuid = normalize_uuid(&uuid);
    let base_dir = config.minecraft_data_dir.clone();
    let uuid_owned = uuid.to_string();

    let target_name = tokio::task::spawn_blocking(move || {
        let (uuid_to_name, _) = load_usercache(&base_dir);
        uuid_to_name
            .get(&norm_uuid)
            .cloned()
            .unwrap_or(uuid_owned)
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?;

    if !is_valid_player_target(&target_name) {
        return Err(AppError::BadRequest("Invalid player username or UUID format".into()));
    }

    let command = format!(
        "effect give {} {} {} {}",
        target_name, clean_effect, payload.duration_seconds, payload.amplifier
    );

    info!("Executing effect apply command via RCON: '{}'", command);

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let output = rcon_client
        .exec(&command)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON command execution failed: {}", err)))?;

    Ok(Json(PlayerActionResponse {
        success: true,
        message: format!("Effect '{}' applied to player '{}'", clean_effect, target_name),
        output,
    }))
}

/// POST /api/players/:uuid/effects/clear
/// Clears status effects for a player via RCON (`effect clear <player> [effect_id]`).
pub async fn clear_player_effects_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(uuid): Path<String>,
    AxumJson(payload): AxumJson<ClearEffectsRequest>,
) -> Result<Json<PlayerActionResponse>, AppError> {
    if !is_valid_uuid(&uuid) {
        return Err(AppError::BadRequest("Invalid player UUID format".into()));
    }

    let command_effect = if let Some(ref raw_effect) = payload.effect_id {
        let trimmed = raw_effect.trim();
        if !trimmed.is_empty() {
            if trimmed.contains('\n')
                || trimmed.contains('\r')
                || trimmed.contains('\0')
                || trimmed.contains(' ')
            {
                return Err(AppError::BadRequest("Invalid effect_id format".into()));
            }

            if !trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':' || c == '-') {
                return Err(AppError::BadRequest("Invalid effect_id format".into()));
            }

            let clean = if !trimmed.contains(':') {
                format!("minecraft:{}", trimmed.to_lowercase())
            } else {
                trimmed.to_lowercase()
            };
            Some(clean)
        } else {
            None
        }
    } else {
        None
    };

    let norm_uuid = normalize_uuid(&uuid);
    let base_dir = config.minecraft_data_dir.clone();
    let uuid_owned = uuid.to_string();

    let target_name = tokio::task::spawn_blocking(move || {
        let (uuid_to_name, _) = load_usercache(&base_dir);
        uuid_to_name
            .get(&norm_uuid)
            .cloned()
            .unwrap_or(uuid_owned)
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?;

    if !is_valid_player_target(&target_name) {
        return Err(AppError::BadRequest("Invalid player username or UUID format".into()));
    }

    let command = if let Some(ref effect) = command_effect {
        format!("effect clear {} {}", target_name, effect)
    } else {
        format!("effect clear {}", target_name)
    };

    info!("Executing effect clear command via RCON: '{}'", command);

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let output = rcon_client
        .exec(&command)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON command execution failed: {}", err)))?;

    Ok(Json(PlayerActionResponse {
        success: true,
        message: format!("Effects cleared for player '{}'", target_name),
        output,
    }))
}

/// GET /api/players/:uuid/permissions
/// Returns player's LuckPerms primary group and permission nodes (parsed from RCON `lp user <player> info` response).
pub async fn get_player_permissions_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(uuid): Path<String>,
) -> Result<Json<PlayerPermissionsResponse>, AppError> {
    if !is_valid_uuid(&uuid) {
        return Err(AppError::BadRequest("Invalid player UUID format".into()));
    }

    let norm_uuid = normalize_uuid(&uuid);
    let base_dir = config.minecraft_data_dir.clone();
    let uuid_owned = uuid.to_string();

    let target_name = tokio::task::spawn_blocking(move || {
        let (uuid_to_name, _) = load_usercache(&base_dir);
        uuid_to_name
            .get(&norm_uuid)
            .cloned()
            .unwrap_or(uuid_owned)
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?;

    if !is_valid_player_target(&target_name) {
        return Err(AppError::BadRequest("Invalid player username or UUID format".into()));
    }

    let command = format!("lp user {} info", target_name);
    info!("Executing LuckPerms info command via RCON: '{}'", command);

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let output = rcon_client
        .exec(&command)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON command execution failed: {}", err)))?;

    let (primary_group, permissions) = parse_luckperms_user_info(&output);

    Ok(Json(PlayerPermissionsResponse {
        primary_group,
        permissions,
        raw_output: output,
    }))
}

/// POST /api/players/:uuid/permissions/group
/// Sets player's LuckPerms parent group via RCON (`lp user <player> parent set <group_name>`).
pub async fn set_player_group_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(uuid): Path<String>,
    AxumJson(payload): AxumJson<SetPlayerGroupRequest>,
) -> Result<Json<PlayerActionResponse>, AppError> {
    if !is_valid_uuid(&uuid) {
        return Err(AppError::BadRequest("Invalid player UUID format".into()));
    }

    let group_clean = payload.group_name.trim();
    if group_clean.is_empty()
        || group_clean.contains('\n')
        || group_clean.contains('\r')
        || group_clean.contains('\0')
        || group_clean.contains(' ')
    {
        return Err(AppError::BadRequest("Invalid group_name format".into()));
    }

    if !group_clean.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(AppError::BadRequest("Invalid group_name format".into()));
    }

    let norm_uuid = normalize_uuid(&uuid);
    let base_dir = config.minecraft_data_dir.clone();
    let uuid_owned = uuid.to_string();

    let target_name = tokio::task::spawn_blocking(move || {
        let (uuid_to_name, _) = load_usercache(&base_dir);
        uuid_to_name
            .get(&norm_uuid)
            .cloned()
            .unwrap_or(uuid_owned)
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?;

    if !is_valid_player_target(&target_name) {
        return Err(AppError::BadRequest("Invalid player username or UUID format".into()));
    }

    let command = format!("lp user {} parent set {}", target_name, group_clean);
    info!("Executing LuckPerms group set command via RCON: '{}'", command);

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let output = rcon_client
        .exec(&command)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON command execution failed: {}", err)))?;

    Ok(Json(PlayerActionResponse {
        success: true,
        message: format!("Group set to '{}' for player '{}'", group_clean, target_name),
        output,
    }))
}

pub fn parse_luckperms_user_info(output: &str) -> (Option<String>, Vec<PermissionNodeInfo>) {
    let mut primary_group: Option<String> = None;
    let mut permissions: Vec<PermissionNodeInfo> = Vec::new();
    let mut in_permissions_section = false;

    for line in output.lines() {
        let clean_line = if let Some(idx) = line.find("]: ") {
            &line[idx + 3..]
        } else if let Some(idx) = line.find("] ") {
            &line[idx + 2..]
        } else {
            line
        };

        let trimmed = clean_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let lower = trimmed.to_lowercase();

        if lower.contains("primary group:") || lower.contains("primary-group:") {
            if let Some(colon_idx) = trimmed.find(':') {
                let val_part = trimmed[colon_idx + 1..].trim();
                let group_clean = if let Some(paren_idx) = val_part.find('(') {
                    val_part[..paren_idx].trim()
                } else {
                    val_part
                };
                if !group_clean.is_empty() {
                    primary_group = Some(group_clean.to_string());
                }
            }
            continue;
        }

        if lower.contains("permissions:") || lower.contains("node tree:") || lower.contains("nodes:") {
            in_permissions_section = true;
            continue;
        }

        if in_permissions_section
            && !trimmed.starts_with('-')
            && !trimmed.starts_with('>')
            && !trimmed.starts_with('+')
            && !trimmed.starts_with('*')
            && !trimmed.starts_with('•')
        {
            if trimmed.contains(':') && !trimmed.contains("(true)") && !trimmed.contains("(false)") {
                in_permissions_section = false;
            }
        }

        let is_node_line = trimmed.starts_with('-')
            || trimmed.starts_with('>')
            || trimmed.starts_with('+')
            || trimmed.starts_with('*')
            || trimmed.starts_with('•')
            || lower.contains("(true)")
            || lower.contains("(false)")
            || (in_permissions_section && trimmed.contains('.'));

        if is_node_line {
            let node_str = trimmed
                .trim_start_matches(|c| c == '-' || c == '>' || c == '+' || c == '*' || c == '•' || c == ' ')
                .trim();

            if node_str.is_empty()
                || node_str.starts_with("User Info")
                || node_str.starts_with("UUID:")
                || node_str.starts_with("Primary Group:")
            {
                continue;
            }

            let value = if node_str.contains("(false)") || node_str.contains("= false") || node_str.ends_with("false") {
                false
            } else {
                true
            };

            let mut expiry: Option<String> = None;
            let mut context: Option<String> = None;

            if let Some(exp_start) = node_str.find("expires in ") {
                let rest = &node_str[exp_start + 11..];
                let exp_val = rest.split(&[')', ']'][..]).next().unwrap_or(rest).trim();
                if !exp_val.is_empty() {
                    expiry = Some(exp_val.to_string());
                }
            } else if let Some(exp_start) = node_str.find("expiry=") {
                let rest = &node_str[exp_start + 7..];
                let exp_val = rest.split(&[')', ']', ' '][..]).next().unwrap_or(rest).trim();
                if !exp_val.is_empty() {
                    expiry = Some(exp_val.to_string());
                }
            }

            if let Some(c_start) = node_str.find('[') {
                if let Some(c_end) = node_str[c_start..].find(']') {
                    let inside = &node_str[c_start + 1..c_start + c_end];
                    if !inside.starts_with("expires") {
                        context = Some(inside.to_string());
                    }
                }
            }

            let perm_part = node_str
                .split(&['(', '=', '['][..])
                .next()
                .unwrap_or(node_str)
                .trim();

            if !perm_part.is_empty() && perm_part.contains('.') {
                permissions.push(PermissionNodeInfo {
                    permission: perm_part.to_string(),
                    value,
                    expiry,
                    context,
                });
            }
        }
    }

    (primary_group, permissions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_luckperms_user_info() {
        let sample_output = r#"[12:34:56 INFO]: User Info: Steve
[12:34:56 INFO]: - UUID: 8667ba71-b85a-4004-af54-457a973afe67
[12:34:56 INFO]: - Status: Online
[12:34:56 INFO]: - Primary Group: admin (default)
[12:34:56 INFO]: - Permissions: (count: 3)
[12:34:56 INFO]:   - chipanel.admin (true)
[12:34:56 INFO]:   - minecraft.command.teleport (true)
[12:34:56 INFO]:   - example.perm (false) (expires in 2d)"#;

        let (group, perms) = parse_luckperms_user_info(sample_output);
        assert_eq!(group, Some("admin".to_string()));
        assert_eq!(perms.len(), 3);
        assert_eq!(perms[0].permission, "chipanel.admin");
        assert_eq!(perms[0].value, true);
        assert_eq!(perms[2].permission, "example.perm");
        assert_eq!(perms[2].value, false);
        assert_eq!(perms[2].expiry, Some("2d".to_string()));
    }
}
