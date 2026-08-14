use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RconCommandRequest {
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RconCommandResponse {
    pub output: String,
    pub success: bool,
}
