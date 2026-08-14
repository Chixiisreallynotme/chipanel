use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::error::AppError;
use crate::rcon::RconClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeNode {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub extension: String,
    pub children: Option<Vec<FileTreeNode>>,
    /// `true` when this node's listing is known to be incomplete (per-directory cap, global
    /// node cap, depth limit, or an I/O error). Bubbles up to ancestors, so the root node
    /// answers "is this whole tree complete?". A truncated listing must never be presented as
    /// authoritative.
    #[serde(default)]
    pub truncated: bool,
    /// Set only when an actual I/O error cut the listing short (as opposed to a configured
    /// limit). Bubbles up so the root carries the first failure message.
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContentResponse {
    pub path: String,
    pub content: String,
    pub size_bytes: u64,
    pub syntax_mode: String,
    pub is_readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSaveRequest {
    pub path: String,
    pub content: String,
    pub trigger_reload: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCreateRequest {
    pub path: String,
    pub is_dir: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileActionResponse {
    pub success: bool,
    pub message: String,
}

/// Maximum items listed per directory.
const MAX_ITEMS_PER_DIR: usize = 500;

/// Maximum directory depth walked in a single tree request.
const MAX_TREE_DEPTH: usize = 4;

/// Hard cap on the total number of nodes in one tree response. Without it, depth 4 × 500
/// items/dir is an unbounded product — a large `world/region` alone blows past the ~30 MB
/// RAM budget this module is written to.
const MAX_TOTAL_NODES: usize = 5_000;

/// Sanitizes and canonicalizes a path relative to the provided base directory.
///
/// Rejects path traversal (`..`), null bytes (`\0`), and control characters (`\n`, `\r`), and
/// verifies the resolved path stays under `base_dir`.
pub async fn canonicalize_and_validate_path_with_base(
    relative_path: &str,
    base_dir: &Path,
) -> Result<PathBuf, AppError> {
    if relative_path.contains('\0') || relative_path.contains('\n') || relative_path.contains('\r') {
        return Err(AppError::BadRequest(
            "Invalid path: control characters forbidden".to_string(),
        ));
    }

    if relative_path.contains("..") {
        return Err(AppError::BadRequest(
            "Invalid path: path traversal forbidden".to_string(),
        ));
    }

    let clean_rel = relative_path.trim_start_matches('/');
    let target_path = if clean_rel.is_empty() {
        base_dir.to_path_buf()
    } else {
        base_dir.join(clean_rel)
    };

    for component in Path::new(clean_rel).components() {
        match component {
            std::path::Component::ParentDir => {
                return Err(AppError::BadRequest(
                    "Path traversal component detected".to_string(),
                ));
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(AppError::BadRequest("Absolute paths forbidden".to_string()));
            }
            _ => {}
        }
    }

    let base_canonical = if base_dir.is_absolute() {
        base_dir.to_path_buf()
    } else if fs::try_exists(base_dir).await.unwrap_or(false) {
        fs::canonicalize(base_dir)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to resolve base directory: {}", e)))?
    } else {
        base_dir.to_path_buf()
    };

    let resolved_path = if fs::try_exists(&target_path).await.unwrap_or(false) {
        fs::canonicalize(&target_path)
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to resolve path: {}", e)))?
    } else if let Some(parent) = target_path.parent() {
        if fs::try_exists(parent).await.unwrap_or(false) {
            let parent_canonical = fs::canonicalize(parent)
                .await
                .map_err(|e| AppError::BadRequest(format!("Failed to resolve parent path: {}", e)))?;
            let file_name = target_path
                .file_name()
                .ok_or_else(|| AppError::BadRequest("Invalid file name".to_string()))?;
            parent_canonical.join(file_name)
        } else {
            target_path
        }
    } else {
        target_path
    };

    if !resolved_path.starts_with(&base_canonical) {
        return Err(AppError::Forbidden);
    }

    Ok(resolved_path)
}

/// Scans a directory tree using a custom base directory, up to [`MAX_TREE_DEPTH`] levels,
/// [`MAX_ITEMS_PER_DIR`] entries per directory and [`MAX_TOTAL_NODES`] nodes overall.
///
/// Hitting any limit, or an I/O error part-way through a listing, sets `truncated` on the
/// affected node and on its ancestors rather than returning a partial tree that looks complete.
pub fn get_file_tree_with_base<'a>(
    subpath: &'a str,
    current_depth: usize,
    base_dir: &'a Path,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FileTreeNode, AppError>> + Send + 'a>> {
    Box::pin(async move {
        let base_canonical = if fs::try_exists(base_dir).await.unwrap_or(false) {
            fs::canonicalize(base_dir)
                .await
                .unwrap_or_else(|_| base_dir.to_path_buf())
        } else {
            base_dir.to_path_buf()
        };

        let node_budget = std::sync::atomic::AtomicUsize::new(0);

        get_file_tree_recursive(subpath, current_depth, &base_canonical, &node_budget).await
    })
}

fn get_file_tree_recursive<'a>(
    subpath: &'a str,
    current_depth: usize,
    base_canonical: &'a Path,
    node_budget: &'a std::sync::atomic::AtomicUsize,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FileTreeNode, AppError>> + Send + 'a>> {
    Box::pin(async move {
        let path_buf = canonicalize_and_validate_path_with_base(subpath, base_canonical).await?;

        let metadata = fs::metadata(&path_buf)
            .await
            .map_err(|e| AppError::NotFound(format!("Path not found: {}", e)))?;

        let name = path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("minecraft-data")
            .to_string();

        let is_dir = metadata.is_dir();
        let size_bytes = if is_dir { 0 } else { metadata.len() };

        let extension = if is_dir {
            String::new()
        } else {
            path_buf
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase()
        };

        let rel_path_buf = path_buf.strip_prefix(base_canonical).unwrap_or(&path_buf);
        let rel_path_str = rel_path_buf.to_string_lossy().replace('\\', "/");

        let mut children = None;
        let mut truncated = false;
        let mut listing_error: Option<String> = None;

        if is_dir {
            if current_depth < MAX_TREE_DEPTH {
                let mut read_dir = fs::read_dir(&path_buf)
                    .await
                    .map_err(|e| AppError::InternalError(format!("Failed to read directory: {}", e)))?;

                let mut child_nodes = Vec::new();
                let mut item_count = 0;

                loop {
                    if item_count >= MAX_ITEMS_PER_DIR {
                        warn!(
                            "Reached {} items limit in directory {:?}",
                            MAX_ITEMS_PER_DIR, path_buf
                        );
                        truncated = true;
                        break;
                    }

                    if node_budget.load(Ordering::Relaxed) >= MAX_TOTAL_NODES {
                        warn!(
                            "Reached {} total node cap while listing {:?}",
                            MAX_TOTAL_NODES, path_buf
                        );
                        truncated = true;
                        break;
                    }

                    // An Err here used to silently end the listing, presenting a partial
                    // directory as complete. Stop, but say so.
                    let entry = match read_dir.next_entry().await {
                        Ok(Some(entry)) => entry,
                        Ok(None) => break,
                        Err(e) => {
                            warn!("Directory listing of {:?} ended early: {}", path_buf, e);
                            truncated = true;
                            listing_error =
                                Some(format!("Directory listing incomplete: {}", e));
                            break;
                        }
                    };

                    item_count += 1;
                    node_budget.fetch_add(1, Ordering::Relaxed);

                    let entry_path = entry.path();
                    let entry_rel = entry_path.strip_prefix(base_canonical).unwrap_or(&entry_path);
                    let entry_rel_str = entry_rel.to_string_lossy().replace('\\', "/");

                    // A single unreadable child must not abort the whole listing either.
                    match get_file_tree_recursive(
                        &entry_rel_str,
                        current_depth + 1,
                        base_canonical,
                        node_budget,
                    )
                    .await
                    {
                        Ok(child_node) => child_nodes.push(child_node),
                        Err(e) => {
                            warn!("Skipping unreadable entry {:?}: {}", entry_path, e);
                            truncated = true;
                            if listing_error.is_none() {
                                listing_error =
                                    Some(format!("Entry '{}' unreadable: {}", entry_rel_str, e));
                            }
                        }
                    }
                }

                child_nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                });

                // Bubble incompleteness up so the root node reflects the whole tree.
                for child in &child_nodes {
                    if child.truncated {
                        truncated = true;
                    }
                    if listing_error.is_none() {
                        listing_error = child.error.clone();
                    }
                }

                children = Some(child_nodes);
            } else {
                // Depth limit: this directory's contents were not walked at all. Reporting an
                // empty `children` without `truncated` would read as "empty folder".
                children = Some(Vec::new());
                truncated = true;
            }
        }

        Ok(FileTreeNode {
            path: rel_path_str,
            name,
            is_dir,
            size_bytes,
            extension,
            children,
            truncated,
            error: listing_error,
        })
    })
}

/// Reads file content up to 5MB max size limit, relative to the specified base directory.
/// Infers syntax_mode based on extension.
pub async fn read_file_content_with_base(
    subpath: &str,
    base_dir: &Path,
) -> Result<FileContentResponse, AppError> {
    let path_buf = canonicalize_and_validate_path_with_base(subpath, base_dir).await?;

    let metadata = fs::metadata(&path_buf)
        .await
        .map_err(|e| AppError::NotFound(format!("File not found: {}", e)))?;

    if metadata.is_dir() {
        return Err(AppError::BadRequest("Target path is a directory".into()));
    }

    const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024; // 5 MB
    if metadata.len() > MAX_FILE_SIZE {
        return Err(AppError::BadRequest(format!(
            "File size ({} bytes) exceeds the maximum limit of 5 MB",
            metadata.len()
        )));
    }

    let extension = path_buf
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let is_binary_ext = matches!(
        extension.as_str(),
        "jar"
            | "dat"
            | "db"
            | "sqlite"
            | "gz"
            | "zip"
            | "tar"
            | "png"
            | "jpg"
            | "jpeg"
            | "nbt"
            | "class"
            | "so"
            | "dll"
            | "exe"
    );

    if is_binary_ext {
        return Err(AppError::BadRequest(format!(
            "Cannot read binary file with extension '.{}'",
            extension
        )));
    }

    let bytes = fs::read(&path_buf)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read file: {}", e)))?;

    let content = String::from_utf8(bytes).map_err(|_| {
        AppError::BadRequest("File contains binary data or invalid UTF-8 encoding".into())
    })?;

    let syntax_mode = infer_syntax_mode(&extension);
    let is_readonly = metadata.permissions().readonly();

    let base_canonical = if fs::try_exists(base_dir).await.unwrap_or(false) {
        fs::canonicalize(base_dir).await.unwrap_or_else(|_| base_dir.to_path_buf())
    } else {
        base_dir.to_path_buf()
    };

    let rel_path_buf = path_buf.strip_prefix(&base_canonical).unwrap_or(&path_buf);
    let rel_path_str = rel_path_buf.to_string_lossy().replace('\\', "/");

    Ok(FileContentResponse {
        path: rel_path_str,
        content,
        size_bytes: metadata.len(),
        syntax_mode,
        is_readonly,
    })
}

/// Infers text editor syntax mode based on file extension.
fn infer_syntax_mode(extension: &str) -> String {
    match extension {
        "yml" | "yaml" => "yaml".to_string(),
        "properties" | "prop" | "conf" | "cfg" => "properties".to_string(),
        "json" => "json".to_string(),
        "toml" => "toml".to_string(),
        "log" => "log".to_string(),
        _ => "txt".to_string(),
    }
}

/// Writes content to a `.tmp_save` file first, then renames to the target path
/// (`tokio::fs::rename`). If `trigger_reload` is true, sends the appropriate RCON command.
pub async fn save_file_content_with_base(
    subpath: &str,
    content: &str,
    trigger_reload: bool,
    config: Option<&AppConfig>,
    base_dir: &Path,
) -> Result<String, AppError> {
    let target_path = canonicalize_and_validate_path_with_base(subpath, base_dir).await?;

    if fs::try_exists(&target_path).await.unwrap_or(false)
        && fs::metadata(&target_path).await.map(|m| m.is_dir()).unwrap_or(false)
    {
        return Err(AppError::BadRequest("Target path is a directory".into()));
    }

    if let Some(parent) = target_path.parent() {
        if !fs::try_exists(parent).await.unwrap_or(false) {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::InternalError(format!("Failed to create directories: {}", e)))?;
        }
    }

    let file_stem = target_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    let tmp_file_name = format!(".{}.tmp_save", file_stem);
    let tmp_path = target_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&tmp_file_name);

    if let Err(e) = fs::write(&tmp_path, content.as_bytes()).await {
        let _ = fs::remove_file(&tmp_path).await;
        return Err(AppError::InternalError(format!("Failed to write temporary file: {}", e)));
    }

    if let Err(e) = fs::rename(&tmp_path, &target_path).await {
        let _ = fs::remove_file(&tmp_path).await;
        return Err(AppError::InternalError(format!("Failed to atomically save file: {}", e)));
    }

    info!("Atomically saved file: {:?}", target_path);

    if trigger_reload {
        execute_live_reload_if_needed(subpath, config).await
    } else {
        Ok("File saved successfully.".to_string())
    }
}

