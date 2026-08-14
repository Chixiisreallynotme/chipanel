use std::{
    collections::VecDeque,
    path::Path,
    sync::{Arc, Mutex as StdMutex, RwLock as StdRwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::{config::AppConfig, error::AppError, rcon::RconClient};

/// A single telemetry sample. Every measured field is `Option` and serializes to `null` when
/// the source was unreachable — an absent reading must never be backfilled with a
/// plausible-looking number (a fabricated `20.0` TPS reads exactly like a healthy server).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSample {
    pub timestamp: u64,
    pub tps: Option<f32>,
    pub cpu_percent: Option<f32>,
    pub ram_used_mb: Option<u64>,
    pub ram_total_mb: Option<u64>,
    pub disk_used_mb: Option<u64>,
    pub disk_total_mb: Option<u64>,
    pub player_count: Option<u32>,
    /// Whether RCON answered when this sample was taken (explains a `null` tps/player_count).
    pub rcon_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparkProfilerRequest {
    pub duration_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparkProfilerResponse {
    pub report_url: Option<String>,
    pub output_log: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub tps_threshold: f32,
    pub cpu_threshold: f32,
    pub ram_threshold: f32,
    pub discord_webhook_url: Option<String>,
    pub alerts_enabled: bool,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            tps_threshold: 15.0,
            cpu_threshold: 85.0,
            ram_threshold: 90.0,
            discord_webhook_url: None,
            alerts_enabled: false,
        }
    }
}

/// Only Discord's own webhook hosts are reachable. The webhook URL is admin-supplied, so
/// without this allow-list it is an SSRF primitive pointing anywhere inside the homelab.
const DISCORD_WEBHOOK_HOSTS: [&str; 4] = [
    "discord.com",
    "discordapp.com",
    "canary.discord.com",
    "ptb.discord.com",
];

/// Rejects any webhook URL that is not an `https` Discord webhook endpoint.
///
/// Uses `reqwest::Url` (re-exported by the `url` crate that reqwest already depends on) — no
/// new dependency.
pub fn validate_discord_webhook_url(url: &str) -> Result<(), AppError> {
    let parsed = reqwest::Url::parse(url.trim())
        .map_err(|e| AppError::BadRequest(format!("Invalid Discord webhook URL: {}", e)))?;

    if parsed.scheme() != "https" {
        return Err(AppError::BadRequest(
            "Discord webhook URL must use https".to_string(),
        ));
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| AppError::BadRequest("Discord webhook URL has no host".to_string()))?
        .to_ascii_lowercase();

    if !DISCORD_WEBHOOK_HOSTS.contains(&host.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Discord webhook host '{}' is not allowed (permitted: {})",
            host,
            DISCORD_WEBHOOK_HOSTS.join(", ")
        )));
    }

    Ok(())
}

/// File name used under `data_dir` for the persisted alert configuration.
const ALERT_CONFIG_FILE: &str = "alert_config.json";

impl AlertConfig {
    /// Loads the persisted alert config from `data_dir`, falling back to defaults when the file
    /// is absent or unreadable. A stored webhook that no longer passes the host allow-list is
    /// dropped rather than silently trusted.
    pub async fn load_or_create(data_dir: &Path) -> Self {
        let storage_path = data_dir.join(ALERT_CONFIG_FILE);

        if !storage_path.exists() {
            return Self::default();
        }

        let mut config = match tokio::fs::read_to_string(&storage_path).await {
            Ok(content) => match serde_json::from_str::<AlertConfig>(&content) {
                Ok(parsed) => {
                    info!("Loaded alert configuration from disk");
                    parsed
                }
                Err(e) => {
                    warn!("Could not parse alert config file, using defaults: {}", e);
                    return Self::default();
                }
            },
            Err(e) => {
                warn!("Could not read alert config file, using defaults: {}", e);
                return Self::default();
            }
        };

        if let Some(ref url) = config.discord_webhook_url {
            if let Err(e) = validate_discord_webhook_url(url) {
                warn!("Discarding persisted Discord webhook URL: {}", e);
                config.discord_webhook_url = None;
                config.alerts_enabled = false;
            }
        }

        config
    }

