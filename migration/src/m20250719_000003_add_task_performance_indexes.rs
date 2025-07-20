use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // マルチテナントタスクのパフォーマンス最適化インデックス

        // 1. user_id + visibility + status の複合インデックス（個人タスクの高速検索）
        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_user_visibility_status")
                    .table(Tasks::Table)
                    .col(Tasks::UserId)
                    .col(Tasks::Visibility)
                    .col(Tasks::Status)
                    .to_owned(),
            )
            .await?;

        // 2. team_id + visibility + created_at の複合インデックス（チームタスクの日付順検索）
        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_team_visibility_created")
                    .table(Tasks::Table)
                    .col(Tasks::TeamId)
                    .col(Tasks::Visibility)
                    .col(Tasks::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // 3. organization_id + visibility + priority の複合インデックス（組織タスクの優先度別検索）
        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_org_visibility_priority")
                    .table(Tasks::Table)
                    .col(Tasks::OrganizationId)
                    .col(Tasks::Visibility)
                    .col(Tasks::Priority)
                    .to_owned(),
            )
            .await?;

        // 4. assigned_to + status + due_date の複合インデックス（割り当てタスクの期限管理）
        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_assigned_status_due")
                    .table(Tasks::Table)
                    .col(Tasks::AssignedTo)
                    .col(Tasks::Status)
                    .col(Tasks::DueDate)
                    .to_owned(),
            )
            .await?;

        // 5. visibility + updated_at の複合インデックス（最近更新されたタスクの検索）
        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_visibility_updated")
                    .table(Tasks::Table)
                    .col(Tasks::Visibility)
                    .col(Tasks::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        // 6. status + priority + created_at の複合インデックス（ステータス別・優先度別のソート）
        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_status_priority_created")
                    .table(Tasks::Table)
                    .col(Tasks::Status)
                    .col(Tasks::Priority)
                    .col(Tasks::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // 7. completed_at のインデックスは既にm20250706_000001で作成済みのためスキップ

        // 8. title と description のフルテキストインデックス（検索機能の高速化）
        // PostgreSQL専用のGINインデックスを使用
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_tasks_fulltext_search 
                ON tasks USING gin(to_tsvector('english', title || ' ' || COALESCE(description, '')))"
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // インデックスを削除（逆順）
        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_tasks_fulltext_search")
            .await?;

        // idx_tasks_completed_atはm20250706_000001で管理されているためスキップ

        manager
            .drop_index(
                Index::drop()
                    .name("idx_tasks_status_priority_created")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_tasks_visibility_updated")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_tasks_assigned_status_due")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_tasks_org_visibility_priority")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_tasks_team_visibility_created")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_tasks_user_visibility_status")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Tasks {
    Table,
    UserId,
    TeamId,
    OrganizationId,
    Visibility,
    Status,
    Priority,
    AssignedTo,
    DueDate,
    CreatedAt,
    UpdatedAt,
    // CompletedAt は m20250706_000001 で管理されているため削除
}
