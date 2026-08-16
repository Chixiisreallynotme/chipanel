use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::client::conn::http1;
use hyper::Request;
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use std::env;
use std::path::{Path, PathBuf};
use tokio::net::UnixStream;
use tracing::{debug, info};

use crate::error::AppError;
use crate::models::podman::{ContainerMetrics, ContainerStatusResponse, ServerStatus};

#[derive(Debug, Clone)]
pub struct PodmanClient {
    socket_path: PathBuf,
}

impl Default for PodmanClient {
    fn default() -> Self {
        Self::new(find_podman_socket_path())
    }
}

impl PodmanClient {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Low-level HTTP request sender over Unix domain socket using Hyper 1.0 and TokioIo.
    async fn send_request(
        &self,
        method: &str,
        path: &str,
        body_bytes: Option<Vec<u8>>,
    ) -> Result<(u16, String), AppError> {
        use std::os::unix::fs::FileTypeExt;

        let meta = std::fs::metadata(&self.socket_path).map_err(|e| {
            AppError::InternalError(format!(
                "Podman socket inaccessible at {:?}: {}",
                self.socket_path, e
            ))
        })?;

        if !meta.file_type().is_socket() {
            return Err(AppError::InternalError(format!(
                "Path {:?} exists but is not a Unix domain socket",
                self.socket_path
            )));
        }

        let stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            AppError::InternalError(format!(
                "Failed to connect to Podman socket at {:?}: {}",
                self.socket_path, e
            ))
        })?;

        let io = TokioIo::new(stream);

        let handshake_fut = http1::handshake(io);
        let (mut sender, conn) = tokio::time::timeout(std::time::Duration::from_secs(5), handshake_fut)
            .await
            .map_err(|_| AppError::InternalError("Podman HTTP handshake timed out after 5s".to_string()))?
            .map_err(|e| AppError::InternalError(format!("Podman HTTP handshake failed: {}", e)))?;

        tokio::spawn(async move {
            if let Err(err) = conn.await {
                debug!("Podman connection closed: {:?}", err);
            }
        });

        let method_obj = match method {
            "POST" => hyper::Method::POST,
            "DELETE" => hyper::Method::DELETE,
            _ => hyper::Method::GET,
        };

        let req_body = match body_bytes {
            Some(vec) => Full::new(Bytes::from(vec)),
            None => Full::new(Bytes::new()),
        };

        let req = Request::builder()
            .method(method_obj)
            .uri(path)
            .header("Host", "localhost")
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .body(req_body)
            .map_err(|e| AppError::InternalError(format!("Failed to build Podman HTTP request: {}", e)))?;

        let response_fut = sender.send_request(req);
        let res = tokio::time::timeout(std::time::Duration::from_secs(5), response_fut)
            .await
            .map_err(|_| AppError::InternalError("Podman API request timed out after 5s".to_string()))?
            .map_err(|e| AppError::InternalError(format!("Podman API request error: {}", e)))?;

        let status = res.status().as_u16();

        let collect_fut = res.into_body().collect();
        let collected = tokio::time::timeout(std::time::Duration::from_secs(5), collect_fut)
            .await
            .map_err(|_| AppError::InternalError("Reading Podman response body timed out after 5s".to_string()))?
            .map_err(|e| AppError::InternalError(format!("Failed to read Podman response body: {}", e)))?;

        let bytes = collected.to_bytes();
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
    ///
    /// Podman 5.x's non-streaming per-container stats snapshot returns Docker
    /// cgroup-stats-compat JSON (`cpu_stats`/`precpu_stats`/`memory_stats`), not the
    /// libpod-native flat `CPU`/`MemUsage`/`MemLimit` shape `parse_podman_stats` expects.
    /// A single such snapshot always has a zeroed `precpu_stats` (no prior sample), so one
    /// query can't yield a live CPU percentage. We take two snapshots ~500ms apart and
    /// compute the standard Docker CPU-delta formula ourselves — the same technique already
    /// used for host CPU via /proc/stat deltas in minecraft/metrics.rs.
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
            // Legacy libpod-native shape already carries a pre-computed percentage.
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

    /// POST /v4.0.0/libpod/containers/{name}/start
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

    /// POST /v4.0.0/libpod/containers/{name}/stop?t=30
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

    /// POST /v4.0.0/libpod/containers/{name}/restart?t=30
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

    // Talk to the user systemd manager over the session bus. The raw hand-rolled
    // DBus encoder that preceded this was retired: it silently produced malformed
    // messages the bus dropped (wrong header-field alignment and the SIGNATURE
    // field mis-labelled as code 6 instead of 8), so every unit action failed.
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

