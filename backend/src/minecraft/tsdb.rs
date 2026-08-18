use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsdbSample {
    pub timestamp: u64,
    pub server_id: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub memory_limit_bytes: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    pub tps: Option<f32>,
    pub player_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsdbPoint {
    pub timestamp: u64,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub memory_limit_bytes: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    pub tps: Option<f32>,
    pub player_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsdbHistoryResponse {
    pub server_id: String,
    pub range: String,
    pub resolution: String,
    pub points: Vec<TsdbPoint>,
}

#[derive(Clone)]
pub struct TsdbEngine {
    #[allow(dead_code)]
    pub db_path: PathBuf,
    writer_conn: Arc<Mutex<Option<Connection>>>,
}

impl TsdbEngine {
    pub fn new(data_dir: &Path) -> Result<Self, AppError> {
        let db_dir = data_dir.join("db");
        std::fs::create_dir_all(&db_dir).map_err(|e| {
            AppError::InternalError(format!("Failed to create telemetry DB dir: {}", e))
        })?;

        let db_path = db_dir.join("telemetry.db");
        let conn = Connection::open(&db_path).map_err(|e| {
            AppError::InternalError(format!("Failed to open telemetry SQLite DB: {}", e))
        })?;

        // Configure WAL mode and tight 2 MB cache
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA cache_size = -2000;
             PRAGMA temp_store = MEMORY;",
        )
        .map_err(|e| AppError::InternalError(format!("Failed to set SQLite pragmas: {}", e)))?;

        // Initialize schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS metrics_raw (
                timestamp INTEGER NOT NULL,
                server_id TEXT NOT NULL,
                cpu_percent REAL NOT NULL,
                memory_bytes INTEGER NOT NULL,
                memory_limit_bytes INTEGER NOT NULL,
                disk_read_bytes INTEGER NOT NULL,
                disk_write_bytes INTEGER NOT NULL,
                net_rx_bytes INTEGER NOT NULL,
                net_tx_bytes INTEGER NOT NULL,
                tps REAL,
                player_count INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_metrics_ts ON metrics_raw (server_id, timestamp);",
        )
        .map_err(|e| AppError::InternalError(format!("Failed to create metrics_raw table: {}", e)))?;

        info!("SQLite TSDB telemetry engine initialized at {:?}", db_path);

        Ok(Self {
            db_path,
            writer_conn: Arc::new(Mutex::new(Some(conn))),
        })
    }

    pub fn insert_sample(&self, sample: &TsdbSample) -> Result<(), AppError> {
        let guard = self.writer_conn.lock().map_err(|_| {
            AppError::InternalError("TSDB writer lock poisoned".to_string())
        })?;

        if let Some(ref conn) = *guard {
            conn.execute(
                "INSERT INTO metrics_raw (
                    timestamp, server_id, cpu_percent, memory_bytes, memory_limit_bytes,
                    disk_read_bytes, disk_write_bytes, net_rx_bytes, net_tx_bytes,
                    tps, player_count
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    sample.timestamp as i64,
                    sample.server_id,
                    sample.cpu_percent as f64,
                    sample.memory_bytes as i64,
                    sample.memory_limit_bytes as i64,
                    sample.disk_read_bytes as i64,
                    sample.disk_write_bytes as i64,
                    sample.net_rx_bytes as i64,
                    sample.net_tx_bytes as i64,
                    sample.tps.map(|v| v as f64),
                    sample.player_count.map(|v| v as i64),
                ],
            )
            .map_err(|e| AppError::InternalError(format!("Failed to insert metric sample: {}", e)))?;
        }

        Ok(())
    }

    pub fn query_history(
        &self,
        server_id: &str,
        range: &str,
        resolution: Option<&str>,
    ) -> Result<TsdbHistoryResponse, AppError> {
        let seconds_back: u64 = match range {
            "30d" => 30 * 86400,
            "7d" => 7 * 86400,
            "24h" => 86400,
            "6h" => 21600,
            "1h" => 3600,
            _ => 3600,
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cutoff = now.saturating_sub(seconds_back);

        let bucket_seconds = match resolution {
            Some("1h") => 3600,
            Some("1m") => 60,
            Some("raw") => 5,
            _ => match range {
                "30d" => 3600,
                "7d" => 300,
                "24h" => 60,
                _ => 5,
            },
        };

        let guard = self.writer_conn.lock().map_err(|_| {
            AppError::InternalError("TSDB lock poisoned".to_string())
        })?;

        let conn = guard.as_ref().ok_or_else(|| {
            AppError::InternalError("TSDB connection unavailable".to_string())
        })?;

        let mut points = Vec::new();

        if bucket_seconds <= 5 {
            let mut stmt = conn
                .prepare(
                    "SELECT timestamp, cpu_percent, memory_bytes, memory_limit_bytes,
                            disk_read_bytes, disk_write_bytes, net_rx_bytes, net_tx_bytes,
                            tps, player_count
                     FROM metrics_raw
                     WHERE server_id = ?1 AND timestamp >= ?2
                     ORDER BY timestamp ASC",
                )
                .map_err(|e| AppError::InternalError(format!("Query prep failed: {}", e)))?;

            let rows = stmt
                .query_map(params![server_id, cutoff as i64], |row| {
                    Ok(TsdbPoint {
                        timestamp: row.get::<_, i64>(0)? as u64,
                        cpu_percent: row.get::<_, f64>(1)? as f32,
                        memory_bytes: row.get::<_, i64>(2)? as u64,
                        memory_limit_bytes: row.get::<_, i64>(3)? as u64,
                        disk_read_bytes: row.get::<_, i64>(4)? as u64,
                        disk_write_bytes: row.get::<_, i64>(5)? as u64,
                        net_rx_bytes: row.get::<_, i64>(6)? as u64,
                        net_tx_bytes: row.get::<_, i64>(7)? as u64,
                        tps: row.get::<_, Option<f64>>(8)?.map(|v| v as f32),
                        player_count: row.get::<_, Option<i64>>(9)?.map(|v| v as u32),
                    })
                })
                .map_err(|e| AppError::InternalError(format!("Query failed: {}", e)))?;

            for r in rows {
                if let Ok(p) = r {
                    points.push(p);
                }
            }
        } else {
            let mut stmt = conn
                .prepare(
                    "SELECT (timestamp / ?3) * ?3 as bucket_ts,
                            AVG(cpu_percent),
                            CAST(AVG(memory_bytes) AS INTEGER),
                            CAST(MAX(memory_limit_bytes) AS INTEGER),
                            CAST(AVG(disk_read_bytes) AS INTEGER),
                            CAST(AVG(disk_write_bytes) AS INTEGER),
                            CAST(AVG(net_rx_bytes) AS INTEGER),
                            CAST(AVG(net_tx_bytes) AS INTEGER),
                            AVG(tps),
                            CAST(MAX(player_count) AS INTEGER)
                     FROM metrics_raw
                     WHERE server_id = ?1 AND timestamp >= ?2
                     GROUP BY bucket_ts
                     ORDER BY bucket_ts ASC",
                )
                .map_err(|e| AppError::InternalError(format!("Query prep failed: {}", e)))?;

            let rows = stmt
                .query_map(params![server_id, cutoff as i64, bucket_seconds as i64], |row| {
                    Ok(TsdbPoint {
                        timestamp: row.get::<_, i64>(0)? as u64,
                        cpu_percent: row.get::<_, f64>(1)? as f32,
                        memory_bytes: row.get::<_, i64>(2)? as u64,
                        memory_limit_bytes: row.get::<_, i64>(3)? as u64,
                        disk_read_bytes: row.get::<_, i64>(4)? as u64,
                        disk_write_bytes: row.get::<_, i64>(5)? as u64,
                        net_rx_bytes: row.get::<_, i64>(6)? as u64,
                        net_tx_bytes: row.get::<_, i64>(7)? as u64,
                        tps: row.get::<_, Option<f64>>(8)?.map(|v| v as f32),
                        player_count: row.get::<_, Option<i64>>(9)?.map(|v| v as u32),
                    })
                })
                .map_err(|e| AppError::InternalError(format!("Grouped query failed: {}", e)))?;

            for r in rows {
                if let Ok(p) = r {
                    points.push(p);
                }
            }
        }

        Ok(TsdbHistoryResponse {
            server_id: server_id.to_string(),
            range: range.to_string(),
            resolution: if bucket_seconds <= 5 { "raw".to_string() } else { format!("{}s", bucket_seconds) },
            points,
        })
    }

    pub fn prune_old_records(&self, retention_days: u32) -> Result<usize, AppError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cutoff = now.saturating_sub((retention_days as u64) * 86400);

        let guard = self.writer_conn.lock().map_err(|_| {
            AppError::InternalError("TSDB lock poisoned".to_string())
        })?;

        if let Some(ref conn) = *guard {
            let deleted = conn
                .execute("DELETE FROM metrics_raw WHERE timestamp < ?1", params![cutoff as i64])
                .map_err(|e| AppError::InternalError(format!("Failed to prune old metrics: {}", e)))?;
            return Ok(deleted);
        }

        Ok(0)
    }

    pub fn format_prometheus_metrics(&self, sample: &Option<TsdbSample>) -> String {
        let mut out = String::with_capacity(1024);
        out.push_str("# HELP chipanel_cpu_usage_percent CPU utilization percentage\n");
        out.push_str("# TYPE chipanel_cpu_usage_percent gauge\n");
        if let Some(s) = sample {
            out.push_str(&format!("chipanel_cpu_usage_percent{{server=\"{}\"}} {:.2}\n", s.server_id, s.cpu_percent));
        }

        out.push_str("# HELP chipanel_memory_used_bytes Memory used in bytes\n");
        out.push_str("# TYPE chipanel_memory_used_bytes gauge\n");
        if let Some(s) = sample {
            out.push_str(&format!("chipanel_memory_used_bytes{{server=\"{}\"}} {}\n", s.server_id, s.memory_bytes));
        }

        out.push_str("# HELP chipanel_memory_limit_bytes Memory limit in bytes\n");
        out.push_str("# TYPE chipanel_memory_limit_bytes gauge\n");
        if let Some(s) = sample {
            out.push_str(&format!("chipanel_memory_limit_bytes{{server=\"{}\"}} {}\n", s.server_id, s.memory_limit_bytes));
        }

        out.push_str("# HELP chipanel_tps Current game server ticks per second\n");
        out.push_str("# TYPE chipanel_tps gauge\n");
        if let Some(s) = sample {
            if let Some(tps) = s.tps {
                out.push_str(&format!("chipanel_tps{{server=\"{}\"}} {:.2}\n", s.server_id, tps));
            }
        }

        out.push_str("# HELP chipanel_online_players Number of players currently connected\n");
        out.push_str("# TYPE chipanel_online_players gauge\n");
        if let Some(s) = sample {
            if let Some(players) = s.player_count {
                out.push_str(&format!("chipanel_online_players{{server=\"{}\"}} {}\n", s.server_id, players));
            }
        }

        out
    }
}

