use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::config::AppConfig;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerBackupMetadata {
    pub filename: String,
    pub file_size_bytes: u64,
    pub created_at_secs: u64,
    pub scope: String, // "full" | "world_only" | "configs_only"
    pub sha256: String,
    pub format: String, // "zip" | "zstd"
    pub file_count: usize,
    pub is_locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3BackupConfig {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub path_prefix: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerBackupSettings {
    pub max_retention_count: usize,
    pub default_exclusions: Vec<String>,
    pub compression_level: i32,
    pub s3_config: Option<S3BackupConfig>,
    pub locked_backups: Vec<String>,
}

impl Default for ServerBackupSettings {
    fn default() -> Self {
        Self {
            max_retention_count: 7,
            default_exclusions: vec![
                "*.log.gz".to_string(),
                "logs/*".to_string(),
                "cache/*".to_string(),
                "backups/*".to_string(),
                "dynmap/web/tiles/*".to_string(),
                "bluemap/web/maps/*".to_string(),
                "crash-reports/*".to_string(),
                ".fabric/*".to_string(),
            ],
            compression_level: 3,
            s3_config: None,
            locked_backups: Vec::new(),
        }
    }
}

/// RAII Guard that pauses in-memory world writes, triggers a flush, and automatically reenables them
pub struct RconSaveGuard {
    rcon: Option<crate::rcon::RconActorHandle>,
}

impl RconSaveGuard {
    pub async fn acquire(rcon: Option<crate::rcon::RconActorHandle>) -> Self {
        if let Some(ref r) = rcon {
            info!("RconSaveGuard: Disabling world auto-save and flushing chunks...");
            let _ = r.exec("save-off").await;
            let _ = r.exec("save-all flush").await;
        }
        Self { rcon }
    }

    pub async fn release(mut self) {
        if let Some(r) = self.rcon.take() {
            info!("RconSaveGuard: Re-enabling world auto-save...");
            let _ = r.exec("save-on").await;
        }
    }
}

impl Drop for RconSaveGuard {
    fn drop(&mut self) {
        if let Some(r) = self.rcon.take() {
            tokio::spawn(async move {
                let _ = r.exec("save-on").await;
            });
        }
    }
}

pub fn get_backups_dir(config: &AppConfig) -> PathBuf {
    config.minecraft_data_dir.join("backups")
}

pub fn get_settings_file(config: &AppConfig) -> PathBuf {
    config.data_dir.join("backup_settings.json")
}

pub async fn load_backup_settings(config: &AppConfig) -> ServerBackupSettings {
    let path = get_settings_file(config);
    if path.exists() {
        if let Ok(data) = tokio::fs::read_to_string(&path).await {
            if let Ok(settings) = serde_json::from_str::<ServerBackupSettings>(&data) {
                return settings;
            }
        }
    }
    ServerBackupSettings::default()
}

pub async fn save_backup_settings(config: &AppConfig, settings: &ServerBackupSettings) -> Result<(), AppError> {
    let path = get_settings_file(config);
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| AppError::InternalError(format!("Failed to serialize backup settings: {}", e)))?;
    tokio::fs::write(&path, json)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to save backup settings: {}", e)))?;
    Ok(())
}

/// Checks if a relative file path matches any exclusion pattern
pub fn is_path_excluded(rel_path: &str, exclusions: &[String]) -> bool {
    let normalized = rel_path.replace('\\', "/");
    let trimmed = normalized.trim_start_matches('/');

    for pattern in exclusions {
        let pat = pattern.replace('\\', "/");
        let pat_trimmed = pat.trim_start_matches('/');

        if let Some(prefix) = pat_trimmed.strip_suffix("/*") {
            if trimmed.starts_with(prefix) {
                return true;
            }
        } else if pat_trimmed.starts_with("*.") {
            let ext = &pat_trimmed[1..];
            if trimmed.ends_with(ext) {
                return true;
            }
        } else if trimmed == pat_trimmed || trimmed.starts_with(&format!("{}/", pat_trimmed)) {
            return true;
        }
    }
    false
}

#[derive(Debug, Deserialize)]
pub struct CreateBackupOptions {
    pub name: Option<String>,
    pub scope: Option<String>, // "full" | "world_only" | "configs_only"
    pub format: Option<String>, // "zip" | "zstd"
    pub extra_exclusions: Option<Vec<String>>,
}

/// Creates a high-performance compressed backup of the Minecraft server
pub async fn create_server_backup(
    config: &AppConfig,
    rcon: Option<crate::rcon::RconActorHandle>,
    options: CreateBackupOptions,
) -> Result<ServerBackupMetadata, AppError> {
    let settings = load_backup_settings(config).await;
    let backups_dir = get_backups_dir(config);
    tokio::fs::create_dir_all(&backups_dir)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to create backups directory: {}", e)))?;

    let scope = options.scope.unwrap_or_else(|| "full".to_string());
    let format_choice = options.format.unwrap_or_else(|| "zstd".to_string());
    let timestamp_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let clean_custom_name = options
        .name
        .map(|n| {
            n.chars()
                .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect::<String>()
        })
        .filter(|n| !n.is_empty());

    let filename = match clean_custom_name {
        Some(name) => format!("{}_{}_{}.zip", name, scope, timestamp_secs),
        None => format!("backup_{}_{}.zip", scope, timestamp_secs),
    };

    let target_path = backups_dir.join(&filename);
    let source_dir = config.minecraft_data_dir.clone();

    let mut exclusions = settings.default_exclusions.clone();
    if let Some(extra) = options.extra_exclusions {
        exclusions.extend(extra);
    }
    exclusions.push("backups/*".to_string());

    let scope_clone = scope.clone();
    let target_path_clone = target_path.clone();
    let format_choice_clone = format_choice.clone();

    // Acquire RCON save-off & flush guard
    let guard = RconSaveGuard::acquire(rcon).await;

    // Perform heavy file scanning, hashing and compression in a blocking thread
    let (file_count, file_size_bytes, sha256_hex) = tokio::task::spawn_blocking(move || -> Result<(usize, u64, String), AppError> {
        let file = File::create(&target_path_clone)
            .map_err(|e| AppError::InternalError(format!("Failed to create backup target file: {}", e)))?;

        let mut zip = ZipWriter::new(file);
        let compression = if format_choice_clone == "zstd" {
            CompressionMethod::Zstd
        } else {
            CompressionMethod::Deflated
        };

        let zip_options = SimpleFileOptions::default()
            .compression_method(compression)
            .unix_permissions(0o644);

        let mut count = 0;
        let mut buffer = vec![0u8; 64 * 1024];

        for entry in WalkDir::new(&source_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path == target_path_clone || path == source_dir {
                continue;
            }

            let rel_path = match path.strip_prefix(&source_dir) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => continue,
            };

            if is_path_excluded(&rel_path, &exclusions) {
                continue;
            }

            if scope_clone == "world_only" {
                let first_segment = rel_path.split('/').next().unwrap_or("");
                let is_world_related = first_segment == "world" 
                    || first_segment == "world_nether" 
                    || first_segment == "world_the_end"
                    || rel_path.contains("region/")
                    || rel_path.contains("entities/")
                    || rel_path.contains("poi/")
                    || rel_path.ends_with("level.dat");
                if !is_world_related {
                    continue;
                }
            } else if scope_clone == "configs_only" {
                if rel_path.contains("/region/") || rel_path.contains("/entities/") {
                    continue;
                }
            }

            if path.is_file() {
                zip.start_file(&rel_path, zip_options)
                    .map_err(|e| AppError::InternalError(format!("Failed to add entry to zip: {}", e)))?;

                let mut src_file = File::open(path)
                    .map_err(|e| AppError::InternalError(format!("Failed to open file {:?}: {}", path, e)))?;

                loop {
                    let bytes_read = src_file.read(&mut buffer)
                        .map_err(|e| AppError::InternalError(format!("Failed reading file {:?}: {}", path, e)))?;
                    if bytes_read == 0 {
                        break;
                    }
                    zip.write_all(&buffer[..bytes_read])
                        .map_err(|e| AppError::InternalError(format!("Failed writing to zip: {}", e)))?;
                }
                count += 1;
            } else if path.is_dir() && !rel_path.is_empty() {
                let dir_path = format!("{}/", rel_path);
                let _ = zip.add_directory(&dir_path, zip_options);
            }
        }

        zip.finish()
            .map_err(|e| AppError::InternalError(format!("Failed to finalize zip file: {}", e)))?;

        // Calculate SHA-256 checksum and size of archive
        let mut final_file = File::open(&target_path_clone)
            .map_err(|e| AppError::InternalError(format!("Failed to open completed archive: {}", e)))?;
        let mut hasher = Sha256::new();
        let mut hash_buf = vec![0u8; 64 * 1024];
        let mut total_size = 0u64;

        loop {
            let n = final_file.read(&mut hash_buf)
                .map_err(|e| AppError::InternalError(format!("Failed reading archive for hash: {}", e)))?;
            if n == 0 {
                break;
            }
            total_size += n as u64;
            hasher.update(&hash_buf[..n]);
        }

        let sha256_hex = hex::encode(hasher.finalize());
        Ok((count, total_size, sha256_hex))
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Backup worker join error: {}", e)))??;

    // Release RCON guard
    guard.release().await;

    info!(
        "Server backup successfully created: {} ({} files, {} bytes, sha256: {})",
        filename, file_count, file_size_bytes, sha256_hex
    );

    // Apply retention policy: delete oldest un-locked backups exceeding max_retention_count
    if settings.max_retention_count > 0 {
        let existing = list_server_backups(config).await?;
        let unlocked: Vec<_> = existing.iter().filter(|b| !b.is_locked).collect();
        if unlocked.len() > settings.max_retention_count {
            for old in unlocked.iter().skip(settings.max_retention_count) {
                warn!("Pruning old backup to enforce retention policy: {}", old.filename);
                let _ = delete_server_backup(config, &old.filename).await;
            }
        }
    }

    Ok(ServerBackupMetadata {
        filename,
        file_size_bytes,
        created_at_secs: timestamp_secs,
        scope,
        sha256: sha256_hex,
        format: format_choice,
        file_count,
        is_locked: false,
    })
}

/// Toggles the lock status on a backup archive
pub async fn toggle_backup_lock(config: &AppConfig, filename: &str) -> Result<bool, AppError> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::BadRequest("Invalid backup filename".to_string()));
    }

    let mut settings = load_backup_settings(config).await;
    let is_locked = if settings.locked_backups.contains(&filename.to_string()) {
        settings.locked_backups.retain(|f| f != filename);
        false
    } else {
        settings.locked_backups.push(filename.to_string());
        true
    };

    save_backup_settings(config, &settings).await?;
    info!("Backup '{}' lock state changed to: {}", filename, is_locked);
    Ok(is_locked)
}

