use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineTypeInfo {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub supports_plugins: bool,
    pub supports_mods: bool,
    pub icon: String,
    pub recommended_versions: Vec<String>,
}

pub fn get_available_engines() -> Vec<EngineTypeInfo> {
    vec![
        EngineTypeInfo {
            id: "PURPUR".to_string(),
            name: "Purpur".to_string(),
            category: "Optimized Paper Fork".to_string(),
            description: "Ultra-optimized Paper fork with extra performance tweaks & customizable gameplay mechanics.".to_string(),
            supports_plugins: true,
            supports_mods: false,
            icon: "zap".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.4".to_string(), "1.21.1".to_string(), "1.20.4".to_string(), "1.20.1".to_string()],
        },
        EngineTypeInfo {
            id: "PAPER".to_string(),
            name: "PaperMC".to_string(),
            category: "Bukkit/Spigot Fork".to_string(),
            description: "Industry-standard high performance Spigot fork with critical bug fixes and exploit prevention.".to_string(),
            supports_plugins: true,
            supports_mods: false,
            icon: "file-text".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.4".to_string(), "1.21.1".to_string(), "1.20.4".to_string(), "1.20.1".to_string()],
        },
        EngineTypeInfo {
            id: "FABRIC".to_string(),
            name: "Fabric".to_string(),
            category: "Modded".to_string(),
            description: "Lightweight, modular modding toolchain with fast updates and modern mod ecosystem.".to_string(),
            supports_plugins: false,
            supports_mods: true,
            icon: "cpu".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.4".to_string(), "1.21.1".to_string(), "1.20.4".to_string(), "1.20.1".to_string()],
        },
        EngineTypeInfo {
            id: "FORGE".to_string(),
            name: "Minecraft Forge".to_string(),
            category: "Modded".to_string(),
            description: "The classic Minecraft modding platform supporting thousands of complex mods and modpacks.".to_string(),
            supports_plugins: false,
            supports_mods: true,
            icon: "hammer".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.1".to_string(), "1.20.1".to_string(), "1.19.2".to_string(), "1.18.2".to_string(), "1.16.5".to_string(), "1.12.2".to_string()],
        },
        EngineTypeInfo {
            id: "NEOFORGE".to_string(),
            name: "NeoForge".to_string(),
            category: "Modded".to_string(),
            description: "Modern community-driven fork of Minecraft Forge for 1.20.2+ with improved APIs and performance.".to_string(),
            supports_plugins: false,
            supports_mods: true,
            icon: "shield-alert".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.4".to_string(), "1.21.1".to_string(), "1.20.4".to_string()],
        },
        EngineTypeInfo {
            id: "SPIGOT".to_string(),
            name: "Spigot".to_string(),
            category: "Bukkit/Spigot".to_string(),
            description: "Classic plugin-compatible server engine built on Bukkit API.".to_string(),
            supports_plugins: true,
            supports_mods: false,
            icon: "layers".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.4".to_string(), "1.21.1".to_string(), "1.20.4".to_string()],
        },
        EngineTypeInfo {
            id: "QUILT".to_string(),
            name: "Quilt".to_string(),
            category: "Modded".to_string(),
            description: "Modern, open-source mod loader compatible with most Fabric mods and enhanced tooling.".to_string(),
            supports_plugins: false,
            supports_mods: true,
            icon: "feather".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.1".to_string(), "1.20.4".to_string()],
        },
        EngineTypeInfo {
            id: "VANILLA".to_string(),
            name: "Vanilla".to_string(),
            category: "Standard".to_string(),
            description: "Official unmodified Minecraft server software directly from Mojang.".to_string(),
            supports_plugins: false,
            supports_mods: false,
            icon: "box".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.4".to_string(), "1.21.1".to_string(), "1.20.4".to_string()],
        },
        EngineTypeInfo {
            id: "FOLIA".to_string(),
            name: "Folia".to_string(),
            category: "Multi-Threaded".to_string(),
            description: "Regionized multithreading Paper fork designed for high player counts across multiple CPU cores.".to_string(),
            supports_plugins: true,
            supports_mods: false,
            icon: "activity".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.4".to_string(), "1.20.4".to_string()],
        },
        EngineTypeInfo {
            id: "MOHIST".to_string(),
            name: "Mohist".to_string(),
            category: "Hybrid (Mods + Plugins)".to_string(),
            description: "Forge + Paper hybrid server allowing running Forge mods alongside Bukkit/Paper plugins.".to_string(),
            supports_plugins: true,
            supports_mods: true,
            icon: "crosshair".to_string(),
            recommended_versions: vec!["1.20.1".to_string(), "1.19.2".to_string(), "1.16.5".to_string(), "1.12.2".to_string()],
        },
        EngineTypeInfo {
            id: "ARCLIGHT".to_string(),
            name: "Arclight".to_string(),
            category: "Hybrid (Mods + Plugins)".to_string(),
            description: "Mixes Forge/NeoForge or Fabric with Bukkit/Mixins for combined mod & plugin support.".to_string(),
            supports_plugins: true,
            supports_mods: true,
            icon: "sparkles".to_string(),
            recommended_versions: vec!["26.2".to_string(), "1.21.1".to_string(), "1.20.4".to_string(), "1.20.1".to_string()],
        },
    ]
}

