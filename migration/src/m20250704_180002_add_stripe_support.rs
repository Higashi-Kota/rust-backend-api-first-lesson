use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 既存のusersテーブルにStripe顧客IDカラムを追加
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::StripeCustomerId)
                            .string()
                            .unique_key()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Stripeサブスクリプション詳細テーブル
        manager
            .create_table(
                Table::create()
                    .table(StripeSubscriptions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StripeSubscriptions::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::StripeSubscriptionId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::StripePriceId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::Status)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::CurrentPeriodStart)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::CurrentPeriodEnd)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::CancelAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::CanceledAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_stripe_subscriptions_user_id")
                            .from(StripeSubscriptions::Table, StripeSubscriptions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Stripe支払い履歴テーブル
        manager
            .create_table(
                Table::create()
                    .table(StripePaymentHistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StripePaymentHistory::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(
                        ColumnDef::new(StripePaymentHistory::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StripePaymentHistory::StripePaymentIntentId)
                            .string()
                            .unique_key()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripePaymentHistory::StripeInvoiceId)
                            .string()
                            .unique_key()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripePaymentHistory::Amount)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StripePaymentHistory::Currency)
                            .string()
                            .not_null()
                            .default("jpy"),
                    )
                    .col(
                        ColumnDef::new(StripePaymentHistory::Status)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StripePaymentHistory::Description)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripePaymentHistory::PaidAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripePaymentHistory::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_stripe_payment_history_user_id")
                            .from(StripePaymentHistory::Table, StripePaymentHistory::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックス作成
        manager
            .create_index(
                Index::create()
                    .name("idx_stripe_subscriptions_user_id")
                    .table(StripeSubscriptions::Table)
                    .col(StripeSubscriptions::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_stripe_subscriptions_status")
                    .table(StripeSubscriptions::Table)
                    .col(StripeSubscriptions::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_stripe_payment_history_user_id")
                    .table(StripePaymentHistory::Table)
                    .col(StripePaymentHistory::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_stripe_payment_history_created_at")
                    .table(StripePaymentHistory::Table)
                    .col(StripePaymentHistory::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // テーブルを削除（逆順）
        manager
            .drop_table(Table::drop().table(StripePaymentHistory::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(StripeSubscriptions::Table).to_owned())
            .await?;

        // usersテーブルからカラムを削除
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::StripeCustomerId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    StripeCustomerId,
}

#[derive(Iden)]
enum StripeSubscriptions {
    Table,
    Id,
    UserId,
    StripeSubscriptionId,
    StripePriceId,
    Status,
    CurrentPeriodStart,
    CurrentPeriodEnd,
    CancelAt,
    CanceledAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum StripePaymentHistory {
    Table,
    Id,
    UserId,
    StripePaymentIntentId,
    StripeInvoiceId,
    Amount,
    Currency,
    Status,
    Description,
    PaidAt,
    CreatedAt,
}
