use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // login_attemptsテーブル作成
        manager
            .create_table(
                Table::create()
                    .table(LoginAttempts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LoginAttempts::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(LoginAttempts::Email)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(LoginAttempts::UserId).uuid().null())
                    .col(ColumnDef::new(LoginAttempts::Success).boolean().not_null())
                    .col(
                        ColumnDef::new(LoginAttempts::FailureReason)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(LoginAttempts::IpAddress)
                            .string_len(45)
                            .not_null(),
                    )
                    .col(ColumnDef::new(LoginAttempts::UserAgent).text().null())
                    .col(
                        ColumnDef::new(LoginAttempts::AttemptedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_login_attempts_user_id")
                            .from(LoginAttempts::Table, LoginAttempts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックス作成
        manager
            .create_index(
                Index::create()
                    .name("idx_login_attempts_email")
                    .table(LoginAttempts::Table)
                    .col(LoginAttempts::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_login_attempts_user_id")
                    .table(LoginAttempts::Table)
                    .col(LoginAttempts::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_login_attempts_attempted_at")
                    .table(LoginAttempts::Table)
                    .col(LoginAttempts::AttemptedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_login_attempts_ip_address")
                    .table(LoginAttempts::Table)
                    .col(LoginAttempts::IpAddress)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LoginAttempts::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum LoginAttempts {
    Table,
    Id,
    Email,
    UserId,
    Success,
    FailureReason,
    IpAddress,
    UserAgent,
    AttemptedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
