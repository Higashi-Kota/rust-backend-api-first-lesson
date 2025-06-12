use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PasswordResetTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PasswordResetTokens::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PasswordResetTokens::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PasswordResetTokens::TokenHash)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(PasswordResetTokens::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PasswordResetTokens::IsUsed)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(PasswordResetTokens::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(PasswordResetTokens::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_password_reset_tokens_user_id")
                            .from(PasswordResetTokens::Table, PasswordResetTokens::UserId)
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
                    .table(PasswordResetTokens::Table)
                    .name("idx_password_reset_tokens_user_id")
                    .col(PasswordResetTokens::UserId)
                    .to_owned(),
            )
            .await?;

        // トークンハッシュ検索用インデックス（リセット時の高速検索）
        manager
            .create_index(
                Index::create()
                    .table(PasswordResetTokens::Table)
                    .name("idx_password_reset_tokens_token_hash")
                    .col(PasswordResetTokens::TokenHash)
                    .to_owned(),
            )
            .await?;

        // 有効期限でのクリーンアップ用インデックス
        manager
            .create_index(
                Index::create()
                    .table(PasswordResetTokens::Table)
                    .name("idx_password_reset_tokens_expires_at")
                    .col(PasswordResetTokens::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        // 有効なトークンの検索用複合インデックス（未使用かつ未期限切れ）
        manager
            .create_index(
                Index::create()
                    .table(PasswordResetTokens::Table)
                    .name("idx_password_reset_tokens_valid")
                    .col(PasswordResetTokens::IsUsed)
                    .col(PasswordResetTokens::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        // ユーザー別の有効なトークン検索用複合インデックス
        manager
            .create_index(
                Index::create()
                    .table(PasswordResetTokens::Table)
                    .name("idx_password_reset_tokens_user_valid")
                    .col(PasswordResetTokens::UserId)
                    .col(PasswordResetTokens::IsUsed)
                    .col(PasswordResetTokens::ExpiresAt)
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
                    .table(PasswordResetTokens::Table)
                    .name("idx_password_reset_tokens_user_valid")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(PasswordResetTokens::Table)
                    .name("idx_password_reset_tokens_valid")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(PasswordResetTokens::Table)
                    .name("idx_password_reset_tokens_expires_at")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(PasswordResetTokens::Table)
                    .name("idx_password_reset_tokens_token_hash")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(PasswordResetTokens::Table)
                    .name("idx_password_reset_tokens_user_id")
                    .to_owned(),
            )
            .await?;

        // テーブルを削除
        manager
            .drop_table(Table::drop().table(PasswordResetTokens::Table).to_owned())
            .await
    }
}

/// Iden enum for the password_reset_tokens table
#[derive(DeriveIden)]
enum PasswordResetTokens {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    IsUsed,
    CreatedAt,
    UpdatedAt,
}

/// Reference to the users table for foreign key
#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
