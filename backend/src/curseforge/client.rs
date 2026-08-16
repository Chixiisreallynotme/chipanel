use std::{
    env,
    fmt::Write,
    path::{Component, Path},
};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use tokio_util::io::StreamReader;
use tracing::info;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeLogo {
    #[serde(default)]
    pub url: String,
    #[serde(default, alias = "thumbnailUrl")]
    pub thumbnail_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeAuthor {
    #[serde(default)]
    pub id: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeFile {
    #[serde(default)]
    pub id: u32,
    #[serde(default, alias = "modId")]
    pub mod_id: u32,
    #[serde(default, alias = "displayName")]
    pub display_name: String,
    #[serde(default, alias = "fileName")]
    pub file_name: String,
    #[serde(default, alias = "downloadUrl")]
    pub download_url: Option<String>,
    #[serde(default, alias = "fileDate")]
    pub file_date: String,
    #[serde(default, alias = "fileLength")]
    pub file_length: u64,
    #[serde(default, alias = "gameVersions")]
    pub game_versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeMod {
    pub id: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default, alias = "downloadCount")]
    pub download_count: f64,
    #[serde(default, alias = "websiteUrl")]
    pub website_url: Option<String>,
    pub logo: Option<CurseForgeLogo>,
    #[serde(default, alias = "latestFiles")]
    pub latest_files: Vec<CurseForgeFile>,
    #[serde(default)]
    pub authors: Vec<CurseForgeAuthor>,
}

#[derive(Debug, Deserialize)]
struct CurseForgeResponse<T> {
    pub data: T,
}

#[derive(Serialize)]
struct BatchFileRequest {
    #[serde(rename = "fileIds")]
    pub file_ids: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct CurseForgeClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl Default for CurseForgeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CurseForgeClient {
    pub fn new() -> Self {
        let api_key = env::var("CURSEFORGE_API_KEY").unwrap_or_default();

        let client = reqwest::Client::builder()
            .user_agent("ChiPanel/0.1.0 (https://github.com/chiserv/chipanel)")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            base_url: "https://api.curseforge.com/v1".to_string(),
            api_key,
        }
    }

    #[allow(dead_code)]
    pub fn with_api_key(api_key: String) -> Self {
        let mut client = Self::new();
        client.api_key = api_key;
        client
    }

    /// Search modpacks on CurseForge (gameId 433 for Minecraft, classId 4471 for Modpacks)
    pub async fn search_modpacks(&self, query: &str) -> Result<Vec<CurseForgeMod>, AppError> {
        let encoded_query = urlencoding_simple(query);
        let url = format!(
            "{}/mods/search?gameId=433&classId=4471&searchFilter={}&pageSize=20&sortField=2&sortOrder=desc",
            self.base_url, encoded_query
        );

        let mut req = self
            .client
            .get(&url)
            .header("Accept", "application/json");

        if !self.api_key.is_empty() {
            req = req.header("x-api-key", &self.api_key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("CurseForge search request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::InternalError(format!(
                "CurseForge API returned status {}",
                resp.status()
            )));
        }

        let result: CurseForgeResponse<Vec<CurseForgeMod>> = resp
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse CurseForge search response: {}", e)))?;

        Ok(result.data)
    }

    /// Fetch details for a specific CurseForge mod/modpack project
    pub async fn get_modpack_details(&self, mod_id: u32) -> Result<CurseForgeMod, AppError> {
        let url = format!("{}/mods/{}", self.base_url, mod_id);

        let mut req = self
            .client
            .get(&url)
            .header("Accept", "application/json");

        if !self.api_key.is_empty() {
            req = req.header("x-api-key", &self.api_key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("CurseForge details request failed: {}", e)))?;

        if resp.status().as_u16() == 404 {
            return Err(AppError::NotFound(format!("CurseForge mod ID {} not found", mod_id)));
        }

        if !resp.status().is_success() {
            return Err(AppError::InternalError(format!(
                "CurseForge API returned status {}",
                resp.status()
            )));
        }

        let result: CurseForgeResponse<CurseForgeMod> = resp
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse CurseForge details response: {}", e)))?;

        Ok(result.data)
    }

    /// Fetch all file releases for a specific CurseForge mod/modpack
    pub async fn get_modpack_files(&self, mod_id: u32) -> Result<Vec<CurseForgeFile>, AppError> {
        let url = format!("{}/mods/{}/files?pageSize=50", self.base_url, mod_id);

        let mut req = self
            .client
            .get(&url)
            .header("Accept", "application/json");

        if !self.api_key.is_empty() {
            req = req.header("x-api-key", &self.api_key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("CurseForge files request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::InternalError(format!(
                "CurseForge API returned status {}",
                resp.status()
            )));
        }

        let result: CurseForgeResponse<Vec<CurseForgeFile>> = resp
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse CurseForge files response: {}", e)))?;

        Ok(result.data)
    }

    /// Fetch direct download URL for a specific CurseForge file ID
    pub async fn get_file_download_url(&self, mod_id: u32, file_id: u32) -> Result<String, AppError> {
        let url = format!("{}/mods/{}/files/{}/download-url", self.base_url, mod_id, file_id);

        let mut req = self
            .client
            .get(&url)
            .header("Accept", "application/json");

        if !self.api_key.is_empty() {
            req = req.header("x-api-key", &self.api_key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("CurseForge download-url request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::InternalError(format!(
                "CurseForge API returned status {}",
                resp.status()
            )));
        }

        let result: CurseForgeResponse<String> = resp
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse CurseForge download URL response: {}", e)))?;

        Ok(result.data)
    }

    /// Batch fetch details for a list of CurseForge file IDs
    pub async fn get_files_batch(&self, file_ids: &[u32]) -> Result<Vec<CurseForgeFile>, AppError> {
        if file_ids.is_empty() {
            return Ok(Vec::new());
        }

        let url = format!("{}/mods/files", self.base_url);
        let payload = BatchFileRequest {
            file_ids: file_ids.to_vec(),
        };

        let mut req = self
            .client
            .post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&payload);

        if !self.api_key.is_empty() {
            req = req.header("x-api-key", &self.api_key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("CurseForge batch files request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::InternalError(format!(
                "CurseForge API returned status {}",
                resp.status()
            )));
        }

        let result: CurseForgeResponse<Vec<CurseForgeFile>> = resp
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse CurseForge batch files response: {}", e)))?;

        Ok(result.data)
    }

    /// Downloads a remote file URL directly into the destination path
    pub async fn download_file(&self, download_url: &str, dest_file_path: &Path) -> Result<u64, AppError> {
        // Validate URL scheme: must start with https://
        if !download_url.to_lowercase().starts_with("https://") {
            return Err(AppError::BadRequest(format!(
                "Invalid download URL scheme: '{}'. Must start with 'https://'",
                download_url
            )));
        }

        // Sanitize dest_file_path against path traversal
        let path_str = dest_file_path.to_string_lossy();
        if path_str.trim().is_empty() {
            return Err(AppError::BadRequest("Destination file path cannot be empty".to_string()));
        }

        for comp in dest_file_path.components() {
            if comp == Component::ParentDir {
                return Err(AppError::BadRequest(
                    "Path traversal ('..') is not allowed in destination file path".to_string(),
                ));
            }
        }

        if let Some(file_name) = dest_file_path.file_name() {
            let name = file_name.to_string_lossy();
            if name.trim().is_empty() || name == "." || name == ".." {
                return Err(AppError::BadRequest("Invalid destination file name".to_string()));
            }
        } else {
            return Err(AppError::BadRequest("Destination path must contain a valid file name".to_string()));
        }

        if let Some(parent) = dest_file_path.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| AppError::InternalError(format!("Failed to create parent directory: {}", e)))?;
            }
        }

        info!("Downloading file from '{}' to {:?}", download_url, dest_file_path);

        let resp = self
            .client
            .get(download_url)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to download file from CurseForge: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::InternalError(format!(
                "File download HTTP status error: {}",
                resp.status()
            )));
        }

        let stream = resp
            .bytes_stream()
            .map_err(std::io::Error::other);

        let mut stream_reader = StreamReader::new(stream);

        let mut file = tokio::fs::File::create(dest_file_path)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create target file: {}", e)))?;

        let bytes_len = tokio::io::copy(&mut stream_reader, &mut file)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to stream file to disk: {}", e)))?;

        Ok(bytes_len)
    }
}

pub fn urlencoding_simple(s: &str) -> String {
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
