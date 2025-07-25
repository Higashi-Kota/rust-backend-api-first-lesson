use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // CLAUDE.mdで要求されている単一カラムインデックスを追加
        // 複合インデックスは既にm20250719_000003で作成済み

        // 1. team_id の単一インデックス
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_tasks_team_id")
                    .table(Tasks::Table)
                    .col(Tasks::TeamId)
                    .to_owned(),
            )
            .await?;

        // 2. organization_id の単一インデックス
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_tasks_organization_id")
                    .table(Tasks::Table)
                    .col(Tasks::OrganizationId)
                    .to_owned(),
            )
            .await?;

        // 3. visibility の単一インデックス
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_tasks_visibility")
                    .table(Tasks::Table)
                    .col(Tasks::Visibility)
                    .to_owned(),
            )
            .await?;

        // 4. assigned_to の単一インデックス（割り当てられたタスクの高速検索用）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_tasks_assigned_to")
                    .table(Tasks::Table)
                    .col(Tasks::AssignedTo)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // インデックスを削除（逆順）
        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_tasks_assigned_to")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_tasks_visibility")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_tasks_organization_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_tasks_team_id")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

/// tasksテーブルへの参照
#[derive(DeriveIden)]
enum Tasks {
    Table,
    TeamId,
    OrganizationId,
    Visibility,
    AssignedTo,
}
