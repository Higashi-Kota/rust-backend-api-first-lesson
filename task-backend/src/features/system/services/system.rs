use crate::error::AppError;
use sea_orm::prelude::*;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use sysinfo::{CpuExt, DiskExt, System, SystemExt};

pub struct SystemService;

impl SystemService {
    /// システムヘルスチェック
    pub async fn health_check(db: &DatabaseConnection) -> Result<HealthStatus, AppError> {
        // データベース接続チェック
        let db_healthy = match db.ping().await {
            Ok(_) => true,
            Err(_) => false,
        };

        // システムリソースチェック
        let mut sys = System::new_all();
        sys.refresh_all();

        let memory_usage = (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0;
        let cpu_usage = sys.global_cpu_info().cpu_usage();

        Ok(HealthStatus {
            status: if db_healthy && memory_usage < 90.0 && cpu_usage < 90.0 {
                "healthy".to_string()
            } else {
                "degraded".to_string()
            },
            database: if db_healthy {
                "connected"
            } else {
                "disconnected"
            }
            .to_string(),
            memory_usage_percent: memory_usage,
            cpu_usage_percent: cpu_usage,
            timestamp: chrono::Utc::now(),
        })
    }

    /// システムメトリクスの取得
    pub async fn get_system_metrics() -> Result<SystemMetrics, AppError> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let disks: Vec<DiskMetrics> = sys
            .disks()
            .iter()
            .map(|disk| DiskMetrics {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                available_space: disk.available_space(),
                total_space: disk.total_space(),
                usage_percent: ((disk.total_space() - disk.available_space()) as f64
                    / disk.total_space() as f64)
                    * 100.0,
            })
            .collect();

        Ok(SystemMetrics {
            cpu_usage: sys.global_cpu_info().cpu_usage(),
            memory_total: sys.total_memory(),
            memory_used: sys.used_memory(),
            memory_available: sys.available_memory(),
            swap_total: sys.total_swap(),
            swap_used: sys.used_swap(),
            load_average: LoadAverage {
                one: sys.load_average().one,
                five: sys.load_average().five,
                fifteen: sys.load_average().fifteen,
            },
            uptime: sys.uptime(),
            disks,
        })
    }

    /// アプリケーション統計の取得
    pub async fn get_application_stats(
        db: &DatabaseConnection,
    ) -> Result<ApplicationStats, AppError> {
        use crate::features::organization::models::organization::Entity as Organization;
        use crate::features::task::models::task_model::Entity as Task;
        use crate::features::team::models::team::Entity as Team;
        use crate::features::user::models::user::Entity as User;
        use sea_orm::{EntityTrait, QuerySelect};

        let total_users = User::find().count(db).await?;
        let total_organizations = Organization::find().count(db).await?;
        let total_teams = Team::find().count(db).await?;
        let total_tasks = Task::find().count(db).await?;

        Ok(ApplicationStats {
            total_users,
            total_organizations,
            total_teams,
            total_tasks,
            timestamp: chrono::Utc::now(),
        })
    }

    /// キャッシュ統計の取得（将来の実装用）
    pub async fn get_cache_stats() -> Result<CacheStats, AppError> {
        // TODO: Redis等のキャッシュシステムと統合時に実装
        Ok(CacheStats {
            hits: 0,
            misses: 0,
            evictions: 0,
            memory_usage: 0,
        })
    }
}

// DTOs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub database: String,
    pub memory_usage_percent: f64,
    pub cpu_usage_percent: f32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub load_average: LoadAverage,
    pub uptime: u64,
    pub disks: Vec<DiskMetrics>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub name: String,
    pub mount_point: String,
    pub available_space: u64,
    pub total_space: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationStats {
    pub total_users: u64,
    pub total_organizations: u64,
    pub total_teams: u64,
    pub total_tasks: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadAverage {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub memory_usage: u64,
}
