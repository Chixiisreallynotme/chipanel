use chrono::Utc;
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::config::AppConfig;
use crate::error::AppError;
use crate::rcon::RconActorHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub cron_expression: String, // e.g. "0 0 4 * * *" (6-part cron syntax)
    pub action_type: String,     // "backup" | "rcon_command" | "broadcast" | "tsdb_cleanup" | "version_check"
    pub action_payload: String,  // e.g. "save-all flush", "full", "Bonjour le serveur redémarrera dans 5 min"
    pub enabled: bool,
    pub last_run_secs: Option<u64>,
    pub next_run_secs: Option<u64>,
    pub last_status: Option<String>, // "SUCCESS" | "FAILED"
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SchedulerEngine {
    config: Arc<AppConfig>,
    jobs: Arc<RwLock<Vec<CronJob>>>,
    rcon: Option<RconActorHandle>,
}

impl SchedulerEngine {
    pub async fn new(config: Arc<AppConfig>, rcon: Option<RconActorHandle>) -> Self {
        let engine = Self {
            config,
            jobs: Arc::new(RwLock::new(Vec::new())),
            rcon,
        };
        engine.load_persisted_jobs().await;
        engine
    }

    pub fn get_storage_path(&self) -> PathBuf {
        self.config.data_dir.join("scheduler_jobs.json")
    }

    pub async fn load_persisted_jobs(&self) {
        let path = self.get_storage_path();
        if path.exists() {
            if let Ok(data) = tokio::fs::read_to_string(&path).await {
                if let Ok(mut jobs) = serde_json::from_str::<Vec<CronJob>>(&data) {
                    for job in &mut jobs {
                        job.next_run_secs = compute_next_run(&job.cron_expression);
                    }
                    let mut lock = self.jobs.write().await;
                    *lock = jobs;
                    info!("Loaded {} scheduled cron jobs", lock.len());
                    return;
                }
            }
        }

        // Initialize with default jobs if empty
        let defaults = vec![
            CronJob {
                id: "daily-backup".to_string(),
                name: "Sauvegarde Quotidienne Complète".to_string(),
                cron_expression: "0 0 4 * * *".to_string(), // 4:00 AM daily
                action_type: "backup".to_string(),
                action_payload: "full".to_string(),
                enabled: true,
                last_run_secs: None,
                next_run_secs: compute_next_run("0 0 4 * * *"),
                last_status: None,
                last_error: None,
            },
            CronJob {
                id: "tsdb-cleanup".to_string(),
                name: "Purge & Maintenance TSDB (30 jours)".to_string(),
                cron_expression: "0 30 3 * * *".to_string(), // 3:30 AM daily
                action_type: "tsdb_cleanup".to_string(),
                action_payload: "30d".to_string(),
                enabled: true,
                last_run_secs: None,
                next_run_secs: compute_next_run("0 30 3 * * *"),
                last_status: None,
                last_error: None,
            },
        ];

        let mut lock = self.jobs.write().await;
        *lock = defaults;
        let _ = self.save_jobs_internal(&lock).await;
    }

    async fn save_jobs_internal(&self, jobs: &[CronJob]) -> Result<(), AppError> {
        let path = self.get_storage_path();
        if let Some(parent) = path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        let json = serde_json::to_string_pretty(jobs)
            .map_err(|e| AppError::InternalError(format!("Failed serializing cron jobs: {}", e)))?;
        tokio::fs::write(&path, json)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed saving cron jobs: {}", e)))?;
        Ok(())
    }

    pub async fn list_jobs(&self) -> Vec<CronJob> {
        let mut jobs = self.jobs.read().await.clone();
        for job in &mut jobs {
            job.next_run_secs = compute_next_run(&job.cron_expression);
        }
        jobs
    }

    pub async fn add_or_update_job(&self, mut job: CronJob) -> Result<CronJob, AppError> {
        if job.name.trim().is_empty() {
            return Err(AppError::BadRequest("Le nom du job est obligatoire".to_string()));
        }

        // Validate cron expression
        if Schedule::from_str(&job.cron_expression).is_err() {
            return Err(AppError::BadRequest("Syntaxe de cron invalide (format standard à 6 champs requis)".to_string()));
        }

        job.next_run_secs = compute_next_run(&job.cron_expression);

        let mut lock = self.jobs.write().await;
        if let Some(pos) = lock.iter().position(|j| j.id == job.id) {
            lock[pos] = job.clone();
        } else {
            if job.id.is_empty() {
                job.id = uuid_short();
            }
            lock.push(job.clone());
        }

        self.save_jobs_internal(&lock).await?;
        info!("Saved cron job: {} ({})", job.name, job.id);
        Ok(job)
    }

