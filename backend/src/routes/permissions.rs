use axum::{
    extract::Path,
    response::Json,
    routing::{get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    auth::middleware::{AuthUser, RequireAdmin},
    config::AppConfig,
    error::AppError,
    minecraft::permissions::{
        parse_group_detail, parse_luckperms_listgroups, sanitize_group_name, sanitize_meta_value,
        sanitize_permission_node, GroupDetail, LuckPermsGroup,
    },
    rcon::RconClient,
};

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub weight: Option<i32>,
    pub prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddPermissionRequest {
    pub permission: String,
    pub value: bool,
}

#[derive(Debug, Deserialize)]
pub struct RemovePermissionRequest {
    pub permission: String,
}

#[derive(Debug, Deserialize)]
pub struct AddParentRequest {
    pub parent_group: String,
}

#[derive(Debug, Deserialize)]
pub struct RemoveParentRequest {
    pub parent_group: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupMetaRequest {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub weight: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub success: bool,
    pub message: String,
}

pub fn permissions_router() -> Router {
    Router::new()
        .route("/groups", get(list_groups_handler))
        .route("/groups/create", post(create_group_handler))
        .route(
            "/groups/:name",
            get(get_group_handler).delete(delete_group_handler),
        )
        .route(
            "/groups/:name/permission",
            post(set_group_permission_handler).delete(unset_group_permission_handler),
        )
        .route(
            "/groups/:name/parent",
            post(add_group_parent_handler).delete(remove_group_parent_handler),
        )
        .route("/groups/:name/meta", post(update_group_meta_handler))
}

/// GET /api/permissions/groups
/// Returns list of all LuckPerms groups.
pub async fn list_groups_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Result<Json<Vec<LuckPermsGroup>>, AppError> {
    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let output = rcon_client
        .exec("lp listgroups")
        .await
        .map_err(|err| AppError::InternalError(format!("RCON command failed: {}", err)))?;

    let base_groups = parse_luckperms_listgroups(&output);
    let mut detailed_groups = Vec::new();

    for group in base_groups {
        let info_cmd = format!("lp group {} info", group.name);
        if let Ok(info_output) = rcon_client.exec(&info_cmd).await {
            let perm_cmd = format!("lp group {} permission info", group.name);
            let perm_output = rcon_client.exec(&perm_cmd).await.ok();
            let detail = parse_group_detail(&group.name, &info_output, perm_output.as_deref());
            detailed_groups.push(LuckPermsGroup::from(detail));
        } else {
            detailed_groups.push(group);
        }
    }

    Ok(Json(detailed_groups))
}

/// POST /api/permissions/groups/create
/// Executes RCON `lp creategroup <name>`.
pub async fn create_group_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<Json<ActionResponse>, AppError> {
    let name = sanitize_group_name(&payload.name)?;

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let create_cmd = format!("lp creategroup {}", name);
    let _output = rcon_client
        .exec(&create_cmd)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON command failed: {}", err)))?;

    // The group now exists, but the follow-up commands are part of what the caller asked for:
    // swallowing their errors reported "group created successfully" even when every one of them
    // failed against a dead RCON. Surface the failure, naming the step that did not apply.
    // `err` stays in the log only - it carries the RCON host:port.
    if let Some(weight) = payload.weight {
        let weight_cmd = format!("lp group {} setweight {}", name, weight);
        rcon_client.exec(&weight_cmd).await.map_err(|err| {
            tracing::error!("setweight for group '{}' failed: {}", name, err);
            AppError::PartialFailure(format!(
                "Group '{}' was created but setting its weight failed",
                name
            ))
        })?;
    }

    if let Some(ref prefix) = payload.prefix {
        if !prefix.trim().is_empty() {
            let clean_prefix = sanitize_meta_value(prefix)?;
            let prefix_cmd = format!("lp group {} meta setprefix \"{}\"", name, clean_prefix);
            rcon_client.exec(&prefix_cmd).await.map_err(|err| {
                tracing::error!("setprefix for group '{}' failed: {}", name, err);
                AppError::PartialFailure(format!(
                    "Group '{}' was created but setting its prefix failed",
                    name
                ))
            })?;
        }
    }

    Ok(Json(ActionResponse {
        success: true,
        message: format!("Group '{}' created successfully", name),
    }))
}

/// DELETE /api/permissions/groups/:name
/// Executes RCON `lp deletegroup <name>`.
pub async fn delete_group_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(name): Path<String>,
) -> Result<Json<ActionResponse>, AppError> {
    let clean_name = sanitize_group_name(&name)?;

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let delete_cmd = format!("lp deletegroup {}", clean_name);
    let _output = rcon_client
        .exec(&delete_cmd)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON command failed: {}", err)))?;

    Ok(Json(ActionResponse {
        success: true,
        message: format!("Group '{}' deleted successfully", clean_name),
    }))
}

/// GET /api/permissions/groups/:name
/// Returns GroupDetail for group.
pub async fn get_group_handler(
    _auth: AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(name): Path<String>,
) -> Result<Json<GroupDetail>, AppError> {
    let clean_name = sanitize_group_name(&name)?;

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let info_cmd = format!("lp group {} info", clean_name);
    let info_output = rcon_client
        .exec(&info_cmd)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON command failed: {}", err)))?;

    let perm_cmd = format!("lp group {} permission info", clean_name);
    let perm_output = rcon_client.exec(&perm_cmd).await.ok();

    let detail = parse_group_detail(&clean_name, &info_output, perm_output.as_deref());

    Ok(Json(detail))
}

/// POST /api/permissions/groups/:name/permission
/// Executes RCON `lp group <name> permission set <permission> <value>`.
pub async fn set_group_permission_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(name): Path<String>,
    Json(payload): Json<AddPermissionRequest>,
) -> Result<Json<ActionResponse>, AppError> {
    let clean_name = sanitize_group_name(&name)?;
    let clean_perm = sanitize_permission_node(&payload.permission)?;

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let command = format!(
        "lp group {} permission set {} {}",
        clean_name, clean_perm, payload.value
    );
    let _output = rcon_client
        .exec(&command)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON command failed: {}", err)))?;

    Ok(Json(ActionResponse {
        success: true,
        message: format!(
            "Permission '{}' set to {} for group '{}'",
            clean_perm, payload.value, clean_name
        ),
    }))
}

