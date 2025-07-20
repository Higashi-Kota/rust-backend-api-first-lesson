use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // organization_idフィールドをusersテーブルに追加
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::OrganizationId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // 外部キー制約を追加
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_users_organization_id")
                    .from(Users::Table, Users::OrganizationId)
                    .to(Organizations::Table, Organizations::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // インデックスを追加（組織別ユーザー検索の高速化）
        manager
            .create_index(
                Index::create()
                    .name("idx_users_organization_id")
                    .table(Users::Table)
                    .col(Users::OrganizationId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // インデックスを削除
        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_organization_id")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        // 外部キー制約を削除
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_users_organization_id")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        // カラムを削除
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::OrganizationId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Users {
    Table,
    OrganizationId,
}

#[derive(Iden)]
enum Organizations {
    Table,
    Id,
}
