use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(EmailVerificationToken::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EmailVerificationToken::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()".to_owned()),
                    )
                    .col(
                        ColumnDef::new(EmailVerificationToken::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EmailVerificationToken::TokenHash)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(EmailVerificationToken::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EmailVerificationToken::IsUsed)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(EmailVerificationToken::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(EmailVerificationToken::UsedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_email_verification_tokens_user_id")
                            .from(
                                EmailVerificationToken::Table,
                                EmailVerificationToken::UserId,
                            )
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックス作成
        manager
            .create_index(
                Index::create()
                    .name("idx_email_verification_tokens_user_id")
                    .table(EmailVerificationToken::Table)
                    .col(EmailVerificationToken::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_email_verification_tokens_expires_at")
                    .table(EmailVerificationToken::Table)
                    .col(EmailVerificationToken::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(EmailVerificationToken::Table)
                    .to_owned(),
            )
            .await
    }
}

enum EmailVerificationToken {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    IsUsed,
    CreatedAt,
    UsedAt,
}

impl Iden for EmailVerificationToken {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "email_verification_tokens",
                Self::Id => "id",
                Self::UserId => "user_id",
                Self::TokenHash => "token_hash",
                Self::ExpiresAt => "expires_at",
                Self::IsUsed => "is_used",
                Self::CreatedAt => "created_at",
                Self::UsedAt => "used_at",
            }
        )
        .unwrap();
    }
}

enum User {
    Table,
    Id,
}

impl Iden for User {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "users",
                Self::Id => "id",
            }
        )
        .unwrap();
    }
}
