use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PermissionMatrices::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PermissionMatrices::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PermissionMatrices::EntityType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PermissionMatrices::EntityId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PermissionMatrices::MatrixVersion)
                            .string()
                            .not_null()
                            .default("v1.0"),
                    )
                    .col(
                        ColumnDef::new(PermissionMatrices::MatrixData)
                            .json()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PermissionMatrices::InheritanceSettings).json())
                    .col(ColumnDef::new(PermissionMatrices::ComplianceSettings).json())
                    .col(
                        ColumnDef::new(PermissionMatrices::UpdatedBy)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PermissionMatrices::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(PermissionMatrices::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(PermissionMatrices::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint to users table for updated_by
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_permission_matrices_updated_by")
                    .from(PermissionMatrices::Table, PermissionMatrices::UpdatedBy)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Add indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_permission_matrices_entity")
                    .table(PermissionMatrices::Table)
                    .col(PermissionMatrices::EntityType)
                    .col(PermissionMatrices::EntityId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_permission_matrices_entity_type")
                    .table(PermissionMatrices::Table)
                    .col(PermissionMatrices::EntityType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_permission_matrices_updated_by")
                    .table(PermissionMatrices::Table)
                    .col(PermissionMatrices::UpdatedBy)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PermissionMatrices::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PermissionMatrices {
    Table,
    Id,
    EntityType,
    EntityId,
    MatrixVersion,
    MatrixData,
    InheritanceSettings,
    ComplianceSettings,
    UpdatedBy,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
