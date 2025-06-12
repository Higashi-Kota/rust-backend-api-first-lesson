use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RefreshTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RefreshTokens::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RefreshTokens::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(RefreshTokens::TokenHash)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(RefreshTokens::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RefreshTokens::IsRevoked)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(RefreshTokens::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(RefreshTokens::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_refresh_tokens_user_id")
                            .from(RefreshTokens::Table, RefreshTokens::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ユーザー別のトークン検索用インデックス
        manager
            .create_index(
                Index::create()
                    .table(RefreshTokens::Table)
                    .name("idx_refresh_tokens_user_id")
                    .col(RefreshTokens::UserId)
                    .to_owned(),
            )
            .await?;

        // トークンハッシュ検索用インデックス（認証時の高速検索）
        manager
            .create_index(
                Index::create()
                    .table(RefreshTokens::Table)
                    .name("idx_refresh_tokens_token_hash")
                    .col(RefreshTokens::TokenHash)
                    .to_owned(),
            )
            .await?;

        // 有効期限でのクリーンアップ用インデックス
        manager
            .create_index(
                Index::create()
                    .table(RefreshTokens::Table)
                    .name("idx_refresh_tokens_expires_at")
                    .col(RefreshTokens::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        // 有効なトークンの検索用複合インデックス
        manager
            .create_index(
                Index::create()
                    .table(RefreshTokens::Table)
                    .name("idx_refresh_tokens_active")
                    .col(RefreshTokens::IsRevoked)
                    .col(RefreshTokens::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        // ユーザー別の有効なトークン検索用複合インデックス
        manager
            .create_index(
                Index::create()
                    .table(RefreshTokens::Table)
                    .name("idx_refresh_tokens_user_active")
                    .col(RefreshTokens::UserId)
                    .col(RefreshTokens::IsRevoked)
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
                    .table(RefreshTokens::Table)
                    .name("idx_refresh_tokens_user_active")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(RefreshTokens::Table)
                    .name("idx_refresh_tokens_active")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(RefreshTokens::Table)
                    .name("idx_refresh_tokens_expires_at")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(RefreshTokens::Table)
                    .name("idx_refresh_tokens_token_hash")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(RefreshTokens::Table)
                    .name("idx_refresh_tokens_user_id")
                    .to_owned(),
            )
            .await?;

        // テーブルを削除
        manager
            .drop_table(Table::drop().table(RefreshTokens::Table).to_owned())
            .await
    }
}

/// Iden enum for the refresh_tokens table
#[derive(DeriveIden)]
enum RefreshTokens {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    IsRevoked,
    CreatedAt,
    UpdatedAt,
}

/// Reference to the users table for foreign key
#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
