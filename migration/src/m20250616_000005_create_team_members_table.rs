use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TeamMembers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TeamMembers::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TeamMembers::TeamId).uuid().not_null())
                    .col(ColumnDef::new(TeamMembers::UserId).uuid().not_null())
                    .col(ColumnDef::new(TeamMembers::Role).string().not_null())
                    .col(
                        ColumnDef::new(TeamMembers::JoinedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(TeamMembers::InvitedBy).uuid())
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraints separately
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_team_members_team_id")
                    .from(TeamMembers::Table, TeamMembers::TeamId)
                    .to(Teams::Table, Teams::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_team_members_user_id")
                    .from(TeamMembers::Table, TeamMembers::UserId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_team_members_invited_by")
                    .from(TeamMembers::Table, TeamMembers::InvitedBy)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        // Add indexes separately
        manager
            .create_index(
                Index::create()
                    .name("idx_team_members_team_id")
                    .table(TeamMembers::Table)
                    .col(TeamMembers::TeamId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_team_members_user_id")
                    .table(TeamMembers::Table)
                    .col(TeamMembers::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_team_members_unique")
                    .table(TeamMembers::Table)
                    .col(TeamMembers::TeamId)
                    .col(TeamMembers::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TeamMembers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TeamMembers {
    Table,
    Id,
    TeamId,
    UserId,
    Role,
    JoinedAt,
    InvitedBy,
}

#[derive(DeriveIden)]
enum Teams {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
