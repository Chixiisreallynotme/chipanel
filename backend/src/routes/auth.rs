use axum::{
    extract::{Extension, Path},
    routing::{delete, get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use crate::{
    auth::{
        jwt::create_token,
        middleware::{resolve_role, AuthUser, RequireAdmin},
        password::{hash_password, verify_password},
        tokens::TokenStore,
        users::{CreateUserAccountRequest, UserAccountPublic, UserStore},
    },
    config::AppConfig,
    error::AppError,
    models::auth::{
        ApiTokenInfo, AuthResponse, CreateTokenRequest, CreateTokenResponse, LoginRequest,
        UserProfile,
    },
};

static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    hash_password("dummy_password_for_timing_side_channel_mitigation")
        .unwrap_or_else(|_| "$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHQ$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string())
});

/// Login throttling: argon2 is deliberately expensive, so an unauthenticated caller
/// hammering /login is a CPU amplification attack. There is no trusted proxy in front of
/// this app, so a client IP cannot be believed - the two instruments that remain are a
/// per-username bucket (stops credential stuffing against one account) and a global
/// argon2 budget (stops a flood that varies the username on every request, which the
/// per-username bucket alone does nothing about).
struct Limits {
    /// Per-username bucket.
    window: Duration,
    per_user_max: u32,
    /// Global ceiling on requests admitted to argon2. A rejected login costs one hash;
    /// a stored-user login costs two (UserStore::authenticate, then the DUMMY_HASH
    /// fallback), so the real CPU ceiling is ~2x `global_max` hashes per window.
    global_window: Duration,
    global_max: u32,
    /// Hard ceiling on tracked usernames, so the map cannot grow with attacker input.
    max_keys: usize,
}

/// 5 tries/min per account, 10 hashes/10s overall, 256 tracked names (~20 KB).
/// A single-admin panel does a handful of logins a day and never approaches any of these;
/// the global window is deliberately short so a flood degrades login for at most 10s
/// at a time rather than for a whole minute.
const LOGIN_LIMITS: Limits = Limits {
    window: Duration::from_secs(60),
    per_user_max: 5,
    global_window: Duration::from_secs(10),
    global_max: 10,
    max_keys: 256,
};

#[derive(Default)]
struct LoginLimiter {
    /// username -> (window start, attempts in window)
    per_user: HashMap<String, (Instant, u32)>,
    /// (window start, attempts in window) across every username
    global: Option<(Instant, u32)>,
}

impl LoginLimiter {
    /// Records one login attempt and returns `true` if it may proceed to argon2.
    fn record(&mut self, key: &str, now: Instant, limits: &Limits) -> bool {
        // Global budget first, and it returns early: once it is spent nothing below runs,
        // so a flood cannot make anyone pay for the map bookkeeping either.
        let global = self.global.get_or_insert((now, 0));
        if now.duration_since(global.0) >= limits.global_window {
            *global = (now, 0);
        }
        global.1 = global.1.saturating_add(1);
        if global.1 > limits.global_max {
            return false;
        }

        match self.per_user.get_mut(key) {
            Some(entry) => {
                // Tracked entries are only expired here now, so this branch is the one
                // thing keeping a stale counter from locking an account out forever.
                if now.duration_since(entry.0) >= limits.window {
                    *entry = (now, 0);
                }
                entry.1 = entry.1.saturating_add(1);
                entry.1 <= limits.per_user_max
            }
            None => {
                // A name we are not tracking only earns a slot if there is room: drop
                // expired entries first, then the oldest. Whether a name is tracked
                // depends purely on request volume, never on whether the account exists -
                // making it identity-dependent would turn this into a user-enumeration
                // oracle, which is exactly what DUMMY_HASH exists to prevent.
                if self.per_user.len() >= limits.max_keys {
                    self.per_user
                        .retain(|_, (started, _)| now.duration_since(*started) < limits.window);
                }
                if self.per_user.len() >= limits.max_keys {
                    let oldest = self
                        .per_user
                        .iter()
                        .min_by_key(|(_, (started, _))| *started)
                        .map(|(name, _)| name.clone());
                    if let Some(oldest) = oldest {
                        self.per_user.remove(&oldest);
                    }
                }
                self.per_user.insert(key.to_string(), (now, 1));
                true
            }
        }
    }
}

// ponytail: one global Mutex and an O(max_keys) prune, but only on the insert path and
// only while global budget remains; shard it if that ever shows up in a profile.
static LOGIN_LIMITER: LazyLock<Mutex<LoginLimiter>> =
    LazyLock::new(|| Mutex::new(LoginLimiter::default()));

/// Takes the limiter lock, recovering it if a previous holder panicked. The state behind
/// it is a pair of counters with no invariant a panic could break, so recovering keeps
/// throttling alive instead of silently disabling it for the life of the process.
fn lock_limiter() -> std::sync::MutexGuard<'static, LoginLimiter> {
    LOGIN_LIMITER.lock().unwrap_or_else(|poisoned| {
        tracing::error!(
            "Login rate limiter mutex was poisoned; recovering counters and continuing to throttle"
        );
        poisoned.into_inner()
    })
}

