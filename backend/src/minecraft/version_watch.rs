//! Background watcher for new Minecraft versions.
//!
//! Polls Mojang's version manifest once an hour (15min retry on failure), keeps the
//! shared engine-catalog cache warm from the same fetch, and raises an alert whenever
//! the latest release or snapshot *changes*. The state is persisted to
//! `data_dir/version_watch.json` so alerts survive restarts; the very first successful
//! check only records a silent baseline (a fresh install must not alert on a version
//! that has been out for weeks).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::routes::engine_catalog::{
    catalog_from_manifest, fetch_remote_manifest, store_catalog, MojangManifest, CATALOG_CACHE_TTL,
};

/// Same cadence as the catalog cache: one fetch serves both.
const WATCH_INTERVAL: Duration = Duration::from_secs(3600);
const WATCH_RETRY_INTERVAL: Duration = Duration::from_secs(900);
const PERSISTED_FILENAME: &str = "version_watch.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersion {
    pub id: String,
    /// ISO-8601 publication timestamp reported by Mojang ("" when unknown).
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewVersionAlert {
    pub id: String,
    /// The latest version before this one took over, when known.
    pub previous: Option<String>,
    pub published_at: String,
    /// Unix seconds — when ChiPanel first saw this version as the latest.
    pub detected_at: u64,
}

/// Snapshot of the watcher state, served on `/api/server/engine`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionWatchInfo {
    pub latest_release: Option<LatestVersion>,
    pub latest_snapshot: Option<LatestVersion>,
    /// Present once the latest release changed under our watch (vs. the persisted
    /// baseline). Stays until superseded by an even newer release.
    pub new_release: Option<NewVersionAlert>,
    pub new_snapshot: Option<NewVersionAlert>,
    pub last_check_at: Option<u64>,
    #[serde(default)]
    pub last_check_ok: bool,
}

#[derive(Debug, Default)]
struct WatchState {
    info: VersionWatchInfo,
    /// False until the first successful check has recorded the silent baseline.
    initialized: bool,
}

/// What a persisted run of the watcher remembers across restarts.
#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedWatch {
    latest_release: Option<LatestVersion>,
    latest_snapshot: Option<LatestVersion>,
    new_release: Option<NewVersionAlert>,
    new_snapshot: Option<NewVersionAlert>,
}

struct WatchUpdate {
    /// True when the baseline moved — the persisted file must be rewritten.
    changed: bool,
    /// Human-readable descriptions of alerts raised this tick (for logs/tests).
    raised: Vec<String>,
}

static WATCH_STATE: LazyLock<RwLock<WatchState>> =
    LazyLock::new(|| RwLock::new(WatchState::default()));

/// Current watcher state for the HTTP layer.
pub async fn get_version_watch_info() -> VersionWatchInfo {
    WATCH_STATE.read().await.info.clone()
}

/// Spawns the hourly watch loop. `data_dir` is where the state file lives.
pub fn start_version_watch(data_dir: PathBuf) {
    tokio::spawn(async move {
        let persist_path = data_dir.join(PERSISTED_FILENAME);
        load_persisted(&persist_path).await;

        loop {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            match fetch_remote_manifest().await {
                Ok(manifest) => {
                    match catalog_from_manifest(&manifest) {
                        Ok(catalog) => {
                            store_catalog(catalog, CATALOG_CACHE_TTL).await;
                        }
                        Err(e) => warn!("Version watcher could not build catalog: {}", e),
                    }

                    let update = {
                        let mut state = WATCH_STATE.write().await;
                        state.info.last_check_at = Some(now);
                        state.info.last_check_ok = true;
                        apply_manifest(&mut state, &manifest, now)
                    };

                    for raised in &update.raised {
                        info!("New Minecraft version detected: {}", raised);
                    }
                    if update.changed {
                        persist(&persist_path).await;
                    }

                    tokio::time::sleep(WATCH_INTERVAL).await;
                }
                Err(e) => {
                    warn!("Version watch check failed: {}", e);
                    let mut state = WATCH_STATE.write().await;
                    state.info.last_check_at = Some(now);
                    state.info.last_check_ok = false;
                    drop(state);
                    tokio::time::sleep(WATCH_RETRY_INTERVAL).await;
                }
            }
        }
    });
}