/// DELETE /api/permissions/groups/:name/permission
/// Executes RCON `lp group <name> permission unset <permission>`.
pub async fn unset_group_permission_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(name): Path<String>,
    Json(payload): Json<RemovePermissionRequest>,
) -> Result<Json<ActionResponse>, AppError> {
    let clean_name = sanitize_group_name(&name)?;
    let clean_perm = sanitize_permission_node(&payload.permission)?;

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let command = format!("lp group {} permission unset {}", clean_name, clean_perm);
    let _output = rcon_client
        .exec(&command)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON command failed: {}", err)))?;

    Ok(Json(ActionResponse {
        success: true,
        message: format!(
            "Permission '{}' unset for group '{}'",
            clean_perm, clean_name
        ),
    }))
}

/// POST /api/permissions/groups/:name/parent
/// Executes RCON `lp group <name> parent add <parent_group>`.
pub async fn add_group_parent_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(name): Path<String>,
    Json(payload): Json<AddParentRequest>,
) -> Result<Json<ActionResponse>, AppError> {
    let clean_name = sanitize_group_name(&name)?;
    let clean_parent = sanitize_group_name(&payload.parent_group)?;

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let command = format!("lp group {} parent add {}", clean_name, clean_parent);
    let _output = rcon_client
        .exec(&command)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON command failed: {}", err)))?;

    Ok(Json(ActionResponse {
        success: true,
        message: format!(
            "Parent group '{}' added to group '{}'",
            clean_parent, clean_name
        ),
    }))
}

/// DELETE /api/permissions/groups/:name/parent
/// Executes RCON `lp group <name> parent remove <parent_group>`.
pub async fn remove_group_parent_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(name): Path<String>,
    Json(payload): Json<RemoveParentRequest>,
) -> Result<Json<ActionResponse>, AppError> {
    let clean_name = sanitize_group_name(&name)?;
    let clean_parent = sanitize_group_name(&payload.parent_group)?;

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    let command = format!("lp group {} parent remove {}", clean_name, clean_parent);
    let _output = rcon_client
        .exec(&command)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON command failed: {}", err)))?;

    Ok(Json(ActionResponse {
        success: true,
        message: format!(
            "Parent group '{}' removed from group '{}'",
            clean_parent, clean_name
        ),
    }))
}

/// POST /api/permissions/groups/:name/meta
/// Executes RCON meta/weight updates for group.
pub async fn update_group_meta_handler(
    _auth: RequireAdmin,
    Extension(config): Extension<Arc<AppConfig>>,
    Path(name): Path<String>,
    Json(payload): Json<UpdateGroupMetaRequest>,
) -> Result<Json<ActionResponse>, AppError> {
    let clean_name = sanitize_group_name(&name)?;

    let mut rcon_client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|err| AppError::InternalError(format!("RCON connection failed: {}", err)))?;

    // Every one of these was previously `let _ = ...`, so this handler reported "Meta updated
    // successfully" even when all five commands failed against a dead RCON.
    let mut applied: Vec<&str> = Vec::new();

    if let Some(ref prefix) = payload.prefix {
        let command = if prefix.trim().is_empty() {
            format!("lp group {} meta clear prefix", clean_name)
        } else {
            let clean_prefix = sanitize_meta_value(prefix)?;
            format!("lp group {} meta setprefix \"{}\"", clean_name, clean_prefix)
        };
        exec_meta_command(&mut rcon_client, &command, "prefix", &clean_name).await?;
        applied.push("prefix");
    }

    if let Some(ref suffix) = payload.suffix {
        let command = if suffix.trim().is_empty() {
            format!("lp group {} meta clear suffix", clean_name)
        } else {
            let clean_suffix = sanitize_meta_value(suffix)?;
            format!("lp group {} meta setsuffix \"{}\"", clean_name, clean_suffix)
        };
        exec_meta_command(&mut rcon_client, &command, "suffix", &clean_name).await?;
        applied.push("suffix");
    }

    if let Some(weight) = payload.weight {
        let command = format!("lp group {} setweight {}", clean_name, weight);
        exec_meta_command(&mut rcon_client, &command, "weight", &clean_name).await?;
        applied.push("weight");
    }

    let message = if applied.is_empty() {
        format!("No meta changes requested for group '{}'", clean_name)
    } else {
        format!(
            "Updated {} for group '{}'",
            applied.join(", "),
            clean_name
        )
    };

    Ok(Json(ActionResponse {
        success: true,
        message,
    }))
}

/// Runs one LuckPerms meta command, turning an RCON failure into an error that names the field
/// that did not apply instead of silently dropping it. The raw `err` is logged, never returned:
/// it carries the RCON host:port that `AppError::InternalError` exists to keep off the wire.
async fn exec_meta_command(
    rcon_client: &mut RconClient,
    command: &str,
    field: &str,
    group: &str,
) -> Result<(), AppError> {
    rcon_client.exec(command).await.map(|_| ()).map_err(|err| {
        tracing::error!("meta update of {} for group '{}' failed: {}", field, group, err);
        AppError::PartialFailure(format!("Failed to update {} for group '{}'", field, group))
    })
}
