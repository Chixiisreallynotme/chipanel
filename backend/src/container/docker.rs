use serde::Deserialize;
use std::env;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::container::client_helper::send_unix_socket_request;
use crate::container::engine::{
    BoxFuture, ContainerEngine, ContainerMetrics, ContainerStatusResponse, ServerStatus,
};
use crate::container::podman::{calculate_uptime_from_rfc3339, get_current_uid};
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct DockerEngine {
    socket_path: PathBuf,
}

impl Default for DockerEngine {
    fn default() -> Self {
        Self::new(find_docker_socket_path())
    }
}

impl DockerEngine {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    async fn send_request(
        &self,
        method: &str,
        path: &str,
        body_bytes: Option<Vec<u8>>,
    ) -> Result<(u16, Vec<u8>), AppError> {
        send_unix_socket_request(&self.socket_path, method, path, body_bytes, "Docker").await
    }

    /// GET /containers/{name}/json
    pub async fn inspect_container(&self, name: &str) -> Result<ContainerStatusResponse, AppError> {
        let path = format!("/containers/{}/json", name);
        let (status, bytes) = self.send_request("GET", &path, None).await?;

        if status == 404 {
            return Ok(ContainerStatusResponse {
                status: ServerStatus::Stopped,
                uptime_seconds: 0,
                container_id: String::new(),
            });
        }

        if status != 200 {
            let err_msg = String::from_utf8_lossy(&bytes);
            return Err(AppError::InternalError(format!(
                "Docker inspect failed with status {}: {}",
                status, err_msg
            )));
        }

        let inspect_res: DockerInspectRaw = serde_json::from_slice(&bytes).map_err(|e| {
            AppError::InternalError(format!("Failed to parse Docker inspect response: {}", e))
        })?;

        let container_id = inspect_res.id.unwrap_or_else(|| name.to_string());
        let raw_status = inspect_res
            .state
            .as_ref()
            .and_then(|s| s.status.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let server_status = match raw_status.to_lowercase().as_str() {
            "running" => ServerStatus::Running,
            "stopped" | "exited" | "dead" => ServerStatus::Stopped,
            "created" | "restarting" => ServerStatus::Starting,
            "paused" | "removing" => ServerStatus::Stopping,
            _ => ServerStatus::Unknown,
        };

        let uptime_seconds = if server_status == ServerStatus::Running {
            if let Some(state) = &inspect_res.state {
                if let Some(started_at) = &state.started_at {
                    calculate_uptime_from_rfc3339(started_at)
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };

        Ok(ContainerStatusResponse {
            status: server_status,
            uptime_seconds,
            container_id,
        })
    }

    /// GET /containers/{name}/stats?stream=false
    pub async fn get_container_metrics(&self, name: &str) -> Result<ContainerMetrics, AppError> {
        let sample_a = self.fetch_stats_snapshot(name).await?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let sample_b = self.fetch_stats_snapshot(name).await?;

        let cpu_delta = sample_b.total_cpu_usage.saturating_sub(sample_a.total_cpu_usage);
        let system_delta = sample_b.system_cpu_usage.saturating_sub(sample_a.system_cpu_usage);
        let online_cpus = sample_b.online_cpus.max(1) as f32;

        let cpu_percent = if system_delta > 0 {
            ((cpu_delta as f64 / system_delta as f64) * online_cpus as f64 * 100.0) as f32
        } else {
            0.0
        }
        .clamp(0.0, online_cpus * 100.0);

        let memory_bytes = sample_b.memory_bytes;
        let memory_limit_bytes = sample_b.memory_limit_bytes;
        let memory_percent = if memory_limit_bytes > 0 {
            (memory_bytes as f32 / memory_limit_bytes as f32) * 100.0
        } else {
            0.0
        };

        Ok(ContainerMetrics {
            cpu_percent,
            memory_bytes,
            memory_limit_bytes,
            memory_percent,
        })
    }

    async fn fetch_stats_snapshot(&self, name: &str) -> Result<DockerStatsSnapshot, AppError> {
        let path = format!("/containers/{}/stats?stream=false", name);
        let (status, bytes) = self.send_request("GET", &path, None).await?;

        if status == 404 {
            return Err(AppError::NotFound(format!("Container '{}' not found", name)));
        }

        if status != 200 {
            let err_msg = String::from_utf8_lossy(&bytes);
            return Err(AppError::InternalError(format!(
                "Docker stats failed with status {}: {}",
                status, err_msg
            )));
        }

        let parsed: DockerStatsShape = serde_json::from_slice(&bytes).map_err(|e| {
            AppError::InternalError(format!("Failed to parse Docker stats response: {}", e))
        })?;

        let cache = parsed.memory_stats.stats.and_then(|s| s.cache).unwrap_or(0);
        let mem_usage = parsed.memory_stats.usage.saturating_sub(cache);

        Ok(DockerStatsSnapshot {
            total_cpu_usage: parsed.cpu_stats.cpu_usage.total_usage,
            system_cpu_usage: parsed.cpu_stats.system_cpu_usage,
            online_cpus: parsed.cpu_stats.online_cpus.max(1),
            memory_bytes: if mem_usage > 0 { mem_usage } else { parsed.memory_stats.usage },
            memory_limit_bytes: parsed.memory_stats.limit,
        })
    }

    /// POST /containers/{name}/start
    pub async fn start_container(&self, name: &str) -> Result<(), AppError> {
        let path = format!("/containers/{}/start", name);
        let (status, bytes) = self.send_request("POST", &path, None).await?;

        if status == 404 {
            return Err(AppError::NotFound(format!("Container '{}' not found", name)));
        }

        if status == 200 || status == 204 || status == 304 {
            info!("Successfully started container '{}' via Docker API", name);
            Ok(())
        } else {
            let err_msg = String::from_utf8_lossy(&bytes);
            Err(AppError::InternalError(format!(
                "Failed to start container '{}' (status {}): {}",
                name, status, err_msg
            )))
        }
    }

    /// POST /containers/{name}/stop?t=30
    pub async fn stop_container(&self, name: &str) -> Result<(), AppError> {
        let path = format!("/containers/{}/stop?t=30", name);
        let (status, bytes) = self.send_request("POST", &path, None).await?;

        if status == 404 {
            return Err(AppError::NotFound(format!("Container '{}' not found", name)));
        }

        if status == 200 || status == 204 || status == 304 {
            info!("Successfully stopped container '{}' via Docker API", name);
            Ok(())
        } else {
            let err_msg = String::from_utf8_lossy(&bytes);
            Err(AppError::InternalError(format!(
                "Failed to stop container '{}' (status {}): {}",
                name, status, err_msg
            )))
        }
    }

    /// POST /containers/{name}/restart?t=30
    pub async fn restart_container(&self, name: &str) -> Result<(), AppError> {
        let path = format!("/containers/{}/restart?t=30", name);
        let (status, bytes) = self.send_request("POST", &path, None).await?;

        if status == 404 {
            return Err(AppError::NotFound(format!("Container '{}' not found", name)));
        }

        if status == 200 || status == 204 {
            info!("Successfully restarted container '{}' via Docker API", name);
            Ok(())
        } else {
            let err_msg = String::from_utf8_lossy(&bytes);
            Err(AppError::InternalError(format!(
                "Failed to restart container '{}' (status {}): {}",
                name, status, err_msg
            )))
        }
    }

    /// GET /containers/{name}/logs?stdout=true&stderr=true&tail={tail}&timestamps=false
    pub async fn get_logs(&self, name: &str, tail: usize) -> Result<Vec<String>, AppError> {
        let path = format!(
            "/containers/{}/logs?stdout=true&stderr=true&tail={}&timestamps=false",
            name, tail
        );
        let (status, bytes) = self.send_request("GET", &path, None).await?;

        if status == 404 {
            return Err(AppError::NotFound(format!("Container '{}' not found", name)));
        }

        if status != 200 {
            let err_msg = String::from_utf8_lossy(&bytes);
            return Err(AppError::InternalError(format!(
                "Failed to read logs for container '{}' (status {}): {}",
                name, status, err_msg
            )));
        }

        Ok(demux_docker_logs(&bytes))
    }
}

impl ContainerEngine for DockerEngine {
    fn engine_name(&self) -> &str {
        "docker"
    }

    fn get_status<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<ContainerStatusResponse, AppError>> {
        Box::pin(self.inspect_container(id))
    }

    fn get_stats<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<ContainerMetrics, AppError>> {
        Box::pin(self.get_container_metrics(id))
    }

    fn start_container<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(self.start_container(id))
    }

    fn stop_container<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(self.stop_container(id))
    }

    fn restart_container<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(self.restart_container(id))
    }