    /// Persists the alert config to `data_dir`, creating the directory if needed.
    pub async fn save(&self, data_dir: &Path) -> Result<(), AppError> {
        let storage_path = data_dir.join(ALERT_CONFIG_FILE);

        let _ = tokio::fs::create_dir_all(data_dir).await;

        let json_data = serde_json::to_string_pretty(self).map_err(|e| {
            AppError::InternalError(format!("Failed to serialize alert config: {}", e))
        })?;

        tokio::fs::write(&storage_path, json_data)
            .await
            .map_err(|e| {
                AppError::InternalError(format!("Failed to write alert config file: {}", e))
            })?;

        Ok(())
    }
}

/// Parallel arrays indexed by `timestamps`. A `null` entry means "not measured at that point",
/// which the UI must render as a gap rather than as a zero or a carried-forward value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsHistoryResponse {
    pub timestamps: Vec<u64>,
    pub tps: Vec<Option<f32>>,
    pub cpu: Vec<Option<f32>>,
    pub ram_mb: Vec<Option<u64>>,
    pub players: Vec<Option<u32>>,
    /// Per-sample RCON availability, same indexing as the arrays above.
    pub rcon_available: Vec<bool>,
}

#[derive(Debug)]
pub struct MetricsStore {
    pub samples: StdRwLock<VecDeque<MetricSample>>,
}

impl MetricsStore {
    pub fn new() -> Self {
        Self {
            samples: StdRwLock::new(VecDeque::with_capacity(3600)),
        }
    }

    pub fn push(&self, sample: MetricSample) {
        if let Ok(mut lock) = self.samples.write() {
            if lock.len() >= 3600 {
                lock.pop_front();
            }
            lock.push_back(sample);
        }
    }

    pub fn get_current(&self) -> Option<MetricSample> {
        if let Ok(lock) = self.samples.read() {
            lock.back().cloned()
        } else {
            None
        }
    }