/// Lists all server backups with metadata sorted newest first
pub async fn list_server_backups(config: &AppConfig) -> Result<Vec<ServerBackupMetadata>, AppError> {
    let backups_dir = get_backups_dir(config);
    if !backups_dir.exists() {
        return Ok(Vec::new());
    }

    let settings = load_backup_settings(config).await;

    let mut entries = tokio::fs::read_dir(&backups_dir)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read backups directory: {}", e)))?;

    let mut backups = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let filename = entry.file_name().to_string_lossy().to_string();
        if !filename.ends_with(".zip") && !filename.ends_with(".tar.zst") {
            continue;
        }

        let metadata = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue,
        };

        let file_size_bytes = metadata.len();
        let created_at_secs = metadata
            .modified()
            .unwrap_or_else(|_| SystemTime::now())
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let scope = if filename.contains("_world_only_") {
            "world_only".to_string()
        } else if filename.contains("_configs_only_") {
            "configs_only".to_string()
        } else {
            "full".to_string()
        };

        let is_locked = settings.locked_backups.contains(&filename);

        backups.push(ServerBackupMetadata {
            filename,
            file_size_bytes,
            created_at_secs,
            scope,
            sha256: "".to_string(),
            format: "zip".to_string(),
            file_count: 0,
            is_locked,
        });
    }

    // Sort newest first
    backups.sort_by_key(|b| std::cmp::Reverse(b.created_at_secs));
    Ok(backups)
}

