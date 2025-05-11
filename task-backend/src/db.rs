// src/db.rs
use sea_orm::{Database, DatabaseConnection, DbErr};
use crate::config::Config;
// use sea_orm_migration::DbErr as MigrationDbErr; // sea_orm_migration::DbErr を使う場合 (今回は不要)

pub type DbPool = DatabaseConnection;

pub async fn create_db_pool(config: &Config) -> Result<DbPool, DbErr> {
    Database::connect(&config.database_url).await
}

/* // アプリケーション起動時の自動マイグレーションは無効化
pub async fn run_migrations(pool: &DbPool) -> Result<(), DbErr> {
    // プロジェクトのルート Cargo.toml に path で migration クレートを指定し、
    // このクレート (task-backend) の lib.rs または main.rs で pub mod migration; が必要
    // migration::Migrator::up(pool, None).await?;
    // Ok(())
    todo!("Automatic migration at startup is disabled. Use sea-orm-cli.");
}
*/