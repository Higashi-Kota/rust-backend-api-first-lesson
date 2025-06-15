use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // role_idカラムを追加（一時的にnullable）
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::RoleId).uuid().null(), // 一時的にnull許可
                    )
                    .to_owned(),
            )
            .await?;

        // memberロールのIDを取得し、既存のユーザーに設定
        let _member_role_id_result = manager
            .exec_stmt(
                Query::select()
                    .from(Roles::Table)
                    .column(Roles::Id)
                    .and_where(Expr::col(Roles::Name).eq("member"))
                    .to_owned(),
            )
            .await?;

        // 既存のユーザーにmemberロールを設定
        manager
            .exec_stmt(
                Query::update()
                    .table(Users::Table)
                    .value(
                        Users::RoleId,
                        Expr::cust("(SELECT id FROM roles WHERE name = 'member')"),
                    )
                    .and_where(Expr::col(Users::RoleId).is_null())
                    .to_owned(),
            )
            .await?;

        // role_idをnot nullに変更
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .modify_column(ColumnDef::new(Users::RoleId).uuid().not_null())
                    .to_owned(),
            )
            .await?;

        // 外部キー制約を追加
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_users_role_id")
                    .from(Users::Table, Users::RoleId)
                    .to(Roles::Table, Roles::Id)
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Restrict) // ロール削除を制限
                    .to_owned(),
            )
            .await?;

        // インデックス作成
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_role_id")
                    .table(Users::Table)
                    .col(Users::RoleId)
                    .to_owned(),
            )
            .await?;

        // 複合インデックス（ロール別の活性ユーザー検索用）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_role_active")
                    .table(Users::Table)
                    .col(Users::RoleId)
                    .col(Users::IsActive)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // インデックス削除
        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_users_role_active")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_users_role_id")
                    .to_owned(),
            )
            .await?;

        // 外部キー制約削除
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_users_role_id")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        // role_idカラム削除
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::RoleId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Users {
    Table,
    RoleId,
    IsActive,
}

#[derive(Iden)]
enum Roles {
    Table,
    Id,
    Name,
}
