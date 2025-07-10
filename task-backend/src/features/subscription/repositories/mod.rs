// task-backend/src/features/subscription/repositories/mod.rs

pub mod history;
pub mod stripe_subscription;

// 公開APIの再エクスポート
// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除
#[allow(unused_imports)]
pub use history::{SubscriptionHistoryRepository, UserSubscriptionStats};

#[allow(unused_imports)]
pub use stripe_subscription::{
    CreateStripeSubscription, StripeSubscriptionRepository, UpdateStripeSubscription,
};
