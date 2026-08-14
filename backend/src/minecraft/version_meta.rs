use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::warn;

/// Protocol + data version for a Minecraft release/snapshot, resolved from
/// PrismarineJS `minecraft-data` (the same source the ecosystem uses to map a
/// version id to its network protocol). This is what lets ChiPanel keep the
/// lazymc `public.protocol` hint correct when the engine version changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionMeta {
    pub protocol: i64,
    pub data_version: i64,
}

const PROTOCOL_VERSIONS_URL: &str =
    "https://raw.githubusercontent.com/PrismarineJS/minecraft-data/master/data/pc/common/protocolVersions.json";

/// Offline fallback for the release list in `engine_catalog.rs`, generated from
/// PrismarineJS `protocolVersions.json`. `data_version` is negative for pre-1.9
/// versions (no real data version existed then) and is only meaningful when > 0.
const FALLBACK_PROTOCOLS: &[(&str, i64, i64)] = &[
    ("26.2", 776, 4903),
    ("26.1.2", 775, 4790),
    ("26.1.1", 775, 4788),
    ("26.1", 775, 4786),
    ("1.21.11", 774, 4671),
    ("1.21.10", 773, 4556),
    ("1.21.9", 773, 4554),
    ("1.21.8", 772, 4440),
    ("1.21.7", 772, 4438),
    ("1.21.6", 771, 4435),
    ("1.21.5", 770, 4325),
    ("1.21.4", 769, 4189),
    ("1.21.3", 768, 4082),
    ("1.21.2", 768, 4080),
    ("1.21.1", 767, 3955),
    ("1.21", 767, 3953),
    ("1.20.6", 766, 3839),
    ("1.20.5", 766, 3837),
    ("1.20.4", 765, 3700),
    ("1.20.3", 765, 3698),
    ("1.20.2", 764, 3578),
    ("1.20.1", 763, 3465),
    ("1.20", 763, 3463),
    ("1.19.4", 762, 3337),
    ("1.19.3", 761, 3218),
    ("1.19.2", 760, 3120),
    ("1.19.1", 760, 3117),
    ("1.19", 759, 3105),
    ("1.18.2", 758, 2975),
    ("1.18.1", 757, 2865),
    ("1.18", 757, 2860),
    ("1.17.1", 756, 2730),
    ("1.17", 755, 2724),
    ("1.16.5", 754, 2586),
    ("1.16.4", 754, 2584),
    ("1.16.3", 753, 2580),
    ("1.16.2", 751, 2578),
    ("1.16.1", 736, 2567),
    ("1.16", 735, 2566),
    ("1.15.2", 578, 2230),
    ("1.15.1", 575, 2227),
    ("1.15", 573, 2225),
    ("1.14.4", 498, 1976),
    ("1.14.3", 490, 1968),
    ("1.14.2", 485, 1963),
    ("1.14.1", 480, 1957),
    ("1.14", 477, 1952),
    ("1.13.2", 404, 1631),
    ("1.13.1", 401, 1628),
    ("1.13", 393, 1519),
    ("1.12.2", 340, 1343),
    ("1.12.1", 338, 1241),
    ("1.12", 335, 1139),
    ("1.11.2", 316, 922),
    ("1.11.1", 316, 921),
    ("1.11", 315, 819),
    ("1.10.2", 210, 512),
    ("1.10.1", 210, 511),
    ("1.10", 210, 510),
    ("1.9.4", 110, 184),
    ("1.9.3", 110, 183),
    ("1.9.2", 109, 176),
    ("1.9.1", 108, 175),
    ("1.9", 107, 169),
    ("1.8.9", 47, 95),
    ("1.8.8", 47, 94),
    ("1.8.7", 47, 93),
    ("1.8.6", 47, 92),
    ("1.8.5", 47, 91),
    ("1.8.4", 47, 90),
    ("1.8.3", 47, 89),
    ("1.8.2", 47, 88),
    ("1.8.1", 47, 80),
    ("1.8", 47, 74),
    ("1.7.10", 5, 18),
    ("1.7.9", 5, 13),
    ("1.7.8", 5, 12),
    ("1.7.7", 5, 11),
    ("1.7.6", 5, 10),
    ("1.7.5", 4, 7),
    ("1.7.4", 4, 6),
    ("1.7.2", 4, -4),
    ("1.6.4", 78, -12),
    ("1.6.2", 74, -13),
    ("1.6.1", 73, -14),
    ("1.5.2", 61, -31),
    ("1.5.1", 60, -32),
    ("1.4.7", 51, -42),
    ("1.4.5", 49, -45),
    ("1.4.6", 51, -43),
    ("1.4.4", 49, -46),
    ("1.4.2", 47, -48),
    ("1.3.2", 39, -54),
    ("1.3.1", 39, -55),
    ("1.2.5", 29, -67),
    ("1.2.4", 29, -68),
    ("1.2.3", 28, -69),
    ("1.2.2", 28, -70),
    ("1.2.1", 28, -71),
    ("1.1", 23, -78),
];