/// Handles live RCON reload logic based on modified file path.
async fn execute_live_reload_if_needed(
    subpath: &str,
    config: Option<&AppConfig>,
) -> Result<String, AppError> {
    let subpath_clean = subpath.replace('\\', "/");
    let lower_path = subpath_clean.to_lowercase();

    if lower_path.ends_with("server.properties") {
        return Ok(
            "File saved successfully. Server restart is required for server.properties changes to take effect."
                .to_string(),
        );
    }

    let rcon_cmd = if lower_path.ends_with("paper-global.yml")
        || lower_path.ends_with("paper-world-defaults.yml")
    {
        Some("paper reload")
    } else if lower_path.ends_with("spigot.yml") || lower_path.ends_with("bukkit.yml") {
        Some("reload confirm")
    } else if lower_path.contains("luckperms") && lower_path.ends_with("config.yml") {
        Some("lp reload")
    } else {
        None
    };

    if let Some(cmd) = rcon_cmd {
        if let Some(cfg) = config {
            // The save itself succeeded, so these stay `Ok` - only the reload step is reported
            // as failed. The RCON error is logged, never interpolated into the message: it
            // carries the RCON host:port.
            match RconClient::connect(&cfg.rcon_host, cfg.rcon_port, &cfg.rcon_password).await {
                Ok(mut client) => match client.exec(cmd).await {
                    Ok(output) => Ok(format!(
                        "File saved successfully. Live RCON reload ('{}') executed: {}",
                        cmd,
                        output.trim()
                    )),
                    Err(e) => {
                        warn!("Live RCON reload '{}' failed: {}", cmd, e);
                        Ok(format!(
                            "File saved successfully, but the live RCON reload ('{}') failed - apply it manually.",
                            cmd
                        ))
                    }
                },
                Err(e) => {
                    warn!("RCON connection for live reload '{}' failed: {}", cmd, e);
                    Ok(format!(
                        "File saved successfully, but the RCON connection failed - the live reload ('{}') was not applied.",
                        cmd
                    ))
                }
            }
        } else {
            Ok(format!(
                "File saved successfully. RCON reload command '{}' was not sent (RCON configuration missing).",
                cmd
            ))
        }
    } else {
        Ok("File saved successfully.".to_string())
    }
}

