use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TeamInvitations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TeamInvitations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TeamInvitations::TeamId).uuid().not_null())
                    .col(
                        ColumnDef::new(TeamInvitations::InvitedEmail)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TeamInvitations::InvitedUserId).uuid())
                    .col(
                        ColumnDef::new(TeamInvitations::InvitedByUserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TeamInvitations::Status)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(TeamInvitations::Message).text())
                    .col(ColumnDef::new(TeamInvitations::ExpiresAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(TeamInvitations::AcceptedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(TeamInvitations::DeclinedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(TeamInvitations::DeclineReason).text())
                    .col(
                        ColumnDef::new(TeamInvitations::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TeamInvitations::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TeamInvitations::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum TeamInvitations {
    Table,
    Id,
    TeamId,
    InvitedEmail,
    InvitedUserId,
    InvitedByUserId,
    Status,
    Message,
    ExpiresAt,
    AcceptedAt,
    DeclinedAt,
    DeclineReason,
    CreatedAt,
    UpdatedAt,
}