fn get_current_uid() -> u32 {
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

/// Normalized single-sample reading, extracted from whichever raw stats shape Podman returned.
#[derive(Debug, Default)]
struct StatsSnapshot {
    memory_bytes: u64,
    memory_limit_bytes: u64,
    /// Present only when the payload was the Docker cgroup-stats-compat shape;
    /// used to compute a CPU delta between two snapshots.
    docker: Option<DockerCpuCounters>,
    /// Present only when the payload was the legacy libpod-native flat shape,
    /// which already carries a pre-computed percentage (no delta needed).
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

/// Parses a single stats snapshot, trying the Docker cgroup-stats-compat shape (what
/// Podman 5.x's per-container `/stats?stream=false` actually returns) first, then
/// falling back to the legacy libpod-native flat shape for older/differently-configured
/// Podman deployments.
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

    // Fall back to the legacy libpod-native flat shape ({"Stats":[{"CPU":..,"MemUsage":..}]}).
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

/// Splits an RFC3339 time-of-day off its UTC offset suffix.
///
/// Returns `(time_without_offset, offset_seconds)` where `offset_seconds` is what must be
/// SUBTRACTED from the local wall-clock reading to obtain UTC. Accepts `Z`, `±HH:MM`, `±HHMM`
/// and `±HH`. A bare time with no suffix is treated as UTC.
fn split_utc_offset(time_str: &str) -> Option<(&str, i64)> {
    if let Some(body) = time_str.strip_suffix('Z').or_else(|| time_str.strip_suffix('z')) {
        return Some((body, 0));
    }

    // The time-of-day itself never contains '+' or '-', so the last one starts the offset.
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

/// Parses an RFC3339 timestamp into a UTC Unix epoch in seconds.
///
/// The previous implementation split the time on `:` and dropped the trailing zone entirely,
/// so `...T21:30:00+02:00` was read as if it were UTC — container uptime came out wrong by the
/// host's offset (2 h in France). No date crate is available here, so the offset is applied by
/// hand. Returns `None` for anything unparsable or pre-1970.
fn parse_rfc3339_to_epoch(started_at: &str) -> Option<u64> {
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
    // Drop any fractional-seconds part.
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

    // `+02:00` means the wall clock is 2 h AHEAD of UTC, so UTC is 2 h earlier.
    let epoch = local_secs as i64 - offset_secs;
    if epoch < 0 {
        return None;
    }

    Some(epoch as u64)
}

fn calculate_uptime_from_rfc3339(started_at: &str) -> u64 {
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
        // 2026-08-08T21:30:00+02:00 == 2026-08-08T19:30:00Z == 1786217400
        assert_eq!(
            parse_rfc3339_to_epoch("2026-08-08T21:30:00+02:00"),
            Some(1_786_217_400)
        );
        assert_eq!(
            parse_rfc3339_to_epoch("2026-08-08T19:30:00Z"),
            Some(1_786_217_400)
        );
        // Same instant expressed west of Greenwich.
        assert_eq!(
            parse_rfc3339_to_epoch("2026-08-08T14:30:00-05:00"),
            Some(1_786_217_400)
        );
        // Compact offset form, and fractional seconds as Podman emits them.
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
