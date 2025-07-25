use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add team_id column (nullable)
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .add_column(ColumnDef::new(Tasks::TeamId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Add organization_id column (nullable)
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .add_column(ColumnDef::new(Tasks::OrganizationId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Create custom enum type for visibility using raw SQL
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TYPE task_visibility AS ENUM ('personal', 'team', 'organization');",
            )
            .await?;

        // Add visibility column with default value
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .add_column(
                        ColumnDef::new(Tasks::Visibility)
                            .enumeration(
                                TaskVisibility::Type,
                                [
                                    TaskVisibility::Personal,
                                    TaskVisibility::Team,
                                    TaskVisibility::Organization,
                                ],
                            )
                            .not_null()
                            .default("personal"),
                    )
                    .to_owned(),
            )
            .await?;

        // Add assigned_to column (nullable) - for team member assignment
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .add_column(ColumnDef::new(Tasks::AssignedTo).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Create foreign key for team_id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_tasks_team_id")
                    .from(Tasks::Table, Tasks::TeamId)
                    .to(Teams::Table, Teams::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Create foreign key for organization_id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_tasks_organization_id")
                    .from(Tasks::Table, Tasks::OrganizationId)
                    .to(Organizations::Table, Organizations::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Create foreign key for assigned_to
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_tasks_assigned_to")
                    .from(Tasks::Table, Tasks::AssignedTo)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Create indexes for better performance
        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_team_id")
                    .table(Tasks::Table)
                    .col(Tasks::TeamId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_organization_id")
                    .table(Tasks::Table)
                    .col(Tasks::OrganizationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_visibility")
                    .table(Tasks::Table)
                    .col(Tasks::Visibility)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_assigned_to")
                    .table(Tasks::Table)
                    .col(Tasks::AssignedTo)
                    .to_owned(),
            )
            .await?;

        // Create composite indexes for common queries
        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_team_visibility")
                    .table(Tasks::Table)
                    .col(Tasks::TeamId)
                    .col(Tasks::Visibility)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_user_visibility")
                    .table(Tasks::Table)
                    .col(Tasks::UserId)
                    .col(Tasks::Visibility)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes
        manager
            .drop_index(Index::drop().name("idx_tasks_user_visibility").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_tasks_team_visibility").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_tasks_assigned_to").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_tasks_visibility").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_tasks_organization_id").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_tasks_team_id").to_owned())
            .await?;

        // Drop foreign keys
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_tasks_assigned_to")
                    .table(Tasks::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_tasks_organization_id")
                    .table(Tasks::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_tasks_team_id")
                    .table(Tasks::Table)
                    .to_owned(),
            )
            .await?;

        // Drop columns
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .drop_column(Tasks::AssignedTo)
                    .drop_column(Tasks::Visibility)
                    .drop_column(Tasks::OrganizationId)
                    .drop_column(Tasks::TeamId)
                    .to_owned(),
            )
            .await?;

        // Drop enum type using raw SQL
        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS task_visibility;")
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Tasks {
    Table,
    TeamId,
    OrganizationId,
    Visibility,
    UserId,
    AssignedTo,
}

#[derive(Iden)]
enum Teams {
    Table,
    Id,
}

#[derive(Iden)]
enum Organizations {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum TaskVisibility {
    #[iden = "task_visibility"]
    Type,
    #[iden = "personal"]
    Personal,
    #[iden = "team"]
    Team,
    #[iden = "organization"]
    Organization,
}
