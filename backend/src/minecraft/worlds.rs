use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldInfo {
    pub folder_name: String,
    pub level_name: String,
    pub seed: i64,
    pub generator: String,
    pub size_bytes: u64,
    pub spawn_x: i32,
    pub spawn_y: i32,
    pub spawn_z: i32,
    pub time: i64,
    pub is_nether: bool,
    pub is_end: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldBorderInfo {
    pub size: f64,
    pub center_x: f64,
    pub center_z: f64,
    pub damage_amount: f64,
    pub warning_distance: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkyStatus {
    pub is_running: bool,
    pub is_paused: bool,
    pub percent_complete: f32,
    pub chunks_rendered: u64,
    pub chunks_total: u64,
    pub current_cps: f32,
    pub eta_seconds: u32,
    pub world: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldBackupInfo {
    pub filename: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub world_name: String,
}

#[derive(Debug, Deserialize, Default)]
struct RawLevelDat {
    #[serde(rename = "Data", default)]
    data: RawDataTag,
}

#[derive(Debug, Deserialize, Default)]
struct RawDataTag {
    #[serde(rename = "RandomSeed", default)]
    random_seed: i64,

    #[serde(rename = "LevelName", default)]
    level_name: String,

    #[serde(rename = "generatorName", default)]
    generator_name: String,

    #[serde(rename = "SpawnX", default)]
    spawn_x: i32,

    #[serde(rename = "SpawnY", default)]
    spawn_y: i32,

    #[serde(rename = "SpawnZ", default)]
    spawn_z: i32,

    #[serde(rename = "Time", default)]
    time: i64,

    #[serde(rename = "WorldBorder", default)]
    world_border: RawWorldBorder,

    #[serde(rename = "WorldGenSettings", default)]
    world_gen_settings: Option<RawWorldGenSettings>,

    /// Save-format version of the world (1.9+). Used to warn about downgrade
    /// incompatibility before switching server versions. NBT Int (i32).
    #[serde(rename = "DataVersion", default)]
    data_version: Option<i32>,

    /// `Version.Id` holds the data version on 1.16+ worlds (preferred when set).
    #[serde(rename = "Version", default)]
    version: Option<RawVersion>,
}

#[derive(Debug, Deserialize, Default)]
struct RawVersion {
    #[serde(rename = "Id", default)]
    id: i32,
}

#[derive(Debug, Deserialize, Default)]
struct RawWorldGenSettings {
    #[serde(rename = "seed", default)]
    seed: i64,
}

#[derive(Debug, Deserialize, Default)]
struct RawWorldBorder {
    #[serde(rename = "Size", default = "default_border_size")]
    size: f64,

    #[serde(rename = "CenterX", default)]
    center_x: f64,

    #[serde(rename = "CenterZ", default)]
    center_z: f64,

    #[serde(rename = "DamagePerBlock", default = "default_damage_amount")]
    damage_amount: f64,

    #[serde(rename = "WarningBlocks", default = "default_warning_distance")]
    warning_distance: i32,
}

fn default_border_size() -> f64 {
    60000000.0
}

fn default_damage_amount() -> f64 {
    0.2
}

fn default_warning_distance() -> i32 {
    5
}

/// Sanitizes world name or folder name against path traversal
pub fn sanitize_name(name: &str) -> Result<(), AppError> {
    if name.contains('\n') || name.contains('\r') || name.contains('\0') {
        return Err(AppError::BadRequest(
            "Control characters not allowed in world or backup name".into(),
        ));
    }
    let name_trimmed = name.trim();
    if name_trimmed.is_empty()
        || name_trimmed == "."
        || name_trimmed.contains('/')
        || name_trimmed.contains('\\')
        || name_trimmed.contains("..")
    {
        return Err(AppError::BadRequest(
            "Invalid world name or folder name: path traversal forbidden".to_string(),
        ));
    }
    Ok(())
}

/// Returns true if name is a valid world or folder name without path traversal
pub fn is_valid_world_name(name: &str) -> bool {
    sanitize_name(name).is_ok()
}

/// Sanitizes backup zip filename against path traversal
pub fn sanitize_backup_filename(filename: &str) -> Result<(), AppError> {
    if filename.contains('\n') || filename.contains('\r') || filename.contains('\0') {
        return Err(AppError::BadRequest(
            "Control characters not allowed in world or backup name".into(),
        ));
    }
    sanitize_name(filename)?;
    if !filename.ends_with(".zip") {
        return Err(AppError::BadRequest(
            "Invalid backup filename: must end with .zip".to_string(),
        ));
    }
    Ok(())
}

/// Returns true if filename is a valid zip backup filename without path traversal
pub fn is_valid_backup_filename(filename: &str) -> bool {
    sanitize_backup_filename(filename).is_ok()
}

/// Reads the save-format version out of a world's `level.dat`, preferring
/// `Data.Version.Id` (1.16+) and falling back to `Data.DataVersion`. Returns
/// `None` when the file is missing or unparsable — callers should treat that as
/// "unknown", not "new world".
pub fn read_world_data_version(level_dat_path: &Path) -> Option<i64> {
    let mut file = match File::open(level_dat_path) {
        Ok(f) => f,
        Err(e) => {
            warn!("read_world_data_version: open {:?} failed: {}", level_dat_path, e);
            return None;
        }
    };

    let mut buffer = Vec::new();
    let mut gz = GzDecoder::new(&file);
    if gz.read_to_end(&mut buffer).is_err() {
        buffer.clear();
        let mut raw_file = match File::open(level_dat_path) {
            Ok(f) => f,
            Err(e) => {
                warn!("read_world_data_version: reopen {:?} failed: {}", level_dat_path, e);
                return None;
            }
        };
        if raw_file.read_to_end(&mut buffer).is_err() {
            warn!("read_world_data_version: raw read {:?} failed", level_dat_path);
            return None;
        }
    }

    let raw: RawLevelDat = match fastnbt::from_bytes(&buffer) {
        Ok(raw) => raw,
        Err(e) => {
            warn!("read_world_data_version: fastnbt parse {:?} failed: {}", level_dat_path, e);
            return None;
        }
    };
    let data = raw.data;
    if let Some(version) = data.version {
        return Some(version.id as i64);
    }
    data.data_version.map(|d| d as i64)
}

/// Parses level.dat at specified path
pub fn parse_level_dat(path: &Path, folder_name: &str) -> Result<(WorldInfo, WorldBorderInfo), AppError> {    let mut file = File::open(path)
        .map_err(|e| AppError::InternalError(format!("Failed to open level.dat at {:?}: {}", path, e)))?;

    let mut buffer = Vec::new();
    let mut gz = GzDecoder::new(&file);
    if gz.read_to_end(&mut buffer).is_err() {
        buffer.clear();
        let mut raw_file = File::open(path)
            .map_err(|e| AppError::InternalError(format!("Failed to reopen level.dat at {:?}: {}", path, e)))?;
        raw_file
            .read_to_end(&mut buffer)
            .map_err(|e| AppError::InternalError(format!("Failed to read level.dat at {:?}: {}", path, e)))?;
    }

    let raw_dat: RawLevelDat = fastnbt::from_bytes(&buffer)
        .map_err(|e| AppError::InternalError(format!("Failed to parse level.dat NBT data: {}", e)))?;

    let data = raw_dat.data;

    let seed = if data.random_seed != 0 {
        data.random_seed
    } else if let Some(wgs) = data.world_gen_settings {
        wgs.seed
    } else {
        0
    };

    let level_name = if data.level_name.is_empty() {
        folder_name.to_string()
    } else {
        data.level_name
    };

    let generator = if data.generator_name.is_empty() {
        "default".to_string()
    } else {
        data.generator_name
    };

    let folder_lower = folder_name.to_lowercase();
    let is_nether = folder_lower.ends_with("_nether") || folder_lower == "nether" || folder_lower == "dim-1";
    let is_end = folder_lower.ends_with("_the_end") || folder_lower.ends_with("_end") || folder_lower == "the_end" || folder_lower == "dim1";

    let border_size = if data.world_border.size > 0.0 {
        data.world_border.size
    } else {
        60000000.0
    };

    let world_info = WorldInfo {
        folder_name: folder_name.to_string(),
        level_name,
        seed,
        generator,
        size_bytes: 0,
        spawn_x: data.spawn_x,
        spawn_y: data.spawn_y,
        spawn_z: data.spawn_z,
        time: data.time,
        is_nether,
        is_end,
    };

    let border_info = WorldBorderInfo {
        size: border_size,
        center_x: data.world_border.center_x,
        center_z: data.world_border.center_z,
        damage_amount: data.world_border.damage_amount,
        warning_distance: data.world_border.warning_distance,
    };

    Ok((world_info, border_info))
}

/// Recursively calculates directory size
pub fn calculate_dir_size(path: &Path) -> u64 {
    let mut total_size = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total_size += calculate_dir_size(&p);
            } else if let Ok(meta) = entry.metadata() {
                total_size += meta.len();
            }
        }
    }
    total_size
}

