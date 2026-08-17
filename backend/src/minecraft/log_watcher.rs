use std::collections::{HashMap, HashSet};
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::config::AppConfig;
use crate::models::websocket::WsServerMessage;

/// Extracts a player's username from a standard Minecraft join log line.
pub fn extract_player_join_from_log_line(line: &str) -> Option<String> {
    // Strip log timestamp / thread prefix (e.g. "[12:00:00 INFO]: ...")
    let clean = if let Some(idx) = line.rfind("]: ") {
        &line[idx + 3..]
    } else if let Some(idx) = line.rfind("] ") {
        &line[idx + 2..]
    } else if let Some(idx) = line.rfind(": ") {
        &line[idx + 2..]
    } else {
        line
    };

    let trimmed = clean.trim();

    // Pattern 1: "<Player> joined the game"
    if let Some(pos) = trimmed.find(" joined the game") {
        let prefix = trimmed[..pos].trim();
        let name = prefix.split_whitespace().last().unwrap_or(prefix);
        let clean_name = name.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_');
        if !clean_name.is_empty() && clean_name.len() <= 32 {
            return Some(clean_name.to_string());
        }
    }

    // Pattern 2: "<Player>[/<IP>] logged in with entity id"
    if let Some(pos) = trimmed.find(" logged in with entity id") {
        let prefix = trimmed[..pos].trim();
        let name_part = prefix.split('[').next().unwrap_or(prefix).trim();
        let clean_name = name_part.split_whitespace().last().unwrap_or(name_part);
        let clean_name = clean_name.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_');
        if !clean_name.is_empty() && clean_name.len() <= 32 {
            return Some(clean_name.to_string());
        }
    }

    // Pattern 3: "UUID of player <Player> is <UUID>"
    if let Some(pos) = trimmed.find("UUID of player ") {
        let rest = &trimmed[pos + 15..];
        if let Some(is_pos) = rest.find(" is ") {
            let name = rest[..is_pos].trim();
            let clean_name = name.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_');
            if !clean_name.is_empty() && clean_name.len() <= 32 {
                return Some(clean_name.to_string());
            }
        }
    }

    None
}

/// Spawns a real-time background file watcher on `logs/latest.log` that detects player joins in <50ms
/// and immediately triggers pending deferred commands without waiting for the 2-second telemetry tick.
pub fn spawn_instant_join_log_watcher(
    config: Arc<AppConfig>,
    tx: broadcast::Sender<WsServerMessage>,
) {
    tokio::spawn(async move {
        let log_path = PathBuf::from(&config.minecraft_data_dir).join("logs").join("latest.log");
        info!("Instant player join watcher initialized for {:?}", log_path);

        let mut last_file_id: Option<(u64, u64)> = None; // (inode/dev or len tracking)
        let mut current_offset: u64 = 0;

        loop {
            // Check if log file exists
            if !log_path.exists() {
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }

            let mut file = match File::open(&log_path).await {
                Ok(f) => f,
                Err(e) => {
                    debug!("Could not open log file for watching: {}", e);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    continue;
                }
            };

            // On first discovery, seek to current end of file to ignore past joins
            if last_file_id.is_none() {
                if let Ok(meta) = file.metadata().await {
                    current_offset = meta.len();
                    let _ = file.seek(SeekFrom::Start(current_offset)).await;
                    last_file_id = Some((meta.len(), 0));
                }
            } else {
                // If file size shrunk (log rotation / truncation), reset offset
                if let Ok(meta) = file.metadata().await {
                    if meta.len() < current_offset {
                        current_offset = 0;
                    }
                }
                let _ = file.seek(SeekFrom::Start(current_offset)).await;
            }

            let mut reader = BufReader::new(file);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        // EOF reached, wait 50ms before checking for new appended lines
                        tokio::time::sleep(Duration::from_millis(50)).await;

                        // Check if file was rotated or deleted
                        if let Ok(meta) = tokio::fs::metadata(&log_path).await {
                            if meta.len() < current_offset {
                                break; // file truncated/rotated -> re-open
                            }
                        }
                    }
                    Ok(bytes_read) => {
                        current_offset += bytes_read as u64;

                        if let Some(player_name) = extract_player_join_from_log_line(&line) {
                            if crate::minecraft::command_queue::has_pending_commands(&config.data_dir) {
                                debug!("Instant log watcher detected player join: '{}'", player_name);
                                let mut online_set = HashSet::new();
                                online_set.insert(player_name.to_lowercase());

                                if let Ok(executed) =
                                    crate::minecraft::command_queue::process_pending_commands(&config, &online_set).await
                                {
                                    if !executed.is_empty() {
                                        info!(
                                            "Instant execution triggered for {} commands upon player '{}' join",
                                            executed.len(),
                                            player_name
                                        );
                                        let mut per_player: HashMap<String, Vec<String>> = HashMap::new();
                                        for cmd in executed {
                                            per_player.entry(cmd.player_name).or_default().push(cmd.action);
                                        }
                                        for (pname, actions) in per_player {
                                            let _ = tx.send(WsServerMessage::PendingCommandsExecuted {
                                                player_name: pname,
                                                commands_count: actions.len(),
                                                actions,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Error reading log line: {}", e);
                        tokio::time::sleep(Duration::from_millis(200)).await;
                        break;
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_player_join_from_log_line() {
        // Standard vanilla / paper
        assert_eq!(
            extract_player_join_from_log_line("[14:23:45 INFO]: Chixi_ joined the game"),
            Some("Chixi_".to_string())
        );

        assert_eq!(
            extract_player_join_from_log_line("[14:23:45] [Server thread/INFO]: Player123 joined the game"),
            Some("Player123".to_string())
        );

        assert_eq!(
            extract_player_join_from_log_line("[14:23:45] [Server thread/INFO] [minecraft/MinecraftServer]: Notch joined the game"),
            Some("Notch".to_string())
        );

        // Entity id login
        assert_eq!(
            extract_player_join_from_log_line("[14:23:45 INFO]: Alex[/192.168.1.100:54321] logged in with entity id 123 at (0.0, 64.0, 0.0)"),
            Some("Alex".to_string())
        );

        // UUID log
        assert_eq!(
            extract_player_join_from_log_line("[14:23:45 INFO]: UUID of player Steve is 11111111-2222-3333-4444-555555555555"),
            Some("Steve".to_string())
        );

        // Non-join lines
        assert_eq!(
            extract_player_join_from_log_line("[14:23:45 INFO]: Chixi_ left the game"),
            None
        );
        assert_eq!(
            extract_player_join_from_log_line("[14:23:45 INFO]: Unknown command. Type \"/help\" for help."),
            None
        );
    }
}