/// Restores a backup file safely with path traversal protection
pub async fn restore_server_backup(config: &AppConfig, filename: &str) -> Result<usize, AppError> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::BadRequest("Invalid backup filename".to_string()));
    }

    let backups_dir = get_backups_dir(config);
    let target_archive = backups_dir.join(filename);

    if !target_archive.exists() {
        return Err(AppError::NotFound(format!("Backup archive '{}' not found", filename)));
    }

    let dest_dir = config.minecraft_data_dir.clone();

    let restored_count = tokio::task::spawn_blocking(move || -> Result<usize, AppError> {
        let file = File::open(&target_archive)
            .map_err(|e| AppError::InternalError(format!("Failed to open archive: {}", e)))?;
        let mut archive = ZipArchive::new(file)
            .map_err(|e| AppError::InternalError(format!("Failed to parse zip archive: {}", e)))?;

        let mut count = 0;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| AppError::InternalError(format!("Failed reading zip index {}: {}", i, e)))?;

            let enclosed_name = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            let outpath = dest_dir.join(&enclosed_name);

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)
                    .map_err(|e| AppError::InternalError(format!("Failed creating directory {:?}: {}", outpath, e)))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)
                            .map_err(|e| AppError::InternalError(format!("Failed creating parent dir {:?}: {}", p, e)))?;
                    }
                }
                let mut outfile = File::create(&outpath)
                    .map_err(|e| AppError::InternalError(format!("Failed creating file {:?}: {}", outpath, e)))?;
                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| AppError::InternalError(format!("Failed writing restored file {:?}: {}", outpath, e)))?;
                count += 1;
            }
        }
        Ok(count)
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Restore task join error: {}", e)))??;

    info!("Successfully restored {} files from backup '{}'", restored_count, filename);
    Ok(restored_count)
}

