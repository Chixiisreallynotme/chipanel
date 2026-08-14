use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LuckPermsGroup {
    pub name: String,
    pub weight: i32,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub parents: Vec<String>,
    pub permissions_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupDetail {
    pub name: String,
    pub weight: i32,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub parents: Vec<String>,
    pub permissions: Vec<LuckPermsPermissionNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LuckPermsPermissionNode {
    pub permission: String,
    pub value: bool,
    pub expiry: Option<String>,
    pub context: Option<String>,
}

impl From<GroupDetail> for LuckPermsGroup {
    fn from(detail: GroupDetail) -> Self {
        LuckPermsGroup {
            name: detail.name,
            weight: detail.weight,
            prefix: detail.prefix,
            suffix: detail.suffix,
            parents: detail.parents,
            permissions_count: detail.permissions.len(),
        }
    }
}

/// Sanitizes and validates a group name.
/// Rejects empty names, spaces, slashes, control characters, and unsafe characters.
pub fn sanitize_group_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest("Group name cannot be empty".to_string()));
    }

    if trimmed.chars().any(|c| c.is_control() || c.is_whitespace() || c == '/' || c == '\\') {
        return Err(AppError::BadRequest(format!(
            "Invalid group name '{}': spaces, slashes, and control characters are not allowed",
            trimmed
        )));
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(AppError::BadRequest(format!(
            "Invalid group name '{}': must contain only alphanumeric characters, underscores, hyphens, or dots",
            trimmed
        )));
    }

    Ok(trimmed.to_string())
}

/// Sanitizes and validates a permission node string.
/// Rejects empty permission nodes, spaces, slashes, control characters, and shell special characters.
pub fn sanitize_permission_node(permission: &str) -> Result<String, AppError> {
    let trimmed = permission.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest("Permission node cannot be empty".to_string()));
    }

    if trimmed.chars().any(|c| c.is_control() || c.is_whitespace() || c == '/' || c == '\\') {
        return Err(AppError::BadRequest(format!(
            "Invalid permission node '{}': spaces, slashes, and control characters are not allowed",
            trimmed
        )));
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' || c == ':' || c == '*')
    {
        return Err(AppError::BadRequest(format!(
            "Invalid permission node '{}': contains forbidden characters",
            trimmed
        )));
    }

    Ok(trimmed.to_string())
}

