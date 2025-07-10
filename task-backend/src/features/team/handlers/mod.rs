pub mod team;
pub mod team_invitation;

use crate::api::AppState;
use axum::Router;

/// Combines team and team_invitation routes into a single router
pub fn team_router_with_state(app_state: AppState) -> Router {
    team::team_router_with_state(app_state)
}