/// Deletes a backup file
pub async fn delete_server_backup(config: &AppConfig, filename: &str) -> Result<(), AppError> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::BadRequest("Invalid backup filename".to_string()));
    }

    let settings = load_backup_settings(config).await;
    if settings.locked_backups.contains(&filename.to_string()) {
        return Err(AppError::BadRequest("Impossible de supprimer une sauvegarde verrouillée".to_string()));
    }

    let backups_dir = get_backups_dir(config);
    let file_path = backups_dir.join(filename);

    if !file_path.exists() {
        return Err(AppError::NotFound(format!("Backup '{}' does not exist", filename)));
    }

    tokio::fs::remove_file(&file_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to delete backup file: {}", e)))?;

    info!("Deleted server backup: {}", filename);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_path_excluded() {
        let exclusions = vec![
            "*.log.gz".to_string(),
            "logs/*".to_string(),
            "cache/*".to_string(),
            "dynmap/web/tiles/*".to_string(),
        ];

        assert!(is_path_excluded("logs/2026-08-17-1.log.gz", &exclusions));
        assert!(is_path_excluded("logs/latest.log", &exclusions));
        assert!(is_path_excluded("cache/mojang_manifest.json", &exclusions));
        assert!(is_path_excluded("dynmap/web/tiles/flat_12_34.png", &exclusions));
        assert!(!is_path_excluded("server.properties", &exclusions));
        assert!(!is_path_excluded("world/level.dat", &exclusions));
    }
}
