use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Users::Email)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Username)
                            .string_len(50)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::PasswordHash)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Users::EmailVerified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Users::LastLoginAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // emailとusernameにインデックスを作成（検索性能向上）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Users::Table)
                    .name("idx_users_email")
                    .col(Users::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Users::Table)
                    .name("idx_users_username")
                    .col(Users::Username)
                    .to_owned(),
            )
            .await?;

        // アクティブユーザーのインデックス（管理用クエリの性能向上）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Users::Table)
                    .name("idx_users_is_active")
                    .col(Users::IsActive)
                    .to_owned(),
            )
            .await?;

        // 作成日でのソート用インデックス
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Users::Table)
                    .name("idx_users_created_at")
                    .col(Users::CreatedAt)
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
                    .name("idx_users_created_at")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(Users::Table)
                    .name("idx_users_is_active")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(Users::Table)
                    .name("idx_users_username")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(Users::Table)
                    .name("idx_users_email")
                    .to_owned(),
            )
            .await?;

        // テーブルを削除
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    Username,
    PasswordHash,
    IsActive,
    EmailVerified,
    LastLoginAt,
    CreatedAt,
    UpdatedAt,
}