    fn get_logs<'a>(
        &'a self,
        id: &'a str,
        tail: usize,
    ) -> BoxFuture<'a, Result<Vec<String>, AppError>> {
        Box::pin(self.get_logs(id, tail))
    }
}

pub fn find_docker_socket_path() -> PathBuf {
    use std::os::unix::fs::FileTypeExt;

    let is_valid_socket = |path: &Path| -> bool {
        std::fs::metadata(path)
            .map(|meta| meta.file_type().is_socket())
            .unwrap_or(false)
    };

    if let Ok(env_path) = env::var("DOCKER_HOST") {
        let clean_path = env_path.strip_prefix("unix://").unwrap_or(&env_path);
        let p = PathBuf::from(clean_path);
        if is_valid_socket(&p) {
            return p;
        }
    }

    if let Ok(env_path) = env::var("DOCKER_SOCKET") {
        let p = PathBuf::from(&env_path);
        if is_valid_socket(&p) {
            return p;
        }
    }

    let uid = get_current_uid();
    let user_dockers = [
        format!("/run/user/{}/docker.sock", uid),
        format!("/var/run/user/{}/docker.sock", uid),
    ];
    for candidate in &user_dockers {
        let p = PathBuf::from(candidate);
        if is_valid_socket(&p) {
            return p;
        }
    }

    let candidates = [
        "/var/run/docker.sock",
        "/run/docker.sock",
    ];

    for candidate in candidates {
        let p = PathBuf::from(candidate);
        if is_valid_socket(&p) {
            return p;
        }
    }

    PathBuf::from("/var/run/docker.sock")
}

