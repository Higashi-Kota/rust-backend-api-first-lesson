use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // subscription_history テーブルを作成
        manager
            .create_table(
                Table::create()
                    .table(SubscriptionHistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SubscriptionHistory::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionHistory::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionHistory::PreviousTier)
                            .string_len(20)
                            .null(), // 初回登録時はnull
                    )
                    .col(
                        ColumnDef::new(SubscriptionHistory::NewTier)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionHistory::ChangedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SubscriptionHistory::ChangedBy).uuid().null(), // システム変更の場合はnull
                    )
                    .col(
                        ColumnDef::new(SubscriptionHistory::Reason)
                            .string_len(255)
                            .null(), // 変更理由（オプション）
                    )
                    .col(
                        ColumnDef::new(SubscriptionHistory::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subscription_history_user_id")
                            .from(SubscriptionHistory::Table, SubscriptionHistory::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade), // ユーザー削除時に履歴も削除
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subscription_history_changed_by")
                            .from(SubscriptionHistory::Table, SubscriptionHistory::ChangedBy)
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
                    .table(SubscriptionHistory::Table)
                    .name("idx_subscription_history_user_id")
                    .col(SubscriptionHistory::UserId)
                    .to_owned(),
            )
            .await?;

        // 変更日時のインデックス（時系列検索用）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(SubscriptionHistory::Table)
                    .name("idx_subscription_history_changed_at")
                    .col(SubscriptionHistory::ChangedAt)
                    .to_owned(),
            )
            .await?;

        // 複合インデックス: user_id + changed_at（ユーザーの履歴を時系列で取得）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(SubscriptionHistory::Table)
                    .name("idx_subscription_history_user_time")
                    .col(SubscriptionHistory::UserId)
                    .col(SubscriptionHistory::ChangedAt)
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
                    .table(SubscriptionHistory::Table)
                    .name("idx_subscription_history_user_time")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(SubscriptionHistory::Table)
                    .name("idx_subscription_history_changed_at")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(SubscriptionHistory::Table)
                    .name("idx_subscription_history_user_id")
                    .to_owned(),
            )
            .await?;

        // テーブルを削除
        manager
            .drop_table(Table::drop().table(SubscriptionHistory::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(DeriveIden)]
enum SubscriptionHistory {
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
