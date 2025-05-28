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
                    .table(Alias::new("tasks"))
                    .name("idx_tasks_status")
                    .col(Alias::new("status"))
                    .to_owned(),
            )
            .await?;

        // due_date カラムにインデックスを追加
        manager
            .create_index(
                Index::create()
                    .table(Alias::new("tasks"))
                    .name("idx_tasks_due_date")
                    .col(Alias::new("due_date"))
                    .to_owned(),
            )
            .await?;

        // created_at カラムにインデックスを追加
        manager
            .create_index(
                Index::create()
                    .table(Alias::new("tasks"))
                    .name("idx_tasks_created_at")
                    .col(Alias::new("created_at"))
                    .to_owned(),
            )
            .await?;

        // title カラムにインデックスを追加（部分一致検索用）
        manager
            .create_index(
                Index::create()
                    .table(Alias::new("tasks"))
                    .name("idx_tasks_title")
                    .col(Alias::new("title"))
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
                    .table(Alias::new("tasks"))
                    .name("idx_tasks_status")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(Alias::new("tasks"))
                    .name("idx_tasks_due_date")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(Alias::new("tasks"))
                    .name("idx_tasks_created_at")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .table(Alias::new("tasks"))
                    .name("idx_tasks_title")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
