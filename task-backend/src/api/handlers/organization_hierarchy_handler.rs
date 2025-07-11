// Temporary stub for organization hierarchy handler
use crate::api::AppState;
use axum::Router;

pub fn organization_hierarchy_router(app_state: AppState) -> Router {
    Router::new().with_state(app_state)
}
