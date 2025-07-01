use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // activity_logsテーブル作成
        manager
            .create_table(
                Table::create()
                    .table(ActivityLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ActivityLogs::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ActivityLogs::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(ActivityLogs::Action)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ActivityLogs::ResourceType)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ActivityLogs::ResourceId).uuid().null())
                    .col(
                        ColumnDef::new(ActivityLogs::IpAddress)
                            .string_len(45)
                            .null(),
                    )
                    .col(ColumnDef::new(ActivityLogs::UserAgent).text().null())
                    .col(ColumnDef::new(ActivityLogs::Details).json().null())
                    .col(
                        ColumnDef::new(ActivityLogs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_activity_logs_user_id")
                            .from(ActivityLogs::Table, ActivityLogs::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックス作成
        manager
            .create_index(
                Index::create()
                    .name("idx_activity_logs_user_id")
                    .table(ActivityLogs::Table)
                    .col(ActivityLogs::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_activity_logs_created_at")
                    .table(ActivityLogs::Table)
                    .col(ActivityLogs::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_activity_logs_action")
                    .table(ActivityLogs::Table)
                    .col(ActivityLogs::Action)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_activity_logs_resource")
                    .table(ActivityLogs::Table)
                    .col(ActivityLogs::ResourceType)
                    .col(ActivityLogs::ResourceId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ActivityLogs::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ActivityLogs {
    Table,
    Id,
    UserId,
    Action,
    ResourceType,
    ResourceId,
    IpAddress,
    UserAgent,
    Details,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
