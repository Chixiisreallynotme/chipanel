use axum::{
    response::Json,
    routing::post,
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;
use std::sync::Arc;

use crate::{
    auth::middleware::RequireAdmin,
    config::AppConfig,
    error::AppError,
    minecraft::files::canonicalize_and_validate_path_with_base,
};

pub fn diff_router() -> Router {
    Router::new().route("/", post(compute_diff_handler))
}

#[derive(Deserialize)]
pub struct ConfigDiffRequest {
    pub path: String,
    pub new_content: String,
}

#[derive(Serialize)]
pub struct DiffLineItem {
    pub tag: String, // "equal", "insert", "delete"
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub content: String,
}

#[derive(Serialize)]
pub struct ConfigDiffResponse {
    pub has_changes: bool,
    pub additions: usize,
    pub deletions: usize,
    pub unified_diff: String,
    pub lines: Vec<DiffLineItem>,
    pub file_path: String,
}

/// Computes unified line-by-line diff between current file on disk and proposed new content.
pub async fn compute_diff_handler(
    _admin: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<ConfigDiffRequest>,
) -> Result<Json<ConfigDiffResponse>, AppError> {
    if payload.path.trim().is_empty() {
        return Err(AppError::BadRequest("File path cannot be empty".to_string()));
    }

    let base_dir = PathBuf::from(&config.minecraft_data_dir);
    let resolved_path = canonicalize_and_validate_path_with_base(&payload.path, &base_dir).await?;

    let old_content = if resolved_path.exists() {
        tokio::fs::read_to_string(&resolved_path)
            .await
            .unwrap_or_default()
    } else {
        String::new()
    };

    let diff = TextDiff::from_lines(&old_content, &payload.new_content);

    let mut additions = 0;
    let mut deletions = 0;
    let mut lines = Vec::new();

    let mut old_idx = 1;
    let mut new_idx = 1;

    for change in diff.iter_all_changes() {
        let (tag, old_line, new_line) = match change.tag() {
            ChangeTag::Delete => {
                deletions += 1;
                let cur = old_idx;
                old_idx += 1;
                ("delete".to_string(), Some(cur), None)
            }
            ChangeTag::Insert => {
                additions += 1;
                let cur = new_idx;
                new_idx += 1;
                ("insert".to_string(), None, Some(cur))
            }
            ChangeTag::Equal => {
                let cur_old = old_idx;
                let cur_new = new_idx;
                old_idx += 1;
                new_idx += 1;
                ("equal".to_string(), Some(cur_old), Some(cur_new))
            }
        };

        lines.push(DiffLineItem {
            tag,
            old_line,
            new_line,
            content: change.value().trim_end_matches(['\r', '\n']).to_string(),
        });
    }

    let unified_diff = diff
        .unified_diff()
        .context_radius(3)
        .header(&payload.path, &payload.path)
        .to_string();

    let has_changes = additions > 0 || deletions > 0;

    Ok(Json(ConfigDiffResponse {
        has_changes,
        additions,
        deletions,
        unified_diff,
        lines,
        file_path: payload.path,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_computation() {
        let old = "difficulty=easy\npvp=true\nmotd=Hello\n";
        let new = "difficulty=hard\npvp=true\nmotd=Hello World\nview-distance=12\n";

        let diff = TextDiff::from_lines(old, new);
        let mut additions = 0;
        let mut deletions = 0;

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Insert => additions += 1,
                ChangeTag::Delete => deletions += 1,
                ChangeTag::Equal => {}
            }
        }

        assert_eq!(deletions, 2); // difficulty=easy, motd=Hello
        assert_eq!(additions, 3); // difficulty=hard, motd=Hello World, view-distance=12
    }
}

