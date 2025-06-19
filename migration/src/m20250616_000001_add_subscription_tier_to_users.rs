use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // subscription_tier 列を users テーブルに追加
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::SubscriptionTier)
                            .string_len(20)
                            .not_null()
                            .default("free"), // デフォルトはFree階層
                    )
                    .to_owned(),
            )
            .await?;

        // subscription_tier のインデックスを作成（権限チェック高速化）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Users::Table)
                    .name("idx_users_subscription_tier")
                    .col(Users::SubscriptionTier)
                    .to_owned(),
            )
            .await?;

        // 複合インデックス: is_active + subscription_tier（動的権限クエリ最適化）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Users::Table)
                    .name("idx_users_active_subscription")
                    .col(Users::IsActive)
                    .col(Users::SubscriptionTier)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // インデックスを削除
        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(Users::Table)
                    .name("idx_users_active_subscription")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(Users::Table)
                    .name("idx_users_subscription_tier")
                    .to_owned(),
            )
            .await?;

        // subscription_tier 列を削除
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::SubscriptionTier)
                    .to_owned(),
            )
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(DeriveIden)]
enum Users {
    Table,
    SubscriptionTier,
    IsActive,
}
