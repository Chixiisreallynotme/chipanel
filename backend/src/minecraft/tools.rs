use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::time::MissedTickBehavior;
use tracing::{info, warn};

use crate::{
    config::AppConfig,
    error::AppError,
    modrinth::client::ModrinthClient,
};

const SPARK_PROJECT: &str = "spark";
const CHUNKY_PROJECT: &str = "chunky";
const FABRIC_API_PROJECT: &str = "fabric-api";
const LUCKPERMS_PROJECT: &str = "luckperms";

const JENKINS_SPARK_JSON: &str =
    "https://ci.lucko.me/job/spark/lastSuccessfulBuild/api/json?tree=artifacts[fileName,relativePath]";
const JENKINS_ARTIFACT_BASE: &str = "https://ci.lucko.me/job/spark/lastSuccessfulBuild/artifact/";

/// Where spark comes from for a given engine. Mod loaders are on Modrinth; the
/// Bukkit/Paper plugin is NOT (Modrinth only carries the mod) — it is built on
/// spark's official Jenkins CI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SparkSource {
    Modrinth(&'static str),
    Jenkins(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ToolTarget {
    target_dir: &'static str,
    chunky_loader: Option<&'static str>,
    spark: Option<SparkSource>,
    luckperms_loader: Option<&'static str>,
}

/// Maps an itzg `TYPE` value to where the tools live and how to fetch them.
/// `None` when the engine is unsupported (no spark AND no chunky for it).
fn tool_target(engine: &str) -> Option<ToolTarget> {
    match engine {
        "FABRIC" => Some(ToolTarget {
            target_dir: "mods",
            chunky_loader: Some("fabric"),
            spark: Some(SparkSource::Modrinth("fabric")),
            luckperms_loader: Some("fabric"),
        }),
        "FORGE" => Some(ToolTarget {
            target_dir: "mods",
            chunky_loader: Some("forge"),
            spark: Some(SparkSource::Modrinth("forge")),
            luckperms_loader: Some("forge"),
        }),
        "NEOFORGE" => Some(ToolTarget {
            target_dir: "mods",
            chunky_loader: Some("neoforge"),
            spark: Some(SparkSource::Modrinth("neoforge")),
            luckperms_loader: Some("neoforge"),
        }),
        "QUILT" => Some(ToolTarget {
            target_dir: "mods",
            chunky_loader: None,
            spark: Some(SparkSource::Modrinth("quilt")),
            luckperms_loader: None,
        }),
        "PAPER" => Some(ToolTarget {
            target_dir: "plugins",
            chunky_loader: Some("paper"),
            spark: Some(SparkSource::Jenkins("paper")),
            luckperms_loader: Some("paper"),
        }),
        "PURPUR" => Some(ToolTarget {
            target_dir: "plugins",
            chunky_loader: Some("paper"),
            spark: Some(SparkSource::Jenkins("paper")),
            luckperms_loader: Some("paper"),
        }),
        "SPIGOT" => Some(ToolTarget {
            target_dir: "plugins",
            chunky_loader: Some("spigot"),
            spark: Some(SparkSource::Jenkins("bukkit")),
            luckperms_loader: Some("spigot"),
        }),
        "BUKKIT" => Some(ToolTarget {
            target_dir: "plugins",
            chunky_loader: Some("bukkit"),
            spark: Some(SparkSource::Jenkins("bukkit")),
            luckperms_loader: Some("bukkit"),
        }),
        "FOLIA" => Some(ToolTarget {
            target_dir: "plugins",
            chunky_loader: Some("folia"),
            spark: None,
            luckperms_loader: Some("folia"),
        }),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct DetectedEngine {
    pub engine: String,
    pub version: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSyncOutcome {
    pub tool: String,
    pub status: String,
    pub version: Option<String>,
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSyncReport {
    pub engine: String,
    pub version: String,
    pub config_source: String,
    pub supported: bool,
    pub target_dir: String,
    pub managed: Vec<String>,
    pub outcomes: Vec<ToolSyncOutcome>,
}

/// Detects the current engine (TYPE) and version. Order of precedence:
/// 1. the `minecraft.container` quadlet (the engine switcher's source of truth),
/// 2. the persisted `engine_config.json` state file,
/// 3. auto-detection from the Minecraft data dir (itzg `.install-*.env` markers).
pub async fn detect_engine(config: &AppConfig) -> DetectedEngine {
    let container_file = config.systemd_config_dir.join("minecraft.container");
    if let Ok(content) = tokio::fs::read_to_string(&container_file).await {
        let (t, v) = parse_type_version(&content);
        if let Some(engine) = t {
            return DetectedEngine {
                engine,
                version: v.unwrap_or_default(),
                source: "quadlet".to_string(),
            };
        }
    }

    let state_file = config.data_dir.join("engine_config.json");
    if let Ok(content) = tokio::fs::read_to_string(&state_file).await {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            let engine = json.get("type").and_then(|v| v.as_str()).map(str::to_string);
            let version = json.get("version").and_then(|v| v.as_str()).map(str::to_string);
            if let Some(engine) = engine {
                return DetectedEngine {
                    engine,
                    version: version.unwrap_or_default(),
                    source: "state_file".to_string(),
                };
            }
        }
    }

    if let Some((engine, version)) = detect_from_data_dir(&config.minecraft_data_dir).await {
        return DetectedEngine {
            engine,
            version,
            source: "data_dir".to_string(),
        };
    }

    DetectedEngine {
        engine: String::new(),
        version: String::new(),
        source: "unavailable".to_string(),
    }
}

/// Lists which tools would be managed for the currently detected engine, without
/// downloading anything. `outcomes` is left empty (this is a read-only snapshot).
pub async fn tools_status(config: &AppConfig) -> ToolSyncReport {
    let detected = detect_engine(config).await;
    let target = tool_target(&detected.engine);
    let mut managed = Vec::new();

    if let Some(t) = target {
        if t.chunky_loader.is_some() {
            managed.push("chunky".to_string());
        }
        if t.spark.is_some() {
            managed.push("spark".to_string());
        }
        if t.luckperms_loader.is_some() {
            managed.push("luckperms".to_string());
        }
        if t.target_dir == "mods" && detected.engine == "FABRIC" {
            managed.push("fabric-api".to_string());
        }
    }

    ToolSyncReport {
        engine: detected.engine,
        version: detected.version,
        config_source: detected.source,
        supported: target.is_some(),
        target_dir: target.map(|t| t.target_dir.to_string()).unwrap_or_default(),
        managed,
        outcomes: Vec::new(),
    }
}

/// Installs or updates spark + chunky (and their dependencies) to match the
/// detected engine and Minecraft version. Idempotent: already-current files are
/// left untouched, stale versions of the same tool are removed.
pub async fn ensure_tools(config: &AppConfig) -> Result<ToolSyncReport, AppError> {
    let detected = detect_engine(config).await;
    let engine = detected.engine.clone();
    let version = detected.version.clone();

    let Some(target) = tool_target(&engine) else {
        return Ok(ToolSyncReport {
            engine,
            version,
            config_source: detected.source,
            supported: false,
            target_dir: String::new(),
            managed: Vec::new(),
            outcomes: Vec::new(),
        });
    };

    let target_path = config.minecraft_data_dir.join(target.target_dir);
    let client = &config.modrinth_client;
    let mut managed = Vec::new();
    let mut outcomes = Vec::new();

    if let Some(loader) = target.chunky_loader {
        managed.push("chunky".to_string());
        outcomes.push(
            sync_modrinth_tool(client, &target_path, "chunky", CHUNKY_PROJECT, loader, &version, "Chunky-").await,
        );
    } else {
        outcomes.push(ToolSyncOutcome {
            tool: "chunky".to_string(),
            status: "unsupported".to_string(),
            version: None,
            filename: None,
        });
    }

    match target.spark {
        Some(SparkSource::Modrinth(loader)) => {
            managed.push("spark".to_string());
            outcomes.push(
                sync_modrinth_tool(client, &target_path, "spark", SPARK_PROJECT, loader, &version, "spark-").await,
            );
        }
        Some(SparkSource::Jenkins(flavor)) => {
            managed.push("spark".to_string());
            outcomes.push(sync_spark_jenkins(client, &target_path, flavor).await);
        }
        None => {
            outcomes.push(ToolSyncOutcome {
                tool: "spark".to_string(),
                status: "unsupported".to_string(),
                version: None,
                filename: None,
            });
        }
    }

    if let Some(loader) = target.luckperms_loader {
        managed.push("luckperms".to_string());
        outcomes.push(
            sync_modrinth_tool(client, &target_path, "luckperms", LUCKPERMS_PROJECT, loader, &version, "LuckPerms-")
                .await,
        );
    } else {
        outcomes.push(ToolSyncOutcome {
            tool: "luckperms".to_string(),
            status: "unsupported".to_string(),
            version: None,
            filename: None,
        });
    }

    // Fabric mods (chunky-fabric, spark-fabric) require Fabric API.
    if target.target_dir == "mods" && engine == "FABRIC" {
        managed.push("fabric-api".to_string());
        outcomes.push(
            sync_modrinth_tool(client, &target_path, "fabric-api", FABRIC_API_PROJECT, "fabric", &version, "fabric-api-")
                .await,
        );
    }

    Ok(ToolSyncReport {
        engine,
        version,
        config_source: detected.source,
        supported: true,
        target_dir: target.target_dir.to_string(),
        managed,
        outcomes,
    })
}

/// Resolves the latest Modrinth version for `project` (loader + game version),
/// then installs it if not already current. Falls back to a loader-only query
/// when no version matches the exact game version (some tools are version-agnostic).
async fn sync_modrinth_tool(
    client: &ModrinthClient,
    target_path: &Path,
    tool: &str,
    project: &str,
    loader: &str,
    game_version: &str,
    file_prefix: &str,
) -> ToolSyncOutcome {
    let versions = match client
        .get_project_versions_filtered(project, &[loader], game_version)
        .await
    {
        Ok(v) if v.is_empty() => client
            .get_project_versions_filtered(project, &[loader], "")
            .await
            .unwrap_or_default(),
        Ok(v) => v,
        Err(e) => {
            return ToolSyncOutcome {
                tool: tool.to_string(),
                status: "error".to_string(),
                version: None,
                filename: Some(e.to_string()),
            }
        }
    };

    let version = match versions.into_iter().find(|v| !v.files.is_empty()) {
        Some(v) => v,
        None => {
            return ToolSyncOutcome {
                tool: tool.to_string(),
                status: "not_found".to_string(),
                version: None,
                filename: None,
            }
        }
    };

    let filename = match primary_filename(&version) {
        Some(f) => f,
        None => {
            return ToolSyncOutcome {
                tool: tool.to_string(),
                status: "not_found".to_string(),
                version: None,
                filename: None,
            }
        }
    };

    if is_current(target_path, &filename).await {
        return ToolSyncOutcome {
            tool: tool.to_string(),
            status: "up_to_date".to_string(),
            version: Some(version.version_number.clone()),
            filename: Some(filename),
        };
    }

    remove_stale(target_path, file_prefix, &filename).await;

    match client.install_version_file(&version, target_path).await {
        Ok((v, f)) => ToolSyncOutcome {
            tool: tool.to_string(),
            status: "installed".to_string(),
            version: Some(v),
            filename: Some(f),
        },
        Err(e) => ToolSyncOutcome {
            tool: tool.to_string(),
            status: "error".to_string(),
            version: None,
            filename: Some(e.to_string()),
        },
    }
}

/// Fetches spark's plugin build from its official Jenkins CI (the only source for
/// the Paper/Purpur/Spigot plugin — Modrinth only carries the mod).
async fn sync_spark_jenkins(
    client: &ModrinthClient,
    target_path: &Path,
    flavor: &str,
) -> ToolSyncOutcome {
    let json = match client.get_json(JENKINS_SPARK_JSON).await {
        Ok(j) => j,
        Err(e) => {
            return ToolSyncOutcome {
                tool: "spark".to_string(),
                status: "error".to_string(),
                version: None,
                filename: Some(e.to_string()),
            }
        }
    };

    let artifacts = json.get("artifacts").and_then(|a| a.as_array()).cloned().unwrap_or_default();

    let wanted_suffix = format!("-{}.jar", flavor);
    let fallback_suffix = "-bukkit.jar".to_string();
    let artifact = artifacts
        .iter()
        .find(|a| file_name_of(a).map(|f| f.ends_with(&wanted_suffix)).unwrap_or(false))
        .or_else(|| artifacts.iter().find(|a| file_name_of(a).map(|f| f.ends_with(&fallback_suffix)).unwrap_or(false)));

    let (file_name, relative_path) = match artifact {
        Some(a) => {
            let f = file_name_of(a).unwrap_or_default().to_string();
            let r = a
                .get("relativePath")
                .and_then(|r| r.as_str())
                .unwrap_or_default()
                .to_string();
            (f, r)
        }
        None => {
            return ToolSyncOutcome {
                tool: "spark".to_string(),
                status: "not_found".to_string(),
                version: None,
                filename: None,
            }
        }
    };

    if file_name.is_empty() || relative_path.is_empty() {
        return ToolSyncOutcome {
            tool: "spark".to_string(),
            status: "not_found".to_string(),
            version: None,
            filename: None,
        };
    }

    if is_current(target_path, &file_name).await {
        return ToolSyncOutcome {
            tool: "spark".to_string(),
            status: "up_to_date".to_string(),
            version: spark_jenkins_version(&file_name, flavor),
            filename: Some(file_name),
        };
    }

    remove_stale(target_path, "spark-", &file_name).await;

    let dest = target_path.join(&file_name);
    match client.download_to(&format!("{}{}", JENKINS_ARTIFACT_BASE, relative_path), &dest).await {
        Ok(_) => ToolSyncOutcome {
            tool: "spark".to_string(),
            status: "installed".to_string(),
            version: spark_jenkins_version(&file_name, flavor),
            filename: Some(file_name),
        },
        Err(e) => ToolSyncOutcome {
            tool: "spark".to_string(),
            status: "error".to_string(),
            version: None,
            filename: Some(e.to_string()),
        },
    }
}

fn file_name_of(artifact: &serde_json::Value) -> Option<&str> {
    artifact.get("fileName").and_then(|f| f.as_str())
}

fn primary_filename(version: &crate::modrinth::client::ModrinthVersion) -> Option<String> {
    version
        .files
        .iter()
        .find(|f| f.primary && f.filename.ends_with(".jar"))
        .or_else(|| version.files.iter().find(|f| f.filename.ends_with(".jar")))
        .or_else(|| version.files.first())
        .map(|f| f.filename.clone())
}

async fn is_current(target_path: &Path, filename: &str) -> bool {
    target_path.join(filename).exists()
}

async fn remove_stale(target_path: &Path, prefix: &str, keep: &str) {
    if !target_path.exists() {
        return;
    }
    let mut entries = match tokio::fs::read_dir(target_path).await {
        Ok(e) => e,
        Err(_) => return,
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue,
        };
        if name == keep || !name.starts_with(prefix) {
            continue;
        }
        if !name.ends_with(".jar") && !name.ends_with(".jar.disabled") {
            continue;
        }
        if let Err(e) = tokio::fs::remove_file(entry.path()).await {
            warn!("Failed to remove stale tool file '{}': {}", name, e);
        } else {
            info!("Removed stale tool file '{}'", name);
        }
    }
}

fn spark_jenkins_version(file_name: &str, flavor: &str) -> Option<String> {
    let suffix = format!("-{}.jar", flavor);
    let body = file_name.strip_prefix("spark-")?.strip_suffix(&suffix)?;
    if body.is_empty() {
        None
    } else {
        Some(body.to_string())
    }
}

fn parse_type_version(content: &str) -> (Option<String>, Option<String>) {
    let mut ty = None;
    let mut ver = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("Environment=TYPE=") {
            let v = val.trim();
            if !v.is_empty() {
                ty = Some(v.to_uppercase());
            }
        } else if let Some(val) = trimmed.strip_prefix("Environment=VERSION=") {
            let v = val.trim();
            if !v.is_empty() {
                ver = Some(v.to_uppercase());
            }
        }
    }
    (ty, ver)
}

async fn detect_from_data_dir(data_dir: &Path) -> Option<(String, String)> {
    let markers = [
        ".install-fabric.env",
        ".install-forge.env",
        ".install-neoforge.env",
        ".install-quilt.env",
        ".paper.env",
        ".purpur.env",
    ];
    for marker in markers {
        let path = data_dir.join(marker);
        if let Ok(content) = tokio::fs::read_to_string(&path).await {
            if let Some((ty, ver)) = parse_env_marker(&content) {
                return Some((ty, ver));
            }
        }
    }
    None
}

/// Parses an itzg install marker (`TYPE="X"` / `FAMILY="X"` / `VERSION="X"`).
fn parse_env_marker(content: &str) -> Option<(String, String)> {
    let mut ty = None;
    let mut ver = None;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let val = v.trim().trim_matches('"').trim_matches('\'').trim();
        if key == "TYPE" || key == "FAMILY" {
            if !val.is_empty() {
                ty = Some(val.to_uppercase());
            }
        } else if key == "VERSION" {
            if !val.is_empty() {
                ver = Some(val.to_uppercase());
            }
        }
    }
    ty.map(|t| (t, ver.unwrap_or_default()))
}

/// Spawns a periodic background task that keeps spark + chunky in sync with the
/// server's engine/version. Default interval 24h, overridable via
/// `TOOLS_SYNC_INTERVAL_SECS`. First tick fires immediately.
pub fn start_tools_sync_loop(config: Arc<AppConfig>) {
    tokio::spawn(async move {
        let interval_secs = std::env::var("TOOLS_SYNC_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(86_400);
        let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            match ensure_tools(&config).await {
                Ok(report) => {
                    if report.supported {
                        info!(
                            "tools sync: engine={} version={} dir={} -> {}",
                            report.engine,
                            report.version,
                            report.target_dir,
                            report
                                .outcomes
                                .iter()
                                .map(|o| format!("{}={}", o.tool, o.status))
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    } else {
                        info!("tools sync skipped: engine '{}' not supported", report.engine);
                    }
                }
                Err(e) => warn!("tools sync failed: {}", e),
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_engine_to_target() {
        assert_eq!(tool_target("FABRIC").unwrap().target_dir, "mods");
        assert_eq!(tool_target("PAPER").unwrap().target_dir, "plugins");
        assert_eq!(tool_target("PURPUR").unwrap().target_dir, "plugins");
        assert_eq!(tool_target("VANILLA"), None);
        assert_eq!(tool_target("MOHIST"), None);
    }

    #[test]
    fn parses_quadlet_type_version() {
        let content = "# comment\nEnvironment=TYPE=fabric\nEnvironment=VERSION=26.2\n";
        let (ty, ver) = parse_type_version(content);
        assert_eq!(ty.as_deref(), Some("FABRIC"));
        assert_eq!(ver.as_deref(), Some("26.2"));
    }

    #[test]
    fn parses_env_marker() {
        let content = "SERVER=\"./fabric-server.jar\"\nFAMILY=\"FABRIC\"\nTYPE=\"FABRIC\"\nVERSION=\"26.2\"\n";
        let (ty, ver) = parse_env_marker(content).unwrap();
        assert_eq!(ty, "FABRIC");
        assert_eq!(ver, "26.2");
    }

    #[test]
    fn extracts_spark_jenkins_version() {
        assert_eq!(spark_jenkins_version("spark-1.10.178-paper.jar", "paper").as_deref(), Some("1.10.178"));
        assert_eq!(spark_jenkins_version("spark-1.10.178-bukkit.jar", "bukkit").as_deref(), Some("1.10.178"));
        assert_eq!(spark_jenkins_version("spark-1.10.178-paper.jar", "bukkit"), None);
    }
}
