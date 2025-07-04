use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 共有リンクアクセスログテーブル
        manager
            .create_table(
                Table::create()
                    .table(ShareLinkAccessLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ShareLinkAccessLogs::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(
                        ColumnDef::new(ShareLinkAccessLogs::ShareLinkId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ShareLinkAccessLogs::IpAddress)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ShareLinkAccessLogs::UserAgent)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ShareLinkAccessLogs::AccessedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_share_link_access_logs_share_link")
                            .from(ShareLinkAccessLogs::Table, ShareLinkAccessLogs::ShareLinkId)
                            .to(AttachmentShareLinks::Table, AttachmentShareLinks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックス
        manager
            .create_index(
                Index::create()
                    .name("idx_share_link_access_logs_share_link_id")
                    .table(ShareLinkAccessLogs::Table)
                    .col(ShareLinkAccessLogs::ShareLinkId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_share_link_access_logs_accessed_at")
                    .table(ShareLinkAccessLogs::Table)
                    .col(ShareLinkAccessLogs::AccessedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ShareLinkAccessLogs::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ShareLinkAccessLogs {
    Table,
    Id,
    ShareLinkId,
    IpAddress,
    UserAgent,
    AccessedAt,
}

#[derive(Iden)]
enum AttachmentShareLinks {
    Table,
    Id,
}
