use axum::{extract::State, routing::get, Json, Router};
use std::sync::Arc;

use super::super::dto::responses::{
    ApplicationStatsResponse, HealthCheckItem, HealthCheckResponse, HealthChecks,
    SystemInfoResponse, SystemMetricsResponse,
};
use super::super::services::system::SystemService;
use crate::{api::AppState, error::AppError, shared::types::common::ApiResponse};

pub async fn get_system_info(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<SystemInfoResponse>>, AppError> {
    let config = &app_state.config;

    Ok(Json(ApiResponse::success(
        "System information retrieved",
        SystemInfoResponse {
            environment: config.environment.clone(),
            is_test: config.is_test(),
            is_production: config.is_production(),
            is_development: config.is_development(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_time: option_env!("BUILD_TIME").map(|s| s.to_string()),
        },
    )))
}

pub async fn health_check(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<HealthCheckResponse>>, AppError> {
    let health_status = SystemService::health_check(&app_state.db).await?;

    let response = HealthCheckResponse {
        status: health_status.status.clone(),
        checks: HealthChecks {
            database: HealthCheckItem {
                status: health_status.database.clone(),
                message: None,
                details: None,
            },
            memory: HealthCheckItem {
                status: if health_status.memory_usage_percent < 90.0 {
                    "healthy"
                } else {
                    "warning"
                }
                .to_string(),
                message: Some(format!(
                    "Memory usage: {:.2}%",
                    health_status.memory_usage_percent
                )),
                details: None,
            },
            cpu: HealthCheckItem {
                status: if health_status.cpu_usage_percent < 90.0 {
                    "healthy"
                } else {
                    "warning"
                }
                .to_string(),
                message: Some(format!(
                    "CPU usage: {:.2}%",
                    health_status.cpu_usage_percent
                )),
                details: None,
            },
            disk: HealthCheckItem {
                status: "healthy".to_string(), // Will be updated when disk checks are implemented
                message: None,
                details: None,
            },
        },
        timestamp: health_status.timestamp,
    };

    Ok(Json(ApiResponse::success(
        "Health check completed",
        response,
    )))
}

pub async fn get_system_metrics(
    State(_app_state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<SystemMetricsResponse>>, AppError> {
    let metrics = SystemService::get_system_metrics().await?;

    use super::super::dto::responses::{CpuMetrics, LoadMetrics, MemoryMetrics, SwapMetrics};

    let response = SystemMetricsResponse {
        cpu: CpuMetrics {
            usage_percent: metrics.cpu_usage,
            core_count: num_cpus::get(),
        },
        memory: MemoryMetrics {
            total_bytes: metrics.memory_total,
            used_bytes: metrics.memory_used,
            available_bytes: metrics.memory_available,
            usage_percent: (metrics.memory_used as f64 / metrics.memory_total as f64) * 100.0,
        },
        swap: SwapMetrics {
            total_bytes: metrics.swap_total,
            used_bytes: metrics.swap_used,
            usage_percent: if metrics.swap_total > 0 {
                (metrics.swap_used as f64 / metrics.swap_total as f64) * 100.0
            } else {
                0.0
            },
        },
        load: LoadMetrics {
            one_minute: metrics.load_average.one,
            five_minutes: metrics.load_average.five,
            fifteen_minutes: metrics.load_average.fifteen,
        },
        uptime_seconds: metrics.uptime,
        disks: metrics
            .disks
            .into_iter()
            .map(|d| super::super::dto::responses::DiskMetrics {
                name: d.name,
                mount_point: d.mount_point,
                available_bytes: d.available_space,
                total_bytes: d.total_space,
                usage_percent: d.usage_percent,
            })
            .collect(),
    };

    Ok(Json(ApiResponse::success(
        "System metrics retrieved",
        response,
    )))
}

pub async fn get_application_stats(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<ApplicationStatsResponse>>, AppError> {
    let stats = SystemService::get_application_stats(&app_state.db).await?;

    use super::super::dto::responses::EntityStats;

    let response = ApplicationStatsResponse {
        users: EntityStats {
            total: stats.total_users,
            active: None, // TODO: Implement active/inactive counts
            inactive: None,
        },
        organizations: EntityStats {
            total: stats.total_organizations,
            active: None,
            inactive: None,
        },
        teams: EntityStats {
            total: stats.total_teams,
            active: None,
            inactive: None,
        },
        tasks: EntityStats {
            total: stats.total_tasks,
            active: None,
            inactive: None,
        },
        timestamp: stats.timestamp,
    };

    Ok(Json(ApiResponse::success(
        "Application statistics retrieved",
        response,
    )))
}

pub fn system_router_with_state(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/admin/system/info", get(get_system_info))
        .route("/admin/system/health", get(health_check))
        .route("/admin/system/metrics", get(get_system_metrics))
        .route("/admin/system/stats", get(get_application_stats))
        .with_state(app_state)
}