/// Demultiplexes standard Docker multiplex stream frames or falls back to plain text lines.
pub fn demux_docker_logs(bytes: &[u8]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cursor = 0;

    let is_multiplexed = bytes.len() >= 8
        && (bytes[0] == 1 || bytes[0] == 2)
        && bytes[1] == 0
        && bytes[2] == 0
        && bytes[3] == 0;

    if is_multiplexed {
        while cursor + 8 <= bytes.len() {
            let size = u32::from_be_bytes([
                bytes[cursor + 4],
                bytes[cursor + 5],
                bytes[cursor + 6],
                bytes[cursor + 7],
            ]) as usize;
            cursor += 8;

            if cursor + size > bytes.len() {
                let chunk = String::from_utf8_lossy(&bytes[cursor..]);
                for line in chunk.lines() {
                    if !line.is_empty() {
                        lines.push(line.to_string());
                    }
                }
                break;
            }

            let chunk = String::from_utf8_lossy(&bytes[cursor..cursor + size]);
            for line in chunk.lines() {
                if !line.is_empty() {
                    lines.push(line.to_string());
                }
            }
            cursor += size;
        }
    } else {
        let text = String::from_utf8_lossy(bytes);
        for line in text.lines() {
            if !line.is_empty() {
                lines.push(line.to_string());
            }
        }
    }

    lines
}

#[derive(Debug, Deserialize)]
struct DockerInspectRaw {
    #[serde(default, alias = "Id", alias = "id")]
    pub id: Option<String>,
    #[serde(default, alias = "State", alias = "state")]
    pub state: Option<DockerInspectStateRaw>,
}

#[derive(Debug, Deserialize)]
struct DockerInspectStateRaw {
    #[serde(default, alias = "Status", alias = "status")]
    pub status: Option<String>,
    #[serde(default, alias = "StartedAt", alias = "startedAt")]
    pub started_at: Option<String>,
}

#[derive(Debug, Default)]
struct DockerStatsSnapshot {
    total_cpu_usage: u64,
    system_cpu_usage: u64,
    online_cpus: u32,
    memory_bytes: u64,
    memory_limit_bytes: u64,
}

#[derive(Debug, Deserialize, Default)]
struct DockerCpuUsage {
    #[serde(default)]
    total_usage: u64,
}

#[derive(Debug, Deserialize, Default)]
struct DockerCpuStats {
    #[serde(default)]
    cpu_usage: DockerCpuUsage,
    #[serde(default)]
    system_cpu_usage: u64,
    #[serde(default)]
    online_cpus: u32,
}

#[derive(Debug, Deserialize, Default)]
struct DockerMemoryStatsDetails {
    #[serde(default)]
    cache: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
struct DockerMemoryStats {
    #[serde(default)]
    usage: u64,
    #[serde(default)]
    limit: u64,
    #[serde(default)]
    stats: Option<DockerMemoryStatsDetails>,
}

#[derive(Debug, Deserialize, Default)]
struct DockerStatsShape {
    #[serde(default)]
    cpu_stats: DockerCpuStats,
    #[serde(default)]
    memory_stats: DockerMemoryStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demuxes_multiplexed_docker_logs() {
        let mut sample = Vec::new();
        // Frame 1: stdout, 11 bytes: "Hello World"
        sample.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 11]);
        sample.extend_from_slice(b"Hello World");
        // Frame 2: stderr, 15 bytes: "Error occurred\n"
        sample.extend_from_slice(&[2, 0, 0, 0, 0, 0, 0, 15]);
        sample.extend_from_slice(b"Error occurred\n");

        let lines = demux_docker_logs(&sample);
        assert_eq!(lines, vec!["Hello World", "Error occurred"]);
    }

    #[test]
    fn demuxes_plain_text_logs() {
        let sample = b"Line 1\nLine 2\nLine 3\n";
        let lines = demux_docker_logs(sample);
        assert_eq!(lines, vec!["Line 1", "Line 2", "Line 3"]);
    }
}
