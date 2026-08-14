use axum::{
    extract::{FromRequestParts, Query},
    http::request::Parts,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    auth::{jwt::verify_token, tokens::TokenStore, users::UserStore},
    config::AppConfig,
    error::AppError,
    models::auth::UserClaims,
};

#[derive(Debug, Clone)]
pub struct AuthUser(pub UserClaims);

/// Authenticated *and* holding the `admin` role. Same token handling as [`AuthUser`].
#[derive(Debug, Clone)]
pub struct RequireAdmin(pub UserClaims);

/// Resolve the effective role of a username.
///
/// The `ADMIN_USERNAME` account is the bootstrap admin and always resolves to `admin`,
/// even if it is absent from the user store - locking it out would lock out the panel.
/// Unknown accounts get the least privilege (`viewer`).
// ponytail: linear scan over list_users() per call; fine for a homelab store of a few
// accounts, swap for a UserStore::get_role() lookup if it ever grows.
pub async fn resolve_role(user_store: &UserStore, config: &AppConfig, username: &str) -> String {
    let key = username.trim().to_lowercase();

    if let Some(user) = user_store
        .list_users()
        .await
        .into_iter()
        .find(|u| u.username.trim().to_lowercase() == key)
    {
        return user.role;
    }

    if key == config.admin_username.trim().to_lowercase() {
        "admin".to_string()
    } else {
        "viewer".to_string()
    }
}

/// True when the request authenticated with a persistent API access token
/// (`chipanel_sec_...`). Those tokens carry no role of their own and are only
/// issuable by an admin, so they are treated as admin-equivalent.
fn used_api_access_token(parts: &Parts) -> bool {
    parts
        .headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .or_else(|| {
            parts
                .headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .map(str::trim)
        })
        .is_some_and(|t| t.starts_with("chipanel_sec_"))
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;

        if used_api_access_token(parts) {
            return Ok(RequireAdmin(claims));
        }

        let config = parts
            .extensions
            .get::<Arc<AppConfig>>()
            .ok_or_else(|| AppError::InternalError("AppConfig missing from request extensions".to_string()))?
            .clone();

        let user_store = parts
            .extensions
            .get::<Arc<UserStore>>()
            .ok_or_else(|| AppError::InternalError("UserStore missing from request extensions".to_string()))?
            .clone();

        if resolve_role(&user_store, &config, &claims.sub).await == "admin" {
            Ok(RequireAdmin(claims))
        } else {
            Err(AppError::Forbidden)
        }
    }
}

#[derive(Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token_query;
        let token_str: &str = if let Some(api_key_header) = parts.headers.get("x-api-key") {
            api_key_header
                .to_str()
                .map_err(|_| AppError::AuthError("Invalid X-API-Key header encoding".to_string()))?
                .trim()
        } else if let Some(auth_header) = parts.headers.get(axum::http::header::AUTHORIZATION) {
            let auth_str = auth_header
                .to_str()
                .map_err(|_| AppError::AuthError("Invalid Authorization header encoding".to_string()))?;
            auth_str
                .strip_prefix("Bearer ")
                .map(str::trim)
                .ok_or_else(|| AppError::AuthError("Authorization header must start with Bearer".to_string()))?
        } else if parts.uri.path().starts_with("/ws") {
            if let Ok(query) = Query::<TokenQuery>::try_from_uri(&parts.uri) {
                token_query = query;
                token_query
                    .token
                    .as_deref()
                    .ok_or_else(|| AppError::AuthError("Missing authentication token".to_string()))?
            } else {
                return Err(AppError::AuthError("Missing authentication token".to_string()));
            }
        } else {
            return Err(AppError::AuthError("Missing authentication token".to_string()));
        };

        let config = parts
            .extensions
            .get::<Arc<AppConfig>>()
            .ok_or_else(|| AppError::InternalError("AppConfig missing from request extensions".to_string()))?;

        // 1. First check if it's a persistent API Access Token (starts with chipanel_sec_)
        if token_str.starts_with("chipanel_sec_") {
            if let Some(token_store) = parts.extensions.get::<Arc<TokenStore>>() {
                if let Some(claims) = token_store.validate_token(token_str).await {
                    return Ok(AuthUser(claims));
                }
            }
            return Err(AppError::AuthError("Invalid or revoked API Access Token".to_string()));
        }

        // 2. Otherwise verify standard JWT token
        let claims = verify_token(token_str, config.jwt_secret.as_bytes())?;

        Ok(AuthUser(claims))
    }
}
