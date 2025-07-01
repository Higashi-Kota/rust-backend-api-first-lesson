use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DepartmentMembers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DepartmentMembers::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DepartmentMembers::DepartmentId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DepartmentMembers::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(DepartmentMembers::Role)
                            .string()
                            .not_null()
                            .default("member"),
                    )
                    .col(
                        ColumnDef::new(DepartmentMembers::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(DepartmentMembers::JoinedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(DepartmentMembers::AddedBy).uuid().not_null())
                    .col(
                        ColumnDef::new(DepartmentMembers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(DepartmentMembers::UpdatedAt)
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
                    .name("fk_department_members_department_id")
                    .from(DepartmentMembers::Table, DepartmentMembers::DepartmentId)
                    .to(OrganizationDepartments::Table, OrganizationDepartments::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_department_members_user_id")
                    .from(DepartmentMembers::Table, DepartmentMembers::UserId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_department_members_added_by")
                    .from(DepartmentMembers::Table, DepartmentMembers::AddedBy)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Add indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_department_members_department_user")
                    .table(DepartmentMembers::Table)
                    .col(DepartmentMembers::DepartmentId)
                    .col(DepartmentMembers::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_department_members_user_id")
                    .table(DepartmentMembers::Table)
                    .col(DepartmentMembers::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_department_members_role")
                    .table(DepartmentMembers::Table)
                    .col(DepartmentMembers::Role)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DepartmentMembers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum DepartmentMembers {
    Table,
    Id,
    DepartmentId,
    UserId,
    Role,
    IsActive,
    JoinedAt,
    AddedBy,
    CreatedAt,
    UpdatedAt,
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
