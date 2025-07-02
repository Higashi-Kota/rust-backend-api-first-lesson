use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BulkOperationHistories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BulkOperationHistories::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(
                        ColumnDef::new(BulkOperationHistories::OperationType)
                            .string()
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BulkOperationHistories::PerformedBy)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BulkOperationHistories::AffectedCount)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BulkOperationHistories::Status)
                            .string()
                            .string_len(20)
                            .not_null(),
                    )
                    .col(ColumnDef::new(BulkOperationHistories::ErrorDetails).json_binary())
                    .col(
                        ColumnDef::new(BulkOperationHistories::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .col(
                        ColumnDef::new(BulkOperationHistories::CompletedAt)
                            .timestamp_with_time_zone(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bulk_operation_histories_performed_by")
                            .from(
                                BulkOperationHistories::Table,
                                BulkOperationHistories::PerformedBy,
                            )
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックスを作成
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_bulk_operation_histories_created_at")
                    .table(BulkOperationHistories::Table)
                    .col(BulkOperationHistories::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_bulk_operation_histories_performed_by")
                    .table(BulkOperationHistories::Table)
                    .col(BulkOperationHistories::PerformedBy)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(BulkOperationHistories::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum BulkOperationHistories {
    Table,
    Id,
    OperationType,
    PerformedBy,
    AffectedCount,
    Status,
    ErrorDetails,
    CreatedAt,
    CompletedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
