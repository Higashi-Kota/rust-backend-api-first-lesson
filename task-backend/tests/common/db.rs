//! テスト用データベースヘルパー（SeaORMとtestcontainers‑rs v0.24を使用）

use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement};
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
    pub schema_name: String,
}

impl TestDatabase {
    // 標準のコンストラクタ
    pub async fn new() -> Self {
        // 新しいスキーマ名を生成
        let schema_name = format!("test_{}", Uuid::new_v4().to_string().replace('-', ""));
        Self::with_schema(schema_name).await
    }

    // 明示的にスキーマ名を指定するコンストラクタを追加
    pub async fn with_schema(schema_name: String) -> Self {
        // 共有コンテナを取得またはポートを作成
        let port = get_or_create_container().await;

        info!("テストスキーマを作成: {}", schema_name);

        // データベース接続URL
        let url = format!("postgres://postgres:postgres@localhost:{}/test_db", port);

        // 基本接続を作成（初期設定用）
        let admin_conn = Self::create_connection(&url, None)
            .await
            .expect("データベース接続の作成に失敗");

        // スキーマを作成
        debug!("スキーマ作成: {}", schema_name);
        Self::create_schema(&admin_conn, &schema_name)
            .await
            .expect("スキーマ作成に失敗");

        // 新しいコネクションを作成（指定されたスキーマを検索パスに設定）
        let connection = Self::create_connection(&url, Some(&schema_name))
            .await
            .expect("テスト用データベース接続の作成に失敗");

        // マイグレーションを実行
        debug!("マイグレーション実行開始: {}", schema_name);
        Migrator::up(&connection, None)
            .await
            .expect("マイグレーション実行に失敗");
        debug!("マイグレーション実行完了: {}", schema_name);

        // テーブルが正しく作成されたか確認
        Self::verify_schema(&connection, &schema_name).await;

        info!("テストデータベースの準備完了: {}", schema_name);

        Self {
            connection,
            schema_name,
        }
    }

    // データベース接続を作成するヘルパーメソッド
    async fn create_connection(
        url: &str,
        schema: Option<&str>,
    ) -> Result<DatabaseConnection, DbErr> {
        use sea_orm::ConnectOptions;

        let mut opt = ConnectOptions::new(url.to_string());
        opt.max_connections(10)
            .min_connections(1)
            .connect_timeout(Duration::from_secs(10))
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(10))
            .max_lifetime(Duration::from_secs(30))
            .sqlx_logging(true); // テスト中のSQLクエリログを有効化

        // スキーマ検索パスを設定（指定されている場合）
        if let Some(schema_name) = schema {
            opt.set_schema_search_path(schema_name.to_string());
        }

        let connection = Database::connect(opt).await?;

        // 明示的に検索パスを設定（指定されている場合）
        if let Some(schema_name) = schema {
            Self::set_schema_search_path(&connection, schema_name).await?;
        }

        Ok(connection)
    }

    // スキーマ検索パスを設定
    async fn set_schema_search_path(conn: &DatabaseConnection, schema: &str) -> Result<(), DbErr> {
        let set_search_path = format!("SET search_path TO \"{}\";", schema);
        conn.execute(Statement::from_string(DbBackend::Postgres, set_search_path))
            .await?;
        Ok(())
    }

    // スキーマを作成
    async fn create_schema(conn: &DatabaseConnection, schema: &str) -> Result<(), DbErr> {
        let create_schema = format!("CREATE SCHEMA IF NOT EXISTS \"{}\";", schema);
        conn.execute(Statement::from_string(DbBackend::Postgres, create_schema))
            .await?;
        Ok(())
    }

    // スキーマの整合性を確認
    async fn verify_schema(conn: &DatabaseConnection, schema: &str) {
        // テーブルが正しく作成されたか確認
        let tables_query = r#"
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = $1
            ORDER BY table_name;
        "#;

        let results = conn
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                tables_query,
                [schema.to_string().into()],
            ))
            .await
            .expect("テーブル一覧取得に失敗");

        if results.is_empty() {
            panic!("スキーマ {} にテーブルが作成されていません", schema);
        }

        let table_names: Vec<String> = results
            .iter()
            .map(|row| row.try_get("", "table_name").unwrap_or_default())
            .collect();

        debug!("スキーマ {} のテーブル一覧: {:?}", schema, table_names);

        // 特に tasks テーブルが存在するか確認
        if !table_names.contains(&"tasks".to_string()) {
            panic!("スキーマ {} に tasks テーブルが見つかりません", schema);
        }
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
