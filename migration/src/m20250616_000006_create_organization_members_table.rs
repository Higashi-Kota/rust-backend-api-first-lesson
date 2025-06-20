use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create table first
        manager
            .create_table(
                Table::create()
                    .table(OrganizationMembers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OrganizationMembers::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(OrganizationMembers::OrganizationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationMembers::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationMembers::Role)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationMembers::JoinedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(OrganizationMembers::InvitedBy).uuid())
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraints separately
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_organization_members_organization_id")
                    .from(
                        OrganizationMembers::Table,
                        OrganizationMembers::OrganizationId,
                    )
                    .to(Organizations::Table, Organizations::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_organization_members_user_id")
                    .from(OrganizationMembers::Table, OrganizationMembers::UserId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_organization_members_invited_by")
                    .from(OrganizationMembers::Table, OrganizationMembers::InvitedBy)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        // Add indexes separately
        manager
            .create_index(
                Index::create()
                    .name("idx_organization_members_organization_id")
                    .table(OrganizationMembers::Table)
                    .col(OrganizationMembers::OrganizationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organization_members_user_id")
                    .table(OrganizationMembers::Table)
                    .col(OrganizationMembers::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organization_members_unique")
                    .table(OrganizationMembers::Table)
                    .col(OrganizationMembers::OrganizationId)
                    .col(OrganizationMembers::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OrganizationMembers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OrganizationMembers {
    Table,
    Id,
    OrganizationId,
    UserId,
    Role,
    JoinedAt,
    InvitedBy,
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
