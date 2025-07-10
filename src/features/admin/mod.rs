//! Admin feature module
//! 
//! 管理者向けの統合機能を提供

pub mod dto;
pub mod handlers;
pub mod services;
pub mod usecases;

// Re-export handlers for main.rs integration
// TODO: Phase 19で古い参照を削除後、#[allow]を削除予定
#[allow(unused_imports)]
pub use handlers::admin_router_with_state;