/// Scans base_dir for directories containing level.dat
pub async fn scan_worlds(base_dir: &Path) -> Result<Vec<WorldInfo>, AppError> {
    if !base_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = tokio::fs::read_dir(base_dir)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read minecraft data directory {:?}: {}", base_dir, e)))?;

    let mut worlds = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let level_dat_path = path.join("level.dat");
        if !level_dat_path.exists() {
            continue;
        }

        let folder_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let path_clone = path.clone();
        let level_dat_clone = level_dat_path.clone();
        let folder_name_clone = folder_name.clone();

        let parsed = tokio::task::spawn_blocking(move || {
            let (mut info, _border) = parse_level_dat(&level_dat_clone, &folder_name_clone)?;
            info.size_bytes = calculate_dir_size(&path_clone);
            Ok::<WorldInfo, AppError>(info)
        })
        .await;

        if let Ok(Ok(info)) = parsed {
            worlds.push(info);
        }
    }

    Ok(worlds)
}

/// Retrieves detailed WorldInfo and WorldBorderInfo for a specific world folder name
pub async fn get_world_detail(
    base_dir: &Path,
    name: &str,
) -> Result<(WorldInfo, WorldBorderInfo), AppError> {
    sanitize_name(name)?;

    let world_dir = base_dir.join(name);
    let level_dat_path = world_dir.join("level.dat");

    if !world_dir.exists() || !level_dat_path.exists() {
        return Err(AppError::NotFound(format!("World '{}' not found", name)));
    }

    let folder_name = name.to_string();
    let parsed = tokio::task::spawn_blocking(move || {
        let (mut info, border) = parse_level_dat(&level_dat_path, &folder_name)?;
        info.size_bytes = calculate_dir_size(&world_dir);
        Ok::<(WorldInfo, WorldBorderInfo), AppError>((info, border))
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

    Ok(parsed)
}

/// Parses Chunky RCON progress output into ChunkyStatus
pub fn parse_chunky_status(output: &str) -> ChunkyStatus {
    let output_lower = output.to_lowercase();

    let is_paused = output_lower.contains("paused");
    let is_running = output_lower.contains("running") && !is_paused;

    let mut percent_complete: f32 = 0.0;
    let mut chunks_rendered: u64 = 0;
    let mut chunks_total: u64 = 0;
    let mut current_cps: f32 = 0.0;
    let mut eta_seconds: u32 = 0;
    let mut world = String::new();

    if let Some(start) = output.find("Task for ") {
        let remainder = &output[start + 9..];
        let cleaned = remainder.strip_prefix("world ").unwrap_or(remainder);
        if let Some(q_start) = cleaned.find('\'') {
            if let Some(q_end) = cleaned[q_start + 1..].find('\'') {
                world = cleaned[q_start + 1..q_start + 1 + q_end].to_string();
            }
        } else if let Some(word) = cleaned.split_whitespace().next() {
            world = word.trim_matches(':').trim_matches('.').to_string();
        }
    }

    for part in output.split_whitespace() {
        if part.ends_with('%') {
            let num_str = part.trim_end_matches('%');
            if let Ok(val) = num_str.parse::<f32>() {
                percent_complete = val;
                break;
            }
        }
    }

    for part in output.split_whitespace() {
        let clean_part = part.trim_matches(',').trim_matches('.');
        if clean_part.contains('/') {
            if let Some((rendered_str, total_str)) = clean_part.split_once('/') {
                if let (Ok(r), Ok(t)) = (rendered_str.parse::<u64>(), total_str.parse::<u64>()) {
                    chunks_rendered = r;
                    chunks_total = t;
                    break;
                }
            }
        }
    }

    if let Some(cps_pos) = output_lower.find("cps:") {
        let after_cps = &output[cps_pos + 4..];
        for word in after_cps.split_whitespace() {
            let clean = word.trim_matches(',').trim_matches('.');
            if let Ok(cps) = clean.parse::<f32>() {
                current_cps = cps;
                break;
            }
        }
    }

    if let Some(eta_pos) = output_lower.find("eta:") {
        let after_eta = &output[eta_pos + 4..];
        eta_seconds = parse_eta_seconds(after_eta);
    }

    ChunkyStatus {
        is_running,
        is_paused,
        percent_complete,
        chunks_rendered,
        chunks_total,
        current_cps,
        eta_seconds,
        world,
    }
}

fn parse_eta_seconds(eta_str: &str) -> u32 {
    let clean = eta_str.trim().trim_matches('.');
    let mut total_secs = 0u32;

    if clean.contains(':') {
        let parts: Vec<&str> = clean.split_whitespace().next().unwrap_or("").split(':').collect();
        if parts.len() == 3 {
            let h = parts[0].parse::<u32>().unwrap_or(0);
            let m = parts[1].parse::<u32>().unwrap_or(0);
            let s = parts[2].parse::<u32>().unwrap_or(0);
            return h * 3600 + m * 60 + s;
        } else if parts.len() == 2 {
            let m = parts[0].parse::<u32>().unwrap_or(0);
            let s = parts[1].parse::<u32>().unwrap_or(0);
            return m * 60 + s;
        }
    }

    let words: Vec<&str> = clean.split_whitespace().take(4).collect();
    for word in words {
        if word.ends_with('h') {
            if let Ok(h) = word.trim_end_matches('h').parse::<u32>() {
                total_secs += h * 3600;
            }
        } else if word.ends_with('m') {
            if let Ok(m) = word.trim_end_matches('m').parse::<u32>() {
                total_secs += m * 60;
            }
        } else if word.ends_with('s') {
            if let Ok(s) = word.trim_end_matches('s').parse::<u32>() {
                total_secs += s;
            }
        }
    }

    total_secs
}

fn format_system_time(system_time: SystemTime) -> String {
    let duration = system_time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let days = secs / 86400;
    let secs_of_day = secs % 86400;
    let hours = secs_of_day / 3600;
    let minutes = (secs_of_day % 3600) / 60;
    let seconds = secs_of_day % 60;

    let mut year = 1970;
    let mut day_rem = days;
    loop {
        let leap = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) { 1 } else { 0 };
        let days_in_year = 365 + leap;
        if day_rem < days_in_year {
            break;
        }
        day_rem -= days_in_year;
        year += 1;
    }
    let leap = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) { 1 } else { 0 };
    let days_in_months = [31, 28 + leap, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1;
    for &dim in &days_in_months {
        if day_rem < dim {
            break;
        }
        day_rem -= dim;
        month += 1;
    }
    let day = day_rem + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Lists all backup .zip files in backups_dir
pub async fn list_backups(backups_dir: &Path) -> Result<Vec<WorldBackupInfo>, AppError> {
    if !backups_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = tokio::fs::read_dir(backups_dir)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read backups dir: {}", e)))?;

    let mut backups = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        if !filename.ends_with(".zip") {
            continue;
        }

        let meta = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue,
        };

        let size_bytes = meta.len();
        let created_at = meta
            .created()
            .or_else(|_| meta.modified())
            .map(format_system_time)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

        let world_name = if let Some((prefix, _)) = filename.rsplit_once("_backup_") {
            prefix.to_string()
        } else if let Some((prefix, _)) = filename.rsplit_once("-backup-") {
            prefix.to_string()
        } else {
            filename.strip_suffix(".zip").unwrap_or(&filename).to_string()
        };

        backups.push(WorldBackupInfo {
            filename,
            size_bytes,
            created_at,
            world_name,
        });
    }

    Ok(backups)
}

