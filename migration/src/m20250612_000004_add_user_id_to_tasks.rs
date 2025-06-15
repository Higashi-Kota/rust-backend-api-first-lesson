use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // タスクテーブルにuser_idカラムを追加
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .add_column(
                        ColumnDef::new(Tasks::UserId).uuid().null(), // 最初はnullableにして、既存データとの互換性を保つ
                    )
                    .to_owned(),
            )
            .await?;

        // 外部キー制約を追加
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_tasks_user_id")
                    .from(Tasks::Table, Tasks::UserId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // user_id検索用インデックスを作成
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_user_id")
                    .col(Tasks::UserId)
                    .to_owned(),
            )
            .await?;

        // ユーザー別のステータス検索用複合インデックス
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_user_status")
                    .col(Tasks::UserId)
                    .col(Tasks::Status)
                    .to_owned(),
            )
            .await?;

        // ユーザー別の作成日順ソート用複合インデックス
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_user_created_at")
                    .col(Tasks::UserId)
                    .col(Tasks::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // ユーザー別の期日順ソート用複合インデックス（NULL値も含む）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_user_due_date")
                    .col(Tasks::UserId)
                    .col(Tasks::DueDate)
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
                    .name("idx_tasks_user_due_date")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_user_created_at")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_user_status")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(Tasks::Table)
                    .name("idx_tasks_user_id")
                    .to_owned(),
            )
            .await?;

        // 外部キー制約を削除
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_tasks_user_id")
                    .table(Tasks::Table)
                    .to_owned(),
            )
            .await?;

        // user_idカラムを削除
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .drop_column(Tasks::UserId)
                    .to_owned(),
            )
            .await
    }
}

/// Iden enum for the tasks table
#[derive(DeriveIden)]
enum Tasks {
    Table,
    UserId,
    Status,
    CreatedAt,
    DueDate,
}

/// Reference to the users table for foreign key
#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
