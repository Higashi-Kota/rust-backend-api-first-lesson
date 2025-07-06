use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // タスク分析サマリーテーブルを作成
        manager
            .create_table(
                Table::create()
                    .table(TaskAnalyticsSummaries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaskAnalyticsSummaries::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TaskAnalyticsSummaries::UserId).uuid().null(), // NULLの場合は全体統計
                    )
                    .col(
                        ColumnDef::new(TaskAnalyticsSummaries::Date)
                            .date()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaskAnalyticsSummaries::TasksCreated)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(TaskAnalyticsSummaries::TasksCompleted)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(TaskAnalyticsSummaries::HighPriorityCompleted)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(TaskAnalyticsSummaries::MediumPriorityCompleted)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(TaskAnalyticsSummaries::LowPriorityCompleted)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(TaskAnalyticsSummaries::AverageCompletionHours)
                            .double()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TaskAnalyticsSummaries::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TaskAnalyticsSummaries::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_analytics_user")
                            .from(
                                TaskAnalyticsSummaries::Table,
                                TaskAnalyticsSummaries::UserId,
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
                    .name("idx_task_analytics_user_date_unique")
                    .table(TaskAnalyticsSummaries::Table)
                    .col(TaskAnalyticsSummaries::UserId)
                    .col(TaskAnalyticsSummaries::Date)
                    .to_owned(),
            )
            .await?;

        // インデックスを作成
        manager
            .create_index(
                Index::create()
                    .name("idx_task_analytics_date")
                    .table(TaskAnalyticsSummaries::Table)
                    .col(TaskAnalyticsSummaries::Date)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(TaskAnalyticsSummaries::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum TaskAnalyticsSummaries {
    Table,
    Id,
    UserId,
    Date,
    TasksCreated,
    TasksCompleted,
    HighPriorityCompleted,
    MediumPriorityCompleted,
    LowPriorityCompleted,
    AverageCompletionHours,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