fn login_allowed(username: &str) -> bool {
    let key = username.trim().to_lowercase();
    lock_limiter().record(&key, Instant::now(), &LOGIN_LIMITS)
}

fn clear_login_attempts(username: &str) {
    lock_limiter()
        .per_user
        .remove(&username.trim().to_lowercase());
}

pub fn auth_router() -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .route("/me", get(me_handler))
        .route("/logout", post(logout_handler))
        .route("/tokens", get(list_tokens_handler).post(create_token_handler))
        .route("/tokens/:id", delete(revoke_token_handler))
        .route("/users", get(list_users_handler).post(create_user_handler))
        .route("/users/:username", delete(delete_user_handler))
}

/// Handler for POST /api/auth/login
pub async fn login_handler(
    Extension(config): Extension<Arc<AppConfig>>,
    Extension(user_store): Extension<Arc<UserStore>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Throttle before any argon2 verification happens. One message for both the
    // per-username and the global limit, and neither consults the user store, so a
    // throttled rejection says nothing about whether the account exists.
    if !login_allowed(&payload.username) {
        return Err(AppError::AuthError(
            "Too many login attempts. Try again shortly.".to_string(),
        ));
    }

    // Check if authenticating via UserStore
    if let Some(user) = user_store.authenticate(&payload.username, &payload.password).await {
        clear_login_attempts(&payload.username);
        let token = create_token(&user.username, config.jwt_secret.as_bytes())?;
        return Ok(Json(AuthResponse {
            token,
            user: UserProfile {
                id: user.id,
                username: user.username,
                role: user.role,
            },
        }));
    }

    // Fallback: Primary Admin Credential Check
    let username_matches = payload.username == config.admin_username;
    let hash = if username_matches {
        config.admin_password_hash.clone()
    } else {
        DUMMY_HASH.clone()
    };

    let password = payload.password.clone();
    let is_valid = tokio::task::spawn_blocking(move || verify_password(&password, &hash))
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))??;

    if !username_matches || !is_valid {
        return Err(AppError::InvalidCredentials);
    }

    clear_login_attempts(&payload.username);
    let token = create_token(&payload.username, config.jwt_secret.as_bytes())?;

    let response = AuthResponse {
        token,
        user: UserProfile {
            id: "1".to_string(),
            username: payload.username,
            role: "admin".to_string(),
        },
    };

    Ok(Json(response))
}

/// Handler for GET /api/auth/me
pub async fn me_handler(
    AuthUser(claims): AuthUser,
    Extension(config): Extension<Arc<AppConfig>>,
    Extension(user_store): Extension<Arc<UserStore>>,
) -> Result<Json<UserProfile>, AppError> {
    let key = claims.sub.trim().to_lowercase();
    let account = user_store
        .list_users()
        .await
        .into_iter()
        .find(|u| u.username.trim().to_lowercase() == key);

    let (id, role) = match account {
        Some(user) => (user.id, user.role),
        // Not in the store: only the ADMIN_USERNAME bootstrap account is an admin.
        None => (
            "1".to_string(),
            resolve_role(&user_store, &config, &claims.sub).await,
        ),
    };

    Ok(Json(UserProfile {
        id,
        username: claims.sub,
        role,
    }))
}

/// Handler for POST /api/auth/logout
pub async fn logout_handler(
    _auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({
        "message": "Logged out successfully"
    })))
}

/// Handler for GET /api/auth/tokens
pub async fn list_tokens_handler(
    _auth: AuthUser,
    Extension(token_store): Extension<Arc<TokenStore>>,
) -> Result<Json<Vec<ApiTokenInfo>>, AppError> {
    let tokens = token_store.list_tokens().await;
    Ok(Json(tokens))
}

/// Handler for POST /api/auth/tokens
pub async fn create_token_handler(
    _auth: RequireAdmin,
    Extension(token_store): Extension<Arc<TokenStore>>,
    Json(payload): Json<CreateTokenRequest>,
) -> Result<Json<CreateTokenResponse>, AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("Token name cannot be empty".to_string()));
    }

    let res = token_store
        .create_token(payload.name.trim(), payload.expires_in_days)
        .await?;

    Ok(Json(res))
}

