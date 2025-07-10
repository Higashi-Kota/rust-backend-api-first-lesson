pub mod dto;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod services;

// Public API exports for the team feature
// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除予定
#[allow(unused_imports)]
pub use handlers::team_router_with_state;
