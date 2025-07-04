use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 外部共有リンクテーブル
        manager
            .create_table(
                Table::create()
                    .table(AttachmentShareLinks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AttachmentShareLinks::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(
                        ColumnDef::new(AttachmentShareLinks::AttachmentId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AttachmentShareLinks::CreatedBy)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AttachmentShareLinks::ShareToken)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(AttachmentShareLinks::Description)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AttachmentShareLinks::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AttachmentShareLinks::MaxAccessCount)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AttachmentShareLinks::CurrentAccessCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AttachmentShareLinks::IsRevoked)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AttachmentShareLinks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .col(
                        ColumnDef::new(AttachmentShareLinks::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_attachment_share_links_attachment")
                            .from(
                                AttachmentShareLinks::Table,
                                AttachmentShareLinks::AttachmentId,
                            )
                            .to(TaskAttachments::Table, TaskAttachments::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_attachment_share_links_user")
                            .from(AttachmentShareLinks::Table, AttachmentShareLinks::CreatedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックス
        manager
            .create_index(
                Index::create()
                    .name("idx_attachment_share_links_attachment_id")
                    .table(AttachmentShareLinks::Table)
                    .col(AttachmentShareLinks::AttachmentId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_attachment_share_links_share_token")
                    .table(AttachmentShareLinks::Table)
                    .col(AttachmentShareLinks::ShareToken)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_attachment_share_links_expires_at")
                    .table(AttachmentShareLinks::Table)
                    .col(AttachmentShareLinks::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AttachmentShareLinks::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum AttachmentShareLinks {
    Table,
    Id,
    AttachmentId,
    CreatedBy,
    ShareToken,
    Description,
    ExpiresAt,
    MaxAccessCount,
    CurrentAccessCount,
    IsRevoked,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum TaskAttachments {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
