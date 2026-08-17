use serde::Deserialize;
use std::env;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::container::client_helper::send_unix_socket_request;
use crate::container::engine::{
    BoxFuture, ContainerEngine, ContainerMetrics, ContainerStatusResponse, ServerStatus,
};
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct PodmanEngine {
    socket_path: PathBuf,
}

impl Default for PodmanEngine {
    fn default() -> Self {
        Self::new(find_podman_socket_path())
    }
}

impl PodmanEngine {
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
    ) -> Result<(u16, String), AppError> {
        let (status, bytes) =
            send_unix_socket_request(&self.socket_path, method, path, body_bytes, "Podman").await?;
        let body_str = String::from_utf8_lossy(&bytes).into_owned();
        Ok((status, body_str))
    }

    /// GET /v4.0.0/libpod/containers/{name}/json
    pub async fn inspect_container(&self, name: &str) -> Result<ContainerStatusResponse, AppError> {
        let path = format!("/v4.0.0/libpod/containers/{}/json", name);
        let (status, body) = self.send_request("GET", &path, None).await?;

        if status == 404 {
            return Ok(ContainerStatusResponse {
                status: ServerStatus::Stopped,
                uptime_seconds: 0,
                container_id: String::new(),
            });
        }

        if status != 200 {
            return Err(AppError::InternalError(format!(
                "Podman inspect failed with status {}: {}",
                status, body
            )));
        }

        let inspect_res: PodmanInspectRaw = serde_json::from_str(&body).map_err(|e| {
            AppError::InternalError(format!("Failed to parse Podman inspect response: {}", e))
        })?;

        let container_id = inspect_res.id.unwrap_or_else(|| name.to_string());
        let raw_status = inspect_res
            .state
            .as_ref()
            .and_then(|s| s.status.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let server_status = match raw_status.to_lowercase().as_str() {
            "running" => ServerStatus::Running,
            "stopped" | "exited" => ServerStatus::Stopped,
            "starting" | "created" | "configured" => ServerStatus::Starting,
            "stopping" | "pausing" => ServerStatus::Stopping,
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

    /// GET /v4.0.0/libpod/containers/{name}/stats?stream=false
    pub async fn get_container_metrics(&self, name: &str) -> Result<ContainerMetrics, AppError> {
        let sample_a = self.fetch_stats_snapshot(name).await?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let sample_b = self.fetch_stats_snapshot(name).await?;

        let cpu_percent = if let (Some(a), Some(b)) = (&sample_a.docker, &sample_b.docker) {
            let cpu_delta = b.total_usage.saturating_sub(a.total_usage);
            let system_delta = b.system_cpu_usage.saturating_sub(a.system_cpu_usage);
            let online_cpus = b.online_cpus.max(1) as f32;
            if system_delta > 0 {
                ((cpu_delta as f64 / system_delta as f64) * online_cpus as f64 * 100.0) as f32
            } else {
                0.0
            }
            .clamp(0.0, online_cpus * 100.0)
        } else {
            sample_b.legacy_cpu_percent.unwrap_or(0.0)
        };

        let memory_bytes = sample_b.memory_bytes;
        let memory_limit_bytes = sample_b.memory_limit_bytes;
        let memory_percent = sample_b.legacy_memory_percent.unwrap_or_else(|| {
            if memory_limit_bytes > 0 {
                (memory_bytes as f32 / memory_limit_bytes as f32) * 100.0
            } else {
                0.0
            }
        });

        Ok(ContainerMetrics {
            cpu_percent,
            memory_bytes,
            memory_limit_bytes,
            memory_percent,
        })
    }

    async fn fetch_stats_snapshot(&self, name: &str) -> Result<StatsSnapshot, AppError> {
        let path = format!("/v4.0.0/libpod/containers/{}/stats?stream=false", name);
        let (status, body) = self.send_request("GET", &path, None).await?;

        if status == 404 {
            return Err(AppError::NotFound(format!("Container '{}' not found", name)));
        }

        if status != 200 {
            return Err(AppError::InternalError(format!(
                "Podman stats failed with status {}: {}",
                status, body
            )));
        }

        Ok(parse_stats_snapshot(&body))
    }

    /// Starts a container via Systemd Quadlet DBus if available, or direct Podman REST API.
    pub async fn start_container(&self, name: &str) -> Result<(), AppError> {
        if let Ok(()) = try_systemd_action(name, "start").await {
            info!("Successfully started container/service '{}' via systemd DBus", name);
            return Ok(());
        }

        let path = format!("/v4.0.0/libpod/containers/{}/start", name);
        let (status, body) = self.send_request("POST", &path, None).await?;

        if status == 404 {
            return Err(AppError::NotFound(format!("Container '{}' not found", name)));
        }

        if status == 200 || status == 204 || status == 304 {
            info!("Successfully started container '{}'", name);
            Ok(())
        } else {
            Err(AppError::InternalError(format!(
                "Failed to start container '{}' (status {}): {}",
                name, status, body
            )))
        }
    }

    /// Stops a container via Systemd Quadlet DBus if available, or direct Podman REST API.
    pub async fn stop_container(&self, name: &str) -> Result<(), AppError> {
        if let Ok(()) = try_systemd_action(name, "stop").await {
            info!("Successfully stopped container/service '{}' via systemd DBus", name);
            return Ok(());
        }

        let path = format!("/v4.0.0/libpod/containers/{}/stop?t=30", name);
        let (status, body) = self.send_request("POST", &path, None).await?;

        if status == 404 {
            return Err(AppError::NotFound(format!("Container '{}' not found", name)));
        }

        if status == 200 || status == 204 || status == 304 {
            info!("Successfully stopped container '{}'", name);
            Ok(())
        } else {
            Err(AppError::InternalError(format!(
                "Failed to stop container '{}' (status {}): {}",
                name, status, body
            )))
        }
    }

    /// Restarts a container via Systemd Quadlet DBus if available, or direct Podman REST API.
    pub async fn restart_container(&self, name: &str) -> Result<(), AppError> {
        if let Ok(()) = try_systemd_action(name, "restart").await {
            info!("Successfully restarted container/service '{}' via systemd DBus", name);
            return Ok(());
        }

        let path = format!("/v4.0.0/libpod/containers/{}/restart?t=30", name);
        let (status, body) = self.send_request("POST", &path, None).await?;

        if status == 404 {
            return Err(AppError::NotFound(format!("Container '{}' not found", name)));
        }

        if status == 200 || status == 204 {
            info!("Successfully restarted container '{}'", name);
            Ok(())
        } else {
            Err(AppError::InternalError(format!(
                "Failed to restart container '{}' (status {}): {}",
                name, status, body
            )))
        }
    }

    /// GET /v4.0.0/libpod/containers/{name}/logs?stdout=true&stderr=true&tail={tail}
    pub async fn get_logs(&self, name: &str, tail: usize) -> Result<Vec<String>, AppError> {
        let path = format!(
            "/v4.0.0/libpod/containers/{}/logs?stdout=true&stderr=true&tail={}",
            name, tail
        );
        let (status, body) = self.send_request("GET", &path, None).await?;

        if status == 404 {
            return Err(AppError::NotFound(format!("Container '{}' not found", name)));
        }

        if status != 200 {
            return Err(AppError::InternalError(format!(
                "Failed to read logs for container '{}' (status {}): {}",
                name, status, body
            )));
        }

        let lines = body
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        Ok(lines)
    }
}

impl ContainerEngine for PodmanEngine {
    fn engine_name(&self) -> &str {
        "podman"
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

pub async fn try_systemd_action(unit_name: &str, action: &str) -> Result<(), AppError> {
    let method_name = match action {
        "start" => "StartUnit",
        "stop" => "StopUnit",
        "restart" => "RestartUnit",
        _ => return Err(AppError::BadRequest(format!("Invalid action {}", action))),
    };

    let service_name = if unit_name.ends_with(".service") {
        unit_name.to_string()
    } else if unit_name == "minecraft-server" || unit_name == "minecraft" {
        "minecraft.service".to_string()
    } else {
        format!("{}.service", unit_name)
    };

    let bus_address = if let Ok(addr) = env::var("DBUS_SESSION_BUS_ADDRESS") {
        addr
    } else {
        let uid = get_current_uid();
        format!("unix:path=/run/user/{}/bus", uid)
    };

    let connection = zbus::connection::Builder::address(bus_address.as_str())
        .map_err(|e| AppError::InternalError(format!("Invalid bus address: {}", e)))?
        .build()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to connect to user bus: {}", e)))?;

    let args = (service_name.clone(), "replace".to_string());
    connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            method_name,
            &args,
        )
        .await
        .map_err(|e| {
            AppError::InternalError(format!(
                "Systemd DBus {} for '{}' failed: {}",
                method_name, service_name, e
            ))
        })?;

    info!("Systemd DBus {} for '{}' completed", method_name, service_name);
    Ok(())
}

pub fn find_podman_socket_path() -> PathBuf {
    use std::os::unix::fs::FileTypeExt;

    let is_valid_socket = |path: &Path| -> bool {
        std::fs::metadata(path)
            .map(|meta| meta.file_type().is_socket())
            .unwrap_or(false)
    };

    if let Ok(env_path) = env::var("PODMAN_SOCKET") {
        let p = PathBuf::from(&env_path);
        if is_valid_socket(&p) {
            return p;
        }
    }

    let uid = get_current_uid();
    let user_sock = PathBuf::from(format!("/run/user/{}/podman/podman.sock", uid));
    if is_valid_socket(&user_sock) {
        return user_sock;
    }

    let candidates = [
        "/run/user/1000/podman/podman.sock",
        "/run/podman/podman.sock",
        "/var/run/podman/podman.sock",
    ];

    for candidate in candidates {
        let p = PathBuf::from(candidate);
        if is_valid_socket(&p) {
            return p;
        }
    }

    user_sock
}

pub fn get_current_uid() -> u32 {
    if let Ok(uid_str) = env::var("UID") {
        if let Ok(uid) = uid_str.parse::<u32>() {
            return uid;
        }
    }

    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("Uid:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    if let Ok(uid) = parts[1].parse::<u32>() {
                        return uid;
                    }
                }
            }
        }
    }

    1000
}

