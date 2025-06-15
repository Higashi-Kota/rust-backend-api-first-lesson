// migration/src/lib.rs
pub use sea_orm_migration::prelude::*;

// マイグレーションモジュール
mod m20250511_073638_create_task_table;
mod m20250512_000001_add_task_indexes; // 追加したインデックスマイグレーション

// 認証関連マイグレーション
mod m20250612_000001_create_users_table;
mod m20250612_000002_create_refresh_tokens_table;
mod m20250612_000003_create_password_reset_tokens_table;
mod m20250612_000004_add_user_id_to_tasks;

// ロール関連マイグレーション
mod m20250613_000001_create_roles_table;
mod m20250613_000002_add_role_id_to_users;
mod m20250613_000003_create_initial_admin;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // 1. 基本テーブル作成（依存関係なし）
            Box::new(m20250511_073638_create_task_table::Migration),
            Box::new(m20250612_000001_create_users_table::Migration),
            // 2. 基本テーブルのインデックス追加
            Box::new(m20250512_000001_add_task_indexes::Migration),
            // 3. 依存テーブル作成（usersテーブルに依存）
            Box::new(m20250612_000002_create_refresh_tokens_table::Migration),
            Box::new(m20250612_000003_create_password_reset_tokens_table::Migration),
            // 4. 既存テーブルの変更（usersテーブルへの外部キー追加）
            Box::new(m20250612_000004_add_user_id_to_tasks::Migration),
            // 5. ロール関連テーブル作成とユーザーテーブル更新
            Box::new(m20250613_000001_create_roles_table::Migration),
            Box::new(m20250613_000002_add_role_id_to_users::Migration),
            // 6. 初期管理者ユーザー作成
            Box::new(m20250613_000003_create_initial_admin::Migration),
        ]
    }
}
