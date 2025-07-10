// task-backend/src/features/security/handlers/mod.rs

pub mod permission;
pub mod role;
pub mod security;

// Re-export router functions
pub use permission::{permission_router, permission_router_with_state};
pub use role::{role_router, role_router_with_state};
pub use security::security_router;

use crate::api::AppState;
use axum::Router;

/// 統合されたセキュリティルーターを作成
pub fn security_router_with_state(app_state: AppState) -> Router {
    Router::new()
        .merge(security_router(app_state.clone()))
        .merge(role_router(app_state.clone()))
        .merge(permission_router(app_state))
}
