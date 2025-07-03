use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DailyActivitySummaries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DailyActivitySummaries::Date)
                            .date()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DailyActivitySummaries::TotalUsers)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(DailyActivitySummaries::ActiveUsers)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(DailyActivitySummaries::NewUsers)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(DailyActivitySummaries::TasksCreated)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(DailyActivitySummaries::TasksCompleted)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(DailyActivitySummaries::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .col(
                        ColumnDef::new(DailyActivitySummaries::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックスを作成（日付範囲検索用）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_daily_activity_summaries_date")
                    .table(DailyActivitySummaries::Table)
                    .col(DailyActivitySummaries::Date)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(DailyActivitySummaries::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum DailyActivitySummaries {
    Table,
    Date,
    TotalUsers,
    ActiveUsers,
    NewUsers,
    TasksCreated,
    TasksCompleted,
    CreatedAt,
    UpdatedAt,
}
