use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // rolesテーブル作成
        manager
            .create_table(
                Table::create()
                    .table(Roles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Roles::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(Roles::Name)
                            .string_len(50)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Roles::DisplayName)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Roles::Description).text().null())
                    .col(
                        ColumnDef::new(Roles::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Roles::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Roles::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックス作成
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_roles_name")
                    .table(Roles::Table)
                    .col(Roles::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_roles_is_active")
                    .table(Roles::Table)
                    .col(Roles::IsActive)
                    .to_owned(),
            )
            .await?;

        // 初期データ投入
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(Roles::Table)
                    .columns([
                        Roles::Name,
                        Roles::DisplayName,
                        Roles::Description,
                        Roles::IsActive,
                    ])
                    .values_panic([
                        "admin".into(),
                        "Administrator".into(),
                        "System administrator with full access to all resources and user management capabilities".into(),
                        true.into(),
                    ])
                    .values_panic([
                        "member".into(),
                        "Member".into(),
                        "Regular user with access to their own data and basic functionality".into(),
                        true.into(),
                    ])
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
                    .name("idx_roles_is_active")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(Index::drop().if_exists().name("idx_roles_name").to_owned())
            .await?;

        // テーブル削除
        manager
            .drop_table(Table::drop().table(Roles::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Roles {
    Table,
    Id,
    Name,
    DisplayName,
    Description,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
