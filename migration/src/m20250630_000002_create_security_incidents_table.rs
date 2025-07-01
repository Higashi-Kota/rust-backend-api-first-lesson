use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // security_incidentsテーブル作成
        manager
            .create_table(
                Table::create()
                    .table(SecurityIncidents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SecurityIncidents::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SecurityIncidents::IncidentType)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SecurityIncidents::Severity)
                            .string_len(20)
                            .not_null()
                            .default("medium"),
                    )
                    .col(ColumnDef::new(SecurityIncidents::UserId).uuid().null())
                    .col(
                        ColumnDef::new(SecurityIncidents::IpAddress)
                            .string_len(45)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SecurityIncidents::Description)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SecurityIncidents::Details).json().null())
                    .col(
                        ColumnDef::new(SecurityIncidents::Status)
                            .string_len(20)
                            .not_null()
                            .default("open"),
                    )
                    .col(
                        ColumnDef::new(SecurityIncidents::ResolvedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(ColumnDef::new(SecurityIncidents::ResolvedBy).uuid().null())
                    .col(
                        ColumnDef::new(SecurityIncidents::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SecurityIncidents::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_security_incidents_user_id")
                            .from(SecurityIncidents::Table, SecurityIncidents::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_security_incidents_resolved_by")
                            .from(SecurityIncidents::Table, SecurityIncidents::ResolvedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックス作成
        manager
            .create_index(
                Index::create()
                    .name("idx_security_incidents_type")
                    .table(SecurityIncidents::Table)
                    .col(SecurityIncidents::IncidentType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_security_incidents_status")
                    .table(SecurityIncidents::Table)
                    .col(SecurityIncidents::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_security_incidents_created_at")
                    .table(SecurityIncidents::Table)
                    .col(SecurityIncidents::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SecurityIncidents::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum SecurityIncidents {
    Table,
    Id,
    IncidentType,
    Severity,
    UserId,
    IpAddress,
    Description,
    Details,
    Status,
    ResolvedAt,
    ResolvedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
