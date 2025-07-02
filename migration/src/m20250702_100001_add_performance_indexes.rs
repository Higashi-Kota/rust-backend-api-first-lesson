use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 組織名検索用インデックス（トライグラム）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_organizations_name_search")
                    .table(Organizations::Table)
                    .col(Organizations::Name)
                    .to_owned(),
            )
            .await?;

        // ユーザー検索用インデックス
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_email_search")
                    .table(Users::Table)
                    .col(Users::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_username_search")
                    .table(Users::Table)
                    .col(Users::Username)
                    .to_owned(),
            )
            .await?;

        // 機能使用状況の集計用インデックスは
        // m20250702_000001_create_feature_usage_metrics_table.rsで既に作成されているためスキップ

        // チームメンバー検索用インデックス
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_team_members_team_user")
                    .table(TeamMembers::Table)
                    .col(TeamMembers::TeamId)
                    .col(TeamMembers::UserId)
                    .to_owned(),
            )
            .await?;

        // 組織メンバー検索用インデックス
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_organization_members_org_user")
                    .table(OrganizationMembers::Table)
                    .col(OrganizationMembers::OrganizationId)
                    .col(OrganizationMembers::UserId)
                    .to_owned(),
            )
            .await?;

        // タスク検索用インデックス（user_idとstatusの複合インデックス）
        // 既存のidx_tasks_user_statusがあるかチェック
        let index_exists_query =
            "SELECT 1 FROM pg_indexes WHERE indexname = 'idx_tasks_user_status'";
        let index_exists = manager
            .get_connection()
            .execute_unprepared(index_exists_query)
            .await
            .is_ok();

        if !index_exists {
            manager
                .create_index(
                    Index::create()
                        .if_not_exists()
                        .name("idx_tasks_user_status")
                        .table(Tasks::Table)
                        .col(Tasks::UserId)
                        .col(Tasks::Status)
                        .to_owned(),
                )
                .await?;
        }

        // idx_tasks_due_dateは既にm20250512_000001_add_task_indexes.rsで作成されているためスキップ

        // 一括操作履歴の検索用インデックス
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_bulk_operation_histories_performed_by")
                    .table(BulkOperationHistories::Table)
                    .col(BulkOperationHistories::PerformedBy)
                    .col(BulkOperationHistories::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_bulk_operation_histories_operation_type")
                    .table(BulkOperationHistories::Table)
                    .col(BulkOperationHistories::OperationType)
                    .col(BulkOperationHistories::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // 日次活動サマリーの日付インデックス（既にプライマリキーになっているが、明示的に追加）
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

        // Trigram indexes are commented out due to pg_trgm extension complexity in test environments
        // In production, these indexes should be created manually after ensuring pg_trgm is installed:
        // CREATE INDEX idx_organizations_name_trgm ON organizations USING gin(name gin_trgm_ops);
        // CREATE INDEX idx_users_username_trgm ON users USING gin(username gin_trgm_ops);

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // インデックスを削除
        manager
            .drop_index(
                Index::drop()
                    .name("idx_organizations_name_search")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(Index::drop().name("idx_users_email_search").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_users_username_search").to_owned())
            .await?;

        // feature_usage_metricsのインデックスは別のマイグレーションで管理されるためスキップ

        manager
            .drop_index(Index::drop().name("idx_team_members_team_user").to_owned())
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_organization_members_org_user")
                    .to_owned(),
            )
            .await?;

        // idx_tasks_user_statusは既存のマイグレーションで管理されているためスキップ

        // idx_tasks_due_dateは別のマイグレーションで管理されているためスキップ

        manager
            .drop_index(
                Index::drop()
                    .name("idx_bulk_operation_histories_performed_by")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_bulk_operation_histories_operation_type")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_daily_activity_summaries_date")
                    .to_owned(),
            )
            .await?;

        // Trigram indexes are not created in up(), so no need to drop them

        Ok(())
    }
}

#[derive(Iden)]
enum Organizations {
    Table,
    Name,
}

#[derive(Iden)]
enum Users {
    Table,
    Email,
    Username,
}

// FeatureUsageMetricsは使用されなくなったため削除

#[derive(Iden)]
enum TeamMembers {
    Table,
    TeamId,
    UserId,
}

#[derive(Iden)]
enum OrganizationMembers {
    Table,
    OrganizationId,
    UserId,
}

#[derive(Iden)]
enum Tasks {
    Table,
    UserId,
    Status,
}

#[derive(Iden)]
enum BulkOperationHistories {
    Table,
    PerformedBy,
    CreatedAt,
    OperationType,
}

#[derive(Iden)]
enum DailyActivitySummaries {
    Table,
    Date,
}