const META_CACHE_TTL: Duration = Duration::from_secs(3600);
const META_ERROR_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Deserialize)]
struct ProtocolEntry {
    #[serde(rename = "minecraftVersion")]
    minecraft_version: String,
    version: i64,
    #[serde(rename = "dataVersion", default)]
    data_version: i64,
}

struct CachedMeta {
    fetched_at: Instant,
    ttl: Duration,
    map: Arc<HashMap<String, VersionMeta>>,
}

static META_CACHE: LazyLock<RwLock<Option<CachedMeta>>> = LazyLock::new(|| RwLock::new(None));

fn fallback_map() -> HashMap<String, VersionMeta> {
    FALLBACK_PROTOCOLS
        .iter()
        .map(|(v, p, d)| {
            (
                v.to_string(),
                VersionMeta {
                    protocol: *p,
                    data_version: *d,
                },
            )
        })
        .collect()
}

async fn fetch_remote_map() -> anyhow::Result<HashMap<String, VersionMeta>> {
    let client = reqwest::Client::builder()
        .user_agent("ChiPanel/0.1.0 (https://github.com/chiserv/chipanel)")
        .timeout(Duration::from_secs(10))
        .build()?;

    let entries = client
        .get(PROTOCOL_VERSIONS_URL)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<ProtocolEntry>>()
        .await?;

    Ok(entries
        .into_iter()
        .map(|e| {
            (
                e.minecraft_version,
                VersionMeta {
                    protocol: e.version,
                    data_version: e.data_version,
                },
            )
        })
        .collect())
}

/// Resolves the protocol + data version for a Minecraft version id, caching the
/// full PrismarineJS table in memory and falling back to the built-in release
/// list when the fetch fails (offline).
pub async fn version_meta(version: &str) -> Option<VersionMeta> {
    // Fast path: a fresh cached table.
    {
        let cache = META_CACHE.read().await;
        if let Some(entry) = cache.as_ref() {
            if entry.fetched_at.elapsed() < entry.ttl {
                if let Some(meta) = entry.map.get(version) {
                    return Some(*meta);
                }
                // Table is fresh but doesn't know this id (e.g. a just-published
                // snapshot). Try the embedded release list before giving up.
                return fallback_map().get(version).copied();
            }
        }
    }

    // Refetch the live table (or fall back to the embedded list offline).
    let (map, ttl) = match fetch_remote_map().await {
        Ok(map) => (map, META_CACHE_TTL),
        Err(e) => {
            warn!("Could not fetch PrismarineJS protocolVersions ({}); serving built-in release table", e);
            (fallback_map(), META_ERROR_TTL)
        }
    };

    let map = Arc::new(map);
    let result = map.get(version).copied();
    {
        let mut cache = META_CACHE.write().await;
        *cache = Some(CachedMeta {
            fetched_at: Instant::now(),
            ttl,
            map,
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_has_recent_and_legacy_versions() {
        let map = fallback_map();
        assert_eq!(map.get("26.2").map(|m| m.protocol), Some(776));
        assert_eq!(map.get("1.21.4").map(|m| m.protocol), Some(769));
        assert_eq!(map.get("1.12.2").map(|m| m.protocol), Some(340));
        // Pre-1.9 releases carry a placeholder (negative) data version.
        assert_eq!(map.get("1.8").map(|m| m.data_version), Some(74));
    }

    #[test]
    fn parses_prismarine_data_version_camel_case() {
        // Regression guard: the PrismarineJS JSON uses `dataVersion` (camelCase),
        // which must map into `data_version` — a missing rename silently defaults
        // it to 0 and disables the world-compatibility warning entirely.
        let json = r#"[
            {"minecraftVersion": "26.2", "version": 776, "dataVersion": 4903, "usesNetty": true, "majorVersion": "26.2", "releaseType": "release"},
            {"minecraftVersion": "1.21.4", "version": 769, "dataVersion": 4189, "usesNetty": true, "majorVersion": "1.21.4", "releaseType": "release"}
        ]"#;
        let entries: Vec<ProtocolEntry> = serde_json::from_str(json).unwrap();
        let map: HashMap<String, VersionMeta> = entries
            .into_iter()
            .map(|e| {
                (
                    e.minecraft_version,
                    VersionMeta {
                        protocol: e.version,
                        data_version: e.data_version,
                    },
                )
            })
            .collect();
        assert_eq!(map.get("26.2").unwrap().data_version, 4903);
        assert_eq!(map.get("1.21.4").unwrap().protocol, 769);
    }
}
