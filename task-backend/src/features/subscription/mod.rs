pub mod dto;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod services;

// 公開APIの再エクスポート
// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除

// ハンドラー
// pub use handlers::{
//     admin_subscription_router, subscription_router_with_state,
// };

// サービス
// pub use services::SubscriptionService;

// リポジトリ
// pub use repositories::{
//     CreateStripeSubscription, StripeSubscriptionRepository, SubscriptionHistoryRepository,
//     UpdateStripeSubscription, UserSubscriptionStats,
// };

// モデル
// pub use models::{
//     StripeSubscription, StripeSubscriptionModel, SubscriptionChangeInfo, SubscriptionHistory,
//     SubscriptionHistoryModel, SubscriptionStatus,
// };
