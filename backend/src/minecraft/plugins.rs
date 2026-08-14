use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use zip::ZipArchive;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub filename: String,
    pub file_size_bytes: u64,
    pub loader_type: String, // "spigot", "paper", "purpur", "fabric", "forge"
    pub target_dir: String,  // "plugins" | "mods"
}

/// Scans both plugins and mods directories under `base_dir`
pub async fn scan_all_installed(base_dir: &Path) -> Result<Vec<InstalledPlugin>, AppError> {
    let mut results = Vec::new();

    let plugins_dir = base_dir.join("plugins");
    let mods_dir = base_dir.join("mods");
    let rp_dir = base_dir.join("resourcepacks");
    let dp_dir = base_dir.join("datapacks");

    results.extend(scan_directory(&plugins_dir, "plugins").await?);
    results.extend(scan_directory(&mods_dir, "mods").await?);
    results.extend(scan_directory(&rp_dir, "resourcepacks").await?);
    results.extend(scan_directory(&dp_dir, "datapacks").await?);

    Ok(results)
}

/// Scans a specific sub-directory ("plugins" or "mods")
pub async fn scan_directory(dir_path: &Path, target_dir: &str) -> Result<Vec<InstalledPlugin>, AppError> {
    if !dir_path.exists() {
        if let Err(e) = tokio::fs::create_dir_all(dir_path).await {
            warn!("Failed to create directory {:?}: {}", dir_path, e);
        }
        return Ok(Vec::new());
    }

    let mut entries = tokio::fs::read_dir(dir_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read directory {:?}: {}", dir_path, e)))?;

    let mut items = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        // Determine if file is a plugin/mod based on extension
        let is_jar = filename.ends_with(".jar") || filename.ends_with(".plugin");
        let is_disabled = filename.ends_with(".jar.disabled")
            || filename.ends_with(".plugin.disabled")
            || filename.ends_with(".disabled");

        if !is_jar && !is_disabled {
            continue;
        }

        let enabled = is_jar && !is_disabled;

        let file_size_bytes = match entry.metadata().await {
            Ok(m) => m.len(),
            Err(_) => 0,
        };

        // Extract metadata via zip archive inspection or fallback to filename
        let path_buf = path.clone();
        let metadata_opt = tokio::task::spawn_blocking(move || inspect_jar_metadata(&path_buf))
            .await
            .ok()
            .flatten();

        let (extracted_name, extracted_version, loader_type) =
            metadata_opt.unwrap_or_else(|| {
                let (name, ver) = parse_name_version_from_filename(&filename);
                let default_loader = if target_dir == "mods" {
                    "fabric".to_string()
                } else {
                    "spigot".to_string()
                };
                (name, ver, default_loader)
            });

        items.push(InstalledPlugin {
            name: extracted_name,
            version: extracted_version,
            enabled,
            filename,
            file_size_bytes,
            loader_type,
            target_dir: target_dir.to_string(),
        });
    }

    Ok(items)
}

