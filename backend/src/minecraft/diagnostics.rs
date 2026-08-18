use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashDiagnosticReport {
    pub id: String,
    pub filename: String,
    pub created_at_secs: u64,
    pub title: String,
    pub category: String, // "OOM" | "MOD_CONFLICT" | "CHUNK_CORRUPTION" | "JAVA_INCOMPATIBILITY" | "TICKING_ENTITY" | "UNKNOWN"
    pub severity: String, // "CRITICAL" | "HIGH" | "MEDIUM"
    pub root_cause: String,
    pub suspected_source: Option<String>,
    pub recommendation: String,
    pub remediation_action: Option<String>, // "increase_ram" | "disable_addon" | "restore_backup" | "delete_corrupt_chunk"
    pub stack_trace_snippet: String,
    pub minecraft_version: Option<String>,
    pub java_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHealthSummary {
    pub score: u8, // 0..100
    pub status: String, // "EXCELLENT" | "GOOD" | "DEGRADED" | "CRITICAL"
    pub total_crashes_detected: usize,
    pub last_crash_timestamp: Option<u64>,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Analyzes a raw crash-report string with deterministic classification heuristics
pub fn parse_and_classify_crash_report(
    id: &str,
    filename: &str,
    created_at_secs: u64,
    content: &str,
) -> CrashDiagnosticReport {
    let mut title = "Serveur Minecraft arrêté de manière inattendue".to_string();
    let mut category = "UNKNOWN".to_string();
    let mut severity = "HIGH".to_string();
    let mut root_cause = "Une exception non gérée a provoqué l'arrêt brutal du serveur.".to_string();
    let mut suspected_source = None;
    let mut recommendation = "Consultez les logs détaillés et vérifiez la compatibilité de vos plugins/mods.".to_string();
    let mut remediation_action = None;
    let mut minecraft_version = None;
    let mut java_version = None;

    // Extract Minecraft & Java Version metadata
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Minecraft Version:") {
            minecraft_version = trimmed.split(':').nth(1).map(|s| s.trim().to_string());
        } else if trimmed.starts_with("Java Version:") {
            java_version = trimmed.split(':').nth(1).map(|s| s.trim().to_string());
        } else if trimmed.starts_with("Description:") {
            title = trimmed.split(':').nth(1).unwrap_or("Erreur de serveur").trim().to_string();
        }
    }

    let lower = content.to_lowercase();

    // 1. Out Of Memory (OOM)
    if lower.contains("outofmemoryerror") || lower.contains("java heap space") || lower.contains("gc overhead limit exceeded") {
        category = "OOM".to_string();
        severity = "CRITICAL".to_string();
        title = "Mémoire JVM Insuffisante (Out Of Memory)".to_string();
        root_cause = "Le serveur Minecraft a épuisé la mémoire RAM allouée au heap Java (-Xmx).".to_string();
        recommendation = "Augmentez l'allocation RAM (-Xmx) dans les paramètres du conteneur ou réduisez 'view-distance' et 'simulation-distance'.".to_string();
        remediation_action = Some("increase_ram".to_string());
    }
    // 2. Ticking Entity / Block Entity Exception
    else if lower.contains("ticking entity") || lower.contains("ticking block entity") || lower.contains("ticking player") {
        category = "TICKING_ENTITY".to_string();
        severity = "HIGH".to_string();
        title = "Entité ou Bloc Corrompu (Ticking Entity Crash)".to_string();
        root_cause = "Une entité vivante ou un conteneur (coffre/machine/spawner) génère une boucle infinie ou un crash de tick.".to_string();
        recommendation = "Activez 'remove-unloaded-ticking-entities' ou 'remove-corrupt-tile-entities' dans paper-world-defaults.yml ou purpur.yml.".to_string();
        remediation_action = Some("delete_corrupt_chunk".to_string());
    }
    // 3. Chunk / Region Corruption
    else if lower.contains("corruptchunkexception") || lower.contains("wrong location in world") || lower.contains("nbt tag too large") || lower.contains("error loading chunk") {
        category = "CHUNK_CORRUPTION".to_string();
        severity = "CRITICAL".to_string();
        title = "Corruption de Chunk / Fichier Région .mca".to_string();
        root_cause = "Un fichier de chunk Minecraft est physiquement corrompu sur le disque.".to_string();
        recommendation = "Régénérez la région endommagée via l'outil de gestion des Mondes ou restaurez votre dernière sauvegarde automatique.".to_string();
        remediation_action = Some("restore_backup".to_string());
    }
    // 4. Mod / Mixin / Plugin Incompatibility & ClassNotFound
    else if lower.contains("mixinapplyerror") || lower.contains("duplicatemodexception") || lower.contains("nosuchmethoderror") || lower.contains("classnotfoundexception") || lower.contains("incompatibleclasschangeerror") {
        category = "MOD_CONFLICT".to_string();
        severity = "HIGH".to_string();
        title = "Incompatibilité ou Conflit de Mod / Plugin".to_string();
        root_cause = "Un plugin ou mod utilise des méthodes Java obsolètes ou incompatibles avec cette version de Minecraft.".to_string();
        
        // Try to identify suspected plugin or mod name from stack trace
        for line in content.lines() {
            let tr = line.trim();
            if tr.starts_with("at ") && (tr.contains("com.") || tr.contains("org.") || tr.contains("net.") || tr.contains("fr.")) {
                if !tr.contains("net.minecraft") && !tr.contains("java.") && !tr.contains("jdk.") && !tr.contains("org.bukkit.craftbukkit") {
                    let segments: Vec<&str> = tr.split('.').collect();
                    if segments.len() >= 2 {
                        suspected_source = Some(segments[1].to_string());
                        break;
                    }
                }
            }
        }

        recommendation = match &suspected_source {
            Some(source) => format!("Mettez à jour ou désactivez le plugin/mod '{}' compatible avec Minecraft {}.", source, minecraft_version.as_deref().unwrap_or("actuel")),
            None => "Vérifiez vos extensions et mettez-les à jour vers des versions compatibles.".to_string(),
        };
        remediation_action = Some("disable_addon".to_string());
    }
    // 5. Java Version Mismatch
    else if lower.contains("unsupported class file major version") || lower.contains("has been compiled by a more recent version of the java runtime") {
        category = "JAVA_INCOMPATIBILITY".to_string();
        severity = "HIGH".to_string();
        title = "Incompatibilité de Runtime Java".to_string();
        root_cause = "Le serveur ou l'un de ses plugins nécessite une version de Java plus récente (Java 17 / 21).".to_string();
        recommendation = "Basculez l'image conteneur ou le profil Java vers Java 21 LTS.".to_string();
        remediation_action = Some("increase_ram".to_string());
    }

    // Extract first 15 lines of stack trace
    let stack_trace_snippet = content
        .lines()
        .take(25)
        .collect::<Vec<&str>>()
        .join("\n");

    CrashDiagnosticReport {
        id: id.to_string(),
        filename: filename.to_string(),
        created_at_secs,
        title,
        category,
        severity,
        root_cause,
        suspected_source,
        recommendation,
        remediation_action,
        stack_trace_snippet,
        minecraft_version,
        java_version,
    }
}

