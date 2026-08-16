use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::config::AppConfig;
use crate::error::AppError;
use crate::minecraft::player::normalize_uuid;
use crate::rcon::client::RconClient;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PendingCommand {
    pub id: String,
    pub player_uuid: String,
    pub player_name: String,
    pub action: String,
    pub command: String,
    pub created_at: u64,
    pub status: String, // "pending" | "executed" | "failed"
    pub executed_at: Option<u64>,
    pub execution_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CommandQueueStore {
    pub pending: Vec<PendingCommand>,
    pub history: Vec<PendingCommand>,
}

/// In-memory cache of the command queue to avoid repeated disk reads on every telemetry tick (every 2s).
static QUEUE_CACHE: OnceLock<RwLock<Option<(PathBuf, CommandQueueStore)>>> = OnceLock::new();

fn get_queue_cache() -> &'static RwLock<Option<(PathBuf, CommandQueueStore)>> {
    QUEUE_CACHE.get_or_init(|| RwLock::new(None))
}

fn get_queue_file_path(data_dir: &Path) -> PathBuf {
    data_dir.join("pending_commands.json")
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn generate_command_id() -> String {
    use argon2::password_hash::rand_core::{OsRng, RngCore};
    let mut bytes = [0u8; 8];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Loads the persistent command queue from memory cache or disk.
/// If cached for the given `data_dir`, returns immediately without disk I/O.
pub fn load_queue(data_dir: &Path) -> CommandQueueStore {
    // Fast path: read from in-memory cache
    if let Ok(guard) = get_queue_cache().read() {
        if let Some((ref cached_path, ref store)) = *guard {
            if cached_path == data_dir {
                return store.clone();
            }
        }
    }

    // Slow path: load from disk on first startup or cache miss
    let path = get_queue_file_path(data_dir);
    let store = if !path.exists() {
        CommandQueueStore::default()
    } else {
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<CommandQueueStore>(&content) {
                Ok(s) => s,
                Err(e) => {
                    warn!("Failed to parse pending_commands.json: {}. Returning empty queue.", e);
                    CommandQueueStore::default()
                }
            },
            Err(e) => {
                warn!("Failed to read pending_commands.json: {}. Returning empty queue.", e);
                CommandQueueStore::default()
            }
        }
    };

    // Update cache
    if let Ok(mut guard) = get_queue_cache().write() {
        *guard = Some((data_dir.to_path_buf(), store.clone()));
    }

    store
}

/// Checks if there are any pending commands in memory with zero disk I/O.
pub fn has_pending_commands(data_dir: &Path) -> bool {
    let store = load_queue(data_dir);
    !store.pending.is_empty()
}

/// Atomically saves the command queue to disk and updates the in-memory cache.
pub fn save_queue(data_dir: &Path, store: &CommandQueueStore) -> Result<(), AppError> {
    if !data_dir.exists() {
        let _ = fs::create_dir_all(data_dir);
    }

    let target_path = get_queue_file_path(data_dir);
    let temp_path = data_dir.join(format!("pending_commands_{}.tmp", generate_command_id()));

    let json_bytes = serde_json::to_vec_pretty(store)
        .map_err(|e| AppError::InternalError(format!("Failed to serialize command queue: {}", e)))?;

    fs::write(&temp_path, json_bytes)
        .map_err(|e| AppError::InternalError(format!("Failed to write temporary command queue file: {}", e)))?;

    fs::rename(&temp_path, &target_path)
        .map_err(|e| AppError::InternalError(format!("Failed to commit command queue file atomically: {}", e)))?;

    // Update in-memory cache
    if let Ok(mut guard) = get_queue_cache().write() {
        *guard = Some((data_dir.to_path_buf(), store.clone()));
    }

    Ok(())
}

/// Enqueues a new command for a player when they are offline or when the server is unavailable.
pub fn enqueue_command(
    data_dir: &Path,
    player_uuid: &str,
    player_name: &str,
    action: &str,
    command: &str,
) -> Result<PendingCommand, AppError> {
    let mut store = load_queue(data_dir);

    let item = PendingCommand {
        id: generate_command_id(),
        player_uuid: player_uuid.to_string(),
        player_name: player_name.to_string(),
        action: action.to_string(),
        command: command.to_string(),
        created_at: current_timestamp(),
        status: "pending".to_string(),
        executed_at: None,
        execution_output: None,
    };

    store.pending.push(item.clone());
    save_queue(data_dir, &store)?;

    info!(
        "Enqueued pending command '{}' ({}) for player '{}' ({})",
        item.action, item.command, item.player_name, item.player_uuid
    );

    Ok(item)
}

/// Returns all pending commands for a specific player (matching normalized UUID or lowercase username).
pub fn get_pending_for_player(data_dir: &Path, player_uuid_or_name: &str) -> Vec<PendingCommand> {
    let store = load_queue(data_dir);
    let norm_target = normalize_uuid(player_uuid_or_name);
    let lower_target = player_uuid_or_name.trim().to_lowercase();

    store
        .pending
        .into_iter()
        .filter(|cmd| {
            if !norm_target.is_empty() && normalize_uuid(&cmd.player_uuid) == norm_target {
                return true;
            }
            cmd.player_name.trim().to_lowercase() == lower_target
        })
        .collect()
}

/// Returns all currently pending commands.
pub fn get_all_pending(data_dir: &Path) -> Vec<PendingCommand> {
    load_queue(data_dir).pending
}

/// Returns the execution history of completed or failed queued commands (up to last 100 items).
pub fn get_history(data_dir: &Path) -> Vec<PendingCommand> {
    load_queue(data_dir).history
}

/// Deletes/cancels a pending command by ID. Returns true if found and removed.
pub fn delete_pending_command(data_dir: &Path, command_id: &str) -> Result<bool, AppError> {
    let mut store = load_queue(data_dir);
    let orig_len = store.pending.len();
    store.pending.retain(|cmd| cmd.id != command_id);

    if store.pending.len() < orig_len {
        save_queue(data_dir, &store)?;
        info!("Cancelled pending command ID '{}'", command_id);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Checks online players against the pending command queue, connects to RCON, and executes queued actions.
/// Returns the list of successfully executed commands.
pub async fn process_pending_commands(
    config: &AppConfig,
    online_players: &HashSet<String>,
) -> Result<Vec<PendingCommand>, AppError> {
    if online_players.is_empty() {
        return Ok(Vec::new());
    }

    let mut store = load_queue(&config.data_dir);
    if store.pending.is_empty() {
        return Ok(Vec::new());
    }

    let mut commands_to_run = Vec::new();
    let mut remaining_pending = Vec::new();

    for cmd in store.pending {
        let norm_u = normalize_uuid(&cmd.player_uuid);
        let name_lower = cmd.player_name.trim().to_lowercase();

        let is_online = (!norm_u.is_empty() && online_players.contains(&norm_u))
            || (!name_lower.is_empty() && online_players.contains(&name_lower));

        if is_online {
            commands_to_run.push(cmd);
        } else {
            remaining_pending.push(cmd);
        }
    }

    if commands_to_run.is_empty() {
        return Ok(Vec::new());
    }

    info!(
        "Found {} pending commands ready to execute for online players",
        commands_to_run.len()
    );

    let mut rcon = match RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password).await {
        Ok(client) => client,
        Err(e) => {
            warn!("Could not connect to RCON to process pending commands: {}", e);
            return Ok(Vec::new());
        }
    };

    let mut executed_commands = Vec::new();

    for mut cmd in commands_to_run {
        info!(
            "Executing deferred command for player '{}': '{}'",
            cmd.player_name, cmd.command
        );

        match rcon.exec(&cmd.command).await {
            Ok(output) => {
                info!("Deferred command succeeded: {}", output);
                cmd.status = "executed".to_string();
                cmd.executed_at = Some(current_timestamp());
                cmd.execution_output = Some(output);
                executed_commands.push(cmd.clone());
                store.history.insert(0, cmd);
            }
            Err(e) => {
                error!("Deferred command execution failed via RCON: {}", e);
                cmd.status = "failed".to_string();
                cmd.executed_at = Some(current_timestamp());
                cmd.execution_output = Some(format!("Error: {}", e));
                executed_commands.push(cmd.clone());
                store.history.insert(0, cmd);
            }
        }
    }

    // Keep history capped to last 150 items
    if store.history.len() > 150 {
        store.history.truncate(150);
    }

    store.pending = remaining_pending;
    save_queue(&config.data_dir, &store)?;

    Ok(executed_commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("chipanel_test_{}_{}", name, generate_command_id()));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_enqueue_and_load_queue() {
        let dir = create_test_dir("enqueue");
        let data_dir = dir.as_path();

        let initial = load_queue(data_dir);
        assert!(initial.pending.is_empty());
        assert!(initial.history.is_empty());

        let cmd = enqueue_command(
            data_dir,
            "68c3afd5-5d04-4afa-bd54-5f65c14f64f1",
            "Chixi_",
            "give",
            "give Chixi_ minecraft:diamond 64",
        )
        .unwrap();

        assert_eq!(cmd.player_name, "Chixi_");
        assert_eq!(cmd.action, "give");
        assert_eq!(cmd.status, "pending");

        let pending = get_pending_for_player(data_dir, "68c3afd5-5d04-4afa-bd54-5f65c14f64f1");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, "give Chixi_ minecraft:diamond 64");

        let all = get_all_pending(data_dir);
        assert_eq!(all.len(), 1);

        let deleted = delete_pending_command(data_dir, &cmd.id).unwrap();
        assert!(deleted);

        let after_del = get_all_pending(data_dir);
        assert!(after_del.is_empty());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_get_pending_by_username() {
        let dir = create_test_dir("username");
        let data_dir = dir.as_path();

        enqueue_command(
            data_dir,
            "11111111-2222-3333-4444-555555555555",
            "Steve",
            "gamemode",
            "gamemode creative Steve",
        )
        .unwrap();

        let pending = get_pending_for_player(data_dir, "steve");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].action, "gamemode");

        let _ = fs::remove_dir_all(dir);
    }
}
