use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // refresh_tokensテーブルにセッション分析フィールドを追加
        manager
            .alter_table(
                Table::alter()
                    .table(RefreshTokens::Table)
                    .add_column(
                        ColumnDef::new(RefreshTokens::LastUsedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(RefreshTokens::UseCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(RefreshTokens::IpAddress)
                            .string_len(45)
                            .null(),
                    )
                    .add_column(ColumnDef::new(RefreshTokens::UserAgent).text().null())
                    .add_column(
                        ColumnDef::new(RefreshTokens::DeviceType)
                            .string_len(50)
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(RefreshTokens::GeolocationCountry)
                            .string_len(100)
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックスを作成
        manager
            .create_index(
                Index::create()
                    .name("idx_refresh_tokens_last_used_at")
                    .table(RefreshTokens::Table)
                    .col(RefreshTokens::LastUsedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_refresh_tokens_device_type")
                    .table(RefreshTokens::Table)
                    .col(RefreshTokens::DeviceType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_refresh_tokens_user_id_last_used")
                    .table(RefreshTokens::Table)
                    .col(RefreshTokens::UserId)
                    .col(RefreshTokens::LastUsedAt)
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
                    .name("idx_refresh_tokens_user_id_last_used")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_refresh_tokens_device_type")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_refresh_tokens_last_used_at")
                    .to_owned(),
            )
            .await?;

        // カラムを削除
        manager
            .alter_table(
                Table::alter()
                    .table(RefreshTokens::Table)
                    .drop_column(RefreshTokens::GeolocationCountry)
                    .drop_column(RefreshTokens::DeviceType)
                    .drop_column(RefreshTokens::UserAgent)
                    .drop_column(RefreshTokens::IpAddress)
                    .drop_column(RefreshTokens::UseCount)
                    .drop_column(RefreshTokens::LastUsedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum RefreshTokens {
    Table,
    LastUsedAt,
    UseCount,
    IpAddress,
    UserAgent,
    DeviceType,
    GeolocationCountry,
    UserId,
}