    pub fn get_history(&self, range: &str) -> MetricsHistoryResponse {
        let seconds_back: u64 = match range {
            "24h" => 86400,
            "6h" => 21600,
            "1h" | _ => 3600,
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let cutoff = now.saturating_sub(seconds_back);

        let mut timestamps = Vec::new();
        let mut tps = Vec::new();
        let mut cpu = Vec::new();
        let mut ram_mb = Vec::new();
        let mut players = Vec::new();
        let mut rcon_available = Vec::new();

        if let Ok(lock) = self.samples.read() {
            let cap = lock.len();
            timestamps.reserve(cap);
            tps.reserve(cap);
            cpu.reserve(cap);
            ram_mb.reserve(cap);
            players.reserve(cap);
            rcon_available.reserve(cap);

            for sample in lock.iter() {
                if sample.timestamp >= cutoff {
                    timestamps.push(sample.timestamp);
                    tps.push(sample.tps);
                    cpu.push(sample.cpu_percent);
                    ram_mb.push(sample.ram_used_mb);
                    players.push(sample.player_count);
                    rcon_available.push(sample.rcon_available);
                }
            }
        }

        MetricsHistoryResponse {
            timestamps,
            tps,
            cpu,
            ram_mb,
            players,
            rcon_available,
        }
    }
}

impl Default for MetricsStore {
    fn default() -> Self {
        Self::new()
    }
}

static LAST_TPS_ALERT: StdMutex<u64> = StdMutex::new(0);
static LAST_CPU_ALERT: StdMutex<u64> = StdMutex::new(0);
static LAST_RAM_ALERT: StdMutex<u64> = StdMutex::new(0);

pub async fn check_and_send_discord_alerts(sample: &MetricSample, config: &AlertConfig) {
    if !config.alerts_enabled {
        return;
    }
    let Some(ref webhook_url) = config.discord_webhook_url else {
        return;
    };
    if webhook_url.trim().is_empty() {
        return;
    }

    // Refuse to talk to a non-Discord host even if a stale/hand-edited config file carries one.
    if let Err(e) = validate_discord_webhook_url(webhook_url) {
        warn!("Skipping Discord alert: {}", e);
        return;
    }

    let now = sample.timestamp;
    let cooldown_secs = 300; // 5 minutes max 1 notification per alert type

    // Check TPS alert. A missing reading (RCON down) is not an alert condition — it is an
    // unknown, and alerting on unknowns would fire continuously during an outage.
    if let Some(tps) = sample.tps.filter(|t| *t < config.tps_threshold && *t > 0.0) {
        let should_send = match LAST_TPS_ALERT.lock() {
            Ok(mut last) => {
                if now.saturating_sub(*last) >= cooldown_secs {
                    *last = now;
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        };

        if should_send {
            let is_danger = tps < 10.0;
            let title = if is_danger {
                "🚨 CRITICAL: Low Server TPS Alert"
            } else {
                "⚠️ WARNING: Low Server TPS Alert"
            };
            let description = format!(
                "Minecraft server TPS dropped to **{:.2}** (Threshold: {:.2})",
                tps, config.tps_threshold
            );
            let color = if is_danger { 15158332 } else { 15844367 };
            send_discord_webhook(webhook_url, title, &description, color, sample).await;
        }
    }

    // Check CPU alert
    if let Some(cpu_percent) = sample.cpu_percent.filter(|c| *c > config.cpu_threshold) {
        let should_send = match LAST_CPU_ALERT.lock() {
            Ok(mut last) => {
                if now.saturating_sub(*last) >= cooldown_secs {
                    *last = now;
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        };

        if should_send {
            let is_danger = cpu_percent > 95.0;
            let title = if is_danger {
                "🚨 CRITICAL: High System CPU Usage"
            } else {
                "⚠️ WARNING: High System CPU Usage"
            };
            let description = format!(
                "Server CPU usage reached **{:.1}%** (Threshold: {:.1}%)",
                cpu_percent, config.cpu_threshold
            );
            let color = if is_danger { 15158332 } else { 15844367 };
            send_discord_webhook(webhook_url, title, &description, color, sample).await;
        }
    }

    // Check RAM alert. Both readings must be present; an unknown is not a threshold breach.
    let (ram_used_mb, ram_total_mb) = match (sample.ram_used_mb, sample.ram_total_mb) {
        (Some(used), Some(total)) if total > 0 => (used, total),
        _ => return,
    };

    let ram_pct = (ram_used_mb as f32 / ram_total_mb as f32) * 100.0;

    let ram_triggered = if config.ram_threshold <= 100.0 {
        ram_pct > config.ram_threshold
    } else {
        ram_used_mb as f32 > config.ram_threshold
    };

    if ram_triggered {
        let should_send = match LAST_RAM_ALERT.lock() {
            Ok(mut last) => {
                if now.saturating_sub(*last) >= cooldown_secs {
                    *last = now;
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        };

        if should_send {
            let is_danger = ram_pct > 95.0;
            let title = if is_danger {
                "🚨 CRITICAL: High Memory Usage"
            } else {
                "⚠️ WARNING: High Memory Usage"
            };
            let description = format!(
                "Server Memory usage reached **{} MB / {} MB ({:.1}%)** (Threshold: {:.1}%)",
                ram_used_mb, ram_total_mb, ram_pct, config.ram_threshold
            );
            let color = if is_danger { 15158332 } else { 15844367 };
            send_discord_webhook(webhook_url, title, &description, color, sample).await;
        }
    }
}

static DISCORD_CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();

/// Hard cap on how long a Discord webhook call may take. Without it, a slow or black-holed
/// webhook host stalls the caller (the telemetry sampler / the alert-config handler) forever.
const DISCORD_HTTP_TIMEOUT: Duration = Duration::from_secs(10);

fn get_discord_client() -> &'static reqwest::Client {
    DISCORD_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent("ChiPanel/0.1.0")
            .timeout(DISCORD_HTTP_TIMEOUT)
            .connect_timeout(Duration::from_secs(5))
            // `validate_discord_webhook_url` pins the host, but reqwest follows up to 10
            // redirects by default without re-checking it — a 302 from Discord's domain to an
            // internal address would walk straight through the allow-list. Never follow one;
            // callers surface the 3xx as an error instead.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            // A builder failure here means the TLS backend is unusable, which `Client::new()`
            // would panic on anyway — and any fallback client would silently drop the timeout
            // and redirect protections above.
            .expect("failed to build Discord HTTP client: TLS backend unavailable")
    })
}

/// Formats an optional reading for a Discord embed field. Missing readings are shown as
/// "unavailable", never as a stand-in number.
fn fmt_opt<T: std::fmt::Display>(value: Option<T>) -> String {
    match value {
        Some(v) => v.to_string(),
        None => "unavailable".to_string(),
    }
}

async fn send_discord_webhook(
    url: &str,
    title: &str,
    description: &str,
    color: u32,
    sample: &MetricSample,
) {
    let payload = serde_json::json!({
        "embeds": [
            {
                "title": title,
                "description": description,
                "color": color,
                "fields": [
                    { "name": "TPS", "value": fmt_opt(sample.tps.map(|v| format!("{:.2}", v))), "inline": true },
                    { "name": "CPU", "value": fmt_opt(sample.cpu_percent.map(|v| format!("{:.1}%", v))), "inline": true },
                    { "name": "RAM", "value": format!("{} / {} MB", fmt_opt(sample.ram_used_mb), fmt_opt(sample.ram_total_mb)), "inline": true },
                    { "name": "Online Players", "value": fmt_opt(sample.player_count), "inline": true }
                ],
                "footer": { "text": "ChiPanel Telemetry Alert Engine" }
            }
        ]
    });

    match get_discord_client().post(url).json(&payload).send().await {
        // Redirects are not followed (see `get_discord_client`), so a 3xx means the alert was
        // never delivered — don't let that pass as a success.
        Ok(resp) if resp.status().is_redirection() => error!(
            "Discord alert webhook returned {} and redirects are not followed; alert not delivered",
            resp.status()
        ),
        Ok(_) => {}
        Err(e) => error!("Failed to send Discord alert webhook: {}", e),
    }
}

pub async fn send_test_discord_webhook(url: &str) -> Result<(), AppError> {
    validate_discord_webhook_url(url)?;

    let payload = serde_json::json!({
        "embeds": [
            {
                "title": "✅ ChiPanel Alert Engine - Test Webhook",
                "description": "Discord webhook alerts have been successfully configured for ChiPanel!",
                "color": 3066993,
                "footer": { "text": "ChiPanel Telemetry Engine" }
            }
        ]
    });

    let resp = get_discord_client()
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to send test Discord webhook: {}", e)))?;

    // Redirects are not followed, so a 3xx never reached Discord. Report it without echoing the
    // `Location` header — this message is client-facing and that target is exactly the host the
    // allow-list exists to keep out of reach.
    if resp.status().is_redirection() {
        return Err(AppError::BadRequest(format!(
            "Discord webhook URL returned a redirect ({}); redirects are not followed",
            resp.status()
        )));
    }

    Ok(())
}

pub fn start_telemetry_sampler(
    store: Arc<MetricsStore>,
    alert_config: Arc<RwLock<AlertConfig>>,
    config: AppConfig,
    ws_hub: crate::websocket::WsHub,
) {
    tokio::spawn(async move {
        let mut prev_cpu = None;
        let mut interval = tokio::time::interval(Duration::from_secs(2));

        loop {
            interval.tick().await;

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let cpu_percent = calc_cpu_percent(&mut prev_cpu);
            let (ram_used_mb, ram_total_mb) = read_ram_stats();
            let (disk_used_mb, disk_total_mb) = read_disk_stats(&config.minecraft_data_dir);

            // Reuse the RCON tps/player-count already fetched by WsHub's own 2s ticker instead
            // of opening a second independent RCON connection here — the two loops were each
            // reconnecting to RCON every 2s, hammering it with a fresh connection roughly once
            // a second combined (confirmed in server logs: "RCON Client started/shutting down"
            // back-to-back). At most ~2s stale, which is fine for a historical/graphing sample.
            let (tps, player_count, rcon_available) = match ws_hub.get_latest_telemetry().await {
                crate::models::websocket::WsServerMessage::Telemetry {
                    tps,
                    online_players,
                    rcon_available,
                    ..
                } => (tps, online_players, rcon_available),
                // The hub only ever stores a Telemetry frame here; anything else means we have
                // no reading at all, which is recorded as such rather than guessed.
                _ => (None, None, false),
            };

            let sample = MetricSample {
                timestamp,
                tps,
                cpu_percent,
                ram_used_mb,
                ram_total_mb,
                disk_used_mb,
                disk_total_mb,
                player_count,
                rcon_available,
            };

            store.push(sample.clone());

            let alerts = alert_config.read().await.clone();
            check_and_send_discord_alerts(&sample, &alerts).await;
        }
    });
}

pub async fn trigger_spark_profiler(
    duration_seconds: u32,
    config: &AppConfig,
) -> Result<SparkProfilerResponse, AppError> {
    let duration = duration_seconds.clamp(5, 300);

    let mut client = RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to connect to RCON for Spark profiler: {}", e)))?;

    let start_output = client
        .exec("spark profiler start")
        .await
        .unwrap_or_else(|e| format!("Error starting spark profiler: {}", e));

    info!("Spark profiler started for {}s", duration);
    tokio::time::sleep(Duration::from_secs(duration as u64)).await;

    let stop_output = client
        .exec("spark profiler stop")
        .await
        .unwrap_or_else(|e| format!("Error stopping spark profiler: {}", e));

    let combined_log = format!("=== Spark Start ===\n{}\n\n=== Spark Stop ===\n{}", start_output, stop_output);
    let report_url = extract_spark_url(&combined_log);

    Ok(SparkProfilerResponse {
        report_url,
        output_log: combined_log,
    })
}

fn extract_spark_url(log: &str) -> Option<String> {
    for word in log.split_whitespace() {
        let clean_word = word.trim_matches(|c: char| {
            c == '§' || c == '[' || c == ']' || c == '(' || c == ')' || c == ',' || c == '.' || c == '<' || c == '>'
        });
        if clean_word.starts_with("https://spark.lucko.me/") || clean_word.starts_with("http://spark.lucko.me/") {
            return Some(clean_word.to_string());
        }
    }
    None
}

struct CpuStats {
    idle: u64,
    total: u64,
}

fn read_cpu_stats() -> Option<CpuStats> {
    let content = std::fs::read_to_string("/proc/stat").ok()?;
    let line = content.lines().next()?;
    if !line.starts_with("cpu ") {
        return None;
    }
    let parts: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();

    if parts.len() < 4 {
        return None;
    }
    let user = parts[0];
    let nice = parts[1];
    let system = parts[2];
    let idle = parts[3];
    let iowait = parts.get(4).copied().unwrap_or(0);
    let irq = parts.get(5).copied().unwrap_or(0);
    let softirq = parts.get(6).copied().unwrap_or(0);
    let steal = parts.get(7).copied().unwrap_or(0);

    let total = user + nice + system + idle + iowait + irq + softirq + steal;
    let idle_total = idle + iowait;

    Some(CpuStats { idle: idle_total, total })
}

/// Returns `None` when no percentage can be computed yet — on the very first tick there is no
/// previous snapshot to diff against, and reporting `0.0` there is indistinguishable from a
/// genuinely idle host.
fn calc_cpu_percent(prev: &mut Option<CpuStats>) -> Option<f32> {
    let curr = read_cpu_stats()?;

    let percent = match prev {
        Some(p) => {
            let total_delta = curr.total.saturating_sub(p.total);
            let idle_delta = curr.idle.saturating_sub(p.idle);
            if total_delta > 0 {
                let usage = 100.0 * (1.0 - (idle_delta as f32 / total_delta as f32));
                Some(usage.clamp(0.0, 100.0))
            } else {
                None
            }
        }
        None => None,
    };

    *prev = Some(curr);
    percent
}

/// Returns `(used_mb, total_mb)`, each `None` when `/proc/meminfo` did not yield the value.
fn read_ram_stats() -> (Option<u64>, Option<u64>) {
    let Ok(content) = std::fs::read_to_string("/proc/meminfo") else {
        return (None, None);
    };

    let mut mem_total_kb: Option<u64> = None;
    let mut mem_avail_kb: Option<u64> = None;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            mem_total_kb = parse_kb_line(line);
        } else if line.starts_with("MemAvailable:") {
            mem_avail_kb = parse_kb_line(line);
        }
    }

    let used_mb = match (mem_total_kb, mem_avail_kb) {
        (Some(total), Some(avail)) => Some(total.saturating_sub(avail) / 1024),
        _ => None,
    };

    (used_mb, mem_total_kb.map(|kb| kb / 1024))
}

fn parse_kb_line(line: &str) -> Option<u64> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].parse::<u64>().ok()
    } else {
        None
    }
}

