// tests/common/db.rs
use migration::Migrator; // migration クレートを直接インポート
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use std::sync::Once;
use task_backend::db::DbPool;
use testcontainers::{clients::Cli, Container, RunnableImage};
use testcontainers_modules::postgres::Postgres; // MigratorTrait をインポート

static INIT: Once = Once::new();

// PostgreSQLコンテナをセットアップ
pub struct TestDatabase {
    pub container: Container<'static, Postgres>,
    pub connection: DatabaseConnection,
}

impl TestDatabase {
    pub async fn new() -> Self {
        // シングルトンパターンでDockerクライアントを初期化
        static mut CLI: Option<Cli> = None;

        INIT.call_once(|| unsafe {
            CLI = Some(Cli::default());
        });

        let cli = unsafe { CLI.as_ref().unwrap() };

        // PostgreSQLコンテナを起動
        let postgres_image = RunnableImage::from(Postgres::default())
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres")
            .with_env_var("POSTGRES_DB", "test_db");

        let container = cli.run(postgres_image);
        let port = container.get_host_port_ipv4(5432);
        let connection_string = format!("postgres://postgres:postgres@localhost:{}/test_db", port);

        // データベース接続を作成
        let connection = Database::connect(&connection_string).await.unwrap();

        // マイグレーションを実行
        Migrator::up(&connection, None).await.unwrap();

        TestDatabase {
            container,
            connection,
        }
    }
}
