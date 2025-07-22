use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 監査ログテーブルの作成
        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuditLogs::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuditLogs::UserId).uuid().not_null())
                    .col(ColumnDef::new(AuditLogs::Action).string().not_null())
                    .col(ColumnDef::new(AuditLogs::ResourceType).string().not_null())
                    .col(ColumnDef::new(AuditLogs::ResourceId).uuid())
                    .col(ColumnDef::new(AuditLogs::TeamId).uuid())
                    .col(ColumnDef::new(AuditLogs::OrganizationId).uuid())
                    .col(ColumnDef::new(AuditLogs::Details).json())
                    .col(ColumnDef::new(AuditLogs::IpAddress).string())
                    .col(ColumnDef::new(AuditLogs::UserAgent).string())
                    .col(ColumnDef::new(AuditLogs::Result).string().not_null())
                    .col(
                        ColumnDef::new(AuditLogs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_logs_user_id")
                            .from(AuditLogs::Table, AuditLogs::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_logs_team_id")
                            .from(AuditLogs::Table, AuditLogs::TeamId)
                            .to(Teams::Table, Teams::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_logs_organization_id")
                            .from(AuditLogs::Table, AuditLogs::OrganizationId)
                            .to(Organizations::Table, Organizations::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックスの作成
        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_user_id")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_resource")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::ResourceType)
                    .col(AuditLogs::ResourceId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_team_id")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::TeamId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_organization_id")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::OrganizationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_created_at")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AuditLogs {
    Table,
    Id,
    UserId,
    Action,
    ResourceType,
    ResourceId,
    TeamId,
    OrganizationId,
    Details,
    IpAddress,
    UserAgent,
    Result,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Teams {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    Id,
}