/// Folds a fetched manifest into the watch state. Pure logic, unit-tested.
fn apply_manifest(state: &mut WatchState, manifest: &MojangManifest, now: u64) -> WatchUpdate {
    let mut update = WatchUpdate {
        changed: false,
        raised: Vec::new(),
    };

    if let Some(latest) = &manifest.latest {
        if let Some(new_id) = &latest.release {
            apply_one(
                new_id,
                "release",
                manifest,
                &mut state.info.latest_release,
                &mut state.info.new_release,
                state.initialized,
                now,
                &mut update,
            );
        }
        if let Some(new_id) = &latest.snapshot {
            apply_one(
                new_id,
                "snapshot",
                manifest,
                &mut state.info.latest_snapshot,
                &mut state.info.new_snapshot,
                state.initialized,
                now,
                &mut update,
            );
        }
    }

    // The first successful check only records the baseline; alerts start afterwards.
    if update.changed && !state.initialized {
        state.initialized = true;
    }

    update
}

#[allow(clippy::too_many_arguments)]
fn apply_one(
    new_id: &str,
    kind: &str,
    manifest: &MojangManifest,
    current: &mut Option<LatestVersion>,
    alert: &mut Option<NewVersionAlert>,
    initialized: bool,
    now: u64,
    update: &mut WatchUpdate,
) {
    if current.as_ref().map(|c| c.id.as_str()) == Some(new_id) {
        return;
    }

    let published_at = manifest
        .versions
        .iter()
        .find(|v| v.id == new_id)
        .map(|v| v.release_time.clone())
        .unwrap_or_default();

    let previous = current.replace(LatestVersion {
        id: new_id.to_string(),
        published_at: published_at.clone(),
    });

    if initialized {
        *alert = Some(NewVersionAlert {
            id: new_id.to_string(),
            previous: previous.map(|p| p.id),
            published_at,
            detected_at: now,
        });
        update.raised.push(match &alert.as_ref().unwrap().previous {
            Some(prev) => format!("{} {} (was {})", kind, new_id, prev),
            None => format!("{} {}", kind, new_id),
        });
    }

    update.changed = true;
}

async fn load_persisted(path: &Path) {
    match tokio::fs::read_to_string(path).await {
        Ok(body) => match serde_json::from_str::<PersistedWatch>(&body) {
            Ok(p) => {
                let mut state = WATCH_STATE.write().await;
                state.info.latest_release = p.latest_release;
                state.info.latest_snapshot = p.latest_snapshot;
                state.info.new_release = p.new_release;
                state.info.new_snapshot = p.new_snapshot;
                // A persisted baseline exists: changes from now on raise alerts.
                state.initialized = true;
                info!("Version watcher resumed from {:?}", path);
            }
            Err(e) => warn!("Could not parse {:?}: {}", path, e),
        },
        // First run ever: baseline will be recorded silently on the first check.
        Err(_) => {}
    }
}

