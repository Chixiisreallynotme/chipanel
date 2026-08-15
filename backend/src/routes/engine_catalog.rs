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
    pub min_version: Option<String>,
    pub max_version: Option<String>,
    pub recommended_ram: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub stability: String,
    pub supports_tools: bool,
}

pub fn get_available_engines() -> Vec<EngineTypeInfo> {
    vec![
        EngineTypeInfo {
            id: "PURPUR".to_string(),
            name: "Purpur".to_string(),
            category: "Optimisation & Plugins".to_string(),
            description: "Fourche ultra-optimisée de Paper offrant les meilleures performances, une configuration poussée et des mécaniques personnalisables.".to_string(),
            supports_plugins: true,
            supports_mods: false,
            icon: "zap".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.4".to_string(), "1.21.1".to_string(), "1.20.4".to_string(), "1.20.1".to_string()],
            min_version: Some("1.14.4".to_string()),
            max_version: None,
            recommended_ram: "2 à 4 Go".to_string(),
            pros: vec![
                "Performances exceptionnelles & TPS stable".to_string(),
                "Hautement personnalisable (gameplay, redstone, chunks)".to_string(),
                "100% compatible plugins Bukkit/Spigot/Paper".to_string(),
                "Outils maison synchronisés (Spark, Chunky, LuckPerms)".to_string(),
            ],
            cons: vec!["Incompatible avec les mods Forge/Fabric".to_string()],
            stability: "Recommandé Production".to_string(),
            supports_tools: true,
        },
        EngineTypeInfo {
            id: "PAPER".to_string(),
            name: "PaperMC".to_string(),
            category: "Optimisation & Plugins".to_string(),
            description: "Le standard de l'industrie pour les serveurs Minecraft avec plugins. Corrige les bugs de Vanilla et prévient les failles d'exploit.".to_string(),
            supports_plugins: true,
            supports_mods: false,
            icon: "file-text".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.4".to_string(), "1.21.1".to_string(), "1.20.4".to_string(), "1.20.1".to_string()],
            min_version: Some("1.8.8".to_string()),
            max_version: None,
            recommended_ram: "2 à 4 Go".to_string(),
            pros: vec![
                "Stabilité et fiabilité éprouvées".to_string(),
                "Écosystème de plugins gigantesque".to_string(),
                "Excellente gestion de mémoire asynchrone".to_string(),
                "Outils maison synchronisés (Spark, Chunky, LuckPerms)".to_string(),
            ],
            cons: vec!["Incompatible avec les mods Forge/Fabric".to_string()],
            stability: "Standard Industrie".to_string(),
            supports_tools: true,
        },
        EngineTypeInfo {
            id: "FABRIC".to_string(),
            name: "Fabric".to_string(),
            category: "Moddé Moderne".to_string(),
            description: "Chargeur de mods moderne, léger et ultra-rapide avec des mises à jour quasi instantanées lors des sorties de nouvelles versions Minecraft.".to_string(),
            supports_plugins: false,
            supports_mods: true,
            icon: "cpu".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.4".to_string(), "1.21.1".to_string(), "1.20.4".to_string(), "1.20.1".to_string()],
            min_version: Some("1.14".to_string()),
            max_version: None,
            recommended_ram: "2 à 6 Go".to_string(),
            pros: vec![
                "Démarrage ultra-rapide et faible empreinte mémoire".to_string(),
                "Mods d'optimisation réputés (Lithium, FerriteCore)".to_string(),
                "Outils maison synchronisés (Spark, Chunky, LuckPerms, fabric-api)".to_string(),
            ],
            cons: vec!["Incompatible avec les plugins Bukkit/Paper natifs".to_string()],
            stability: "Moddé Recommandé".to_string(),
            supports_tools: true,
        },
        EngineTypeInfo {
            id: "FORGE".to_string(),
            name: "Minecraft Forge".to_string(),
            category: "Moddé Classique".to_string(),
            description: "La plateforme de modding historique de référence, indispensable pour les grands modpacks d'aventure et techniques complexes.".to_string(),
            supports_plugins: false,
            supports_mods: true,
            icon: "hammer".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.1".to_string(), "1.20.1".to_string(), "1.19.2".to_string(), "1.18.2".to_string(), "1.16.5".to_string(), "1.12.2".to_string()],
            min_version: Some("1.7.10".to_string()),
            max_version: None,
            recommended_ram: "4 à 8 Go+".to_string(),
            pros: vec![
                "Immense catalogue de mods historiques (1.7.10 - 1.20.1)".to_string(),
                "Compatible avec les grands modpacks CurseForge/Modrinth".to_string(),
                "Outils maison synchronisés (Spark, Chunky, LuckPerms)".to_string(),
            ],
            cons: vec![
                "Consommation de RAM et temps de chargement plus élevés".to_string(),
                "Incompatible avec les plugins Bukkit".to_string(),
            ],
            stability: "Modpack Établi".to_string(),
            supports_tools: true,
        },
        EngineTypeInfo {
            id: "NEOFORGE".to_string(),
            name: "NeoForge".to_string(),
            category: "Moddé Moderne".to_string(),
            description: "Fork communautaire moderne de Forge pour Minecraft 1.20.2+, avec une architecture refondue, de meilleures performances et des API modernes.".to_string(),
            supports_plugins: false,
            supports_mods: true,
            icon: "shield-alert".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.4".to_string(), "1.21.1".to_string(), "1.20.4".to_string()],
            min_version: Some("1.20.2".to_string()),
            max_version: None,
            recommended_ram: "4 à 8 Go+".to_string(),
            pros: vec![
                "Architecture moderne plus rapide et propre que Forge".to_string(),
                "Plateforme principale pour les mods récents (1.20.2+)".to_string(),
                "Outils maison synchronisés (Spark, Chunky, LuckPerms)".to_string(),
            ],
            cons: vec![
                "Uniquement disponible sur Minecraft 1.20.2 et supérieur".to_string(),
                "Incompatible avec les plugins Bukkit".to_string(),
            ],
            stability: "Nouvelle Génération".to_string(),
            supports_tools: true,
        },
        EngineTypeInfo {
            id: "SPIGOT".to_string(),
            name: "Spigot".to_string(),
            category: "Bukkit / Plugins".to_string(),
            description: "Moteur historique basé sur Bukkit, offrant une compatibilité étendue avec les anciens plugins.".to_string(),
            supports_plugins: true,
            supports_mods: false,
            icon: "layers".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.4".to_string(), "1.21.1".to_string(), "1.20.4".to_string()],
            min_version: Some("1.8".to_string()),
            max_version: None,
            recommended_ram: "2 à 4 Go".to_string(),
            pros: vec![
                "Grande compatibilité avec les anciens plugins Bukkit".to_string(),
                "Outils maison synchronisés (Spark, Chunky, LuckPerms)".to_string(),
            ],
            cons: vec![
                "Moins de correctifs et d'optimisations que Paper/Purpur".to_string(),
                "Incompatible avec les mods".to_string(),
            ],
            stability: "Legacy Stable".to_string(),
            supports_tools: true,
        },
        EngineTypeInfo {
            id: "QUILT".to_string(),
            name: "Quilt".to_string(),
            category: "Moddé Moderne".to_string(),
            description: "Projet open-source moderne dérivé de Fabric, compatible avec la quasi-totalité des mods Fabric tout en offrant des fonctionnalités supplémentaires.".to_string(),
            supports_plugins: false,
            supports_mods: true,
            icon: "feather".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.1".to_string(), "1.20.4".to_string()],
            min_version: Some("1.14".to_string()),
            max_version: None,
            recommended_ram: "2 à 6 Go".to_string(),
            pros: vec![
                "Compatible avec la majorité de l'écosystème Fabric".to_string(),
                "Outils modernes de gestion de dépendances".to_string(),
                "Outils maison synchronisés".to_string(),
            ],
            cons: vec!["Incompatible avec les plugins Bukkit".to_string()],
            stability: "Moddé Alternatif".to_string(),
            supports_tools: true,
        },
        EngineTypeInfo {
            id: "VANILLA".to_string(),
            name: "Vanilla".to_string(),
            category: "Officiel Mojang".to_string(),
            description: "Le logiciel serveur officiel pur produit par Mojang, garantissant une fidélité 100% absolue aux mécaniques du jeu original.".to_string(),
            supports_plugins: false,
            supports_mods: false,
            icon: "box".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.4".to_string(), "1.21.1".to_string(), "1.20.4".to_string()],
            min_version: Some("1.0".to_string()),
            max_version: None,
            recommended_ram: "2 à 4 Go".to_string(),
            pros: vec![
                "100% conforme au gameplay et redstone Mojang sans modification".to_string(),
                "Supporte tous les snapshots et pre-releases sans attente".to_string(),
            ],
            cons: vec![
                "Aucun plugin ni mod supporté".to_string(),
                "Aucune optimisation de TPS ni d'outils d'administration".to_string(),
            ],
            stability: "Officiel".to_string(),
            supports_tools: false,
        },
        EngineTypeInfo {
            id: "FOLIA".to_string(),
            name: "Folia".to_string(),
            category: "SMP Multithread".to_string(),
            description: "Fourche Paper révolutionnaire avec multithreading régionalisé par chunks, conçu pour accueillir des centaines de joueurs simultanés sur plusieurs cœurs CPU.".to_string(),
            supports_plugins: true,
            supports_mods: false,
            icon: "activity".to_string(),
            recommended_versions: vec!["LATEST".to_string(), "26.2".to_string(), "1.21.4".to_string(), "1.20.4".to_string()],
            min_version: Some("1.19.4".to_string()),
            max_version: None,
            recommended_ram: "6 à 12 Go+".to_string(),
            pros: vec![
                "Multi-threading par région : TPS parfait à 100+ joueurs".to_string(),
                "Exploite pleinement les processeurs multi-cœurs modernes".to_string(),
                "Outils maison synchronisés".to_string(),
            ],
            cons: vec![
                "Uniquement disponible sur 1.19.4+".to_string(),
                "Nécessite des plugins compatibles Folia (les plugins Bukkit standards crasheront)".to_string(),
                "Incompatible avec les mods".to_string(),
            ],
            stability: "Haute Concurrence".to_string(),
            supports_tools: true,
        },
        EngineTypeInfo {
            id: "MOHIST".to_string(),
            name: "Mohist".to_string(),
            category: "Hybride (Mods + Plugins)".to_string(),
            description: "Serveur hybride permettant d'exécuter simultanément des mods Forge et des plugins Bukkit/Paper sur un même serveur.".to_string(),
            supports_plugins: true,
            supports_mods: true,
            icon: "crosshair".to_string(),
            recommended_versions: vec!["1.20.1".to_string(), "1.19.2".to_string(), "1.16.5".to_string(), "1.12.2".to_string()],
            min_version: Some("1.12.2".to_string()),
            max_version: Some("1.20.1".to_string()),
            recommended_ram: "4 à 8 Go".to_string(),
            pros: vec![
                "Combine mods Forge et plugins Bukkit/Paper".to_string(),
                "Idéal pour administrer un modpack avec LuckPerms/WorldEdit Bukkit".to_string(),
            ],
            cons: vec![
                "Limité aux versions 1.12.2 jusqu'à 1.20.1".to_string(),
                "Possibles incompatibilités entre certains mods et plugins complexes".to_string(),
            ],
            stability: "Hybride Spécialisé".to_string(),
            supports_tools: true,
        },
        EngineTypeInfo {
            id: "ARCLIGHT".to_string(),
            name: "Arclight".to_string(),
            category: "Hybride Moderne".to_string(),
            description: "Solution hybride moderne basée sur Mixins permettant de combiner mods Forge/NeoForge/Fabric et plugins Bukkit sur les versions récentes.".to_string(),
            supports_plugins: true,
            supports_mods: true,
            icon: "sparkles".to_string(),
            recommended_versions: vec!["26.2".to_string(), "1.21.1".to_string(), "1.20.4".to_string(), "1.20.1".to_string()],
            min_version: Some("1.16.5".to_string()),
            max_version: None,
            recommended_ram: "4 à 8 Go".to_string(),
            pros: vec![
                "Supporte les versions modernes (1.16.5 à 1.21+)".to_string(),
                "Exécute mods et plugins ensemble avec une bonne stabilité".to_string(),
            ],
            cons: vec![
                "Expérimental : nécessite de tester la compatibilité du pack d'addons".to_string(),
            ],
            stability: "Hybride Moderne".to_string(),
            supports_tools: true,
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

/// Checks whether a Minecraft version is supported by a specific engine,
/// respecting `min_version` and `max_version` boundaries based on the official
/// chronological release list in `VersionCatalog`.
pub fn is_engine_version_supported(
    engine: &EngineTypeInfo,
    version: &str,
    catalog: &VersionCatalog,
) -> bool {
    let ver_upper = version.trim().to_uppercase();
    if ver_upper == "LATEST" {
        // LATEST is the newest release. If the engine has a max_version limit
        // (like Mohist max 1.20.1), LATEST resolves beyond that limit.
        if let Some(ref max_v) = engine.max_version {
            if let Some(first_rel) = catalog.releases.first() {
                if !first_rel.eq_ignore_ascii_case(max_v) {
                    return false;
                }
            }
        }
        return true;
    }
    if ver_upper == "SNAPSHOT" {
        // Snapshots are bleeding edge. Valid only if engine has no max_version.
        return engine.max_version.is_none();
    }

    // Check if the version is in releases list
    if let Some(target_idx) = catalog.releases.iter().position(|r| r.eq_ignore_ascii_case(version)) {
        if let Some(ref min_v) = engine.min_version {
            if let Some(min_idx) = catalog.releases.iter().position(|r| r.eq_ignore_ascii_case(min_v)) {
                // Since catalog.releases is sorted newest first (index 0 is newest),
                // target_idx > min_idx means target is older than min_version.
                if target_idx > min_idx {
                    return false;
                }
            }
        }
        if let Some(ref max_v) = engine.max_version {
            if let Some(max_idx) = catalog.releases.iter().position(|r| r.eq_ignore_ascii_case(max_v)) {
                // target_idx < max_idx means target is newer than max_version.
                if target_idx < max_idx {
                    return false;
                }
            }
        }
        return true;
    }

    // If it's a snapshot version (in catalog.snapshots)
    if catalog.snapshots.iter().any(|s| s.eq_ignore_ascii_case(version)) {
        return engine.max_version.is_none();
    }

    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderVersionItem {
    pub version: String,
    pub label: String,
    pub is_stable: bool,
    pub is_recommended: bool,
}

#[derive(Debug, Deserialize)]
pub struct LoaderVersionsQuery {
    pub engine_type: String,
    #[serde(default)]
    pub game_version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoaderVersionsResponse {
    pub engine_type: String,
    pub game_version: String,
    pub env_variable: String,
    pub default_version: String,
    pub versions: Vec<LoaderVersionItem>,
}

pub async fn fetch_engine_loader_versions(engine_type: &str, game_version: Option<&str>) -> LoaderVersionsResponse {
    let engine_upper = engine_type.trim().to_uppercase();
    let gv = game_version.unwrap_or("").trim();

    match engine_upper.as_str() {
        "FABRIC" => fetch_fabric_loader_versions(gv).await,
        "QUILT" => fetch_quilt_loader_versions(gv).await,
        "PAPER" => fetch_paper_builds(gv).await,
        "PURPUR" => fetch_purpur_builds(gv).await,
        "FORGE" => fetch_forge_versions(gv).await,
        "NEOFORGE" => fetch_neoforge_versions(gv).await,
        _ => LoaderVersionsResponse {
            engine_type: engine_upper,
            game_version: gv.to_string(),
            env_variable: "".to_string(),
            default_version: "LATEST".to_string(),
            versions: vec![
                LoaderVersionItem {
                    version: "LATEST".to_string(),
                    label: "Dernière version recommandée (LATEST)".to_string(),
                    is_stable: true,
                    is_recommended: true,
                }
            ],
        },
    }
}

#[derive(Debug, Deserialize)]
struct FabricLoaderEntry {
    version: String,
    #[serde(default)]
    stable: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct FabricLoaderWrapper {
    loader: FabricLoaderEntry,
}

async fn fetch_fabric_loader_versions(game_version: &str) -> LoaderVersionsResponse {
    let client = reqwest::Client::builder()
        .user_agent("ChiPanel/0.1.0 (https://github.com/chiserv/chipanel)")
        .timeout(Duration::from_secs(5))
        .build();

    let mut versions = Vec::new();

    if let Ok(c) = client {
        let url = if !game_version.is_empty() && !game_version.eq_ignore_ascii_case("LATEST") && !game_version.eq_ignore_ascii_case("SNAPSHOT") {
            format!("https://meta.fabricmc.net/v2/versions/loader/{}", game_version)
        } else {
            "https://meta.fabricmc.net/v2/versions/loader".to_string()
        };

        if let Ok(resp) = c.get(&url).send().await {
            if resp.status().is_success() {
                if url.contains("/loader/") {
                    if let Ok(entries) = resp.json::<Vec<FabricLoaderWrapper>>().await {
                        for (idx, item) in entries.into_iter().enumerate() {
                            let is_stable = item.loader.stable.unwrap_or(true);
                            versions.push(LoaderVersionItem {
                                label: format!("Fabric Loader {} ({})", item.loader.version, if is_stable { "Stable" } else { "Beta" }),
                                version: item.loader.version,
                                is_stable,
                                is_recommended: idx == 0,
                            });
                        }
                    }
                } else if let Ok(entries) = resp.json::<Vec<FabricLoaderEntry>>().await {
                    for (idx, item) in entries.into_iter().enumerate() {
                        let is_stable = item.stable.unwrap_or(true);
                        versions.push(LoaderVersionItem {
                            label: format!("Fabric Loader {} ({})", item.version, if is_stable { "Stable" } else { "Beta" }),
                            version: item.version,
                            is_stable,
                            is_recommended: idx == 0,
                        });
                    }
                }
            }
        }
    }

    if versions.is_empty() {
        let fallback_list = ["0.16.10", "0.16.9", "0.16.8", "0.16.7", "0.15.11", "0.15.7", "0.14.25"];
        for (idx, ver) in fallback_list.iter().enumerate() {
            versions.push(LoaderVersionItem {
                version: ver.to_string(),
                label: format!("Fabric Loader {} (Stable)", ver),
                is_stable: true,
                is_recommended: idx == 0,
            });
        }
    }

    LoaderVersionsResponse {
        engine_type: "FABRIC".to_string(),
        game_version: game_version.to_string(),
        env_variable: "FABRIC_LOADER_VERSION".to_string(),
        default_version: "LATEST".to_string(),
        versions,
    }
}

#[derive(Debug, Deserialize)]
struct QuiltLoaderEntry {
    version: String,
}

async fn fetch_quilt_loader_versions(game_version: &str) -> LoaderVersionsResponse {
    let client = reqwest::Client::builder()
        .user_agent("ChiPanel/0.1.0")
        .timeout(Duration::from_secs(5))
        .build();

    let mut versions = Vec::new();

    if let Ok(c) = client {
        if let Ok(resp) = c.get("https://meta.quiltmc.org/v3/versions/loader").send().await {
            if let Ok(entries) = resp.json::<Vec<QuiltLoaderEntry>>().await {
                for (idx, item) in entries.into_iter().enumerate() {
                    versions.push(LoaderVersionItem {
                        label: format!("Quilt Loader {}", item.version),
                        version: item.version,
                        is_stable: true,
                        is_recommended: idx == 0,
                    });
                }
            }
        }
    }

    if versions.is_empty() {
        let fallback_list = ["0.26.3", "0.26.0", "0.25.0", "0.24.0", "0.23.1"];
        for (idx, ver) in fallback_list.iter().enumerate() {
            versions.push(LoaderVersionItem {
                version: ver.to_string(),
                label: format!("Quilt Loader {}", ver),
                is_stable: true,
                is_recommended: idx == 0,
            });
        }
    }

    LoaderVersionsResponse {
        engine_type: "QUILT".to_string(),
        game_version: game_version.to_string(),
        env_variable: "QUILT_LOADER_VERSION".to_string(),
        default_version: "LATEST".to_string(),
        versions,
    }
}

#[derive(Debug, Deserialize)]
struct PaperBuildsResponse {
    #[serde(default)]
    builds: Vec<PaperBuildEntry>,
}

#[derive(Debug, Deserialize)]
struct PaperBuildEntry {
    build: u32,
    #[serde(default)]
    channel: String,
}

async fn fetch_paper_builds(game_version: &str) -> LoaderVersionsResponse {
    let client = reqwest::Client::builder()
        .user_agent("ChiPanel/0.1.0")
        .timeout(Duration::from_secs(5))
        .build();

    let mut versions = Vec::new();

    if !game_version.is_empty() && !game_version.eq_ignore_ascii_case("LATEST") {
        if let Ok(c) = client {
            let url = format!("https://api.papermc.io/v2/projects/paper/versions/{}/builds", game_version);
            if let Ok(resp) = c.get(&url).send().await {
                if let Ok(data) = resp.json::<PaperBuildsResponse>().await {
                    for b in data.builds.into_iter().rev().take(20) {
                        let is_default = b.channel == "default" || b.channel.is_empty();
                        let is_first = versions.is_empty();
                        versions.push(LoaderVersionItem {
                            label: format!("Paper Build #{} ({})", b.build, if is_default { "Stable" } else { &b.channel }),
                            version: b.build.to_string(),
                            is_stable: is_default,
                            is_recommended: is_first,
                        });
                    }
                }
            }
        }
    }

    LoaderVersionsResponse {
        engine_type: "PAPER".to_string(),
        game_version: game_version.to_string(),
        env_variable: "PAPER_BUILD".to_string(),
        default_version: "LATEST".to_string(),
        versions,
    }
}

#[derive(Debug, Deserialize)]
struct PurpurBuildsResponse {
    #[serde(default)]
    builds: Option<PurpurBuildsMap>,
}

#[derive(Debug, Deserialize)]
struct PurpurBuildsMap {
    #[serde(default)]
    all: Vec<String>,
}

async fn fetch_purpur_builds(game_version: &str) -> LoaderVersionsResponse {
    let client = reqwest::Client::builder()
        .user_agent("ChiPanel/0.1.0")
        .timeout(Duration::from_secs(5))
        .build();

    let mut versions = Vec::new();

    if !game_version.is_empty() && !game_version.eq_ignore_ascii_case("LATEST") {
        if let Ok(c) = client {
            let url = format!("https://api.purpurmc.org/v2/purpur/{}", game_version);
            if let Ok(resp) = c.get(&url).send().await {
                if let Ok(data) = resp.json::<PurpurBuildsResponse>().await {
                    if let Some(map) = data.builds {
                        for b in map.all.into_iter().rev().take(20) {
                            let is_first = versions.is_empty();
                            versions.push(LoaderVersionItem {
                                label: format!("Purpur Build #{}", b),
                                version: b,
                                is_stable: true,
                                is_recommended: is_first,
                            });
                        }
                    }
                }
            }
        }
    }

    LoaderVersionsResponse {
        engine_type: "PURPUR".to_string(),
        game_version: game_version.to_string(),
        env_variable: "PURPUR_BUILD".to_string(),
        default_version: "LATEST".to_string(),
        versions,
    }
}

async fn fetch_forge_versions(game_version: &str) -> LoaderVersionsResponse {
    let versions = vec![
        LoaderVersionItem {
            version: "RECOMMENDED".to_string(),
            label: "Forge Recommandé (Stable)".to_string(),
            is_stable: true,
            is_recommended: true,
        },
        LoaderVersionItem {
            version: "LATEST".to_string(),
            label: "Dernière version Forge (LATEST)".to_string(),
            is_stable: false,
            is_recommended: false,
        },
    ];

    LoaderVersionsResponse {
        engine_type: "FORGE".to_string(),
        game_version: game_version.to_string(),
        env_variable: "FORGE_VERSION".to_string(),
        default_version: "RECOMMENDED".to_string(),
        versions,
    }
}

async fn fetch_neoforge_versions(game_version: &str) -> LoaderVersionsResponse {
    let versions = vec![
        LoaderVersionItem {
            version: "LATEST".to_string(),
            label: "Dernière version NeoForge (LATEST)".to_string(),
            is_stable: true,
            is_recommended: true,
        },
    ];

    LoaderVersionsResponse {
        engine_type: "NEOFORGE".to_string(),
        game_version: game_version.to_string(),
        env_variable: "NEOFORGE_VERSION".to_string(),
        default_version: "LATEST".to_string(),
        versions,
    }
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

    #[test]
    fn test_engine_version_bounds() {
        let catalog = fallback_version_catalog();
        let engines = get_available_engines();

        let neoforge = engines.iter().find(|e| e.id == "NEOFORGE").unwrap();
        assert!(is_engine_version_supported(neoforge, "1.21.4", &catalog));
        assert!(is_engine_version_supported(neoforge, "1.20.4", &catalog));
        assert!(is_engine_version_supported(neoforge, "1.20.2", &catalog));
        assert!(!is_engine_version_supported(neoforge, "1.20.1", &catalog));
        assert!(!is_engine_version_supported(neoforge, "1.16.5", &catalog));
        assert!(!is_engine_version_supported(neoforge, "1.12.2", &catalog));

        let folia = engines.iter().find(|e| e.id == "FOLIA").unwrap();
        assert!(is_engine_version_supported(folia, "1.21.4", &catalog));
        assert!(is_engine_version_supported(folia, "1.19.4", &catalog));
        assert!(!is_engine_version_supported(folia, "1.19.2", &catalog));
        assert!(!is_engine_version_supported(folia, "1.16.5", &catalog));

        let mohist = engines.iter().find(|e| e.id == "MOHIST").unwrap();
        assert!(is_engine_version_supported(mohist, "1.20.1", &catalog));
        assert!(is_engine_version_supported(mohist, "1.16.5", &catalog));
        assert!(is_engine_version_supported(mohist, "1.12.2", &catalog));
        assert!(!is_engine_version_supported(mohist, "1.21.4", &catalog));
        assert!(!is_engine_version_supported(mohist, "LATEST", &catalog));

        let purpur = engines.iter().find(|e| e.id == "PURPUR").unwrap();
        assert!(is_engine_version_supported(purpur, "1.21.4", &catalog));
        assert!(is_engine_version_supported(purpur, "1.14.4", &catalog));
        assert!(!is_engine_version_supported(purpur, "1.12.2", &catalog));
    }

    #[tokio::test]
    async fn test_fetch_engine_loader_versions() {
        let fabric = fetch_engine_loader_versions("FABRIC", Some("1.21.4")).await;
        assert_eq!(fabric.engine_type, "FABRIC");
        assert_eq!(fabric.env_variable, "FABRIC_LOADER_VERSION");
        assert!(!fabric.versions.is_empty());

        let forge = fetch_engine_loader_versions("FORGE", Some("1.20.1")).await;
        assert_eq!(forge.engine_type, "FORGE");
        assert_eq!(forge.env_variable, "FORGE_VERSION");
        assert!(forge.versions.iter().any(|v| v.version == "RECOMMENDED"));

        let paper = fetch_engine_loader_versions("PAPER", Some("1.21.4")).await;
        assert_eq!(paper.engine_type, "PAPER");
        assert_eq!(paper.env_variable, "PAPER_BUILD");
    }
}