/// Creates a new file or directory.
pub async fn create_file_or_dir(
    subpath: &str,
    is_dir: bool,
    base_dir: &Path,
) -> Result<String, AppError> {
    let target_path = canonicalize_and_validate_path_with_base(subpath, base_dir).await?;

    if fs::try_exists(&target_path).await.unwrap_or(false) {
        return Err(AppError::BadRequest(format!(
            "Path '{}' already exists",
            subpath
        )));
    }

    if is_dir {
        fs::create_dir_all(&target_path)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create directory: {}", e)))?;
        info!("Created directory: {:?}", target_path);
        Ok(format!("Directory '{}' created successfully", subpath))
    } else {
        if let Some(parent) = target_path.parent() {
            if !fs::try_exists(parent).await.unwrap_or(false) {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| AppError::InternalError(format!("Failed to create parent directories: {}", e)))?;
            }
        }

        fs::write(&target_path, b"")
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create file: {}", e)))?;

        info!("Created empty file: {:?}", target_path);
        Ok(format!("File '{}' created successfully", subpath))
    }
}

/// Deletes a file or an empty directory.
pub async fn delete_file_or_dir(subpath: &str, base_dir: &Path) -> Result<String, AppError> {
    let target_path = canonicalize_and_validate_path_with_base(subpath, base_dir).await?;

    if !fs::try_exists(&target_path).await.unwrap_or(false) {
        return Err(AppError::NotFound(format!("Path '{}' not found", subpath)));
    }

    let base_canonical = if fs::try_exists(base_dir).await.unwrap_or(false) {
        fs::canonicalize(base_dir).await.unwrap_or_else(|_| base_dir.to_path_buf())
    } else {
        base_dir.to_path_buf()
    };

    if target_path == base_canonical {
        return Err(AppError::Forbidden);
    }

    let metadata = fs::metadata(&target_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read metadata: {}", e)))?;

    if metadata.is_dir() {
        fs::remove_dir(&target_path).await.map_err(|e| {
            AppError::BadRequest(format!(
                "Cannot delete directory '{}' (ensure it is empty): {}",
                subpath, e
            ))
        })?;
        info!("Deleted empty directory: {:?}", target_path);
        Ok(format!("Directory '{}' deleted successfully", subpath))
    } else {
        fs::remove_file(&target_path)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to delete file: {}", e)))?;
        info!("Deleted file: {:?}", target_path);
        Ok(format!("File '{}' deleted successfully", subpath))
    }
}