/// Sanitizes meta prefix/suffix strings.
/// Rejects control characters, newlines, null bytes, and quote characters to prevent RCON injection.
pub fn sanitize_meta_value(val: &str) -> Result<String, AppError> {
    let trimmed = val.trim();
    if trimmed.chars().any(|c| c.is_control() || c == '"' || c == '\'' || c == '\\') {
        return Err(AppError::BadRequest(
            "Meta value contains invalid control characters or quotes".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

#[inline]
fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.len() < needle.len() {
        return false;
    }
    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

fn strip_log_prefix(line: &str) -> &str {
    if let Some(idx) = line.find("]: ") {
        &line[idx + 3..]
    } else if let Some(idx) = line.find("] ") {
        &line[idx + 2..]
    } else {
        line
    }
}

/// Parses the output of RCON `lp listgroups` into group names.
pub fn parse_luckperms_group_names(output: &str) -> Vec<String> {
    let mut names = Vec::with_capacity(output.lines().count());

    for line in output.lines() {
        let clean = strip_log_prefix(line).trim();
        if clean.is_empty() {
            continue;
        }

        if contains_ignore_ascii_case(clean, "groups:") {
            if let Some(idx) = clean.find(':') {
                let rest = clean[idx + 1..].trim();
                if !rest.is_empty() {
                    for item in rest.split(',') {
                        let item_clean = item
                            .trim()
                            .trim_start_matches(|c| c == '>' || c == '-' || c == '*' || c == '•' || c == ' ')
                            .trim();
                        let name_part = item_clean
                            .split(&['(', ' ', '['][..])
                            .next()
                            .unwrap_or(item_clean)
                            .trim();
                        if !name_part.is_empty() && !name_part.eq_ignore_ascii_case("groups") {
                            if let Ok(valid_name) = sanitize_group_name(name_part) {
                                if !names.contains(&valid_name) {
                                    names.push(valid_name);
                                }
                            }
                        }
                    }
                }
            }
            continue;
        }

        if clean.starts_with('-') || clean.starts_with('>') || clean.starts_with('*') || clean.starts_with('•') {
            let item_clean = clean
                .trim_start_matches(|c| c == '>' || c == '-' || c == '*' || c == '•' || c == ' ')
                .trim();
            let name_part = item_clean
                .split(&['(', ' ', '['][..])
                .next()
                .unwrap_or(item_clean)
                .trim();

            if !name_part.is_empty() && !name_part.eq_ignore_ascii_case("groups") {
                if let Ok(valid_name) = sanitize_group_name(name_part) {
                    if !names.contains(&valid_name) {
                        names.push(valid_name);
                    }
                }
            }
        }
    }

    names
}

/// Parses the output of RCON `lp listgroups` into `Vec<LuckPermsGroup>`.
pub fn parse_luckperms_listgroups(output: &str) -> Vec<LuckPermsGroup> {
    let names = parse_luckperms_group_names(output);
    let mut groups = Vec::with_capacity(names.len().max(output.lines().count()));

    for line in output.lines() {
        let clean = strip_log_prefix(line).trim();
        if clean.is_empty() {
            continue;
        }

        let item_clean = clean
            .trim_start_matches(|c| c == '>' || c == '-' || c == '*' || c == '•' || c == ' ')
            .trim();

        let name_part = item_clean
            .split(&['(', ' ', '['][..])
            .next()
            .unwrap_or(item_clean)
            .trim();

        if let Ok(valid_name) = sanitize_group_name(name_part) {
            let mut weight = 0;
            if let Some(w_start) = item_clean.find("weight:") {
                let rest = &item_clean[w_start + 7..];
                let num_str = rest.split(&[')', ']', ',', ' '][..]).next().unwrap_or(rest).trim();
                if let Ok(w) = num_str.parse::<i32>() {
                    weight = w;
                }
            }

            if !groups.iter().any(|g: &LuckPermsGroup| g.name == valid_name) {
                groups.push(LuckPermsGroup {
                    name: valid_name,
                    weight,
                    prefix: None,
                    suffix: None,
                    parents: Vec::new(),
                    permissions_count: 0,
                });
            }
        }
    }

    if groups.is_empty() {
        for name in names {
            groups.push(LuckPermsGroup {
                name,
                weight: 0,
                prefix: None,
                suffix: None,
                parents: Vec::new(),
                permissions_count: 0,
            });
        }
    }

    groups
}

/// Parses RCON `lp group <name> permission info` output into `Vec<LuckPermsPermissionNode>`.
pub fn parse_group_permission_info(output: &str) -> Vec<LuckPermsPermissionNode> {
    let mut nodes = Vec::with_capacity(output.lines().count());

    for line in output.lines() {
        let clean = strip_log_prefix(line).trim();
        if clean.is_empty() {
            continue;
        }

        if contains_ignore_ascii_case(clean, "permissions:")
            || contains_ignore_ascii_case(clean, "page ")
            || contains_ignore_ascii_case(clean, "group info:")
        {
            if !contains_ignore_ascii_case(clean, "(true)") && !contains_ignore_ascii_case(clean, "(false)") {
                continue;
            }
        }

        let is_node_line = clean.starts_with('-')
            || clean.starts_with('>')
            || clean.starts_with('+')
            || clean.starts_with('*')
            || clean.starts_with('•')
            || contains_ignore_ascii_case(clean, "(true)")
            || contains_ignore_ascii_case(clean, "(false)")
            || clean.contains('.');

        if !is_node_line {
            continue;
        }

        let node_str = clean
            .trim_start_matches(|c| c == '-' || c == '>' || c == '+' || c == '*' || c == '•' || c == ' ')
            .trim();

        if node_str.is_empty()
            || node_str.starts_with("Group Info")
            || node_str.starts_with("Weight:")
            || node_str.starts_with("Parents:")
            || node_str.starts_with("Display Name:")
        {
            continue;
        }

        let value = !(contains_ignore_ascii_case(clean, "(false)")
            || contains_ignore_ascii_case(clean, "value=false")
            || contains_ignore_ascii_case(clean, "= false"));

        let mut expiry: Option<String> = None;
        let mut context: Option<String> = None;

        if let Some(exp_start) = node_str.find("expires in ") {
            let rest = &node_str[exp_start + 11..];
            let exp_val = rest.split(&[')', ']'][..]).next().unwrap_or(rest).trim();
            if !exp_val.is_empty() {
                expiry = Some(exp_val.to_string());
            }
        } else if let Some(exp_start) = node_str.find("expiry=") {
            let rest = &node_str[exp_start + 7..];
            let exp_val = rest.split(&[')', ']', ' '][..]).next().unwrap_or(rest).trim();
            if !exp_val.is_empty() {
                expiry = Some(exp_val.to_string());
            }
        }

        if let Some(c_start) = node_str.find('[') {
            if let Some(c_end) = node_str[c_start..].find(']') {
                let inside = &node_str[c_start + 1..c_start + c_end];
                if !inside.starts_with("expires") {
                    context = Some(inside.to_string());
                }
            }
        }

        let perm_part = node_str
            .split(&['(', '=', '['][..])
            .next()
            .unwrap_or(node_str)
            .trim();

        if !perm_part.is_empty() && (perm_part.contains('.') || perm_part.contains('*')) {
            if let Ok(valid_perm) = sanitize_permission_node(perm_part) {
                if !nodes.iter().any(|n: &LuckPermsPermissionNode| n.permission == valid_perm) {
                    nodes.push(LuckPermsPermissionNode {
                        permission: valid_perm,
                        value,
                        expiry,
                        context,
                    });
                }
            }
        }
    }

    nodes
}

/// Parses RCON `lp group <name> info` and `lp group <name> permission info` into `GroupDetail`.
pub fn parse_group_detail(group_name: &str, info_output: &str, perm_output: Option<&str>) -> GroupDetail {
    let clean_name = sanitize_group_name(group_name).unwrap_or_else(|_| group_name.to_string());

    let mut weight: i32 = 0;
    let mut prefix: Option<String> = None;
    let mut suffix: Option<String> = None;
    let info_lines = info_output.lines().count();
    let perm_lines = perm_output.map_or(0, |s| s.lines().count());
    let mut parents: Vec<String> = Vec::with_capacity(info_lines);
    let mut permissions: Vec<LuckPermsPermissionNode> = Vec::with_capacity(info_lines + perm_lines);

    for line in info_output.lines() {
        let clean = strip_log_prefix(line).trim();
        if clean.is_empty() {
            continue;
        }

        if contains_ignore_ascii_case(clean, "weight:") || contains_ignore_ascii_case(clean, "weight =") {
            if let Some(colon_idx) = clean.find(':') {
                let val_str = clean[colon_idx + 1..].trim();
                let num_part = val_str.split(&[' ', '(', ')'][..]).next().unwrap_or(val_str).trim();
                if let Ok(w) = num_part.parse::<i32>() {
                    weight = w;
                }
            }
        } else if contains_ignore_ascii_case(clean, "prefix:") || contains_ignore_ascii_case(clean, "prefix =") {
            if let Some(colon_idx) = clean.find(':') {
                let val_str = clean[colon_idx + 1..].trim();
                let trimmed_val = val_str.trim_matches(|c| c == '"' || c == '\'' || c == ' ');
                if !trimmed_val.is_empty() && !trimmed_val.eq_ignore_ascii_case("none") {
                    prefix = Some(trimmed_val.to_string());
                }
            }
        } else if contains_ignore_ascii_case(clean, "suffix:") || contains_ignore_ascii_case(clean, "suffix =") {
            if let Some(colon_idx) = clean.find(':') {
                let val_str = clean[colon_idx + 1..].trim();
                let trimmed_val = val_str.trim_matches(|c| c == '"' || c == '\'' || c == ' ');
                if !trimmed_val.is_empty() && !trimmed_val.eq_ignore_ascii_case("none") {
                    suffix = Some(trimmed_val.to_string());
                }
            }
        } else if contains_ignore_ascii_case(clean, "parents:")
            || contains_ignore_ascii_case(clean, "inheritance:")
            || contains_ignore_ascii_case(clean, "inherited groups:")
        {
            if let Some(colon_idx) = clean.find(':') {
                let val_str = clean[colon_idx + 1..].trim();
                for item in val_str.split(',') {
                    let parent_name = item.trim().trim_matches(|c| c == '"' || c == '\'' || c == ' ');
                    if !parent_name.is_empty() && !parent_name.eq_ignore_ascii_case("none") {
                        if let Ok(valid_parent) = sanitize_group_name(parent_name) {
                            if !parents.contains(&valid_parent) {
                                parents.push(valid_parent);
                            }
                        }
                    }
                }
            }
        }
    }

    let parsed_nodes_info = parse_group_permission_info(info_output);
    for node in parsed_nodes_info {
        if let Some(parent_name) = node.permission.strip_prefix("group.") {
            if let Ok(valid_parent) = sanitize_group_name(parent_name) {
                if !parents.contains(&valid_parent) {
                    parents.push(valid_parent);
                }
            }
        } else if let Some(parent_name) = node.permission.strip_prefix("parent.") {
            if let Ok(valid_parent) = sanitize_group_name(parent_name) {
                if !parents.contains(&valid_parent) {
                    parents.push(valid_parent);
                }
            }
        } else if prefix.is_none() && node.permission.starts_with("prefix.") {
            if let Some(rest) = node.permission.strip_prefix("prefix.") {
                if let Some((_weight, val)) = rest.split_once('.') {
                    prefix = Some(val.to_string());
                }
            }
        } else if suffix.is_none() && node.permission.starts_with("suffix.") {
            if let Some(rest) = node.permission.strip_prefix("suffix.") {
                if let Some((_weight, val)) = rest.split_once('.') {
                    suffix = Some(val.to_string());
                }
            }
        }

        if !permissions.iter().any(|n| n.permission == node.permission) {
            permissions.push(node);
        }
    }

    if let Some(p_output) = perm_output {
        let extra_nodes = parse_group_permission_info(p_output);
        for node in extra_nodes {
            if let Some(parent_name) = node.permission.strip_prefix("group.") {
                if let Ok(valid_parent) = sanitize_group_name(parent_name) {
                    if !parents.contains(&valid_parent) {
                        parents.push(valid_parent);
                    }
                }
            } else if let Some(parent_name) = node.permission.strip_prefix("parent.") {
                if let Ok(valid_parent) = sanitize_group_name(parent_name) {
                    if !parents.contains(&valid_parent) {
                        parents.push(valid_parent);
                    }
                }
            }

            if !permissions.iter().any(|n| n.permission == node.permission) {
                permissions.push(node);
            }
        }
    }

    GroupDetail {
        name: clean_name,
        weight,
        prefix,
        suffix,
        parents,
        permissions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_group_name() {
        assert_eq!(sanitize_group_name("admin").unwrap(), "admin");
        assert_eq!(sanitize_group_name(" vip_group-1 ").unwrap(), "vip_group-1");
        assert!(sanitize_group_name("admin group").is_err());
        assert!(sanitize_group_name("admin/mod").is_err());
        assert!(sanitize_group_name("admin\n").is_err());
        assert!(sanitize_group_name("").is_err());
    }

    #[test]
    fn test_sanitize_permission_node() {
        assert_eq!(sanitize_permission_node("minecraft.command.kick").unwrap(), "minecraft.command.kick");
        assert_eq!(sanitize_permission_node("essentials.*").unwrap(), "essentials.*");
        assert!(sanitize_permission_node("permission node").is_err());
        assert!(sanitize_permission_node("perm/node").is_err());
        assert!(sanitize_permission_node("perm;drop").is_err());
    }

    #[test]
    fn test_parse_luckperms_listgroups() {
        let output = r#"[12:34:56 INFO]: > Groups: default, admin, moderator
[12:34:56 INFO]: - default (weight: 0)
[12:34:56 INFO]: - admin (weight: 100)"#;

        let groups = parse_luckperms_listgroups(output);
        assert!(!groups.is_empty());
        assert!(groups.iter().any(|g| g.name == "admin"));
        assert!(groups.iter().any(|g| g.name == "default"));
    }

    #[test]
    fn test_contains_ignore_ascii_case() {
        assert!(contains_ignore_ascii_case("GROUPS: default", "groups:"));
        assert!(contains_ignore_ascii_case("[12:34:56 INFO]: > Groups: default", "groups:"));
        assert!(!contains_ignore_ascii_case("user list", "groups:"));
        assert!(contains_ignore_ascii_case("Permissions: (count: 2)", "permissions:"));
        assert!(contains_ignore_ascii_case("some.permission (FALSE)", "(false)"));
    }

    #[test]
    fn test_parse_group_detail() {
        let info_output = r#"[12:34:56 INFO]: > Group Info: admin
[12:34:56 INFO]: - Weight: 100
[12:34:56 INFO]: - Prefix: "[Admin] "
[12:34:56 INFO]: - Suffix: " [Staff]"
[12:34:56 INFO]: - Parents: default, builder
[12:34:56 INFO]: - Permissions: (count: 4)
[12:34:56 INFO]:   - minecraft.command.kick (true)
[12:34:56 INFO]:   - example.perm (false) (expires in 2d)
[12:34:56 INFO]:   - prefix.100.[Admin] (true)
[12:34:56 INFO]:   - suffix.100. [Staff] (true)"#;

        let detail = parse_group_detail("admin", info_output, None);
        assert_eq!(detail.name, "admin");
        assert_eq!(detail.weight, 100);
        assert_eq!(detail.prefix, Some("[Admin] ".to_string()));
        assert_eq!(detail.suffix, Some(" [Staff]".to_string()));
        assert_eq!(detail.parents, vec!["default".to_string(), "builder".to_string()]);
        assert_eq!(detail.permissions.len(), 4);
        assert_eq!(detail.permissions[0].permission, "minecraft.command.kick");
        assert_eq!(detail.permissions[0].value, true);
        assert_eq!(detail.permissions[1].permission, "example.perm");
        assert_eq!(detail.permissions[1].value, false);
        assert_eq!(detail.permissions[1].expiry, Some("2d".to_string()));
        assert_eq!(detail.permissions[2].permission, "prefix.100.[Admin]");
        assert_eq!(detail.permissions[3].permission, "suffix.100. [Staff]");
    }
}
