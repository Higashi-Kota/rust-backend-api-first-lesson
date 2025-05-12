//! Async TestDatabase helper for SeaORM using testcontainers‑rs v0.24.

use migration::Migrator;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt},
};

pub struct TestDatabase {
    _container: ContainerAsync<Postgres>,
    pub connection: DatabaseConnection,
}

impl TestDatabase {
    pub async fn new() -> Self {
        // ① Configure Postgres image with env‑vars
        let image = Postgres::default()
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres")
            .with_env_var("POSTGRES_DB", "test_db");

        // ② Start container (AsyncRunner implicit)
        let container = image.start().await.expect("start pg container");

        // ③ Build connection string using mapped port
        let port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("get mapped port");
        let url = format!("postgres://postgres:postgres@localhost:{}/test_db", port);
        let connection = Database::connect(&url).await.unwrap();

        // ④ Run migrations
        Migrator::up(&connection, None).await.unwrap();

        Self {
            _container: container,
            connection,
        }
    }
}