/// Returns `(used_mb, total_mb)`, or `(None, None)` when `statvfs` could not read the mount.
fn read_disk_stats(path: &std::path::Path) -> (Option<u64>, Option<u64>) {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let path_cstring = match CString::new(path.as_os_str().as_bytes()) {
        Ok(s) => s,
        Err(_) => match CString::new("/") {
            Ok(s) => s,
            Err(_) => return (None, None),
        },
    };

    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(path_cstring.as_ptr(), &mut stat) == 0 {
            let block_size = if stat.f_frsize > 0 {
                stat.f_frsize as u64
            } else {
                stat.f_bsize as u64
            };
            let total_bytes = stat.f_blocks as u64 * block_size;
            let free_bytes = stat.f_bavail as u64 * block_size;
            let used_bytes = total_bytes.saturating_sub(free_bytes);

            let total_mb = total_bytes / (1024 * 1024);
            let used_mb = used_bytes / (1024 * 1024);
            (Some(used_mb), Some(total_mb))
        } else {
            (None, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_internal_and_non_discord_webhook_urls() {
        // SSRF into the homelab — plain HTTP loopback and a private-range host.
        assert!(validate_discord_webhook_url("http://127.0.0.1:8080/x").is_err());
        assert!(validate_discord_webhook_url("https://192.168.1.5/hook").is_err());
        // Right host, wrong scheme.
        assert!(validate_discord_webhook_url("http://discord.com/api/webhooks/1/abc").is_err());
        // Look-alike host that merely contains the allowed one.
        assert!(validate_discord_webhook_url("https://discord.com.evil.tld/api/webhooks/1/abc").is_err());
        assert!(validate_discord_webhook_url("not a url").is_err());
    }

    #[test]
    fn accepts_discord_webhook_urls() {
        assert!(validate_discord_webhook_url("https://discord.com/api/webhooks/123/abcDEF").is_ok());
        assert!(
            validate_discord_webhook_url("https://discordapp.com/api/webhooks/123/abcDEF").is_ok()
        );
        assert!(
            validate_discord_webhook_url("https://ptb.discord.com/api/webhooks/123/abc").is_ok()
        );
        assert!(
            validate_discord_webhook_url("https://canary.discord.com/api/webhooks/123/abc").is_ok()
        );
    }
}

