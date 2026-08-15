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
            .timeout(std::time::Duration::from_secs(30))
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
        let mut url = format!("{}/search?query={}&limit=24", self.base_url, urlencoding_simple(query));

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
        if project_id.trim().is_empty()
            || project_id.contains('/')
            || project_id.contains('\\')
            || project_id.contains(' ')
            || project_id.contains("..")
        {
            return Err(AppError::BadRequest("Invalid project_id: path traversal or invalid characters forbidden".to_string()));
        }

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

        let primary_file = target_version
            .files
            .iter()
            .find(|f| f.primary && f.filename.ends_with(".jar"))
            .or_else(|| target_version.files.iter().find(|f| f.filename.ends_with(".jar")))
            .or_else(|| target_version.files.first())
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

        info!("Downloading Modrinth plugin file from '{}' to {:?}", primary_file.url, dest_file_path);

        let response = self
            .client
            .get(&primary_file.url)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to download plugin file: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::InternalError(format!(
                "Download HTTP status error: {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to read plugin file bytes: {}", e)))?;

        let bytes_len = bytes.len() as u64;

        tokio::fs::write(&dest_file_path, &bytes)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to write plugin file to disk: {}", e)))?;

        info!("Successfully installed plugin file '{}' ({} bytes)", filename, bytes_len);

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

    /// Downloads an arbitrary file URL to `dest_path`, returning the byte count.
    /// Used for sources outside Modrinth (e.g. the spark Jenkins CI).
    pub async fn download_to(&self, url: &str, dest_path: &Path) -> Result<u64, AppError> {        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to download file: {}", e)))?;
        if !response.status().is_success() {
            return Err(AppError::InternalError(format!(
                "Download HTTP status error: {}",
                response.status()
            )));
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to read file bytes: {}", e)))?;
        let len = bytes.len() as u64;
        tokio::fs::write(dest_path, &bytes)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to write file to disk: {}", e)))?;
        Ok(len)
    }

    /// Installs the latest version of `project` matching `loaders` + `game_version`
    /// into `target_dir`, returning `(version_number, filename)`.
    pub async fn install_latest_filtered(
        &self,
        project: &str,
        loaders: &[&str],
        game_version: &str,
        target_dir: &Path,
    ) -> Result<(String, String), AppError> {
        let versions = self
            .get_project_versions_filtered(project, loaders, game_version)
            .await?;

        let version = versions
            .into_iter()
            .find(|v| !v.files.is_empty())
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "No compatible version found for project '{}' (loaders {:?}, MC {})",
                    project, loaders, game_version
                ))
            })?;

        self.install_version_file(&version, target_dir).await
    }

    /// Installs the primary jar of a specific version into `target_dir`.
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
        .find(|f| f.primary && f.filename.ends_with(".jar"))
        .or_else(|| version.files.iter().find(|f| f.filename.ends_with(".jar")))
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
