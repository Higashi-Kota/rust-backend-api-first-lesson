use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement};
use std::sync::{Arc, Weak};
use std::time::Duration;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt},
};
use tokio::sync::{Mutex, OnceCell};
use tracing::{debug, info};
use uuid::Uuid;

// 全テスト共有のPostgresコンテナとポート
static POSTGRES_CONTAINER: OnceCell<Mutex<Weak<Mutex<ContainerAsync<Postgres>>>>> =
    OnceCell::const_new();
static DB_PORT: OnceCell<u16> = OnceCell::const_new();

async fn get_or_create_container() -> Arc<Mutex<ContainerAsync<Postgres>>> {
    // Weak ポインタでコンテナを管理し、必要なら起動
    let mut guard = POSTGRES_CONTAINER
        .get_or_init(|| async { Mutex::new(Weak::new()) })
        .await
        .lock()
        .await;
    if let Some(container_arc) = guard.upgrade() {
        // すでにコンテナが存在する場合はそれを利用
        drop(guard);
        container_arc
    } else {
        // 新しい Postgres コンテナを起動（初回のみ）
        let image = Postgres::default()
            .with_tag("15-alpine")
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres")
            .with_env_var("POSTGRES_DB", "test_db");
        let container = image.start().await.expect("PostgreSQLコンテナの起動に失敗");
        let port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("マップされたポートの取得に失敗");
        DB_PORT.set(port).expect("DB_PORTの設定に失敗");

        // コンテナ安定化のため少し待機
        tokio::time::sleep(Duration::from_secs(2)).await;

        // コンテナを Arc 管理下に置き、Weak ポインタを格納
        let new_container_arc = Arc::new(Mutex::new(container));
        *guard = Arc::downgrade(&new_container_arc);
        drop(guard);
        new_container_arc
    }
}

pub struct TestDatabase {
    pub connection: DatabaseConnection,
    pub schema_name: String,
    #[allow(dead_code)]
    // この行を追加して警告を抑制 フィールドを 削除してしまうと強参照が失われ、最後のテストが終わる前にコンテナが Drop される可能性があるので、保持は必須です。
    _container: Arc<Mutex<ContainerAsync<Postgres>>>, // コンテナ参照を保持
}

impl TestDatabase {
    // 標準のコンストラクタ
    pub async fn new() -> Self {
        let schema_name = format!("test_{}", Uuid::new_v4().to_string().replace('-', ""));
        Self::with_schema(schema_name).await
    }

    // スキーマ名を指定するコンストラクタ
    pub async fn with_schema(schema_name: String) -> Self {
        // 共有コンテナを取得（必要に応じ起動）
        let container_arc = get_or_create_container().await;
        let port = *DB_PORT.get().expect("DB_PORTが設定されていません");

        info!("テストスキーマを作成: {}", schema_name);

        // 管理者用の基本接続を作成
        let url = format!("postgres://postgres:postgres@localhost:{}/test_db", port);
        let admin_conn = Self::create_connection(&url, None)
            .await
            .expect("データベース接続の作成に失敗");

        // スキーマを作成
        debug!("スキーマ作成: {}", schema_name);
        Self::create_schema(&admin_conn, &schema_name)
            .await
            .expect("スキーマ作成に失敗");

        // テスト用の接続を作成（検索パスにスキーマを設定）
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
            _container: container_arc, // コンテナ参照を保存
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
            .sqlx_logging(true);
        if let Some(schema_name) = schema {
            opt.set_schema_search_path(schema_name.to_string());
        }
        let connection = Database::connect(opt).await?;
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
