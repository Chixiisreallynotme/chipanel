use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, sleep, Duration, MissedTickBehavior};
use tracing::{debug, error};

use crate::config::AppConfig;
use crate::models::podman::ServerStatus;
use crate::models::websocket::WsServerMessage;
use crate::podman::PodmanClient;
use crate::rcon::RconClient;

/// Telemetry frame emitted before the first poll completes, and whenever nothing could be
/// reached. Everything is `None`/false — the UI must render "unavailable", not a fake value.
fn unavailable_telemetry() -> WsServerMessage {
    WsServerMessage::Telemetry {
        podman_available: false,
        container_running: None,
        cpu_percent: None,
        memory_bytes: None,
        memory_limit_bytes: None,
        memory_percent: None,
        rcon_available: false,
        tps: None,
        online_players: None,
        max_players: None,
    }
}

#[derive(Clone)]
pub struct WsHub {
    tx: broadcast::Sender<WsServerMessage>,
    latest_telemetry: Arc<RwLock<WsServerMessage>>,
    config: Arc<AppConfig>,
}

impl WsHub {
    /// Creates a new `WsHub` with a broadcast channel capacity of 100
    /// and spawns the 2-second background telemetry ticker.
    pub fn new(config: Arc<AppConfig>) -> Self {
        let (tx, _rx) = broadcast::channel(100);

        let latest_telemetry = Arc::new(RwLock::new(unavailable_telemetry()));

        let hub = Self {
            tx,
            latest_telemetry,
            config: config.clone(),
        };

        hub.spawn_telemetry_loop();

        hub
    }

    /// Subscribes a client to broadcasted `WsServerMessage` events.
    pub fn subscribe(&self) -> broadcast::Receiver<WsServerMessage> {
        self.tx.subscribe()
    }

    /// Retrieves the most recent `Telemetry` payload for immediate delivery on new WebSocket connection.
    pub async fn get_latest_telemetry(&self) -> WsServerMessage {
        self.latest_telemetry.read().await.clone()
    }

    /// Spawns a supervisor that keeps the 2-second telemetry ticker alive.
    ///
    /// The ticker runs in its own child task so that a panic inside it surfaces as a
    /// `JoinError` here instead of silently freezing telemetry forever; the supervisor
    /// logs it and respawns after a short backoff.
    ///
    /// Caveat: the release profile sets `panic = "abort"`, so in release builds a panic
    /// takes the whole process down before this can react. Supervision is still correct
    /// for unwinding builds (dev/test) and for a loop that merely returns early.
    fn spawn_telemetry_loop(&self) {
        let tx = self.tx.clone();
        let latest_telemetry = self.latest_telemetry.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                let handle = tokio::spawn(telemetry_loop(
                    tx.clone(),
                    latest_telemetry.clone(),
                    config.clone(),
                ));

                match handle.await {
                    Ok(()) => error!("Telemetry loop exited unexpectedly; restarting in 5s"),
                    Err(err) => error!("Telemetry loop panicked ({}); restarting in 5s", err),
                }

                sleep(Duration::from_secs(5)).await;
            }
        });
    }
}

/// Polls Podman + RCON every 2 seconds and broadcasts the result. Only returns if it panics
/// (caught by the supervisor above).
async fn telemetry_loop(
    tx: broadcast::Sender<WsServerMessage>,
    latest_telemetry: Arc<RwLock<WsServerMessage>>,
    config: Arc<AppConfig>,
) {
    let mut ticker = interval(Duration::from_secs(2));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let podman_client = PodmanClient::default();

    loop {
        ticker.tick().await;

        let telemetry = fetch_telemetry(&podman_client, &config).await;

        {
            let mut lock = latest_telemetry.write().await;
            *lock = telemetry.clone();
        }

        // Broadcast telemetry update to active subscribers.
        // An error here indicates zero active receivers, which is expected behavior.
        let _ = tx.send(telemetry);
    }
}

/// Whether it is worth opening an RCON connection this tick.
///
/// A container we *know* is stopped has no RCON listener, and retrying every 2s indefinitely is
/// the RCON churn this codebase was already restructured once to remove. `None` means Podman
/// itself was unreachable, so the container state is unknown — the Minecraft server may well be
/// running while ChiPanel merely lost its Podman socket, and RCON is then the only source of
/// tps/players, so we still try.
fn should_try_rcon(container_running: Option<bool>) -> bool {
    container_running != Some(false)
}

