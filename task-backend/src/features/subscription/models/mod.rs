// task-backend/src/features/subscription/models/mod.rs

pub mod history;
pub mod stripe_subscription;

// 公開APIの再エクスポート
// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除
#[allow(unused_imports)]
pub use history::{
    ActiveModel as SubscriptionHistoryActiveModel, ActiveModelBehavior,
    Column as SubscriptionHistoryColumn, Entity as SubscriptionHistory,
    Model as SubscriptionHistoryModel, PrimaryKey as SubscriptionHistoryPrimaryKey,
    Relation as SubscriptionHistoryRelation, SubscriptionChangeInfo,
};

#[allow(unused_imports)]
pub use stripe_subscription::{
    ActiveModel as StripeSubscriptionActiveModel, Column as StripeSubscriptionColumn,
    Entity as StripeSubscription, Model as StripeSubscriptionModel,
    PrimaryKey as StripeSubscriptionPrimaryKey, Relation as StripeSubscriptionRelation,
    SubscriptionStatus,
};
