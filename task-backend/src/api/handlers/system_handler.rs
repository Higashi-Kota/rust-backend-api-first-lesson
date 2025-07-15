use axum::{extract::State, routing::get, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{api::AppState, error::AppError, types::ApiResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfoResponse {
    pub environment: String,
    pub is_test: bool,
    pub is_production: bool,
    pub is_development: bool,
}

pub async fn get_system_info(
    State(app_state): State<Arc<AppState>>,
) -> Result<ApiResponse<SystemInfoResponse>, AppError> {
    let config = &app_state.config;

    Ok(ApiResponse::success(SystemInfoResponse {
        environment: config.environment.clone(),
        is_test: config.is_test(),
        is_production: config.is_production(),
        is_development: config.is_development(),
    }))
}

pub fn system_router_with_state(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/admin/system/info", get(get_system_info))
        .with_state(app_state)
}
