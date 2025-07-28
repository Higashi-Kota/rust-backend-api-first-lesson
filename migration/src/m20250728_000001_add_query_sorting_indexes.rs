use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Payment History のソート用インデックス
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_payment_history_user_paid_at")
                    .table(StripePaymentHistory::Table)
                    .col(StripePaymentHistory::UserId)
                    .col(StripePaymentHistory::PaidAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_payment_history_user_amount")
                    .table(StripePaymentHistory::Table)
                    .col(StripePaymentHistory::UserId)
                    .col(StripePaymentHistory::Amount)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_payment_history_user_status")
                    .table(StripePaymentHistory::Table)
                    .col(StripePaymentHistory::UserId)
                    .col(StripePaymentHistory::Status)
                    .to_owned(),
            )
            .await?;

        // Subscription History のソート用インデックス
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_subscription_history_user_changed_at")
                    .table(SubscriptionHistories::Table)
                    .col(SubscriptionHistories::UserId)
                    .col(SubscriptionHistories::ChangedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_subscription_history_user_tiers")
                    .table(SubscriptionHistories::Table)
                    .col(SubscriptionHistories::UserId)
                    .col(SubscriptionHistories::PreviousTier)
                    .col(SubscriptionHistories::NewTier)
                    .to_owned(),
            )
            .await?;

        // Activity Log の既存インデックスを拡張（resource_type + action の複合インデックス）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_activity_logs_resource_action")
                    .table(ActivityLogs::Table)
                    .col(ActivityLogs::ResourceType)
                    .col(ActivityLogs::Action)
                    .col(ActivityLogs::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Audit Log のソート用インデックス
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_audit_logs_action_created")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::Action)
                    .col(AuditLogs::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_audit_logs_resource_type_created")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::ResourceType)
                    .col(AuditLogs::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Payment History インデックスの削除
        manager
            .drop_index(
                Index::drop()
                    .name("idx_payment_history_user_paid_at")
                    .table(StripePaymentHistory::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_payment_history_user_amount")
                    .table(StripePaymentHistory::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_payment_history_user_status")
                    .table(StripePaymentHistory::Table)
                    .to_owned(),
            )
            .await?;

        // Subscription History インデックスの削除
        manager
            .drop_index(
                Index::drop()
                    .name("idx_subscription_history_user_changed_at")
                    .table(SubscriptionHistories::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_subscription_history_user_tiers")
                    .table(SubscriptionHistories::Table)
                    .to_owned(),
            )
            .await?;

        // Activity Log インデックスの削除
        manager
            .drop_index(
                Index::drop()
                    .name("idx_activity_logs_resource_action")
                    .table(ActivityLogs::Table)
                    .to_owned(),
            )
            .await?;

        // Audit Log インデックスの削除
        manager
            .drop_index(
                Index::drop()
                    .name("idx_audit_logs_action_created")
                    .table(AuditLogs::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_audit_logs_resource_type_created")
                    .table(AuditLogs::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum StripePaymentHistory {
    Table,
    UserId,
    PaidAt,
    Amount,
    Status,
}

#[derive(DeriveIden)]
enum SubscriptionHistories {
    Table,
    UserId,
    ChangedAt,
    PreviousTier,
    NewTier,
}

#[derive(DeriveIden)]
enum ActivityLogs {
    Table,
    ResourceType,
    Action,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AuditLogs {
    Table,
    Action,
    ResourceType,
    CreatedAt,
}