/// Scans crash-reports/ directory and returns list of diagnosed reports
pub async fn scan_and_diagnose_crashes(config: &AppConfig) -> Result<Vec<CrashDiagnosticReport>, AppError> {
    let crash_dir = config.minecraft_data_dir.join("crash-reports");
    if !crash_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = tokio::fs::read_dir(&crash_dir)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed reading crash-reports dir: {}", e)))?;

    let mut reports = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let filename = entry.file_name().to_string_lossy().to_string();
        if !filename.starts_with("crash-") || !filename.ends_with(".txt") {
            continue;
        }

        let metadata = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue,
        };

        let created_at_secs = metadata
            .modified()
            .unwrap_or_else(|_| SystemTime::now())
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let id = filename.trim_start_matches("crash-").trim_end_matches(".txt").to_string();

        if let Ok(content) = tokio::fs::read_to_string(&path).await {
            let report = parse_and_classify_crash_report(&id, &filename, created_at_secs, &content);
            reports.push(report);
        }
    }

    // Sort newest first
    reports.sort_by_key(|r| std::cmp::Reverse(r.created_at_secs));
    Ok(reports)
}

/// Computes overall server health score and issue summary
pub async fn compute_server_health(config: &AppConfig) -> Result<ServerHealthSummary, AppError> {
    let crashes = scan_and_diagnose_crashes(config).await.unwrap_or_default();
    let total_crashes = crashes.len();
    let last_crash = crashes.first().map(|c| c.created_at_secs);

    let mut score = 100u8;
    let mut issues = Vec::new();
    let mut recommendations = Vec::new();

    // Check recent crashes within last 24h
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let recent_crashes = crashes.iter().filter(|c| now_secs.saturating_sub(c.created_at_secs) < 86400).count();

    if recent_crashes > 0 {
        score = score.saturating_sub((recent_crashes * 20).min(60) as u8);
        issues.push(format!("{} crash(s) détecté(s) au cours des dernières 24 heures.", recent_crashes));
    }

    // Inspect crash categories
    for c in crashes.iter().take(3) {
        if c.category == "OOM" {
            issues.push("Épuisement de mémoire JVM (OOM) détecté.".to_string());
            recommendations.push("Augmentez la mémoire allouée au serveur.".to_string());
        } else if c.category == "CHUNK_CORRUPTION" {
            issues.push("Corruption de chunk enregistrée dans les rapports.".to_string());
            recommendations.push("Vérifiez l'intégrité de vos régions de mondes.".to_string());
        }
    }

    if score > 85 {
        recommendations.push("Le serveur fonctionne de manière optimale. Pensez à planifier des sauvegardes régulières.".to_string());
    }

    let status = match score {
        90..=100 => "EXCELLENT".to_string(),
        70..=89 => "GOOD".to_string(),
        40..=69 => "DEGRADED".to_string(),
        _ => "CRITICAL".to_string(),
    };

    Ok(ServerHealthSummary {
        score,
        status,
        total_crashes_detected: total_crashes,
        last_crash_timestamp: last_crash,
        issues,
        recommendations,
    })
}

