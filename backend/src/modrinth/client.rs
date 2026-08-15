use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use tokio_util::io::StreamReader;
use tracing::info;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthProject {
    #[serde(alias = "id")]
    pub project_id: String,
    #[serde(default)]
    pub project_type: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub client_side: String,
    #[serde(default)]
    pub server_side: String,
    #[serde(default)]
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub latest_version: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    pub hits: Vec<ModrinthProject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthVersion {
    pub id: String,
    pub project_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version_number: String,
    pub changelog: Option<String>,
    #[serde(default)]
    pub date_published: String,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub version_type: String,
    #[serde(default)]
    pub files: Vec<ModrinthFile>,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<ModrinthDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthFile {
    pub url: String,
    pub filename: String,
    #[serde(default)]
    pub primary: bool,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub hashes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthDependency {
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub version_id: Option<String>,
    #[serde(default)]
    pub dependency_type: String,
}

#[derive(Debug, Serialize)]
struct VersionFilesUpdateRequest<'a> {
    hashes: &'a [String],
    algorithm: &'a str,
    loaders: &'a [&'a str],
    game_versions: &'a [&'a str],
}

#[derive(Debug, Serialize)]
struct VersionFilesRequest<'a> {
    hashes: &'a [String],
    algorithm: &'a str,
}