#[derive(Debug, Deserialize)]
struct PodmanInspectRaw {
    #[serde(default, alias = "Id", alias = "id")]
    pub id: Option<String>,
    #[serde(default, alias = "State", alias = "state")]
    pub state: Option<PodmanInspectStateRaw>,
}

#[derive(Debug, Deserialize)]
struct PodmanInspectStateRaw {
    #[serde(default, alias = "Status", alias = "status")]
    pub status: Option<String>,
    #[serde(default, alias = "StartedAt", alias = "startedAt")]
    pub started_at: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct PodmanStatsWrapper {
    #[serde(default, alias = "Stats", alias = "stats")]
    pub stats: Option<Vec<PodmanStatsItem>>,
}

#[derive(Debug, Deserialize, Default)]
struct PodmanStatsItem {
    #[serde(default, alias = "cpu", alias = "cpu_percent", alias = "CPUPerc", alias = "CPU")]
    pub cpu: Option<f32>,
    #[serde(default, alias = "memUsage", alias = "memory_bytes", alias = "MemUsage")]
    pub memory_bytes: Option<u64>,
    #[serde(default, alias = "memLimit", alias = "memory_limit_bytes", alias = "MemLimit")]
    pub memory_limit_bytes: Option<u64>,
    #[serde(default, alias = "memPerc", alias = "memory_percent", alias = "MemPerc")]
    pub memory_percent: Option<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PodmanStatsPayload {
    Wrapper(PodmanStatsWrapper),
    List(Vec<PodmanStatsItem>),
    Single(PodmanStatsItem),
}

#[derive(Debug, Default)]
struct StatsSnapshot {
    memory_bytes: u64,
    memory_limit_bytes: u64,
    docker: Option<DockerCpuCounters>,
    legacy_cpu_percent: Option<f32>,
    legacy_memory_percent: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
struct DockerCpuCounters {
    total_usage: u64,
    system_cpu_usage: u64,
    online_cpus: u32,
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
struct DockerMemoryStats {
    #[serde(default)]
    usage: u64,
    #[serde(default)]
    limit: u64,
}

#[derive(Debug, Deserialize, Default)]
struct DockerStatsShape {
    #[serde(default)]
    cpu_stats: DockerCpuStats,
    #[serde(default)]
    memory_stats: DockerMemoryStats,
}

fn parse_stats_snapshot(body: &str) -> StatsSnapshot {
    if let Ok(docker) = serde_json::from_str::<DockerStatsShape>(body) {
        if docker.memory_stats.usage > 0 || docker.cpu_stats.system_cpu_usage > 0 {
            return StatsSnapshot {
                memory_bytes: docker.memory_stats.usage,
                memory_limit_bytes: docker.memory_stats.limit,
                docker: Some(DockerCpuCounters {
                    total_usage: docker.cpu_stats.cpu_usage.total_usage,
                    system_cpu_usage: docker.cpu_stats.system_cpu_usage,
                    online_cpus: docker.cpu_stats.online_cpus,
                }),
                legacy_cpu_percent: None,
                legacy_memory_percent: None,
            };
        }
    }

    let item = parse_podman_stats(body).unwrap_or_default();
    StatsSnapshot {
        memory_bytes: item.memory_bytes.unwrap_or(0),
        memory_limit_bytes: item.memory_limit_bytes.unwrap_or(0),
        docker: None,
        legacy_cpu_percent: item.cpu,
        legacy_memory_percent: item.memory_percent,
    }
}

fn parse_podman_stats(body: &str) -> Result<PodmanStatsItem, AppError> {
    let payload: PodmanStatsPayload = serde_json::from_str(body).map_err(|e| {
        AppError::InternalError(format!("Failed to parse Podman stats response: {}", e))
    })?;

    let item = match payload {
        PodmanStatsPayload::Wrapper(wrapper) => {
            if let Some(mut list) = wrapper.stats {
                if !list.is_empty() {
                    list.remove(0)
                } else {
                    PodmanStatsItem::default()
                }
            } else {
                PodmanStatsItem::default()
            }
        }
        PodmanStatsPayload::List(mut list) => {
            if !list.is_empty() {
                list.remove(0)
            } else {
                PodmanStatsItem::default()
            }
        }
        PodmanStatsPayload::Single(item) => item,
    };

    Ok(item)
}

pub fn split_utc_offset(time_str: &str) -> Option<(&str, i64)> {
    if let Some(body) = time_str.strip_suffix('Z').or_else(|| time_str.strip_suffix('z')) {
        return Some((body, 0));
    }

    let Some(idx) = time_str.rfind(['+', '-']) else {
        return Some((time_str, 0));
    };

    let (body, offset) = time_str.split_at(idx);
    let sign: i64 = if offset.starts_with('-') { -1 } else { 1 };
    let digits = &offset[1..];

    let (hours_str, minutes_str) = match digits.split_once(':') {
        Some((h, m)) => (h, m),
        None => match digits.len() {
            4 => (&digits[..2], &digits[2..]),
            2 => (digits, "0"),
            _ => return None,
        },
    };

    let hours: i64 = hours_str.parse().ok()?;
    let minutes: i64 = minutes_str.parse().ok()?;

    Some((body, sign * (hours * 3600 + minutes * 60)))
}

pub fn parse_rfc3339_to_epoch(started_at: &str) -> Option<u64> {
    let (date_str, time_str) = started_at.split_once('T')?;

    let date_parts: Vec<&str> = date_str.split('-').collect();
    if date_parts.len() < 3 {
        return None;
    }

    let (time_body, offset_secs) = split_utc_offset(time_str)?;
    let time_parts: Vec<&str> = time_body.split(':').collect();
    if time_parts.len() < 3 {
        return None;
    }

    let year: i32 = date_parts[0].parse().ok()?;
    let month: u32 = date_parts[1].parse().ok()?;
    let day: u32 = date_parts[2].parse().ok()?;

    let hour: u32 = time_parts[0].parse().ok()?;
    let minute: u32 = time_parts[1].parse().ok()?;
    let second: u32 = time_parts[2].split('.').next()?.parse().ok()?;

    if year < 1970 || month == 0 || month > 12 || day == 0 || day > 31 {
        return None;
    }

    let days_before_month = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let mut days = (year - 1970) as u64 * 365 + ((year - 1969) / 4) as u64;
    days += days_before_month[(month - 1) as usize];
    if month > 2 && (year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)) {
        days += 1;
    }
    days += (day - 1) as u64;

