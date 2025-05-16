// src/db.rs
use crate::config::Config;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr, Statement};
use std::time::Duration;

pub type DbPool = DatabaseConnection;

pub async fn create_db_pool(config: &Config) -> Result<DbPool, DbErr> {
    Database::connect(&config.database_url).await
}

// スキーマを指定して接続するバージョンを追加
pub async fn create_db_pool_with_schema(config: &Config, schema: &str) -> Result<DbPool, DbErr> {
    let mut opt = ConnectOptions::new(config.database_url.clone());

    // 接続オプションを設定
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8 * 60))
        .set_schema_search_path(schema.to_string());

    Database::connect(opt).await
}

// 既存の接続にスキーマを設定するヘルパー関数
pub async fn set_schema(conn: &DbPool, schema: &str) -> Result<(), DbErr> {
    let set_search_path = format!("SET search_path TO \"{}\";", schema);
    conn.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        set_search_path,
    ))
    .await?;
    Ok(())
}

// スキーマが存在するか確認するヘルパー関数
pub async fn schema_exists(conn: &DbPool, schema: &str) -> Result<bool, DbErr> {
    let query = format!(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.schemata 
            WHERE schema_name = '{}'
        );",
        schema
    );

    // query_oneはOption<QueryResult>を返すため、まずそれを処理する
    let result_opt = conn
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            query,
        ))
        .await?;

    // 結果が存在すれば処理、なければfalseを返す
    if let Some(result) = result_opt {
        let exists: bool = result.try_get("", "exists")?;
        Ok(exists)
    } else {
        Ok(false) // クエリ結果がない場合はfalseとみなす
    }
}

// スキーマを作成するヘルパー関数
pub async fn create_schema(conn: &DbPool, schema: &str) -> Result<(), DbErr> {
    let create_schema = format!("CREATE SCHEMA IF NOT EXISTS \"{}\";", schema);
    conn.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        create_schema,
    ))
    .await?;
    Ok(())
}
