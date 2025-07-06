use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // password_reset_tokensテーブルにIPアドレス追跡フィールドを追加
        manager
            .alter_table(
                Table::alter()
                    .table(PasswordResetTokens::Table)
                    .add_column(
                        ColumnDef::new(PasswordResetTokens::IpAddress)
                            .string_len(45)
                            .not_null()
                            .default("0.0.0.0"),
                    )
                    .add_column(ColumnDef::new(PasswordResetTokens::UserAgent).text().null())
                    .add_column(
                        ColumnDef::new(PasswordResetTokens::RequestedFrom)
                            .string_len(50)
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックスを作成
        manager
            .create_index(
                Index::create()
                    .name("idx_password_reset_tokens_ip_address")
                    .table(PasswordResetTokens::Table)
                    .col(PasswordResetTokens::IpAddress)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_password_reset_tokens_created_at_ip")
                    .table(PasswordResetTokens::Table)
                    .col(PasswordResetTokens::CreatedAt)
                    .col(PasswordResetTokens::IpAddress)
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
                    .name("idx_password_reset_tokens_created_at_ip")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_password_reset_tokens_ip_address")
                    .to_owned(),
            )
            .await?;

        // カラムを削除
        manager
            .alter_table(
                Table::alter()
                    .table(PasswordResetTokens::Table)
                    .drop_column(PasswordResetTokens::RequestedFrom)
                    .drop_column(PasswordResetTokens::UserAgent)
                    .drop_column(PasswordResetTokens::IpAddress)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum PasswordResetTokens {
    Table,
    IpAddress,
    UserAgent,
    RequestedFrom,
    CreatedAt,
}
