use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfoResponse {
    pub environment: String,
    pub is_test: bool,
    pub is_production: bool,
    pub is_development: bool,
    pub version: String,
    pub build_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub checks: HealthChecks,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthChecks {
    pub database: HealthCheckItem,
    pub memory: HealthCheckItem,
    pub cpu: HealthCheckItem,
    pub disk: HealthCheckItem,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckItem {
    pub status: String,
    pub message: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMetricsResponse {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub swap: SwapMetrics,
    pub load: LoadMetrics,
    pub uptime_seconds: u64,
    pub disks: Vec<DiskMetrics>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage_percent: f32,
    pub core_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadMetrics {
    pub one_minute: f64,
    pub five_minutes: f64,
    pub fifteen_minutes: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub name: String,
    pub mount_point: String,
    pub available_bytes: u64,
    pub total_bytes: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationStatsResponse {
    pub users: EntityStats,
    pub organizations: EntityStats,
    pub teams: EntityStats,
    pub tasks: EntityStats,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityStats {
    pub total: u64,
    pub active: Option<u64>,
    pub inactive: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStatsResponse {
    pub hit_rate: f64,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub memory_usage_bytes: u64,
    pub entries_count: Option<u64>,
}