pub fn start_tsdb_maintenance_loop(tsdb: Arc<TsdbEngine>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Run hourly
        loop {
            interval.tick().await;
            match tsdb.prune_old_records(30) {
                Ok(n) if n > 0 => info!("TSDB maintenance: pruned {} records older than 30 days", n),
                Ok(_) => {}
                Err(e) => warn!("TSDB maintenance prune error: {}", e),
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tsdb_insert_and_query() {
        let temp_dir = std::env::temp_dir().join(format!("chipanel_test_tsdb_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);

        let engine = TsdbEngine::new(&temp_dir).expect("Failed to create TSDB");
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let sample1 = TsdbSample {
            timestamp: now,
            server_id: "test-server".to_string(),
            cpu_percent: 15.5,
            memory_bytes: 1024 * 1024 * 500,
            memory_limit_bytes: 1024 * 1024 * 4096,
            disk_read_bytes: 1000,
            disk_write_bytes: 2000,
            net_rx_bytes: 500,
            net_tx_bytes: 1200,
            tps: Some(20.0),
            player_count: Some(3),
        };

        engine.insert_sample(&sample1).expect("Insert sample failed");

        let res = engine.query_history("test-server", "30d", Some("raw")).expect("Query failed");
        assert_eq!(res.points.len(), 1);
        assert_eq!(res.points[0].cpu_percent, 15.5);
        assert_eq!(res.points[0].player_count, Some(3));

        let prom = engine.format_prometheus_metrics(&Some(sample1));
        assert!(prom.contains("chipanel_cpu_usage_percent"));
        assert!(prom.contains("15.50"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
