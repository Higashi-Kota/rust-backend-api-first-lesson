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
                    .table(Organizations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Organizations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Organizations::Name).string().not_null())
                    .col(ColumnDef::new(Organizations::Description).text())
                    .col(ColumnDef::new(Organizations::OwnerId).uuid().not_null())
                    .col(
                        ColumnDef::new(Organizations::SubscriptionTier)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Organizations::MaxTeams).integer().not_null())
                    .col(
                        ColumnDef::new(Organizations::MaxMembers)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Organizations::SettingsJson)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Organizations::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Organizations::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint separately
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_organizations_owner_id")
                    .from(Organizations::Table, Organizations::OwnerId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Add indexes separately
        manager
            .create_index(
                Index::create()
                    .name("idx_organizations_name")
                    .table(Organizations::Table)
                    .col(Organizations::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_organizations_owner_id")
                    .table(Organizations::Table)
                    .col(Organizations::OwnerId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Organizations::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    Id,
    Name,
    Description,
    OwnerId,
    SubscriptionTier,
    MaxTeams,
    MaxMembers,
    SettingsJson,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
