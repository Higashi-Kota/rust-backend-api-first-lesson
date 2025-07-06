use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // セッション分析サマリーテーブルを作成
        manager
            .create_table(
                Table::create()
                    .table(SessionAnalyticsSummaries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SessionAnalyticsSummaries::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SessionAnalyticsSummaries::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SessionAnalyticsSummaries::Date)
                            .date()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SessionAnalyticsSummaries::TotalSessionMinutes)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SessionAnalyticsSummaries::LoginCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SessionAnalyticsSummaries::ActiveDeviceCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SessionAnalyticsSummaries::PrimaryDeviceType)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SessionAnalyticsSummaries::PrimaryCountry)
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SessionAnalyticsSummaries::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SessionAnalyticsSummaries::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_session_analytics_user")
                            .from(
                                SessionAnalyticsSummaries::Table,
                                SessionAnalyticsSummaries::UserId,
                            )
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ユニーク制約を追加
        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx_session_analytics_user_date_unique")
                    .table(SessionAnalyticsSummaries::Table)
                    .col(SessionAnalyticsSummaries::UserId)
                    .col(SessionAnalyticsSummaries::Date)
                    .to_owned(),
            )
            .await?;

        // インデックスを作成
        manager
            .create_index(
                Index::create()
                    .name("idx_session_analytics_date")
                    .table(SessionAnalyticsSummaries::Table)
                    .col(SessionAnalyticsSummaries::Date)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(SessionAnalyticsSummaries::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum SessionAnalyticsSummaries {
    Table,
    Id,
    UserId,
    Date,
    TotalSessionMinutes,
    LoginCount,
    ActiveDeviceCount,
    PrimaryDeviceType,
    PrimaryCountry,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