async fn fetch_telemetry(podman_client: &PodmanClient, config: &AppConfig) -> WsServerMessage {
    let container_name = &config.podman_container;

    let (podman_available, container_running) =
        match podman_client.inspect_container(container_name).await {
            Ok(status) => (true, Some(status.status == ServerStatus::Running)),
            Err(err) => {
                debug!("Podman inspect unavailable: {}", err);
                (false, None)
            }
        };

    // Metrics only exist for a running container; a stopped one reports `null`, not zeroes.
    let (cpu_percent, memory_bytes, memory_limit_bytes, memory_percent) =
        if container_running == Some(true) {
            match podman_client.get_container_metrics(container_name).await {
                Ok(m) => (
                    Some(m.cpu_percent),
                    Some(m.memory_bytes),
                    Some(m.memory_limit_bytes),
                    Some(m.memory_percent),
                ),
                Err(err) => {
                    debug!("Failed to fetch container metrics: {}", err);
                    (None, None, None, None)
                }
            }
        } else {
            (None, None, None, None)
        };

    let (rcon_available, tps, online_players, max_players) = if should_try_rcon(container_running) {
        match RconClient::connect(&config.rcon_host, config.rcon_port, &config.rcon_password).await
        {
            Ok(mut rcon) => {
                let (online, max) = match rcon.exec("list").await {
                    Ok(output) => parse_player_list(&output),
                    Err(err) => {
                        debug!("RCON 'list' failed: {}", err);
                        (None, None)
                    }
                };

                let parsed_tps = match rcon.exec("tps").await {
                    Ok(output) => parse_tps_output(&output),
                    Err(err) => {
                        debug!("RCON 'tps' failed: {}", err);
                        None
                    }
                };

                (true, parsed_tps, online, max)
            }
            Err(err) => {
                debug!("RCON metrics query unavailable: {}", err);
                (false, None, None, None)
            }
        }
    } else {
        (false, None, None, None)
    };

    WsServerMessage::Telemetry {
        podman_available,
        container_running,
        cpu_percent,
        memory_bytes,
        memory_limit_bytes,
        memory_percent,
        rcon_available,
        tps,
        online_players,
        max_players,
    }
}

/// Parses `/list` output into `(online, max)`. Returns `None` for either value that could not
/// be read out of the response — never a guessed default such as "max 20".
fn parse_player_list(output: &str) -> (Option<u32>, Option<u32>) {
    if output.is_empty() {
        return (None, None);
    }

    // Standard Vanilla/Paper Minecraft format: "There are X of a max of Y players online: ..."
    if let Some(idx) = output.find("of a max of") {
        let before = &output[..idx];
        let after = &output[idx + "of a max of".len()..];

        let online = before
            .split_whitespace()
            .last()
            .and_then(|s| s.parse::<u32>().ok());

        let max = after
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<u32>().ok());

        if online.is_some() || max.is_some() {
            return (online, max);
        }
    }

    // Alternative format: "There are X/Y players online"
    if let Some(slash_idx) = output.find('/') {
        let before = &output[..slash_idx];
        let after = &output[slash_idx + 1..];

        let online = before
            .split_whitespace()
            .last()
            .and_then(|s| s.parse::<u32>().ok());

        let max = after
            .split_whitespace()
            .next()
            .and_then(|s| s.trim_matches(|c: char| !c.is_numeric()).parse::<u32>().ok());

        if online.is_some() && max.is_some() {
            return (online, max);
        }
    }

    (None, None)
}

/// Parses `/tps` output. Returns `None` when the server did not answer with a parsable TPS
/// (e.g. vanilla, which has no `tps` command) rather than pretending the server runs at 20.0.
fn parse_tps_output(output: &str) -> Option<f32> {
    if output.is_empty() {
        return None;
    }

    // Spigot / Paper format: "TPS from last 1m, 5m, 15m: 20.0, 20.0, 20.0"
    if let Some(idx) = output.find("1m, 5m, 15m:") {
        let rest = &output[idx + "1m, 5m, 15m:".len()..];
        if let Some(first_tps) = rest.split(',').next() {
            if let Ok(tps) = first_tps.trim().parse::<f32>() {
                return Some(tps.clamp(0.0, 20.0));
            }
        }
    }

    // Generic format: "TPS: 19.8" or "Mean tick time: 10.5ms (TPS: 20.0)"
    if let Some(idx) = output.find("TPS:") {
        let rest = &output[idx + 4..];
        let num_str: String = rest
            .trim_start()
            .chars()
            .take_while(|c| c.is_numeric() || *c == '.')
            .collect();
        if let Ok(tps) = num_str.parse::<f32>() {
            return Some(tps.clamp(0.0, 20.0));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_rcon_only_when_container_is_known_stopped() {
        assert!(should_try_rcon(Some(true)));
        assert!(!should_try_rcon(Some(false)));
        // Podman unreachable: state unknown, still worth trying RCON.
        assert!(should_try_rcon(None));
    }

    #[test]
    fn parses_paper_player_list() {
        let (online, max) = parse_player_list("There are 3 of a max of 40 players online: a, b, c");
        assert_eq!(online, Some(3));
        assert_eq!(max, Some(40));
    }

    #[test]
    fn returns_none_for_unreadable_player_list() {
        assert_eq!(parse_player_list(""), (None, None));
        assert_eq!(parse_player_list("Unknown command"), (None, None));
    }

    #[test]
    fn parses_paper_tps() {
        assert_eq!(
            parse_tps_output("TPS from last 1m, 5m, 15m: 19.4, 19.9, 20.0"),
            Some(19.4)
        );
        assert_eq!(parse_tps_output("Mean tick time: 10.5ms (TPS: 20.0)"), Some(20.0));
    }

    #[test]
    fn returns_none_when_tps_is_unavailable() {
        assert_eq!(parse_tps_output(""), None);
        assert_eq!(parse_tps_output("Unknown or incomplete command"), None);
    }
}
