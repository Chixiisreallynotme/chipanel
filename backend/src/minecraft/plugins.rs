use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use serde::{Deserialize, Serialize};
use sha1::Digest as Sha1Digest;
use sha2::Digest as Sha2Digest;
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
    pub loader_type: String, // "spigot", "paper", "purpur", "fabric", "forge", "neoforge", "quilt", "resourcepack", "datapack"
    pub target_dir: String,  // "plugins" | "mods" | "resourcepacks" | "datapacks"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack_format: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha512: Option<String>,
}

/// Scans plugins, mods, resourcepacks, and datapacks directories under `base_dir`
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

/// Scans a specific sub-directory ("plugins", "mods", "resourcepacks", or "datapacks")
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

        // Determine if file is recognized extension based on target_dir
        let is_jar = filename.ends_with(".jar") || filename.ends_with(".plugin");
        let is_zip = filename.ends_with(".zip");
        let is_disabled = filename.ends_with(".disabled");

        let matches_type = if target_dir == "resourcepacks" || target_dir == "datapacks" {
            is_zip || is_disabled || is_jar
        } else {
            is_jar || is_disabled
        };

        if !matches_type {
            continue;
        }

        let enabled = !is_disabled;

        let file_size_bytes = match entry.metadata().await {
            Ok(m) => m.len(),
            Err(_) => 0,
        };

        let path_buf = path.clone();
        let target_dir_owned = target_dir.to_string();
        let filename_clone = filename.clone();

        let meta = tokio::task::spawn_blocking(move || {
            inspect_file_metadata(&path_buf, &target_dir_owned, &filename_clone)
        })
        .await
        .ok()
        .flatten();

        let (extracted_name, extracted_version, loader_type, description, pack_format) = meta.unwrap_or_else(|| {
            let (name, ver) = parse_name_version_from_filename(&filename);
            let default_loader = match target_dir {
                "mods" => "fabric".to_string(),
                "resourcepacks" => "resourcepack".to_string(),
                "datapacks" => "datapack".to_string(),
                _ => "spigot".to_string(),
            };
            (name, ver, default_loader, None, None)
        });

        items.push(InstalledPlugin {
            name: extracted_name,
            version: extracted_version,
            enabled,
            filename,
            file_size_bytes,
            loader_type,
            target_dir: target_dir.to_string(),
            description,
            pack_format,
            sha1: None,
            sha512: None,
        });
    }

    Ok(items)
}

/// Toggles plugin enabled state (renames `file` <-> `file.disabled`)
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
        "Toggled addon state: renamed '{}' -> '{}'",
        filename, new_filename
    );

    let metadata = match tokio::fs::metadata(&new_file_path).await {
        Ok(m) => m.len(),
        Err(_) => 0,
    };

    let new_file_path_buf = new_file_path.clone();
    let target_dir_owned = target_dir.to_string();
    let new_filename_clone = new_filename.clone();

    let meta = tokio::task::spawn_blocking(move || {
        inspect_file_metadata(&new_file_path_buf, &target_dir_owned, &new_filename_clone)
    })
    .await
    .ok()
    .flatten();

    let (extracted_name, extracted_version, loader_type, description, pack_format) = meta.unwrap_or_else(|| {
        let (name, ver) = parse_name_version_from_filename(&new_filename);
        let default_loader = match target_dir {
            "mods" => "fabric".to_string(),
            "resourcepacks" => "resourcepack".to_string(),
            "datapacks" => "datapack".to_string(),
            _ => "spigot".to_string(),
        };
        (name, ver, default_loader, None, None)
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
        description,
        pack_format,
        sha1: None,
        sha512: None,
    })
}

/// Deletes plugin/addon file from plugins, mods, resourcepacks, or datapacks directory
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

    info!("Deleted addon file: {:?}", file_path);

    Ok(())
}

