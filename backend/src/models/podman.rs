use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStatusResponse {
    pub status: ServerStatus,
    pub uptime_seconds: u64,
    pub container_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerMetrics {
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub memory_limit_bytes: u64,
    pub memory_percent: f32,
}
