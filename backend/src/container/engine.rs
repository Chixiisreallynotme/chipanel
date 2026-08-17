use std::future::Future;
use std::pin::Pin;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContainerMetrics {
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub memory_limit_bytes: u64,
    pub memory_percent: f32,
}

/// Core asynchronous container runtime abstraction trait.
///
/// Implementations must be thread-safe (`Send + Sync`) and object-safe (`dyn ContainerEngine`).
pub trait ContainerEngine: Send + Sync {
    /// Identifier for the container engine (e.g. "podman", "docker")
    fn engine_name(&self) -> &str;

    /// Inspects a container and returns its operational state and uptime.
    fn get_status<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<ContainerStatusResponse, AppError>>;

    /// Queries CPU and Memory metrics for a specified container.
    fn get_stats<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<ContainerMetrics, AppError>>;

    /// Starts a container.
    fn start_container<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<(), AppError>>;

    /// Stops a running container gracefully.
    fn stop_container<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<(), AppError>>;

    /// Restarts a container.
    fn restart_container<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<(), AppError>>;

    /// Fetches the last `tail` log lines from the container's stdout/stderr.
    fn get_logs<'a>(
        &'a self,
        id: &'a str,
        tail: usize,
    ) -> BoxFuture<'a, Result<Vec<String>, AppError>>;
}
