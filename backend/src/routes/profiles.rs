use axum::{
    extract::{Extension, Path},
    routing::{delete, get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::{
    auth::{middleware::AuthUser, profiles::ProfileStore},
    error::AppError,
    models::profiles::{ApplyProfileRequest, CreateProfileRequest, ServerProfile},
};

pub fn profiles_router() -> Router {
    Router::new()
        .route("/", get(list_profiles_handler).post(create_profile_handler))
        .route("/active", get(get_active_profile_handler))
        .route("/:id/apply", post(apply_profile_handler))
        .route("/:id", delete(delete_profile_handler))
        .route("/export/:id", get(export_profile_handler))
        .route("/import", post(import_profile_handler))
}

/// Handler for GET /api/profiles
pub async fn list_profiles_handler(
    _auth: AuthUser,
    Extension(profile_store): Extension<Arc<ProfileStore>>,
) -> Result<Json<Vec<ServerProfile>>, AppError> {
    let list = profile_store.list_profiles().await;
    Ok(Json(list))
}

/// Handler for GET /api/profiles/active
pub async fn get_active_profile_handler(
    _auth: AuthUser,
    Extension(profile_store): Extension<Arc<ProfileStore>>,
) -> Result<Json<Value>, AppError> {
    let active = profile_store.get_active_profile().await;
    Ok(Json(json!({
        "active_profile": active
    })))
}

/// Handler for POST /api/profiles
pub async fn create_profile_handler(
    _auth: AuthUser,
    Extension(profile_store): Extension<Arc<ProfileStore>>,
    Json(payload): Json<CreateProfileRequest>,
) -> Result<Json<ServerProfile>, AppError> {
    let profile = profile_store.create_profile(payload).await?;
    Ok(Json(profile))
}

/// Handler for POST /api/profiles/:id/apply
pub async fn apply_profile_handler(
    _auth: AuthUser,
    Extension(profile_store): Extension<Arc<ProfileStore>>,
    Path(id): Path<String>,
    Json(payload): Json<ApplyProfileRequest>,
) -> Result<Json<Value>, AppError> {
    let profile = profile_store.set_active_profile(&id).await?;

    let restart_msg = if payload.restart_server.unwrap_or(false) {
        "Le serveur redémarre pour appliquer le nouveau profil."
    } else {
        "Profil sélectionné avec succès."
    };

    Ok(Json(json!({
        "success": true,
        "message": format!("Profil '{}' appliqué avec succès. {}", profile.name, restart_msg),
        "profile": profile
    })))
}

/// Handler for DELETE /api/profiles/:id
pub async fn delete_profile_handler(
    _auth: AuthUser,
    Extension(profile_store): Extension<Arc<ProfileStore>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let deleted = profile_store.delete_profile(&id).await?;
    if !deleted {
        return Err(AppError::NotFound(format!("Profil ID '{}' introuvable", id)));
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("Profil '{}' supprimé avec succès", id)
    })))
}

/// Handler for GET /api/profiles/export/:id
pub async fn export_profile_handler(
    _auth: AuthUser,
    Extension(profile_store): Extension<Arc<ProfileStore>>,
    Path(id): Path<String>,
) -> Result<Json<ServerProfile>, AppError> {
    let profile = profile_store
        .get_profile(&id)
        .await
        .ok_or_else(|| AppError::NotFound(format!("Profil ID '{}' introuvable", id)))?;

    Ok(Json(profile))
}

/// Handler for POST /api/profiles/import
pub async fn import_profile_handler(
    _auth: AuthUser,
    Extension(profile_store): Extension<Arc<ProfileStore>>,
    Json(payload): Json<CreateProfileRequest>,
) -> Result<Json<ServerProfile>, AppError> {
    let imported = profile_store.create_profile(payload).await?;
    Ok(Json(imported))
}
