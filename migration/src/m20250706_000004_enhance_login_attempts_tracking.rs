use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // login_attemptsテーブルに拡張追跡フィールドを追加
        manager
            .alter_table(
                Table::alter()
                    .table(LoginAttempts::Table)
                    .add_column(
                        ColumnDef::new(LoginAttempts::DeviceType)
                            .string_len(50)
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(LoginAttempts::BrowserName)
                            .string_len(100)
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(LoginAttempts::Country)
                            .string_len(100)
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(LoginAttempts::SuspiciousScore)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックスを作成
        manager
            .create_index(
                Index::create()
                    .name("idx_login_attempts_suspicious")
                    .table(LoginAttempts::Table)
                    .col(LoginAttempts::IpAddress)
                    .col(LoginAttempts::Success)
                    .col(LoginAttempts::AttemptedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_login_attempts_country")
                    .table(LoginAttempts::Table)
                    .col(LoginAttempts::Country)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // インデックスを削除
        manager
            .drop_index(Index::drop().name("idx_login_attempts_country").to_owned())
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_login_attempts_suspicious")
                    .to_owned(),
            )
            .await?;

        // カラムを削除
        manager
            .alter_table(
                Table::alter()
                    .table(LoginAttempts::Table)
                    .drop_column(LoginAttempts::SuspiciousScore)
                    .drop_column(LoginAttempts::Country)
                    .drop_column(LoginAttempts::BrowserName)
                    .drop_column(LoginAttempts::DeviceType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum LoginAttempts {
    Table,
    DeviceType,
    BrowserName,
    Country,
    SuspiciousScore,
    IpAddress,
    Success,
    AttemptedAt,
}