/// Handler for DELETE /api/auth/tokens/:id
pub async fn revoke_token_handler(
    _auth: RequireAdmin,
    Extension(token_store): Extension<Arc<TokenStore>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let revoked = token_store.revoke_token(&id).await?;
    if !revoked {
        return Err(AppError::NotFound(format!("API Token ID '{}' not found", id)));
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("Token '{}' revoked successfully", id)
    })))
}

/// Handler for GET /api/auth/users
pub async fn list_users_handler(
    _auth: AuthUser,
    Extension(user_store): Extension<Arc<UserStore>>,
) -> Result<Json<Vec<UserAccountPublic>>, AppError> {
    let users = user_store.list_users().await;
    Ok(Json(users))
}

/// Handler for POST /api/auth/users
pub async fn create_user_handler(
    _auth: RequireAdmin,
    Extension(user_store): Extension<Arc<UserStore>>,
    Json(payload): Json<CreateUserAccountRequest>,
) -> Result<Json<UserAccountPublic>, AppError> {
    let user = user_store
        .create_user(&payload.username, &payload.password, &payload.role, payload.permissions)
        .await?;
    Ok(Json(user))
}

/// Handler for DELETE /api/auth/users/:username
pub async fn delete_user_handler(
    RequireAdmin(claims): RequireAdmin,
    Extension(user_store): Extension<Arc<UserStore>>,
    Path(username): Path<String>,
) -> Result<Json<Value>, AppError> {
    let deleted = user_store.delete_user(&username, &claims.sub).await?;
    if !deleted {
        return Err(AppError::NotFound(format!("User '{}' not found", username)));
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("User account '{}' deleted successfully", username)
    })))
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Small numbers so the intent of each test is readable; `global_max` is set out of
    /// the way unless the test is specifically about it.
    fn test_limits() -> Limits {
        Limits {
            window: Duration::from_secs(60),
            per_user_max: 3,
            global_window: Duration::from_secs(10),
            global_max: 10_000,
            max_keys: 4,
        }
    }

    #[test]
    fn login_limiter_blocks_one_username_after_max_and_recovers_after_window() {
        let limits = test_limits();
        let start = Instant::now();
        let mut limiter = LoginLimiter::default();

        for i in 1..=limits.per_user_max {
            assert!(
                limiter.record("bob", start, &limits),
                "attempt {} should be allowed",
                i
            );
        }
        assert!(!limiter.record("bob", start, &limits));

        // Same window, another username is unaffected.
        assert!(limiter.record("alice", start, &limits));

        // Once bob's window has elapsed his counter resets.
        let later = start + limits.window + Duration::from_secs(1);
        assert!(limiter.record("bob", later, &limits));
    }

    #[test]
    fn login_limiter_global_cap_stops_a_rotating_username_flood() {
        let limits = Limits {
            global_max: 5,
            ..test_limits()
        };
        let start = Instant::now();
        let mut limiter = LoginLimiter::default();

        // Every request uses a fresh username, so the per-username bucket never trips -
        // only the global argon2 budget can stop this.
        for i in 0..limits.global_max {
            assert!(
                limiter.record(&format!("user{}", i), start, &limits),
                "request {} is still within the global budget",
                i
            );
        }
        assert!(!limiter.record("user-overflow", start, &limits));

        // The budget refills once the global window rolls over, so a real admin is locked
        // out for at most one global window, not indefinitely.
        let later = start + limits.global_window + Duration::from_millis(1);
        assert!(limiter.record("admin", later, &limits));
    }

    #[test]
    fn login_limiter_tracked_usernames_stay_bounded() {
        let limits = test_limits();
        let start = Instant::now();
        let mut limiter = LoginLimiter::default();

        for i in 0..200 {
            limiter.record(&format!("user{}", i), start, &limits);
        }

        assert!(
            limiter.per_user.len() <= limits.max_keys,
            "map grew to {} entries, cap is {}",
            limiter.per_user.len(),
            limits.max_keys
        );
    }

    #[test]
    fn login_limiter_rejects_before_touching_the_username_map() {
        // Once the global budget is spent the request never reaches the map, so a flood
        // cannot make the map churn either.
        let limits = Limits {
            global_max: 1,
            ..test_limits()
        };
        let start = Instant::now();
        let mut limiter = LoginLimiter::default();

        assert!(limiter.record("first", start, &limits));
        assert!(!limiter.record("second", start, &limits));
        assert_eq!(limiter.per_user.len(), 1);
    }
}
