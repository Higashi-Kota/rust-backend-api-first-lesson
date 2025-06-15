use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // status カラムにインデックスを追加
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_status")
                    .col(Tasks::Status)
                    .to_owned(),
            )
            .await?;

        // due_date カラムにインデックスを追加
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_due_date")
                    .col(Tasks::DueDate)
                    .to_owned(),
            )
            .await?;

        // created_at カラムにインデックスを追加
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_created_at")
                    .col(Tasks::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // title カラムにインデックスを追加（部分一致検索用）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_title")
                    .col(Tasks::Title)
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
                    .if_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_status")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_due_date")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_created_at")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_title")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

/// Reference to the tasks table
#[derive(DeriveIden)]
enum Tasks {
    Table,
    Status,
    DueDate,
    CreatedAt,
    Title,
}
