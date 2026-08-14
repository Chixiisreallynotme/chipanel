use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserProfile,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserClaims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiTokenInfo {
    pub id: String,
    pub name: String,
    pub token_prefix: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateTokenRequest {
    pub name: String,
    pub expires_in_days: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateTokenResponse {
    pub token_info: ApiTokenInfo,
    pub raw_token: String,
}
