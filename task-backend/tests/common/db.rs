//! テスト用データベースヘルパー（SeaORMとtestcontainers‑rs v0.24を使用）

use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, Statement};
use std::sync::Arc;
use std::time::Duration;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt},
};
use tokio::sync::{Mutex, OnceCell};
use uuid::Uuid;

// 全テスト共有のPostgresコンテナとポート
static POSTGRES_CONTAINER: OnceCell<Arc<Mutex<ContainerAsync<Postgres>>>> = OnceCell::const_new();
static DB_PORT: OnceCell<u16> = OnceCell::const_new();

async fn get_or_create_container() -> u16 {
    // OnceCell を初期化（tokio::sync::OnceCell は get_or_init が async 対応）
    // 未使用変数の警告を避けるため、アンダースコアを追加
    let _container_ref = POSTGRES_CONTAINER
        .get_or_init(|| async {
            // Postgres設定（環境変数含む）
            let image = Postgres::default()
                .with_tag("15-alpine") // 安定性のためバージョン固定
                .with_env_var("POSTGRES_USER", "postgres")
                .with_env_var("POSTGRES_PASSWORD", "postgres")
                .with_env_var("POSTGRES_DB", "test_db");

            // コンテナ起動
            let container = image.start().await.expect("PostgreSQLコンテナの起動に失敗");

            // ポート番号を取得して保存
            let port = container
                .get_host_port_ipv4(5432)
                .await
                .expect("マップされたポートの取得に失敗");

            DB_PORT.set(port).expect("DB_PORTの設定に失敗");

            Arc::new(Mutex::new(container))
        })
        .await;

    // 共有コンテナのポートを返す
    *DB_PORT.get().expect("DB_PORTが設定されていません")
}

pub struct TestDatabase {
    pub connection: DatabaseConnection,
    schema_name: String,
}

impl TestDatabase {
    pub async fn new() -> Self {
        // 共有コンテナを取得またはポートを作成
        let port = get_or_create_container().await;

        // このテスト用の一意なスキーマ名を生成（データ分離用）
        let schema_name = format!("test_{}", Uuid::new_v4().to_string().replace("-", ""));

        // データベースに接続
        let url = format!("postgres://postgres:postgres@localhost:{}/test_db", port);

        let mut opt = ConnectOptions::new(url);
        opt.max_connections(10) // 並列テストに適切な接続数
            .min_connections(1)
            .connect_timeout(Duration::from_secs(10))
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(10))
            .max_lifetime(Duration::from_secs(30))
            .sqlx_logging(false); // テスト時のログ出力を減らす

        let connection = Database::connect(opt).await.unwrap();

        // このテスト用のスキーマを作成して使用
        let create_schema = format!("CREATE SCHEMA IF NOT EXISTS \"{}\";", schema_name);
        connection
            .execute(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                create_schema,
            ))
            .await
            .expect("スキーマ作成に失敗");

        let set_search_path = format!("SET search_path TO \"{}\";", schema_name);
        connection
            .execute(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                set_search_path,
            ))
            .await
            .expect("search pathの設定に失敗");

        // このスキーマ内でマイグレーションを実行
        Migrator::up(&connection, None).await.unwrap();

        Self {
            connection,
            schema_name,
        }
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // テスト終了時にスキーマをクリーンアップ
        let schema_name = self.schema_name.clone();
        let connection = self.connection.clone();

        tokio::spawn(async move {
            let drop_schema = format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE;", schema_name);
            let _ = connection
                .execute(Statement::from_string(
                    sea_orm::DatabaseBackend::Postgres,
                    drop_schema,
                ))
                .await;
        });
    }
}