/// Every official Minecraft release, newest first, mirrored from Mojang's version
/// manifest (piston-meta.mojang.com). This is the offline fallback served when the
/// live manifest cannot be fetched — snapshots are too volatile to hardcode, so the
/// fallback simply has none.
const FALLBACK_RELEASES: &[&str] = &[
    "26.2", "26.1.2", "26.1.1", "26.1", "1.21.11", "1.21.10",
    "1.21.9", "1.21.8", "1.21.7", "1.21.6", "1.21.5", "1.21.4",
    "1.21.3", "1.21.2", "1.21.1", "1.21", "1.20.6", "1.20.5",
    "1.20.4", "1.20.3", "1.20.2", "1.20.1", "1.20", "1.19.4",
    "1.19.3", "1.19.2", "1.19.1", "1.19", "1.18.2", "1.18.1",
    "1.18", "1.17.1", "1.17", "1.16.5", "1.16.4", "1.16.3",
    "1.16.2", "1.16.1", "1.16", "1.15.2", "1.15.1", "1.15",
    "1.14.4", "1.14.3", "1.14.2", "1.14.1", "1.14", "1.13.2",
    "1.13.1", "1.13", "1.12.2", "1.12.1", "1.12", "1.11.2",
    "1.11.1", "1.11", "1.10.2", "1.10.1", "1.10", "1.9.4",
    "1.9.3", "1.9.2", "1.9.1", "1.9", "1.8.9", "1.8.8",
    "1.8.7", "1.8.6", "1.8.5", "1.8.4", "1.8.3", "1.8.2",
    "1.8.1", "1.8", "1.7.10", "1.7.9", "1.7.8", "1.7.7",
    "1.7.6", "1.7.5", "1.7.4", "1.7.3", "1.7.2", "1.6.4",
    "1.6.2", "1.6.1", "1.5.2", "1.5.1", "1.4.7", "1.4.5",
    "1.4.6", "1.4.4", "1.4.2", "1.3.2", "1.3.1", "1.2.5",
    "1.2.4", "1.2.3", "1.2.2", "1.2.1", "1.1", "1.0",
];

/// The official Mojang launcher manifest listing every release and snapshot ever published.
const MOJANG_MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
/// Snapshots drop roughly weekly; an hour keeps the panel current without hammering Mojang.
/// pub(crate): the background version watcher stores its fetches with the same TTL.
pub(crate) const CATALOG_CACHE_TTL: Duration = Duration::from_secs(3600);
/// When Mojang is unreachable the fallback is cached only briefly so the panel
/// recovers quickly and a down network can't turn every request into a 10s timeout.
const CATALOG_ERROR_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Clone)]
pub struct VersionCatalog {
    /// Official releases, newest first.
    pub releases: Vec<String>,
    /// Official snapshots / pre-releases / release candidates, newest first.
    pub snapshots: Vec<String>,
}