pub fn sanitize_target_dir_and_filename(target_dir: &str, filename: &str) -> Result<(), AppError> {
    if target_dir != "plugins"
        && target_dir != "mods"
        && target_dir != "resourcepacks"
        && target_dir != "datapacks"
    {
        return Err(AppError::BadRequest(format!(
            "Invalid target_dir '{}'. Must be 'plugins', 'mods', 'resourcepacks', or 'datapacks'",
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

/// Computes the SHA-1 hex hash of a file on disk (for Minecraft server resource-pack-sha1)
pub async fn compute_file_sha1(file_path: &Path) -> Result<String, AppError> {
    let mut file = tokio::fs::File::open(file_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to open file for SHA-1: {}", e)))?;
    
    let mut hasher = sha1::Sha1::new();
    let mut buffer = [0u8; 65536];

    use tokio::io::AsyncReadExt;
    loop {
        let n = file.read(&mut buffer).await
            .map_err(|e| AppError::InternalError(format!("Error reading file for SHA-1: {}", e)))?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Computes the SHA-512 hex hash of a file on disk (for Modrinth version update checks)
pub async fn compute_file_sha512(file_path: &Path) -> Result<String, AppError> {
    let mut file = tokio::fs::File::open(file_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to open file for SHA-512: {}", e)))?;

    let mut hasher = sha2::Sha512::new();
    let mut buffer = [0u8; 65536];

    use tokio::io::AsyncReadExt;
    loop {
        let n = file.read(&mut buffer).await
            .map_err(|e| AppError::InternalError(format!("Error reading file for SHA-512: {}", e)))?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

type JarMetadataTuple = (String, String, String, Option<String>, Option<u32>);

/// Inspects archive files (JARs or ZIPs) to extract metadata
fn inspect_file_metadata(
    file_path: &Path,
    target_dir: &str,
    filename: &str,
) -> Option<JarMetadataTuple> {
    if target_dir == "resourcepacks" || target_dir == "datapacks" {
        return inspect_pack_metadata(file_path, target_dir, filename);
    }

    inspect_jar_metadata(file_path)
}

/// Inspects Resource Pack or Data Pack ZIP files for `pack.mcmeta`
fn inspect_pack_metadata(
    file_path: &Path,
    target_dir: &str,
    filename: &str,
) -> Option<JarMetadataTuple> {
    let (name, ver) = parse_name_version_from_filename(filename);
    let default_loader = if target_dir == "resourcepacks" {
        "resourcepack".to_string()
    } else {
        "datapack".to_string()
    };

    let file = File::open(file_path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;

    if let Ok(entry) = archive.by_name("pack.mcmeta") {
        let mut content = String::new();
        if entry.take(65536).read_to_string(&mut content).is_ok() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                let pack = val.get("pack");
                let desc = pack.and_then(|p| {
                    if let Some(s) = p.get("description").and_then(|d| d.as_str()) {
                        Some(s.to_string())
                    } else {
                        p.get("description").and_then(|d| d.get("text")).and_then(|t| t.as_str()).map(|t| t.to_string())
                    }
                });
                let pack_format = pack
                    .and_then(|p| p.get("pack_format"))
                    .and_then(|f| f.as_u64())
                    .map(|f| f as u32);

                return Some((name, ver, default_loader, desc, pack_format));
            }
        }
    }

    Some((name, ver, default_loader, None, None))
}

/// Inspects jar file ZIP archive for metadata files (`paper-plugin.yml`, `plugin.yml`, `fabric.mod.json`, `neoforge.mods.toml`, `mods.toml`, `mcmod.info`)
fn inspect_jar_metadata(file_path: &Path) -> Option<JarMetadataTuple> {
    let file = File::open(file_path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;

    // 1. Check paper-plugin.yml
    if let Ok(entry) = archive.by_name("paper-plugin.yml") {
        let mut content = String::new();
        if entry.take(65536).read_to_string(&mut content).is_ok() {
            let name = parse_yaml_key(&content, "name");
            let version = parse_yaml_key(&content, "version");
            let desc = parse_yaml_key(&content, "description");
            if let (Some(n), Some(v)) = (name, version) {
                return Some((n, v, "paper".to_string(), desc, None));
            }
        }
    }

    // 2. Check plugin.yml
    if let Ok(entry) = archive.by_name("plugin.yml") {
        let mut content = String::new();
        if entry.take(65536).read_to_string(&mut content).is_ok() {
            let name = parse_yaml_key(&content, "name");
            let version = parse_yaml_key(&content, "version");
            let desc = parse_yaml_key(&content, "description");
            let loader = if content.contains("paper-plugin") || content.contains("paper:") {
                "paper".to_string()
            } else if content.contains("purpur:") {
                "purpur".to_string()
            } else {
                "spigot".to_string()
            };

            if let (Some(n), Some(v)) = (name, version) {
                return Some((n, v, loader, desc, None));
            }
        }
    }

    // 3. Check fabric.mod.json
    if let Ok(entry) = archive.by_name("fabric.mod.json") {
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
                let desc = val
                    .get("description")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string());

                if let (Some(n), Some(v)) = (name, version) {
                    return Some((n, v, "fabric".to_string(), desc, None));
                }
            }
        }
    }

    // 4. Check META-INF/neoforge.mods.toml
    if let Ok(entry) = archive.by_name("META-INF/neoforge.mods.toml") {
        let mut content = String::new();
        if entry.take(65536).read_to_string(&mut content).is_ok() {
            let name = parse_toml_key(&content, "displayName")
                .or_else(|| parse_toml_key(&content, "name"))
                .or_else(|| parse_toml_key(&content, "modId"));
            let version = parse_toml_key(&content, "version");
            let desc = parse_toml_key(&content, "description");

            if let (Some(n), Some(v)) = (name, version) {
                return Some((n, v, "neoforge".to_string(), desc, None));
            }
        }
    }

    // 5. Check META-INF/mods.toml (Forge)
    if let Ok(entry) = archive.by_name("META-INF/mods.toml") {
        let mut content = String::new();
        if entry.take(65536).read_to_string(&mut content).is_ok() {
            let name = parse_toml_key(&content, "displayName")
                .or_else(|| parse_toml_key(&content, "name"))
                .or_else(|| parse_toml_key(&content, "modId"));
            let version = parse_toml_key(&content, "version");
            let desc = parse_toml_key(&content, "description");

            if let (Some(n), Some(v)) = (name, version) {
                return Some((n, v, "forge".to_string(), desc, None));
            }
        }
    }

    // 6. Check mcmod.info
    if let Ok(entry) = archive.by_name("mcmod.info") {
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
                    let desc = mod_obj
                        .get("description")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string());

                    if let (Some(n), Some(v)) = (name, version) {
                        return Some((n, v, "forge".to_string(), desc, None));
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
        .or_else(|| clean.strip_suffix(".zip"))
        .unwrap_or(clean);

    if let Some((name, ver)) = clean.rsplit_once('-') {
        if !ver.is_empty() && ver.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return (name.to_string(), ver.to_string());
        }
    }

    if let Some((name, ver)) = clean.rsplit_once('_') {
        if !ver.is_empty() && ver.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return (name.to_string(), ver.to_string());
        }
    }

    (clean.to_string(), "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_target_dir_and_filename() {
        assert!(sanitize_target_dir_and_filename("plugins", "WorldEdit-7.3.0.jar").is_ok());
        assert!(sanitize_target_dir_and_filename("mods", "lithium-0.12.1.jar").is_ok());
        assert!(sanitize_target_dir_and_filename("resourcepacks", "Faithful-64x.zip").is_ok());
        assert!(sanitize_target_dir_and_filename("datapacks", "custom-recipes.zip").is_ok());

        assert!(sanitize_target_dir_and_filename("worlds", "world.zip").is_err());
        assert!(sanitize_target_dir_and_filename("plugins", "../secrets.txt").is_err());
        assert!(sanitize_target_dir_and_filename("plugins", "foo/bar.jar").is_err());
    }

    #[test]
    fn test_parse_name_version_from_filename() {
        let (n1, v1) = parse_name_version_from_filename("WorldEdit-7.3.0.jar");
        assert_eq!(n1, "WorldEdit");
        assert_eq!(v1, "7.3.0");

        let (n2, v2) = parse_name_version_from_filename("Fabric-API_0.92.0.jar.disabled");
        assert_eq!(n2, "Fabric-API");
        assert_eq!(v2, "0.92.0");

        let (n3, v3) = parse_name_version_from_filename("Faithful-32x.zip");
        assert_eq!(n3, "Faithful");
        assert_eq!(v3, "32x");
    }
}

