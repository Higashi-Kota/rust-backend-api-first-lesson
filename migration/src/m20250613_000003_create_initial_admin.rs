use sea_orm_migration::prelude::*;
use std::env;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 環境変数から管理者情報を取得
        let admin_email =
            env::var("INITIAL_ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string());
        let admin_username =
            env::var("INITIAL_ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
        let admin_password_hash = env::var("INITIAL_ADMIN_PASSWORD_HASH")
            .unwrap_or_else(|_| {
                // デフォルトパスワード: "Adm1n$ecurE2024!" をArgon2でハッシュ化した値
                "$argon2id$v=19$m=65536,t=3,p=4$rwjnw7itO1QP7YiQLYYPuw$bwYljZ/eNoieCwcPydAbagPt05UT9wcs+n0zH58ZxS4".to_string()
            });

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

        // 既存の管理者ユーザーが存在しない場合のみ作成
        // 重複した場合はエラーを無視する（既に存在している場合）
        let result = manager
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
                        admin_email.into(),
                        admin_username.into(),
                        admin_password_hash.into(),
                        true.into(),
                        true.into(),
                        Expr::cust("(SELECT id FROM roles WHERE name = 'admin')"),
                    ])
                    .to_owned(),
            )
            .await;

        match result {
            Ok(_) => println!("Initial admin user created successfully"),
            Err(e) => {
                // エラーが重複キーエラーの場合は無視
                if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                    println!("Admin user already exists, skipping creation");
                } else {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 環境変数から管理者メールアドレスを取得
        let admin_email =
            env::var("INITIAL_ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string());

        // 初期管理者ユーザーを削除
        manager
            .exec_stmt(
                Query::delete()
                    .from_table(Users::Table)
                    .and_where(Expr::col(Users::Email).eq(&admin_email))
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
