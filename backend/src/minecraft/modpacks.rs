use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tracing::{info, warn};

use crate::{
    curseforge::client::{CurseForgeClient, CurseForgeFile},
    error::AppError,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub name: String,
    pub path: String,
    pub is_active: bool,
    pub created_at: Option<String>,
    pub modpack_name: Option<String>,
    pub modpack_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMeta {
    pub name: String,
    pub created_at: String,
    pub modpack_name: Option<String>,
    pub modpack_id: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct DeployModpackRequest {
    pub modpack_id: u32,
    pub file_id: Option<u32>,
    pub profile_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeployModpackResponse {
    pub success: bool,
    pub profile_name: String,
    pub modpack_id: u32,
    pub installed_mods_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeManifestFile {
    #[serde(rename = "projectID")]
    pub project_id: u32,
    #[serde(rename = "fileID")]
    pub file_id: u32,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeManifestMinecraft {
    pub version: String,
    #[serde(default, rename = "modLoaders")]
    pub mod_loaders: Vec<CurseForgeModLoader>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeModLoader {
    pub id: String,
    #[serde(default)]
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeManifest {
    pub minecraft: CurseForgeManifestMinecraft,
    #[serde(default, rename = "manifestType")]
    pub manifest_type: String,
    #[serde(default, rename = "manifestVersion")]
    pub manifest_version: u32,
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub files: Vec<CurseForgeManifestFile>,
    #[serde(default)]
    pub overrides: String,
}

pub fn validate_profile_name(name: &str) -> Result<(), AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest("Profile name cannot be empty".to_string()));
    }
    if trimmed == "." || trimmed == ".." {
        return Err(AppError::BadRequest("Invalid profile name: cannot be '.' or '..'".to_string()));
    }
    if trimmed.contains("..") || trimmed.contains('/') || trimmed.contains('\\') {
        return Err(AppError::BadRequest(
            "Invalid profile name: cannot contain '..', '/', or '\\'".to_string(),
        ));
    }
    Ok(())
}

pub async fn get_active_profile_name(profiles_dir: &Path) -> Option<String> {
    let active_file = profiles_dir.join("active_profile.txt");
    if active_file.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&active_file).await {
            let name = content.trim().to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

pub async fn set_active_profile_name(profiles_dir: &Path, name: &str) -> Result<(), AppError> {
    let active_file = profiles_dir.join("active_profile.txt");
    tokio::fs::write(&active_file, name)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to write active profile marker: {}", e)))?;
    Ok(())
}

pub async fn list_profiles(profiles_dir: &Path) -> Result<Vec<ProfileInfo>, AppError> {
    if !profiles_dir.exists() {
        tokio::fs::create_dir_all(profiles_dir)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create profiles dir: {}", e)))?;
    }

    let active_name = get_active_profile_name(profiles_dir).await;

    let mut entries = tokio::fs::read_dir(profiles_dir)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read profiles directory: {}", e)))?;

    let mut profiles = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| AppError::InternalError(format!("Error reading profiles entry: {}", e)))?
    {
        let file_type = entry.file_type().await.map_err(|e| AppError::InternalError(e.to_string()))?;
        if file_type.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_active = active_name.as_deref() == Some(&name);
            let profile_path = entry.path();

            let meta_path = profile_path.join("profile.json");
            let mut created_at = None;
            let mut modpack_name = None;
            let mut modpack_id = None;

            if meta_path.exists() {
                if let Ok(content) = tokio::fs::read_to_string(&meta_path).await {
                    if let Ok(meta) = serde_json::from_str::<ProfileMeta>(&content) {
                        created_at = Some(meta.created_at);
                        modpack_name = meta.modpack_name;
                        modpack_id = meta.modpack_id;
                    }
                }
            }

            profiles.push(ProfileInfo {
                name,
                path: profile_path.to_string_lossy().to_string(),
                is_active,
                created_at,
                modpack_name,
                modpack_id,
            });
        }
    }

    Ok(profiles)
}

pub async fn switch_profile(
    profiles_dir: &Path,
    minecraft_data_dir: &Path,
    profile_name: &str,
) -> Result<ProfileInfo, AppError> {
    validate_profile_name(profile_name)?;

    let target_profile_path = profiles_dir.join(profile_name);
    if !target_profile_path.exists() || !target_profile_path.is_dir() {
        return Err(AppError::NotFound(format!("Profile '{}' does not exist", profile_name)));
    }

    if !minecraft_data_dir.exists() {
        tokio::fs::create_dir_all(minecraft_data_dir)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create minecraft_data_dir: {}", e)))?;
    }

    let src_dir = target_profile_path.clone();
    let dest_dir = minecraft_data_dir.to_path_buf();

    tokio::task::spawn_blocking(move || -> Result<(), AppError> {
        let src_mods = src_dir.join("mods");
        let dest_mods = dest_dir.join("mods");
        if src_mods.exists() {
            if dest_mods.exists() {
                let _ = std::fs::remove_dir_all(&dest_mods);
            }
            copy_dir_all(&src_mods, &dest_mods)?;
        }

        let src_config = src_dir.join("config");
        let dest_config = dest_dir.join("config");
        if src_config.exists() {
            if dest_config.exists() {
                let _ = std::fs::remove_dir_all(&dest_config);
            }
            copy_dir_all(&src_config, &dest_config)?;
        }

        Ok(())
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Task error during profile switch: {}", e)))??;

    set_active_profile_name(profiles_dir, profile_name).await?;

    let meta_path = target_profile_path.join("profile.json");
    let mut created_at = None;
    let mut modpack_name = None;
    let mut modpack_id = None;

    if meta_path.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&meta_path).await {
            if let Ok(meta) = serde_json::from_str::<ProfileMeta>(&content) {
                created_at = Some(meta.created_at);
                modpack_name = meta.modpack_name;
                modpack_id = meta.modpack_id;
            }
        }
    }

    Ok(ProfileInfo {
        name: profile_name.to_string(),
        path: target_profile_path.to_string_lossy().to_string(),
        is_active: true,
        created_at,
        modpack_name,
        modpack_id,
    })
}

pub async fn delete_profile(profiles_dir: &Path, profile_name: &str) -> Result<(), AppError> {
    validate_profile_name(profile_name)?;

    let target_profile_path = profiles_dir.join(profile_name);
    if !target_profile_path.exists() {
        return Err(AppError::NotFound(format!("Profile '{}' does not exist", profile_name)));
    }

    let canonical_profiles = profiles_dir.canonicalize().unwrap_or_else(|_| profiles_dir.to_path_buf());
    let canonical_target = target_profile_path.canonicalize().unwrap_or_else(|_| target_profile_path.clone());
    if !canonical_target.starts_with(&canonical_profiles) {
        return Err(AppError::BadRequest("Invalid profile path".to_string()));
    }

    if let Some(active) = get_active_profile_name(profiles_dir).await {
        if active == profile_name {
            return Err(AppError::BadRequest(format!(
                "Cannot delete profile '{}' because it is currently active",
                profile_name
            )));
        }
    }

    tokio::fs::remove_dir_all(&target_profile_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to delete profile directory: {}", e)))?;

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(dst)
        .map_err(|e| AppError::InternalError(format!("Failed to create directory {:?}: {}", dst, e)))?;
    for entry in std::fs::read_dir(src)
        .map_err(|e| AppError::InternalError(format!("Failed to read directory {:?}: {}", src, e)))?
    {
        let entry = entry.map_err(|e| AppError::InternalError(e.to_string()))?;
        let file_type = entry.file_type().map_err(|e| AppError::InternalError(e.to_string()))?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)
                .map_err(|e| AppError::InternalError(format!("Failed to copy file {:?}: {}", entry.path(), e)))?;
        }
    }
    Ok(())
}

pub async fn deploy_modpack(
    cf_client: &CurseForgeClient,
    profiles_dir: &Path,
    minecraft_data_dir: &Path,
    req: DeployModpackRequest,
) -> Result<DeployModpackResponse, AppError> {
    let mod_details = cf_client.get_modpack_details(req.modpack_id).await?;

    let file_id = match req.file_id {
        Some(fid) => fid,
        None => {
            if let Some(first_file) = mod_details.latest_files.first() {
                first_file.id
            } else {
                let files = cf_client.get_modpack_files(req.modpack_id).await?;
                let first = files.first().ok_or_else(|| {
                    AppError::NotFound(format!("No download files found for modpack ID {}", req.modpack_id))
                })?;
                first.id
            }
        }
    };

    let profile_name = match req.profile_name {
        Some(ref name) if !name.trim().is_empty() => name.trim().to_string(),
        _ => mod_details.name.clone(),
    };

    validate_profile_name(&profile_name)?;

    if !profiles_dir.exists() {
        tokio::fs::create_dir_all(profiles_dir)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create profiles dir: {}", e)))?;
    }

    let profile_dir = profiles_dir.join(&profile_name);
    tokio::fs::create_dir_all(&profile_dir)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to create profile dir: {}", e)))?;

    let download_url = cf_client.get_file_download_url(req.modpack_id, file_id).await?;

    let temp_zip_path = profiles_dir.join(format!("temp_{}_{}.zip", req.modpack_id, file_id));

    cf_client.download_file(&download_url, &temp_zip_path).await?;

    let target_dir = profile_dir.clone();
    let zip_path = temp_zip_path.clone();

    // Offload extraction to spawn_blocking
    let (manifest_opt, skipped_entries) = tokio::task::spawn_blocking(move || -> Result<(Option<CurseForgeManifest>, usize), AppError> {
        let file = std::fs::File::open(&zip_path)
            .map_err(|e| AppError::InternalError(format!("Failed to open downloaded modpack zip: {}", e)))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| AppError::InternalError(format!("Failed to read zip archive: {}", e)))?;

        let mut manifest_data = None;
        let mut skipped_entries = 0usize;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| AppError::InternalError(format!("Failed to read zip file entry: {}", e)))?;
            let outpath = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            let outpath_str = outpath.to_string_lossy();

            if outpath_str == "manifest.json" {
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .map_err(|e| AppError::InternalError(format!("Failed to read manifest.json: {}", e)))?;
                manifest_data = Some(contents);
            } else if outpath_str.starts_with("overrides/") || outpath_str.starts_with("overrides\\") {
                let normalized_str = outpath_str.replace('\\', "/");
                let relative_str = if let Some(stripped) = normalized_str.strip_prefix("overrides/") {
                    stripped
                } else {
                    continue;
                };
                if relative_str.is_empty() {
                    continue;
                }
                let dest_path = target_dir.join(relative_str);
                if file.name().ends_with('/') || file.name().ends_with('\\') {
                    std::fs::create_dir_all(&dest_path)
                        .map_err(|e| AppError::InternalError(format!("Failed to create override dir: {}", e)))?;
                } else {
                    if let Some(p) = dest_path.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p)
                                .map_err(|e| AppError::InternalError(format!("Failed to create parent dir: {}", e)))?;
                        }
                    }
                    let mut outfile = std::fs::File::create(&dest_path)
                        .map_err(|e| AppError::InternalError(format!("Failed to create override file: {}", e)))?;
                    std::io::copy(&mut file, &mut outfile)
                        .map_err(|e| AppError::InternalError(format!("Failed to copy override file: {}", e)))?;
                }
            }
        }

        let _ = std::fs::remove_file(&zip_path);

        if let Some(json_str) = manifest_data {
            let manifest: CurseForgeManifest = serde_json::from_str(&json_str)
                .map_err(|e| AppError::InternalError(format!("Failed to parse manifest.json: {}", e)))?;
            Ok((Some(manifest), skipped_entries))
        } else {
            Ok((None, skipped_entries))
        }
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Join error during zip extraction: {}", e)))??;

    let mut installed_mods_count = 0;

    if let Some(manifest) = manifest_opt {
        let mods_dir = profile_dir.join("mods");
        tokio::fs::create_dir_all(&mods_dir)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create mods dir: {}", e)))?;

        let file_ids: Vec<u32> = manifest.files.iter().map(|f| f.file_id).collect();
        let batch_files = cf_client.get_files_batch(&file_ids).await.unwrap_or_default();
        let file_map: HashMap<u32, CurseForgeFile> = batch_files
            .into_iter()
            .map(|f| (f.id, f))
            .collect();

        let semaphore = Arc::new(Semaphore::new(4));
        let mut set = tokio::task::JoinSet::new();

        for mod_file in manifest.files {
            let sem = semaphore.clone();
            let client = cf_client.clone();
            let mods_dir = mods_dir.clone();
            let cached_file_info = file_map.get(&mod_file.file_id).cloned();

            set.spawn(async move {
                let _permit = sem
                    .acquire_owned()
                    .await
                    .map_err(|e| AppError::InternalError(e.to_string()))?;

                // Destructure rather than `is_some()` + `unwrap()` so no panic path exists on
                // this externally-supplied CurseForge metadata.
                let cached = cached_file_info
                    .as_ref()
                    .and_then(|info| {
                        info.download_url
                            .clone()
                            .map(|url| (url, info.file_name.clone()))
                    });

                let (download_url, file_name) = match cached {
                    Some(pair) => pair,
                    None => {
                        let url = client.get_file_download_url(mod_file.project_id, mod_file.file_id).await?;
                        let name = format!("{}_{}.jar", mod_file.project_id, mod_file.file_id);
                        (url, name)
                    }
                };

                let dest_file_path = mods_dir.join(&file_name);
                client.download_file(&download_url, &dest_file_path).await?;
                Ok::<String, AppError>(file_name)
            });
        }

        while let Some(res) = set.join_next().await {
            match res {
                Ok(Ok(_)) => {
                    installed_mods_count += 1;
                }
                Ok(Err(e)) => {
                    warn!("Mod download error: {:?}", e);
                }
                Err(e) => {
                    warn!("Mod download task join error: {:?}", e);
                }
            }
        }
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();

    let meta = ProfileMeta {
        name: profile_name.clone(),
        created_at: now,
        modpack_name: Some(mod_details.name.clone()),
        modpack_id: Some(req.modpack_id),
    };
    if let Ok(meta_json) = serde_json::to_string_pretty(&meta) {
        let _ = tokio::fs::write(profile_dir.join("profile.json"), meta_json).await;
    }

    switch_profile(profiles_dir, minecraft_data_dir, &profile_name).await?;

    let message = if skipped_entries > 0 {
        format!(
            "Deployed modpack '{}', but {} override entr{} were skipped (malformed path separators in the archive)",
            mod_details.name,
            skipped_entries,
            if skipped_entries == 1 { "y" } else { "ies" }
        )
    } else {
        format!("Successfully deployed modpack '{}'", mod_details.name)
    };

    Ok(DeployModpackResponse {
        success: true,
        profile_name,
        modpack_id: req.modpack_id,
        installed_mods_count,
        message,
    })
}