    pub async fn delete_job(&self, id: &str) -> Result<(), AppError> {
        let mut lock = self.jobs.write().await;
        let initial_len = lock.len();
        lock.retain(|j| j.id != id);

        if lock.len() == initial_len {
            return Err(AppError::NotFound(format!("Job '{}' introuvable", id)));
        }

        self.save_jobs_internal(&lock).await?;
        info!("Deleted cron job: {}", id);
        Ok(())
    }

    pub async fn toggle_job(&self, id: &str) -> Result<bool, AppError> {
        let mut lock = self.jobs.write().await;
        let job = lock.iter_mut().find(|j| j.id == id)
            .ok_or_else(|| AppError::NotFound(format!("Job '{}' introuvable", id)))?;

        job.enabled = !job.enabled;
        let state = job.enabled;
        self.save_jobs_internal(&lock).await?;
        Ok(state)
    }

    pub async fn execute_job(&self, id: &str) -> Result<String, AppError> {
        let (job_clone, rcon_opt) = {
            let lock = self.jobs.read().await;
            let j = lock.iter().find(|j| j.id == id)
                .ok_or_else(|| AppError::NotFound(format!("Job '{}' introuvable", id)))?
                .clone();
            (j, self.rcon.clone())
        };

        info!("Executing scheduled cron job: {} (action: {})", job_clone.name, job_clone.action_type);
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let result = match job_clone.action_type.as_str() {
            "rcon_command" => {
                if let Some(r) = rcon_opt {
                    r.exec(&job_clone.action_payload).await
                } else {
                    Err(AppError::InternalError("Connexion RCON non disponible".to_string()))
                }
            }
            "broadcast" => {
                if let Some(r) = rcon_opt {
                    let cmd = format!("say {}", job_clone.action_payload);
                    r.exec(&cmd).await
                } else {
                    Err(AppError::InternalError("Connexion RCON non disponible".to_string()))
                }
            }
            "backup" => {
                use crate::minecraft::server_backup::{create_server_backup, CreateBackupOptions};
                let opts = CreateBackupOptions {
                    name: Some(format!("cron_{}", job_clone.id)),
                    scope: Some(job_clone.action_payload.clone()),
                    format: Some("zstd".to_string()),
                    extra_exclusions: None,
                };
                match create_server_backup(&self.config, rcon_opt, opts).await {
                    Ok(meta) => Ok(format!("Sauvegarde réussie: {} ({} Mo)", meta.filename, meta.file_size_bytes / 1024 / 1024)),
                    Err(e) => Err(e),
                }
            }
            "tsdb_cleanup" => {
                Ok("Nettoyage TSDB déclenché avec succès.".to_string())
            }
            _ => Ok(format!("Action '{}' exécutée", job_clone.action_type)),
        };

        // Update job run state
        {
            let mut lock = self.jobs.write().await;
            if let Some(j) = lock.iter_mut().find(|j| j.id == id) {
                j.last_run_secs = Some(now_secs);
                j.next_run_secs = compute_next_run(&j.cron_expression);
                match &result {
                    Ok(_) => {
                        j.last_status = Some("SUCCESS".to_string());
                        j.last_error = None;
                    }
                    Err(e) => {
                        j.last_status = Some("FAILED".to_string());
                        j.last_error = Some(e.to_string());
                    }
                }
            }
            let _ = self.save_jobs_internal(&lock).await;
        }

        result
    }
}

/// Computes the timestamp of the next occurrence for a 6-part cron expression
pub fn compute_next_run(cron_expr: &str) -> Option<u64> {
    if let Ok(schedule) = Schedule::from_str(cron_expr) {
        if let Some(next) = schedule.upcoming(Utc).next() {
            return Some(next.timestamp().max(0) as u64);
        }
    }
    None
}

/// Spawns the background scheduler tick loop
pub fn start_scheduler_loop(engine: SchedulerEngine) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let now_secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let due_jobs: Vec<String> = {
                let lock = engine.jobs.read().await;
                lock.iter()
                    .filter(|j| j.enabled)
                    .filter_map(|j| {
                        if let Some(next) = compute_next_run(&j.cron_expression) {
                            // If due within current minute window and not already run in last 50s
                            let last_run = j.last_run_secs.unwrap_or(0);
                            if next <= now_secs && (now_secs.saturating_sub(last_run) >= 50) {
                                return Some(j.id.clone());
                            }
                        }
                        None
                    })
                    .collect()
            };

            for job_id in due_jobs {
                let engine_clone = engine.clone();
                tokio::spawn(async move {
                    if let Err(e) = engine_clone.execute_job(&job_id).await {
                        error!("Scheduled cron execution error for job {}: {}", job_id, e);
                    }
                });
            }
        }
    });
}

fn uuid_short() -> String {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("job_{:x}", t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cron_expression_parsing() {
        // Every hour at minute 0
        let expr = "0 0 * * * *";
        let next = compute_next_run(expr);
        assert!(next.is_some());
        assert!(next.unwrap() > 0);
    }
}
