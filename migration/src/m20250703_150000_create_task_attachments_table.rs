use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // task_attachmentsテーブルを作成
        manager
            .create_table(
                Table::create()
                    .table(TaskAttachments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaskAttachments::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(ColumnDef::new(TaskAttachments::TaskId).uuid().not_null())
                    .col(
                        ColumnDef::new(TaskAttachments::UploadedBy)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaskAttachments::FileName)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaskAttachments::FileSize)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaskAttachments::MimeType)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaskAttachments::StorageKey)
                            .string_len(500)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(TaskAttachments::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TaskAttachments::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 外部キー制約を追加 - タスクへの参照
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_task_attachment_task")
                    .from(TaskAttachments::Table, TaskAttachments::TaskId)
                    .to(Tasks::Table, Tasks::Id)
                    .on_delete(ForeignKeyAction::Cascade) // タスクが削除されたら添付ファイルも削除
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // 外部キー制約を追加 - ユーザーへの参照
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_task_attachment_user")
                    .from(TaskAttachments::Table, TaskAttachments::UploadedBy)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Restrict) // ユーザーが削除されても添付ファイルは保持
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // インデックスを作成 - タスクIDで検索
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(TaskAttachments::Table)
                    .name("idx_task_attachments_task_id")
                    .col(TaskAttachments::TaskId)
                    .to_owned(),
            )
            .await?;

        // インデックスを作成 - アップロードユーザーで検索
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(TaskAttachments::Table)
                    .name("idx_task_attachments_uploaded_by")
                    .col(TaskAttachments::UploadedBy)
                    .to_owned(),
            )
            .await?;

        // 複合インデックス - タスクとアップロード日時での検索
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(TaskAttachments::Table)
                    .name("idx_task_attachments_task_created")
                    .col(TaskAttachments::TaskId)
                    .col(TaskAttachments::CreatedAt)
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
                    .table(TaskAttachments::Table)
                    .name("idx_task_attachments_task_created")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(TaskAttachments::Table)
                    .name("idx_task_attachments_uploaded_by")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(TaskAttachments::Table)
                    .name("idx_task_attachments_task_id")
                    .to_owned(),
            )
            .await?;

        // 外部キー制約を削除
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_task_attachment_user")
                    .table(TaskAttachments::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_task_attachment_task")
                    .table(TaskAttachments::Table)
                    .to_owned(),
            )
            .await?;

        // テーブルを削除
        manager
            .drop_table(Table::drop().table(TaskAttachments::Table).to_owned())
            .await
    }
}

/// Iden enum for the task_attachments table
#[derive(DeriveIden)]
enum TaskAttachments {
    Table,
    Id,
    TaskId,
    UploadedBy,
    FileName,
    FileSize,
    MimeType,
    StorageKey,
    CreatedAt,
    UpdatedAt,
}

/// Reference to the tasks table for foreign key
#[derive(DeriveIden)]
enum Tasks {
    Table,
    Id,
}

/// Reference to the users table for foreign key
#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