impl VersionCatalog {
    /// itzg/minecraft-server keywords first, then every release, then every snapshot.
    pub fn all_versions(&self) -> Vec<String> {
        let mut out = Vec::with_capacity(self.releases.len() + self.snapshots.len() + 2);
        out.push("LATEST".to_string());
        out.push("SNAPSHOT".to_string());
        out.extend(self.releases.iter().cloned());
        out.extend(self.snapshots.iter().cloned());
        out
    }

    /// Case-insensitive lookup returning the canonical (manifest-exact) version id.
    /// itzg/minecraft-server resolves VERSION case-sensitively against Mojang's
    /// manifest, so "24W14A" from the panel must be written down as "24w14a".
    pub fn canonical(&self, version: &str) -> Option<String> {
        self.all_versions()
            .into_iter()
            .find(|v| v.eq_ignore_ascii_case(version))
    }
}

pub fn fallback_version_catalog() -> VersionCatalog {
    VersionCatalog {
        releases: FALLBACK_RELEASES.iter().map(|s| s.to_string()).collect(),
        snapshots: Vec::new(),
    }
}

struct CachedCatalog {
    fetched_at: Instant,
    ttl: Duration,
    catalog: Arc<VersionCatalog>,
}

static CATALOG_CACHE: LazyLock<RwLock<Option<CachedCatalog>>> =
    LazyLock::new(|| RwLock::new(None));

/// Returns the live version catalog (Mojang manifest, cached), falling back to the
/// built-in release list when the network or Mojang is unavailable.
pub async fn fetch_version_catalog() -> Arc<VersionCatalog> {
    {
        let cache = CATALOG_CACHE.read().await;
        if let Some(entry) = cache.as_ref() {
            if entry.fetched_at.elapsed() < entry.ttl {
                return Arc::clone(&entry.catalog);
            }
        }
    }

    let (catalog, ttl) = match fetch_remote_manifest().await.and_then(|m| catalog_from_manifest(&m)) {
        Ok(catalog) => (catalog, CATALOG_CACHE_TTL),
        Err(e) => {
            warn!("Could not fetch Mojang version manifest ({}); serving built-in release list", e);
            (fallback_version_catalog(), CATALOG_ERROR_TTL)
        }
    };

    store_catalog(catalog, ttl).await
}

/// Writes a freshly built catalog into the shared cache. Called by the on-demand
/// path above and by the background version watcher, so both never double-fetch.
pub(crate) async fn store_catalog(catalog: VersionCatalog, ttl: Duration) -> Arc<VersionCatalog> {
    let catalog = Arc::new(catalog);
    let mut cache = CATALOG_CACHE.write().await;
    *cache = Some(CachedCatalog {
        fetched_at: Instant::now(),
        ttl,
        catalog: Arc::clone(&catalog),
    });
    catalog
}

