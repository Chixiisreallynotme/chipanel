use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{error::AppError, models::auth::UserClaims};

pub fn create_token(username: &str, secret: &[u8]) -> Result<String, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::InternalError(format!("System time error: {}", e)))?
        .as_secs();

    // 24 hours duration in seconds (24 * 60 * 60)
    let exp = (now + 86_400) as usize;
    let iat = now as usize;

    let claims = UserClaims {
        sub: username.to_string(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| AppError::InternalError(format!("Token creation failed: {}", e)))?;

    Ok(token)
}

pub fn verify_token(token: &str, secret: &[u8]) -> Result<UserClaims, AppError> {
    let validation = Validation::default();
    let token_data = decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(secret),
        &validation,
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
        _ => AppError::AuthError(format!("Invalid token: {}", e)),
    })?;

    Ok(token_data.claims)
}
