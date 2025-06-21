use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // subscription_histories テーブルを作成
        manager
            .create_table(
                Table::create()
                    .table(SubscriptionHistories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SubscriptionHistories::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionHistories::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionHistories::PreviousTier)
                            .string_len(20)
                            .null(), // 初回登録時はnull
                    )
                    .col(
                        ColumnDef::new(SubscriptionHistories::NewTier)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionHistories::ChangedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SubscriptionHistories::ChangedBy)
                            .uuid()
                            .null(), // システム変更の場合はnull
                    )
                    .col(
                        ColumnDef::new(SubscriptionHistories::Reason)
                            .string_len(255)
                            .null(), // 変更理由（オプション）
                    )
                    .col(
                        ColumnDef::new(SubscriptionHistories::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subscription_histories_user_id")
                            .from(SubscriptionHistories::Table, SubscriptionHistories::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade), // ユーザー削除時に履歴も削除
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subscription_histories_changed_by")
                            .from(
                                SubscriptionHistories::Table,
                                SubscriptionHistories::ChangedBy,
                            )
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull), // 変更者削除時はnullに設定
                    )
                    .to_owned(),
            )
            .await?;

        // ユーザーIDのインデックス（履歴検索用）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(SubscriptionHistories::Table)
                    .name("idx_subscription_histories_user_id")
                    .col(SubscriptionHistories::UserId)
                    .to_owned(),
            )
            .await?;

        // 変更日時のインデックス（時系列検索用）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(SubscriptionHistories::Table)
                    .name("idx_subscription_histories_changed_at")
                    .col(SubscriptionHistories::ChangedAt)
                    .to_owned(),
            )
            .await?;

        // 複合インデックス: user_id + changed_at（ユーザーの履歴を時系列で取得）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(SubscriptionHistories::Table)
                    .name("idx_subscription_histories_user_time")
                    .col(SubscriptionHistories::UserId)
                    .col(SubscriptionHistories::ChangedAt)
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
                    .table(SubscriptionHistories::Table)
                    .name("idx_subscription_histories_user_time")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(SubscriptionHistories::Table)
                    .name("idx_subscription_histories_changed_at")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(SubscriptionHistories::Table)
                    .name("idx_subscription_histories_user_id")
                    .to_owned(),
            )
            .await?;

        // テーブルを削除
        manager
            .drop_table(Table::drop().table(SubscriptionHistories::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(DeriveIden)]
enum SubscriptionHistories {
    Table,
    Id,
    UserId,
    PreviousTier,
    NewTier,
    ChangedAt,
    ChangedBy,
    Reason,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
