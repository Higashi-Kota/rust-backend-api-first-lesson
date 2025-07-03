use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the existing foreign key constraint
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_teams_owner_id")
                    .table(Teams::Table)
                    .to_owned(),
            )
            .await?;

        // First, alter the column to allow NULL values
        manager
            .alter_table(
                Table::alter()
                    .table(Teams::Table)
                    .modify_column(
                        ColumnDef::new(Teams::OwnerId).uuid().null(), // Allow NULL for when owner is deleted
                    )
                    .to_owned(),
            )
            .await?;

        // Recreate the foreign key with SET NULL on delete
        // This will set owner_id to NULL when the owner is deleted, preserving the team
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_teams_owner_id")
                    .from(Teams::Table, Teams::OwnerId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::SetNull) // Set to NULL when owner is deleted
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the modified foreign key
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_teams_owner_id")
                    .table(Teams::Table)
                    .to_owned(),
            )
            .await?;

        // Recreate the original foreign key with CASCADE
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

        // Restore NOT NULL constraint
        manager
            .alter_table(
                Table::alter()
                    .table(Teams::Table)
                    .modify_column(ColumnDef::new(Teams::OwnerId).uuid().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Teams {
    Table,
    OwnerId,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