/// Creates a zip archive of target world folder in backups_dir
pub async fn create_backup(
    minecraft_data_dir: &Path,
    backups_dir: &Path,
    world_name: &str,
) -> Result<WorldBackupInfo, AppError> {
    sanitize_name(world_name)?;

    let world_dir = minecraft_data_dir.join(world_name);
    let level_dat_path = world_dir.join("level.dat");

    if !world_dir.exists() || !level_dat_path.exists() {
        return Err(AppError::NotFound(format!("World '{}' not found", world_name)));
    }

    if !backups_dir.exists() {
        tokio::fs::create_dir_all(backups_dir)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create backups directory: {}", e)))?;
    }

    let timestamp_secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let filename = format!("{}_backup_{}.zip", world_name, timestamp_secs);
    let zip_path = backups_dir.join(&filename);

    let world_dir_clone = world_dir.clone();
    let zip_path_clone = zip_path.clone();

    tokio::task::spawn_blocking(move || -> Result<(), AppError> {
        let zip_file = File::create(&zip_path_clone)
            .map_err(|e| AppError::InternalError(format!("Failed to create backup file {:?}: {}", zip_path_clone, e)))?;
        let mut zip = ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        fn collect_files(current: &Path, acc: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
            for entry in fs::read_dir(current)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    collect_files(&path, acc)?;
                } else {
                    acc.push(path);
                }
            }
            Ok(())
        }

        let mut files = Vec::new();
        collect_files(&world_dir_clone, &mut files)
            .map_err(|e| AppError::InternalError(format!("Failed to collect world files: {}", e)))?;

        for file_path in files {
            let relative_path = match file_path.strip_prefix(&world_dir_clone) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let name_str = relative_path.to_string_lossy();

            zip.start_file(name_str, options)
                .map_err(|e| AppError::InternalError(format!("Failed to start zip file entry: {}", e)))?;

            let mut f = File::open(&file_path)
                .map_err(|e| AppError::InternalError(format!("Failed to open file {:?}: {}", file_path, e)))?;

            std::io::copy(&mut f, &mut zip)
                .map_err(|e| AppError::InternalError(format!("Failed to write zip content for {:?}: {}", file_path, e)))?;
        }

        zip.finish()
            .map_err(|e| AppError::InternalError(format!("Failed to finish zip archive: {}", e)))?;

        Ok(())
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Join error during backup creation: {}", e)))??;

    let meta = tokio::fs::metadata(&zip_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to get backup metadata: {}", e)))?;

    let created_at = meta
        .created()
        .or_else(|_| meta.modified())
        .map(format_system_time)
        .unwrap_or_else(|_| format_system_time(SystemTime::now()));

    info!("Successfully created world backup: {:?}", zip_path);

    Ok(WorldBackupInfo {
        filename,
        size_bytes: meta.len(),
        created_at,
        world_name: world_name.to_string(),
    })
}

/// Restores world backup from a .zip file in backups_dir into minecraft_data_dir
pub async fn restore_backup(
    minecraft_data_dir: &Path,
    backups_dir: &Path,
    filename: &str,
) -> Result<(), AppError> {
    sanitize_backup_filename(filename)?;

    let zip_path = backups_dir.join(filename);
    if !zip_path.exists() {
        return Err(AppError::NotFound(format!("Backup file '{}' not found", filename)));
    }

    let world_name = if let Some((prefix, _)) = filename.rsplit_once("_backup_") {
        prefix.to_string()
    } else if let Some((prefix, _)) = filename.rsplit_once("-backup-") {
        prefix.to_string()
    } else {
        filename.strip_suffix(".zip").unwrap_or(filename).to_string()
    };

    sanitize_name(&world_name)?;

    let target_dir = minecraft_data_dir.join(&world_name);
    let target_dir_clone = target_dir.clone();
    let zip_path_clone = zip_path.clone();

    tokio::task::spawn_blocking(move || -> Result<(), AppError> {
        let file = File::open(&zip_path_clone)
            .map_err(|e| AppError::InternalError(format!("Failed to open backup zip file: {}", e)))?;
        let mut archive = ZipArchive::new(file)
            .map_err(|e| AppError::InternalError(format!("Failed to read backup zip archive: {}", e)))?;

        for i in 0..archive.len() {
            let mut entry_file = archive.by_index(i)
                .map_err(|e| AppError::InternalError(format!("Failed to read zip entry: {}", e)))?;

            let enclosed = match entry_file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            let dest_path = target_dir_clone.join(&enclosed);

            if entry_file.name().ends_with('/') || entry_file.name().ends_with('\\') {
                fs::create_dir_all(&dest_path)
                    .map_err(|e| AppError::InternalError(format!("Failed to create directory {:?}: {}", dest_path, e)))?;
            } else {
                if let Some(parent) = dest_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)
                            .map_err(|e| AppError::InternalError(format!("Failed to create parent directory {:?}: {}", parent, e)))?;
                    }
                }
                let mut out_file = File::create(&dest_path)
                    .map_err(|e| AppError::InternalError(format!("Failed to create output file {:?}: {}", dest_path, e)))?;
                std::io::copy(&mut entry_file, &mut out_file)
                    .map_err(|e| AppError::InternalError(format!("Failed to extract file {:?}: {}", dest_path, e)))?;
            }
        }

        Ok(())
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Join error during backup restore: {}", e)))??;

    info!("Successfully restored backup '{}' to {:?}", filename, target_dir);

    Ok(())
}

/// Deletes backup file from backups_dir
pub async fn delete_backup(backups_dir: &Path, filename: &str) -> Result<(), AppError> {
    sanitize_backup_filename(filename)?;

    let file_path = backups_dir.join(filename);
    if !file_path.exists() {
        return Err(AppError::NotFound(format!("Backup file '{}' not found", filename)));
    }

    tokio::fs::remove_file(&file_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to delete backup file '{}': {}", filename, e)))?;

    info!("Deleted backup file: {:?}", file_path);

    Ok(())
}

