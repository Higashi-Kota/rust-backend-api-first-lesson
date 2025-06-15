use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 初期管理者ユーザーを作成
        // パスワード: "Adm1n$ecurE2024!" をArgon2でハッシュ化した値
        let admin_password_hash = "$argon2id$v=19$m=65536,t=3,p=4$rwjnw7itO1QP7YiQLYYPuw$bwYljZ/eNoieCwcPydAbagPt05UT9wcs+n0zH58ZxS4";

        // adminロールのIDを取得
        manager
            .exec_stmt(
                Query::select()
                    .from(Roles::Table)
                    .column(Roles::Id)
                    .and_where(Expr::col(Roles::Name).eq("admin"))
                    .to_owned(),
            )
            .await?;

        // 管理者ユーザーを挿入
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(Users::Table)
                    .columns([
                        Users::Id,
                        Users::Email,
                        Users::Username,
                        Users::PasswordHash,
                        Users::IsActive,
                        Users::EmailVerified,
                        Users::RoleId,
                    ])
                    .values_panic([
                        Expr::cust("gen_random_uuid()"),
                        "admin@example.com".into(),
                        "admin".into(),
                        admin_password_hash.into(),
                        true.into(),
                        true.into(),
                        Expr::cust("(SELECT id FROM roles WHERE name = 'admin')"),
                    ])
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 初期管理者ユーザーを削除
        manager
            .exec_stmt(
                Query::delete()
                    .from_table(Users::Table)
                    .and_where(Expr::col(Users::Email).eq("admin@example.com"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Email,
    Username,
    PasswordHash,
    IsActive,
    EmailVerified,
    RoleId,
}

#[derive(Iden)]
enum Roles {
    Table,
    Id,
    Name,
}