#[derive(Debug, Deserialize)]
pub(crate) struct MojangManifest {
    #[serde(default)]
    pub(crate) latest: Option<MojangLatest>,
    pub(crate) versions: Vec<MojangManifestEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MojangLatest {
    pub(crate) release: Option<String>,
    pub(crate) snapshot: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MojangManifestEntry {
    pub(crate) id: String,
    #[serde(rename = "type")]
    pub(crate) kind: String,
    /// ISO-8601 publication timestamp, used by the version watcher.
    #[serde(rename = "releaseTime", default)]
    pub(crate) release_time: String,
}

pub(crate) async fn fetch_remote_manifest() -> anyhow::Result<MojangManifest> {
    let client = reqwest::Client::builder()
        .user_agent("ChiPanel/0.1.0 (https://github.com/chiserv/chipanel)")
        .timeout(Duration::from_secs(10))
        .build()?;

    let manifest = client
        .get(MOJANG_MANIFEST_URL)
        .send()
        .await?
        .error_for_status()?
        .json::<MojangManifest>()
        .await?;

    Ok(manifest)
}

pub(crate) fn catalog_from_manifest(manifest: &MojangManifest) -> anyhow::Result<VersionCatalog> {
    let mut releases = Vec::new();
    let mut snapshots = Vec::new();
    for entry in &manifest.versions {
        // Anything outside the safe charset (e.g. old pre-releases with spaces in
        // their id) can never be written into the quadlet — drop it at ingestion.
        if !is_version_charset_safe(&entry.id) {
            continue;
        }
        match entry.kind.as_str() {
            "release" => releases.push(entry.id.clone()),
            "snapshot" => snapshots.push(entry.id.clone()),
            _ => {} // old_alpha / old_beta are not offered
        }
    }

    if releases.is_empty() {
        anyhow::bail!("Mojang manifest contained no usable releases");
    }

    Ok(VersionCatalog { releases, snapshots })
}

/// Version strings are written verbatim into the systemd Quadlet file, so anything
/// that could carry an extra systemd directive (newline, space, `=`…) is rejected —
/// this holds even if a future Mojang manifest entry is hostile or malformed.
pub fn is_version_charset_safe(version: &str) -> bool {
    !version.is_empty()
        && version.len() <= 64
        && version
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// Validates a user-supplied version against the live catalog (or fallback) and
/// returns the canonical casing to write into the quadlet. `None` = rejected.
pub async fn resolve_minecraft_version(version: &str) -> Option<String> {
    if !is_version_charset_safe(version) {
        return None;
    }
    fetch_version_catalog().await.canonical(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_version_with_injected_systemd_directive() {
        let payload = "LATEST\nExecStartPre=/bin/sh -c 'id > /tmp/pwn'";
        assert!(!is_version_charset_safe(payload));
        assert!(fallback_version_catalog().canonical(payload).is_none());
        // Uppercasing must not let it through either.
        assert!(fallback_version_catalog()
            .canonical(&payload.to_uppercase())
            .is_none());
    }

    #[test]
    fn version_charset_guard() {
        assert!(is_version_charset_safe("24w14a"));
        assert!(is_version_charset_safe("1.21.4-rc1"));
        assert!(!is_version_charset_safe(""));
        // Spaces (old 1.14 pre-release ids) can never go into a quadlet line.
        assert!(!is_version_charset_safe("1.14 Pre-Release 1"));
        assert!(!is_version_charset_safe(&"a".repeat(65)));
    }

    #[test]
    fn accepts_known_versions() {
        let catalog = fallback_version_catalog();
        assert_eq!(catalog.canonical("LATEST"), Some("LATEST".to_string()));
        assert_eq!(catalog.canonical("SNAPSHOT"), Some("SNAPSHOT".to_string()));
        assert_eq!(catalog.canonical("1.21.4"), Some("1.21.4".to_string()));
        // Case-insensitive input resolves to the canonical manifest casing.
        assert_eq!(catalog.canonical("latest"), Some("LATEST".to_string()));
        // 26.3 is still a snapshot line, not a release - it must not validate.
        assert!(catalog.canonical("26.3").is_none());
    }

    #[test]
    fn fallback_mirrors_mojang_release_order() {
        let catalog = fallback_version_catalog();
        // Snapshot of the official manifest (102 releases, 26.2 newest, 1.0 oldest).
        assert_eq!(catalog.releases.len(), 102);
        assert_eq!(catalog.releases.first().unwrap(), "26.2");
        assert_eq!(catalog.releases.last().unwrap(), "1.0");
        // LATEST / SNAPSHOT keywords head the combined list.
        let all = catalog.all_versions();
        assert_eq!(all[0], "LATEST");
        assert_eq!(all[1], "SNAPSHOT");
    }
}
