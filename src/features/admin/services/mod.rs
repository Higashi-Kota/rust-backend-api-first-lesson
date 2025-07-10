//! Admin services module
//! 
//! 管理者向けの統合サービスを提供

pub mod admin;
pub mod analytics;

// Re-export for backward compatibility
// TODO: Phase 19で古い参照を削除後、#[allow]を削除予定
#[allow(unused_imports)]
pub use admin::AdminService;
#[allow(unused_imports)]
pub use analytics::AnalyticsService;