/// Toggles plugin enabled state (renames `.jar` <-> `.jar.disabled`)
pub async fn toggle_plugin(
    base_dir: &Path,
    target_dir: &str,
    filename: &str,
) -> Result<InstalledPlugin, AppError> {
    sanitize_target_dir_and_filename(target_dir, filename)?;

    let dir_path = base_dir.join(target_dir);
    let old_file_path = dir_path.join(filename);

    if !old_file_path.exists() {
        return Err(AppError::NotFound(format!(
            "File '{}' not found in {}",
            filename, target_dir
        )));
    }

    let new_filename = if filename.ends_with(".disabled") {
        filename.strip_suffix(".disabled").unwrap_or(filename).to_string()
    } else {
        format!("{}.disabled", filename)
    };

    let new_file_path = dir_path.join(&new_filename);

    tokio::fs::rename(&old_file_path, &new_file_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to rename file: {}", e)))?;

    info!(
        "Toggled plugin/mod state: renamed '{}' -> '{}'",
        filename, new_filename
    );

    let metadata = match tokio::fs::metadata(&new_file_path).await {
        Ok(m) => m.len(),
        Err(_) => 0,
    };

    let new_file_path_buf = new_file_path.clone();
    let metadata_opt = tokio::task::spawn_blocking(move || inspect_jar_metadata(&new_file_path_buf))
        .await
        .ok()
        .flatten();

    let (extracted_name, extracted_version, loader_type) =
        metadata_opt.unwrap_or_else(|| {
            let (name, ver) = parse_name_version_from_filename(&new_filename);
            let default_loader = if target_dir == "mods" {
                "fabric".to_string()
            } else {
                "spigot".to_string()
            };
            (name, ver, default_loader)
        });

    let enabled = !new_filename.ends_with(".disabled");

    Ok(InstalledPlugin {
        name: extracted_name,
        version: extracted_version,
        enabled,
        filename: new_filename,
        file_size_bytes: metadata,
        loader_type,
        target_dir: target_dir.to_string(),
    })
}

/// Deletes plugin file from plugins or mods directory
pub async fn delete_plugin(
    base_dir: &Path,
    target_dir: &str,
    filename: &str,
) -> Result<(), AppError> {
    sanitize_target_dir_and_filename(target_dir, filename)?;

    let file_path = base_dir.join(target_dir).join(filename);

    if !file_path.exists() {
        return Err(AppError::NotFound(format!(
            "File '{}' not found in {}",
            filename, target_dir
        )));
    }

    tokio::fs::remove_file(&file_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to delete file '{}': {}", filename, e)))?;

    info!("Deleted plugin/mod file: {:?}", file_path);

    Ok(())
}

fn sanitize_target_dir_and_filename(target_dir: &str, filename: &str) -> Result<(), AppError> {
    if target_dir != "plugins" && target_dir != "mods" {
        return Err(AppError::BadRequest(format!(
            "Invalid target_dir '{}'. Must be 'plugins' or 'mods'",
            target_dir
        )));
    }

    if filename.trim().is_empty()
        || filename == "."
        || filename.contains('/')
        || filename.contains('\\')
        || filename.contains("..")
    {
        return Err(AppError::BadRequest(
            "Invalid filename: path traversal forbidden".to_string(),
        ));
    }

    Ok(())
}

/// Inspects jar file ZIP archive for metadata files (`paper-plugin.yml`, `plugin.yml`, `fabric.mod.json`, `mods.toml`, `mcmod.info`)
fn inspect_jar_metadata(file_path: &Path) -> Option<(String, String, String)> {
    let file = File::open(file_path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;

    // 1. Check paper-plugin.yml
    if let Ok(mut entry) = archive.by_name("paper-plugin.yml") {
        let mut content = String::new();
        if entry.take(65536).read_to_string(&mut content).is_ok() {
            let name = parse_yaml_key(&content, "name");
            let version = parse_yaml_key(&content, "version");
            if let (Some(n), Some(v)) = (name, version) {
                return Some((n, v, "paper".to_string()));
            }
        }
    }

    // 2. Check plugin.yml
    if let Ok(mut entry) = archive.by_name("plugin.yml") {
        let mut content = String::new();
        if entry.take(65536).read_to_string(&mut content).is_ok() {
            let name = parse_yaml_key(&content, "name");
            let version = parse_yaml_key(&content, "version");
            let loader = if content.contains("paper-plugin") || content.contains("paper:") {
                "paper".to_string()
            } else if content.contains("purpur:") {
                "purpur".to_string()
            } else {
                "spigot".to_string()
            };

            if let (Some(n), Some(v)) = (name, version) {
                return Some((n, v, loader));
            }
        }
    }

    // 3. Check fabric.mod.json
    if let Ok(mut entry) = archive.by_name("fabric.mod.json") {
        let mut content = String::new();
        if entry.take(65536).read_to_string(&mut content).is_ok() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                let name = val
                    .get("name")
                    .and_then(|s| s.as_str())
                    .or_else(|| val.get("id").and_then(|s| s.as_str()))
                    .map(|s| s.to_string());
                let version = val
                    .get("version")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string());

                if let (Some(n), Some(v)) = (name, version) {
                    return Some((n, v, "fabric".to_string()));
                }
            }
        }
    }

    // 4. Check META-INF/mods.toml
    if let Ok(mut entry) = archive.by_name("META-INF/mods.toml") {
        let mut content = String::new();
        if entry.take(65536).read_to_string(&mut content).is_ok() {
            let name = parse_toml_key(&content, "displayName")
                .or_else(|| parse_toml_key(&content, "name"))
                .or_else(|| parse_toml_key(&content, "modId"));
            let version = parse_toml_key(&content, "version");

            if let (Some(n), Some(v)) = (name, version) {
                return Some((n, v, "forge".to_string()));
            }
        }
    }

    // 5. Check mcmod.info
    if let Ok(mut entry) = archive.by_name("mcmod.info") {
        let mut content = String::new();
        if entry.take(65536).read_to_string(&mut content).is_ok() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                let item = if val.is_array() {
                    val.get(0)
                } else {
                    val.get("modList").and_then(|m| m.get(0)).or(Some(&val))
                };
                if let Some(mod_obj) = item {
                    let name = mod_obj
                        .get("name")
                        .and_then(|s| s.as_str())
                        .or_else(|| mod_obj.get("modid").and_then(|s| s.as_str()))
                        .map(|s| s.to_string());
                    let version = mod_obj
                        .get("version")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string());

                    if let (Some(n), Some(v)) = (name, version) {
                        return Some((n, v, "forge".to_string()));
                    }
                }
            }
        }
    }

    None
}

fn parse_yaml_key(yaml: &str, key: &str) -> Option<String> {
    for line in yaml.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            if k.trim() == key {
                let val = v.trim().trim_matches('\'').trim_matches('"').to_string();
                if !val.is_empty() {
                    return Some(val);
                }
            }
        }
    }
    None
}

fn parse_toml_key(toml: &str, key: &str) -> Option<String> {
    for line in toml.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                let val = v.trim().trim_matches('\'').trim_matches('"').to_string();
                if !val.is_empty() {
                    return Some(val);
                }
            }
        }
    }
    None
}

fn parse_name_version_from_filename(filename: &str) -> (String, String) {
    let clean = filename.strip_suffix(".disabled").unwrap_or(filename);
    let clean = clean
        .strip_suffix(".jar")
        .or_else(|| clean.strip_suffix(".plugin"))
        .unwrap_or(clean);

    if let Some((name, ver)) = clean.rsplit_once('-') {
        if !ver.is_empty() && ver.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            return (name.to_string(), ver.to_string());
        }
    }

    if let Some((name, ver)) = clean.rsplit_once('_') {
        if !ver.is_empty() && ver.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            return (name.to_string(), ver.to_string());
        }
    }

    (clean.to_string(), "unknown".to_string())
}