#[derive(Debug, Clone)]
pub struct ModrinthClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for ModrinthClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ModrinthClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("ChiPanel/0.1.0 (https://github.com/chiserv/chipanel)")
            .timeout(std::time::Duration::from_secs(45))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            base_url: "https://api.modrinth.com/v2".to_string(),
        }
    }

    pub async fn search_projects(
        &self,
        query: &str,
        project_type: &str,
        game_version: &str,
        loader: &str,
        sort: &str,
    ) -> Result<Vec<ModrinthProject>, AppError> {
        let mut url = format!("{}/search?query={}&limit=30", self.base_url, urlencoding_simple(query));

        if !sort.is_empty() {
            let index_val = match sort {
                "downloads" => "downloads",
                "follows" => "follows",
                "newest" => "newest",
                "updated" => "updated",
                _ => "relevance",
            };
            url.push_str(&format!("&index={}", index_val));
        }

        let mut facets = Vec::new();
        if !project_type.is_empty() {
            facets.push(format!("\"project_type:{}\"", project_type));
        }
        if !game_version.is_empty() && game_version != "all" {
            facets.push(format!("\"versions:{}\"", game_version));
        }
        if !loader.is_empty() && loader != "all" {
            let ldr_clean = loader.to_lowercase();
            facets.push(format!("\"categories:{}\"", ldr_clean));
        }

        if !facets.is_empty() {
            let inner_facets: Vec<String> = facets.iter().map(|f| format!("[{}]", f)).collect();
            let facets_str = format!("[{}]", inner_facets.join(","));
            url.push_str("&facets=");
            url.push_str(&urlencoding_simple(&facets_str));
        }

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Modrinth API request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::InternalError(format!(
                "Modrinth API returned status {}",
                resp.status()
            )));
        }

        let search_res: SearchResponse = resp
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse Modrinth response: {}", e)))?;

        Ok(search_res.hits)
    }

    pub async fn get_project_versions(&self, project_id: &str) -> Result<Vec<ModrinthVersion>, AppError> {
        validate_project_ref(project_id)?;

        let url = format!("{}/project/{}/version", self.base_url, project_id);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Modrinth API request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::InternalError(format!(
                "Modrinth API returned status {}",
                resp.status()
            )));
        }

        let versions: Vec<ModrinthVersion> = resp
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse Modrinth versions response: {}", e)))?;

        Ok(versions)
    }

    pub async fn download_and_install_version(
        &self,
        project_id: &str,
        version_id: Option<&str>,
        target_dir_path: &Path,
    ) -> Result<(String, u64), AppError> {
        let versions = self.get_project_versions(project_id).await?;

        let target_version = match version_id {
            Some(vid) if !vid.is_empty() => versions
                .into_iter()
                .find(|v| v.id == vid)
                .ok_or_else(|| AppError::NotFound(format!("Version ID '{}' not found for project", vid)))?,
            _ => versions
                .into_iter()
                .next()
                .ok_or_else(|| AppError::NotFound("No versions available for project".to_string()))?,
        };

        let primary_file = pick_primary_file(&target_version)
            .ok_or_else(|| AppError::NotFound("No suitable download file found in version".to_string()))?;

        let filename = &primary_file.filename;

        if filename.trim().is_empty()
            || filename == "."
            || filename.contains('/')
            || filename.contains('\\')
            || filename.contains("..")
        {
            return Err(AppError::BadRequest("Invalid download filename".to_string()));
        }

        if !target_dir_path.exists() {
            tokio::fs::create_dir_all(target_dir_path)
                .await
                .map_err(|e| AppError::InternalError(format!("Failed to create target dir: {}", e)))?;
        }

        let dest_file_path = target_dir_path.join(filename);

        info!("Downloading Modrinth file from '{}' to {:?}", primary_file.url, dest_file_path);

        let bytes_len = self.download_to(&primary_file.url, &dest_file_path).await?;

        info!("Successfully installed file '{}' ({} bytes)", filename, bytes_len);

        Ok((filename.clone(), bytes_len))
    }

    /// Fetches project versions filtered by loaders and game version, newest first.
    pub async fn get_project_versions_filtered(
        &self,
        project: &str,
        loaders: &[&str],
        game_version: &str,
    ) -> Result<Vec<ModrinthVersion>, AppError> {
        validate_project_ref(project)?;

        let mut url = format!("{}/project/{}/version", self.base_url, project);
        let mut params: Vec<String> = Vec::new();

        if !loaders.is_empty() {
            let inner = loaders
                .iter()
                .map(|l| format!("\"{}\"", l.replace('"', "")))
                .collect::<Vec<_>>()
                .join(",");
            params.push(format!("loaders={}", urlencoding_simple(&format!("[{}]", inner))));
        }
        if !game_version.is_empty() {
            params.push(format!(
                "game_versions={}",
                urlencoding_simple(&format!("[\"{}\"]", game_version.replace('"', "")))
            ));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Modrinth API request failed: {}", e)))?;
        if !resp.status().is_success() {
            return Err(AppError::InternalError(format!(
                "Modrinth API returned status {}",
                resp.status()
            )));
        }
        resp.json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse Modrinth versions response: {}", e)))
    }

    /// Batch checks updates for multiple file hashes via `POST /v2/version_files/update`.
    /// Returns a map of `[original_hash -> latest_compatible_version]`.
    pub async fn check_version_files_update(
        &self,
        hashes: &[String],
        loaders: &[&str],
        game_versions: &[&str],
    ) -> Result<HashMap<String, ModrinthVersion>, AppError> {
        if hashes.is_empty() {
            return Ok(HashMap::new());
        }

        let url = format!("{}/version_files/update", self.base_url);
        let req_body = VersionFilesUpdateRequest {
            hashes,
            algorithm: "sha512",
            loaders,
            game_versions,
        };

        let resp = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Modrinth update check request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::InternalError(format!(
                "Modrinth update check returned status {}",
                resp.status()
            )));
        }

        let map: HashMap<String, ModrinthVersion> = resp
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse Modrinth update response: {}", e)))?;

        Ok(map)
    }

    /// Batch looks up version metadata for multiple file hashes via `POST /v2/version_files`.
    /// Returns a map of `[hash -> version_info]`.
    pub async fn get_version_files(
        &self,
        hashes: &[String],
    ) -> Result<HashMap<String, ModrinthVersion>, AppError> {
        if hashes.is_empty() {
            return Ok(HashMap::new());
        }

        let url = format!("{}/version_files", self.base_url);
        let req_body = VersionFilesRequest {
            hashes,
            algorithm: "sha512",
        };

        let resp = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Modrinth version files request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::InternalError(format!(
                "Modrinth version files returned status {}",
                resp.status()
            )));
        }

        let map: HashMap<String, ModrinthVersion> = resp
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse Modrinth version files response: {}", e)))?;

        Ok(map)
    }

    /// Fetches an arbitrary JSON endpoint (used for non-Modrinth sources such as
    /// the spark Jenkins CI), returning the raw parsed value.
    pub async fn get_json(&self, url: &str) -> Result<serde_json::Value, AppError> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Request failed: {}", e)))?;
        if !resp.status().is_success() {
            return Err(AppError::InternalError(format!(
                "Request returned status {}",
                resp.status()
            )));
        }
        resp.json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse JSON response: {}", e)))
    }

    /// Streams an arbitrary file URL to `dest_path` without buffering the entire
    /// payload in RAM. Returns the total byte count written.
    pub async fn download_to(&self, url: &str, dest_path: &Path) -> Result<u64, AppError> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to download file from '{}': {}", url, e)))?;

        if !response.status().is_success() {
            return Err(AppError::InternalError(format!(
                "Download HTTP status error: {}",
                response.status()
            )));
        }

        let byte_stream = response
            .bytes_stream()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
        let mut reader = StreamReader::new(byte_stream);

        let mut file = tokio::fs::File::create(dest_path)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create destination file {:?}: {}", dest_path, e)))?;

        let bytes_written = tokio::io::copy(&mut reader, &mut file)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to stream file to disk: {}", e)))?;

        Ok(bytes_written)
    }

    /// Installs the primary jar/zip of a specific version into `target_dir`.
    pub async fn install_version_file(
        &self,
        version: &ModrinthVersion,
        target_dir: &Path,
    ) -> Result<(String, String), AppError> {
        let primary_file = pick_primary_file(version)
            .ok_or_else(|| AppError::NotFound("No suitable download file found in version".to_string()))?;

        let filename = &primary_file.filename;
        if filename.trim().is_empty()
            || filename == "."
            || filename.contains('/')
            || filename.contains('\\')
            || filename.contains("..")
        {
            return Err(AppError::BadRequest("Invalid download filename".to_string()));
        }

        if !target_dir.exists() {
            tokio::fs::create_dir_all(target_dir)
                .await
                .map_err(|e| AppError::InternalError(format!("Failed to create target dir: {}", e)))?;
        }

        let dest_file_path = target_dir.join(filename);
        info!(
            "Downloading Modrinth file '{}' ({}) from '{}' to {:?}",
            version.version_number, filename, primary_file.url, dest_file_path
        );

        self.download_to(&primary_file.url, &dest_file_path).await?;

        Ok((version.version_number.clone(), filename.clone()))
    }
}

fn validate_project_ref(project: &str) -> Result<(), AppError> {
    if project.trim().is_empty()
        || project.contains('/')
        || project.contains('\\')
        || project.contains(' ')
        || project.contains("..")
    {
        return Err(AppError::BadRequest(
            "Invalid project id: path traversal or invalid characters forbidden".to_string(),
        ));
    }
    Ok(())
}

fn pick_primary_file(version: &ModrinthVersion) -> Option<&ModrinthFile> {
    version
        .files
        .iter()
        .find(|f| f.primary && (f.filename.ends_with(".jar") || f.filename.ends_with(".zip")))
        .or_else(|| version.files.iter().find(|f| f.filename.ends_with(".jar") || f.filename.ends_with(".zip")))
        .or_else(|| version.files.first())
}

fn urlencoding_simple(s: &str) -> String {
    let mut encoded = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                let _ = write!(encoded, "%{:02X}", b);
            }
        }
    }
    encoded
}