/// Executes automated remediation for a diagnosed crash
pub async fn execute_remediation(
    config: &AppConfig,
    report_id: &str,
) -> Result<String, AppError> {
    let crashes = scan_and_diagnose_crashes(config).await?;
    let report = crashes.into_iter().find(|c| c.id == report_id)
        .ok_or_else(|| AppError::NotFound(format!("Crash report '{}' not found", report_id)))?;

    match report.remediation_action.as_deref() {
        Some("disable_addon") => {
            if let Some(ref source) = report.suspected_source {
                let plugins_dir = config.minecraft_data_dir.join("plugins");
                let mut found = false;
                if let Ok(mut entries) = tokio::fs::read_dir(&plugins_dir).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let name = entry.file_name().to_string_lossy().to_lowercase();
                        if name.contains(&source.to_lowercase()) && name.ends_with(".jar") {
                            let old_path = entry.path();
                            let new_path = plugins_dir.join(format!("{}.disabled", entry.file_name().to_string_lossy()));
                            tokio::fs::rename(&old_path, &new_path).await
                                .map_err(|e| AppError::InternalError(format!("Failed renaming addon: {}", e)))?;
                            found = true;
                            info!("Auto-remediation: disabled offending plugin {:?}", new_path);
                            break;
                        }
                    }
                }
                if found {
                    return Ok(format!("Le plugin '{}' a été désactivé avec succès (.disabled).", source));
                }
            }
            Ok("Aucun plugin suspect n'a pu être isolé avec certitude pour désactivation automatique.".to_string())
        }
        Some("increase_ram") => {
            Ok("Recommandation appliquée : Veuillez ajuster les paramètres de mémoire dans l'onglet Configuration / Quadlet.".to_string())
        }
        Some("restore_backup") => {
            Ok("Restauration recommandée : Rendez-vous dans l'onglet Sauvegardes pour restaurer la dernière version saine.".to_string())
        }
        _ => Ok("Aucune action corrective automatisée spécifique pour ce type d'incident.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oom_classification() {
        let sample = r#"---- Minecraft Crash Report ----
Description: Exception in server tick loop

java.lang.OutOfMemoryError: Java heap space
	at java.base/java.util.Arrays.copyOf(Arrays.java:3537)
	at net.minecraft.world.level.chunk.LevelChunk.getBlockState(LevelChunk.java:120)
"#;
        let rep = parse_and_classify_crash_report("123", "crash-123.txt", 1700000000, sample);
        assert_eq!(rep.category, "OOM");
        assert_eq!(rep.severity, "CRITICAL");
        assert_eq!(rep.remediation_action.as_deref(), Some("increase_ram"));
    }

    #[test]
    fn test_chunk_corruption_classification() {
        let sample = r#"---- Minecraft Crash Report ----
Description: Exception reading chunk

net.minecraft.world.level.chunk.storage.CorruptChunkException: Wrong location in world: (12, -4)
	at net.minecraft.world.level.chunk.storage.RegionFile.read(RegionFile.java:140)
"#;
        let rep = parse_and_classify_crash_report("124", "crash-124.txt", 1700000000, sample);
        assert_eq!(rep.category, "CHUNK_CORRUPTION");
        assert_eq!(rep.remediation_action.as_deref(), Some("restore_backup"));
    }
}
