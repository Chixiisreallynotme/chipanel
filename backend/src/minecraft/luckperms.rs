use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{config::AppConfig, error::AppError};

/// LuckPerms stores its data in a *different* directory depending on the server
/// platform, so an engine switch (e.g. Fabric -> Paper) would silently orphan the
/// permissions unless we move them. This module backs the data up before a switch
/// and restores it into the new platform's expected location afterwards.
const BACKUPS_DIR: &str = "luckperms-backups";

/// Primary data directory for a given engine, relative to the Minecraft data dir.
fn luckperms_primary_rel(engine: &str) -> Option<&'static str> {
    match engine {
        // Fabric: `getGameDir().resolve("mods").resolve("luckperms")`.
        "FABRIC" => Some("mods/luckperms"),
        // Forge / NeoForge: `FMLPaths.CONFIGDIR.get().resolve(<modid>)`.
        "FORGE" | "NEOFORGE" => Some("config/luckperms"),
        // Bukkit-family: the plugin's data folder (plugin.yml `name: LuckPerms`).
        "PAPER" | "PURPUR" | "SPIGOT" | "BUKKIT" | "FOLIA" => Some("plugins/LuckPerms"),
        _ => None,
    }
}

fn luckperms_primary_dir(data_dir: &Path, engine: &str) -> Option<PathBuf> {
    luckperms_primary_rel(engine).map(|rel| data_dir.join(rel))
}

/// All plausible LuckPerms data locations, engine-specific first, then generic
/// fallbacks (covers unusual setups and mis-detected engines). Deduplicated while
/// preserving priority order (the primary location is checked first on backup).
fn luckperms_candidate_dirs(data_dir: &Path, engine: &str) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(p) = luckperms_primary_dir(data_dir, engine) {
        dirs.push(p);
    }
    for rel in [
        "plugins/LuckPerms",
        "mods/luckperms",
        "config/luckperms",
        "LuckPerms",
        "luckperms",
    ] {
        dirs.push(data_dir.join(rel));
    }
    let mut seen = std::collections::HashSet::new();
    dirs.retain(|d| seen.insert(d.clone()));
    dirs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuckPermsBackup {
    pub name: String,
    pub created_secs: u64,
}

fn backup_root(config: &AppConfig) -> PathBuf {
    config.data_dir.join(BACKUPS_DIR)
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Snapshots the current LuckPerms data dir (for `engine`) into a timestamped
/// folder under ChiPanel's data dir. Returns `None` when there is no data to back
/// up. Best-effort by design: callers decide how to surface a failure.
pub async fn backup_luckperms(config: &AppConfig, engine: &str) -> Result<Option<PathBuf>, AppError> {
    for dir in luckperms_candidate_dirs(&config.minecraft_data_dir, engine) {
        if !dir.exists() {
            continue;
        }
        let dest = backup_root(config).join(timestamp().to_string());
        copy_dir_all(&dir, &dest).await?;
        info!("Backed up LuckPerms data {:?} -> {:?}", dir, dest);
        return Ok(Some(dest));
    }
    Ok(None)
}

/// Restores LuckPerms data into the new engine's expected location. Uses the
/// latest backup unless `backup_name` is given. Skips (returns `Ok(false)`) when
/// there is no backup or the target already contains data — never clobbers live
/// configuration.
pub async fn restore_luckperms(
    config: &AppConfig,
    engine: &str,
    backup_name: Option<&str>,
) -> Result<bool, AppError> {
    let Some(target) = luckperms_primary_dir(&config.minecraft_data_dir, engine) else {
        return Ok(false);
    };

    let source = match backup_name {
        Some(name) => {
            let name = name.trim();
            if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
                return Err(AppError::BadRequest("Invalid backup name".to_string()));
            }
            backup_root(config).join(name)
        }
        None => match latest_backup(&backup_root(config)).await {
            Some(b) => b,
            None => return Ok(false),
        },
    };

    if !source.exists() {
        return Err(AppError::NotFound(format!(
            "LuckPerms backup '{}' not found",
            source.file_name().and_then(|n| n.to_str()).unwrap_or_default()
        )));
    }

    if dir_non_empty(&target).await {
        warn!(
            "LuckPerms target {:?} already contains data; skipping restore",
            target
        );
        return Ok(false);
    }

    copy_dir_all(&source, &target).await?;
    info!("Restored LuckPerms data {:?} -> {:?}", source, target);
    Ok(true)
}

/// Lists available LuckPerms backups, newest first.
pub async fn list_luckperms_backups(config: &AppConfig) -> Result<Vec<LuckPermsBackup>, AppError> {
    let root = backup_root(config);
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    let mut entries = tokio::fs::read_dir(&root)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read {:?}: {}", root, e)))?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue,
        };
        let created_secs = name.parse::<u64>().unwrap_or(0);
        backups.push(LuckPermsBackup { name, created_secs });
    }

    backups.sort_by_key(|b| std::cmp::Reverse(b.created_secs));
    Ok(backups)
}

async fn latest_backup(root: &Path) -> Option<PathBuf> {
    let backups = list_backup_dir_names(root).await;
    let newest = backups.into_iter().max();
    newest.map(|name| root.join(name))
}

async fn list_backup_dir_names(root: &Path) -> Vec<String> {
    if !root.exists() {
        return Vec::new();
    }
    let mut names = Vec::new();
    let mut entries = match tokio::fs::read_dir(root).await {
        Ok(e) => e,
        Err(_) => return names,
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Ok(name) = entry.file_name().into_string() {
            if name.parse::<u64>().is_ok() {
                names.push(name);
            }
        }
    }
    names
}

async fn dir_non_empty(dir: &Path) -> bool {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(e) => e,
        Err(_) => return false,
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        if entry.path().exists() {
            return true;
        }
    }
    false
}

async fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), AppError> {
    tokio::fs::create_dir_all(dst)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to create {:?}: {}", dst, e)))?;

    let mut entries = tokio::fs::read_dir(src)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read {:?}: {}", src, e)))?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let ty = entry
            .file_type()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to stat {:?}: {}", entry.path(), e)))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            Box::pin(copy_dir_all(&from, &to)).await?;
        } else {
            tokio::fs::copy(&from, &to).await.map_err(|e| {
                AppError::InternalError(format!("Failed to copy {:?} -> {:?}: {}", from, to, e))
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_data_dir_per_engine() {
        assert_eq!(luckperms_primary_rel("FABRIC"), Some("mods/luckperms"));
        assert_eq!(luckperms_primary_rel("FORGE"), Some("config/luckperms"));
        assert_eq!(luckperms_primary_rel("PAPER"), Some("plugins/LuckPerms"));
        assert_eq!(luckperms_primary_rel("PURPUR"), Some("plugins/LuckPerms"));
        assert_eq!(luckperms_primary_rel("VANILLA"), None);
    }

    #[test]
    fn candidate_dirs_are_deduped_and_prioritized() {
        let base = Path::new("/data");
        let dirs = luckperms_candidate_dirs(base, "FABRIC");
        assert_eq!(dirs.first(), Some(&base.join("mods/luckperms")));
        let unique: std::collections::HashSet<_> = dirs.iter().collect();
        assert_eq!(unique.len(), dirs.len());
    }
}
