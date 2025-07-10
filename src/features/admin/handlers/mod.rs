//! Admin handlers module
//! 
//! 管理者向けのHTTPハンドラー実装

pub mod admin;
pub mod analytics;

use axum::Router;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

// Re-export handlers
// TODO: Phase 19で古い参照を削除後、#[allow]を削除予定
#[allow(unused_imports)]
pub use admin::*;
#[allow(unused_imports)]
pub use analytics::*;

/// Create admin router with database connection
/// 
/// 管理者向けの統合ルーターを作成
pub fn admin_router_with_state(db: Arc<DatabaseConnection>) -> Router {
    Router::new()
        .nest("/admin", admin::create_admin_routes(db.clone()))
        .nest("/analytics", analytics::create_analytics_routes(db))
}