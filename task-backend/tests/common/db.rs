//! Async TestDatabase helper for SeaORM using testcontainers‑rs v0.24.

use migration::Migrator;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use std::time::Duration;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt},
};
use uuid::Uuid;

pub struct TestDatabase {
    _container: ContainerAsync<Postgres>,
    pub connection: DatabaseConnection,
}

impl TestDatabase {
    pub async fn new() -> Self {
        // ランダムなデータベース名を生成して並列テストでの衝突を避ける
        let db_name = format!("test_db_{}", Uuid::new_v4().to_string().replace("-", ""));

        // ① Configure Postgres image with env‑vars
        let image = Postgres::default()
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres")
            .with_env_var("POSTGRES_DB", &db_name);

        // ② Start container (AsyncRunner implicit)
        let container = image.start().await.expect("start pg container");

        // ③ Build connection string using mapped port
        let port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("get mapped port");
        let url = format!(
            "postgres://postgres:postgres@localhost:{}/{}",
            port, db_name
        );

        // 接続オプションを調整してタイムアウトとプール設定を最適化
        let mut opt = ConnectOptions::new(url);
        opt.max_connections(5) // 並列テストに十分な接続数を確保
            .min_connections(1)
            .connect_timeout(Duration::from_secs(10))
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(10))
            .max_lifetime(Duration::from_secs(30))
            .sqlx_logging(false); // テスト時のログ出力を減らす

        let connection = Database::connect(opt).await.unwrap();

        // ④ Run migrations
        Migrator::up(&connection, None).await.unwrap();

        Self {
            _container: container,
            connection,
        }
    }
}
