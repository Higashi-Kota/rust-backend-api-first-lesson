//! テスト用データベースヘルパー（SeaORMとtestcontainers‑rs v0.24を使用）

use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement,
};
use std::sync::Arc;
use std::time::Duration;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt},
};
use tokio::sync::{Mutex, OnceCell};
use tracing::{debug, info};
use uuid::Uuid;

// 全テスト共有のPostgresコンテナとポート
static POSTGRES_CONTAINER: OnceCell<Arc<Mutex<ContainerAsync<Postgres>>>> = OnceCell::const_new();
static DB_PORT: OnceCell<u16> = OnceCell::const_new();

async fn get_or_create_container() -> u16 {
    let _container_ref = POSTGRES_CONTAINER
        .get_or_init(|| async {
            // Postgres設定
            let image = Postgres::default()
                .with_tag("15-alpine")
                .with_env_var("POSTGRES_USER", "postgres")
                .with_env_var("POSTGRES_PASSWORD", "postgres")
                .with_env_var("POSTGRES_DB", "test_db");

            // コンテナ起動
            let container = image.start().await.expect("PostgreSQLコンテナの起動に失敗");
            let port = container
                .get_host_port_ipv4(5432)
                .await
                .expect("マップされたポートの取得に失敗");

            DB_PORT.set(port).expect("DB_PORTの設定に失敗");

            // コンテナが起動したら少し待機する（安定化のため）
            tokio::time::sleep(Duration::from_secs(2)).await;

            Arc::new(Mutex::new(container))
        })
        .await;

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

        // このテスト用の一意なスキーマ名を生成
        let schema_name = format!("test_{}", Uuid::new_v4().to_string().replace('-', ""));

        info!("新しいテストスキーマを作成: {}", schema_name);

        // 基本接続を作成（初期設定用）
        let url = format!("postgres://postgres:postgres@localhost:{}/test_db", port);
        let mut opt = ConnectOptions::new(url.clone());
        opt.max_connections(10)
            .min_connections(2)
            .connect_timeout(Duration::from_secs(10))
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(10))
            .max_lifetime(Duration::from_secs(30))
            .sqlx_logging(true); // テスト中のSQLクエリログを有効化

        let admin_conn = Database::connect(opt).await.unwrap();

        // スキーマを作成
        debug!("スキーマ作成: {}", schema_name);
        let create_schema = format!("CREATE SCHEMA IF NOT EXISTS \"{}\";", schema_name);
        admin_conn
            .execute(Statement::from_string(DbBackend::Postgres, create_schema))
            .await
            .expect("スキーマ作成に失敗");

        // 新しいコネクションを作成（このテストのみで使用）
        let mut test_opt = ConnectOptions::new(url);
        test_opt
            .max_connections(5)
            .min_connections(1)
            .connect_timeout(Duration::from_secs(10))
            .acquire_timeout(Duration::from_secs(10))
            .sqlx_logging(true)
            // 明示的に新しいスキーマをデフォルトの検索パスとして設定
            .set_schema_search_path(schema_name.clone());

        let connection = Database::connect(test_opt).await.unwrap();

        // 明示的に検索パスを設定（確認のため）
        let set_search_path = format!("SET search_path TO \"{}\";", schema_name);
        connection
            .execute(Statement::from_string(DbBackend::Postgres, set_search_path))
            .await
            .expect("search pathの設定に失敗");

        // 現在のsearch_pathを確認
        let show_search_path = "SHOW search_path;";
        let result = connection
            .query_one(Statement::from_string(
                DbBackend::Postgres,
                show_search_path.to_string(),
            ))
            .await
            .expect("search pathの確認に失敗");

        debug!("設定されたsearch_path: {:?}", result);

        // マイグレーションを実行
        debug!("マイグレーション実行開始: {}", schema_name);
        Migrator::up(&connection, None)
            .await
            .expect("マイグレーション実行に失敗");
        debug!("マイグレーション実行完了: {}", schema_name);

        // テーブルが正しく作成されたか確認
        let tables_query = r#"
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = $1
            ORDER BY table_name;
        "#;

        let results = connection
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                tables_query,
                [schema_name.clone().into()],
            ))
            .await
            .expect("テーブル一覧取得に失敗");

        if results.is_empty() {
            panic!("スキーマ {} にテーブルが作成されていません", schema_name);
        }

        let table_names: Vec<String> = results
            .iter()
            .map(|row| row.try_get("", "table_name").unwrap_or_default())
            .collect();

        debug!("スキーマ {} のテーブル一覧: {:?}", schema_name, table_names);

        // 特に tasks テーブルが存在するか確認
        if !table_names.contains(&"tasks".to_string()) {
            panic!("スキーマ {} に tasks テーブルが見つかりません", schema_name);
        }

        info!("テストデータベースの準備完了: {}", schema_name);

        Self {
            connection,
            schema_name,
        }
    }

    // スキーマ名を取得するメソッドを追加
    pub fn get_schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // テスト終了時にスキーマをクリーンアップ
        let schema_name = self.schema_name.clone();
        let connection = self.connection.clone();

        tokio::spawn(async move {
            debug!("スキーマ削除開始: {}", schema_name);
            let drop_schema = format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE;", schema_name);
            match connection
                .execute(Statement::from_string(DbBackend::Postgres, drop_schema))
                .await
            {
                Ok(_) => debug!("スキーマ正常に削除: {}", schema_name),
                Err(e) => debug!("スキーマ削除でエラー発生: {} - {:?}", schema_name, e),
            }
        });
    }
}
