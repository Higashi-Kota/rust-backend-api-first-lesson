use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Teams::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Teams::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Teams::Name).string().not_null())
                    .col(ColumnDef::new(Teams::Description).text())
                    .col(ColumnDef::new(Teams::OrganizationId).uuid())
                    .col(ColumnDef::new(Teams::OwnerId).uuid().not_null())
                    .col(
                        ColumnDef::new(Teams::SubscriptionTier)
                            .string_len(20)
                            .not_null()
                            .default("Free"),
                    )
                    .col(
                        ColumnDef::new(Teams::MaxMembers)
                            .integer()
                            .not_null()
                            .default(10),
                    )
                    .col(
                        ColumnDef::new(Teams::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Teams::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraints separately
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_teams_owner_id")
                    .from(Teams::Table, Teams::OwnerId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Add organization foreign key constraint
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_teams_organization_id")
                    .from(Teams::Table, Teams::OrganizationId)
                    .to(Organizations::Table, Organizations::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        // Add indexes separately
        manager
            .create_index(
                Index::create()
                    .name("idx_teams_name")
                    .table(Teams::Table)
                    .col(Teams::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_teams_owner_id")
                    .table(Teams::Table)
                    .col(Teams::OwnerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_teams_organization_id")
                    .table(Teams::Table)
                    .col(Teams::OrganizationId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Teams::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Teams {
    Table,
    Id,
    Name,
    Description,
    OrganizationId,
    OwnerId,
    SubscriptionTier,
    MaxMembers,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    Id,
}
