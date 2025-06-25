use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OrganizationAnalytics::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OrganizationAnalytics::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(OrganizationAnalytics::OrganizationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(OrganizationAnalytics::DepartmentId).uuid())
                    .col(
                        ColumnDef::new(OrganizationAnalytics::AnalyticsType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationAnalytics::MetricName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationAnalytics::MetricValue)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationAnalytics::Period)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationAnalytics::PeriodStart)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationAnalytics::PeriodEnd)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationAnalytics::CalculatedBy)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationAnalytics::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(OrganizationAnalytics::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraints
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_organization_analytics_organization_id")
                    .from(
                        OrganizationAnalytics::Table,
                        OrganizationAnalytics::OrganizationId,
                    )
                    .to(Organizations::Table, Organizations::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_organization_analytics_department_id")
                    .from(
                        OrganizationAnalytics::Table,
                        OrganizationAnalytics::DepartmentId,
                    )
                    .to(OrganizationDepartments::Table, OrganizationDepartments::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_organization_analytics_calculated_by")
                    .from(
                        OrganizationAnalytics::Table,
                        OrganizationAnalytics::CalculatedBy,
                    )
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Add indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_organization_analytics_organization_id")
                    .table(OrganizationAnalytics::Table)
                    .col(OrganizationAnalytics::OrganizationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organization_analytics_department_id")
                    .table(OrganizationAnalytics::Table)
                    .col(OrganizationAnalytics::DepartmentId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organization_analytics_type_metric")
                    .table(OrganizationAnalytics::Table)
                    .col(OrganizationAnalytics::AnalyticsType)
                    .col(OrganizationAnalytics::MetricName)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organization_analytics_period")
                    .table(OrganizationAnalytics::Table)
                    .col(OrganizationAnalytics::Period)
                    .col(OrganizationAnalytics::PeriodStart)
                    .col(OrganizationAnalytics::PeriodEnd)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OrganizationAnalytics::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OrganizationAnalytics {
    Table,
    Id,
    OrganizationId,
    DepartmentId,
    AnalyticsType,
    MetricName,
    MetricValue,
    Period,
    PeriodStart,
    PeriodEnd,
    CalculatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum OrganizationDepartments {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
