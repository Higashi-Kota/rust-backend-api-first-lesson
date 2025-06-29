use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OrganizationDepartments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OrganizationDepartments::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(OrganizationDepartments::Name)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(OrganizationDepartments::Description).text())
                    .col(
                        ColumnDef::new(OrganizationDepartments::OrganizationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(OrganizationDepartments::ParentDepartmentId).uuid())
                    .col(
                        ColumnDef::new(OrganizationDepartments::HierarchyLevel)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(OrganizationDepartments::HierarchyPath)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(OrganizationDepartments::ManagerUserId).uuid())
                    .col(
                        ColumnDef::new(OrganizationDepartments::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(OrganizationDepartments::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(OrganizationDepartments::UpdatedAt)
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
                    .name("fk_organization_departments_organization_id")
                    .from(
                        OrganizationDepartments::Table,
                        OrganizationDepartments::OrganizationId,
                    )
                    .to(Organizations::Table, Organizations::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_organization_departments_parent_department_id")
                    .from(
                        OrganizationDepartments::Table,
                        OrganizationDepartments::ParentDepartmentId,
                    )
                    .to(OrganizationDepartments::Table, OrganizationDepartments::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_organization_departments_manager_user_id")
                    .from(
                        OrganizationDepartments::Table,
                        OrganizationDepartments::ManagerUserId,
                    )
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        // Add indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_organization_departments_organization_id")
                    .table(OrganizationDepartments::Table)
                    .col(OrganizationDepartments::OrganizationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organization_departments_parent_department_id")
                    .table(OrganizationDepartments::Table)
                    .col(OrganizationDepartments::ParentDepartmentId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organization_departments_hierarchy_path")
                    .table(OrganizationDepartments::Table)
                    .col(OrganizationDepartments::HierarchyPath)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organization_departments_name")
                    .table(OrganizationDepartments::Table)
                    .col(OrganizationDepartments::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(OrganizationDepartments::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum OrganizationDepartments {
    Table,
    Id,
    Name,
    Description,
    OrganizationId,
    ParentDepartmentId,
    HierarchyLevel,
    HierarchyPath,
    ManagerUserId,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
