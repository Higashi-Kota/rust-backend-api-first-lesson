use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FeatureUsageMetrics::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FeatureUsageMetrics::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(
                        ColumnDef::new(FeatureUsageMetrics::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FeatureUsageMetrics::FeatureName)
                            .string()
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FeatureUsageMetrics::ActionType)
                            .string()
                            .string_len(50)
                            .not_null(),
                    )
                    .col(ColumnDef::new(FeatureUsageMetrics::Metadata).json_binary())
                    .col(
                        ColumnDef::new(FeatureUsageMetrics::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_feature_usage_metrics_user_id")
                            .from(FeatureUsageMetrics::Table, FeatureUsageMetrics::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックスを作成
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_feature_usage_metrics_date")
                    .table(FeatureUsageMetrics::Table)
                    .col(FeatureUsageMetrics::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_feature_usage_metrics_user_feature")
                    .table(FeatureUsageMetrics::Table)
                    .col(FeatureUsageMetrics::UserId)
                    .col(FeatureUsageMetrics::FeatureName)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FeatureUsageMetrics::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum FeatureUsageMetrics {
    Table,
    Id,
    UserId,
    FeatureName,
    ActionType,
    Metadata,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