async fn persist(path: &Path) {
    let body = {
        let state = WATCH_STATE.read().await;
        PersistedWatch {
            latest_release: state.info.latest_release.clone(),
            latest_snapshot: state.info.latest_snapshot.clone(),
            new_release: state.info.new_release.clone(),
            new_snapshot: state.info.new_snapshot.clone(),
        }
    };

    match serde_json::to_string_pretty(&body) {
        Ok(json) => {
            if let Some(parent) = path.parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }
            if let Err(e) = tokio::fs::write(path, json).await {
                warn!("Could not persist version watch state to {:?}: {}", path, e);
            }
        }
        Err(e) => warn!("Could not serialize version watch state: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::engine_catalog::{MojangLatest, MojangManifestEntry};

    fn manifest(release: &str, snapshot: &str) -> MojangManifest {
        MojangManifest {
            latest: Some(MojangLatest {
                release: Some(release.to_string()),
                snapshot: Some(snapshot.to_string()),
            }),
            versions: vec![
                MojangManifestEntry {
                    id: snapshot.to_string(),
                    kind: "snapshot".to_string(),
                    release_time: "2026-08-01T10:00:00+00:00".to_string(),
                },
                MojangManifestEntry {
                    id: release.to_string(),
                    kind: "release".to_string(),
                    release_time: "2026-07-15T10:00:00+00:00".to_string(),
                },
            ],
        }
    }

    #[test]
    fn first_check_baselines_silently() {
        let mut state = WatchState::default();
        let update = apply_manifest(&mut state, &manifest("26.2", "26.3-snapshot-7"), 1000);

        assert!(update.changed);
        assert!(update.raised.is_empty(), "fresh installs must not alert");
        assert!(state.initialized);
        assert_eq!(state.info.latest_release.as_ref().unwrap().id, "26.2");
        assert_eq!(state.info.latest_snapshot.as_ref().unwrap().id, "26.3-snapshot-7");
        assert!(state.info.new_release.is_none());
        assert!(state.info.new_snapshot.is_none());
    }

    #[test]
    fn detects_new_release_and_snapshot() {
        let mut state = WatchState::default();
        apply_manifest(&mut state, &manifest("26.2", "26.3-snapshot-7"), 1000);

        let update = apply_manifest(&mut state, &manifest("26.3", "26.3-snapshot-8"), 2000);

        assert!(update.changed);
        assert_eq!(update.raised.len(), 2);

        let alert = state.info.new_release.clone().unwrap();
        assert_eq!(alert.id, "26.3");
        assert_eq!(alert.previous.as_deref(), Some("26.2"));
        assert_eq!(alert.published_at, "2026-07-15T10:00:00+00:00");
        assert_eq!(alert.detected_at, 2000);

        let snap = state.info.new_snapshot.clone().unwrap();
        assert_eq!(snap.id, "26.3-snapshot-8");
        assert_eq!(snap.previous.as_deref(), Some("26.3-snapshot-7"));
    }

    #[test]
    fn unchanged_manifest_is_quiet() {
        let mut state = WatchState::default();
        apply_manifest(&mut state, &manifest("26.2", "26.3-snapshot-7"), 1000);

        let update = apply_manifest(&mut state, &manifest("26.2", "26.3-snapshot-7"), 2000);

        assert!(!update.changed);
        assert!(update.raised.is_empty());
        assert!(state.info.new_release.is_none());
    }

    #[test]
    fn newer_alert_supersedes_older_one() {
        let mut state = WatchState::default();
        apply_manifest(&mut state, &manifest("26.2", "26.3-snapshot-7"), 1000);
        apply_manifest(&mut state, &manifest("26.3", "26.3-snapshot-8"), 2000);
        apply_manifest(&mut state, &manifest("26.3.1", "26.3-snapshot-8"), 3000);

        let alert = state.info.new_release.clone().unwrap();
        assert_eq!(alert.id, "26.3.1");
        assert_eq!(alert.previous.as_deref(), Some("26.3"));
        assert_eq!(alert.detected_at, 3000);
    }

    #[test]
    fn persisted_roundtrip_keeps_alerts() {
        let persisted = PersistedWatch {
            latest_release: Some(LatestVersion {
                id: "26.3".to_string(),
                published_at: "t".to_string(),
            }),
            latest_snapshot: None,
            new_release: Some(NewVersionAlert {
                id: "26.3".to_string(),
                previous: Some("26.2".to_string()),
                published_at: "t".to_string(),
                detected_at: 42,
            }),
            new_snapshot: None,
        };

        let json = serde_json::to_string(&persisted).unwrap();
        let back: PersistedWatch = serde_json::from_str(&json).unwrap();

        assert_eq!(back.latest_release.unwrap().id, "26.3");
        let alert = back.new_release.unwrap();
        assert_eq!(alert.previous.as_deref(), Some("26.2"));
        assert_eq!(alert.detected_at, 42);
    }
}
