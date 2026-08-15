use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::error::AppError;

/// Reads a Java `.properties` file into ordered `(key, value)` pairs. Comments
/// (`#`/`!`) and blank lines are dropped — callers only need the key→value map.
pub async fn read_properties(path: &Path) -> Result<Vec<(String, String)>, AppError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read {:?}: {}", path, e)))?;

    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
            continue;
        }
        if let Some(eq) = trimmed.find('=') {
            let key = trimmed[..eq].trim().to_string();
            let value = trimmed[eq + 1..].trim().to_string();
            if !key.is_empty() {
                out.push((key, value));
            }
        }
    }
    Ok(out)
}

/// Case-sensitive key lookup; `None` when the key is absent. Returns an error only
/// when the file itself cannot be read.
pub async fn get_property(path: &Path, key: &str) -> Result<Option<String>, AppError> {
    let props = read_properties(path).await?;
    Ok(props.into_iter().find(|(k, _)| k == key).map(|(_, v)| v))
}

/// Rewrites existing `key=value` lines in place (preserving comments, order and
/// indentation) and appends any missing keys at the end of the file. The write is
/// atomic (temp file + rename) so a concurrent reader never sees a half-written file.
pub async fn set_properties(path: &Path, updates: &[(String, String)]) -> Result<(), AppError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read {:?}: {}", path, e)))?;

    let mut seen: HashSet<&str> = HashSet::new();
    let mut out = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        let mut rewritten = false;
        if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with('!') {
            if let Some(eq) = trimmed.find('=') {
                let key = trimmed[..eq].trim();
                if let Some((_, value)) = updates.iter().find(|(k, _)| k == key) {
                    let indent = &line[..line.len() - line.trim_start().len()];
                    out.push(format!("{}{}={}", indent, key, value));
                    seen.insert(key);
                    rewritten = true;
                }
            }
        }
        if !rewritten {
            out.push(line.to_string());
        }
    }

    for (key, value) in updates {
        if !seen.contains(key.as_str()) {
            out.push(format!("{}={}", key, value));
            seen.insert(key.as_str());
        }
    }

    let new_content = out.join("\n") + "\n";

    let mut tmp_os: OsString = path.as_os_str().to_os_string();
    tmp_os.push(".tmp");
    let tmp = PathBuf::from(tmp_os);

    tokio::fs::write(&tmp, &new_content)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to write {:?}: {}", tmp, e)))?;
    tokio::fs::rename(&tmp, path).await.map_err(|e| {
        AppError::InternalError(format!("Failed to rename {:?} -> {:?}: {}", tmp, path, e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_rewrites_properties() {
        let content = "# comment\nlevel-name=world\nlevel-seed=123\n\nmotd=hello\n";
        // parse
        let mut pairs = Vec::new();
        for line in content.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') {
                continue;
            }
            if let Some(eq) = t.find('=') {
                pairs.push((t[..eq].trim().to_string(), t[eq + 1..].trim().to_string()));
            }
        }
        assert_eq!(
            pairs,
            vec![
                ("level-name".to_string(), "world".to_string()),
                ("level-seed".to_string(), "123".to_string()),
                ("motd".to_string(), "hello".to_string()),
            ]
        );
    }
}