    let local_secs =
        days * 86400 + (hour as u64) * 3600 + (minute as u64) * 60 + (second as u64);

    let epoch = local_secs as i64 - offset_secs;
    if epoch < 0 {
        return None;
    }

    Some(epoch as u64)
}

pub fn calculate_uptime_from_rfc3339(started_at: &str) -> u64 {
    if started_at.is_empty() || started_at.starts_with("0001-01-01") {
        return 0;
    }

    let Some(timestamp_secs) = parse_rfc3339_to_epoch(started_at) else {
        return 0;
    };

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    now_secs.saturating_sub(timestamp_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_non_utc_offset() {
        assert_eq!(
            parse_rfc3339_to_epoch("2026-08-08T21:30:00+02:00"),
            Some(1_786_217_400)
        );
        assert_eq!(
            parse_rfc3339_to_epoch("2026-08-08T19:30:00Z"),
            Some(1_786_217_400)
        );
        assert_eq!(
            parse_rfc3339_to_epoch("2026-08-08T14:30:00-05:00"),
            Some(1_786_217_400)
        );
        assert_eq!(
            parse_rfc3339_to_epoch("2026-08-08T21:30:00.123456789+0200"),
            Some(1_786_217_400)
        );
    }

    #[test]
    fn rejects_unparsable_timestamps() {
        assert_eq!(parse_rfc3339_to_epoch(""), None);
        assert_eq!(parse_rfc3339_to_epoch("0001-01-01T00:00:00Z"), None);
        assert_eq!(parse_rfc3339_to_epoch("not-a-timestamp"), None);
        assert_eq!(calculate_uptime_from_rfc3339("garbage"), 0);
    }
}
