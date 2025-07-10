// task-backend/src/features/subscription/handlers/mod.rs

pub mod subscription;

// 公開APIの再エクスポート
// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除
#[allow(unused_imports)]
pub use subscription::{
    admin_subscription_router, subscription_router, subscription_router_with_state,
    AdminStatsQuery, PeriodQuery, UuidPath,
};
