use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // tasksテーブルに優先度と完了追跡フィールドを追加
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .add_column(
                        ColumnDef::new(Tasks::Priority)
                            .string_len(20)
                            .not_null()
                            .default("medium"),
                    )
                    .add_column(
                        ColumnDef::new(Tasks::CompletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(Tasks::CompletionDurationHours)
                            .double()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックスを作成
        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_priority")
                    .table(Tasks::Table)
                    .col(Tasks::Priority)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_completed_at")
                    .table(Tasks::Table)
                    .col(Tasks::CompletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_status_completed_at")
                    .table(Tasks::Table)
                    .col(Tasks::Status)
                    .col(Tasks::CompletedAt)
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
                    .name("idx_tasks_status_completed_at")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(Index::drop().name("idx_tasks_completed_at").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_tasks_priority").to_owned())
            .await?;

        // カラムを削除
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .drop_column(Tasks::CompletionDurationHours)
                    .drop_column(Tasks::CompletedAt)
                    .drop_column(Tasks::Priority)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Tasks {
    Table,
    Priority,
    CompletedAt,
    CompletionDurationHours,
    Status,
}
