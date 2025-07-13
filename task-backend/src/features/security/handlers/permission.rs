// task-backend/src/features/security/handlers/permission.rs

use crate::api::AppState;
use axum::Router;

/// Permission management routes (minimal for backward compatibility)
pub fn permission_routes() -> Router<AppState> {
    // Routes are now handled by permission_handler::permission_router()
    Router::new()
}
