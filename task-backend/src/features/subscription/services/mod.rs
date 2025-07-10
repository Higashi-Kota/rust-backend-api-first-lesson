// task-backend/src/features/subscription/services/mod.rs

pub mod subscription;

// 公開APIの再エクスポート
// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除
#[allow(unused_imports)]
pub use subscription::SubscriptionService;
