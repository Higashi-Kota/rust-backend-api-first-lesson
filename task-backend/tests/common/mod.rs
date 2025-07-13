// tests/common/mod.rs
pub mod app_helper;
pub mod auth_helper;
pub mod db;
pub mod mock_storage;
pub mod stripe_helper;
pub mod test_data;

use std::sync::Once;

// テスト環境の初期化を一度だけ実行
static INIT: Once = Once::new();

/// テスト環境を初期化
pub fn init_test_env() {
    INIT.call_once(|| {
        // .env.testファイルから環境変数を読み込む
        if std::path::Path::new(".env.test").exists() {
            dotenvy::from_filename(".env.test").ok();
        } else if std::path::Path::new("../.env.test").exists() {
            // task-backendディレクトリから実行される場合
            dotenvy::from_filename("../.env.test").ok();
        } else {
            // デフォルトの.envを読み込む
            dotenvy::dotenv().ok();
        }

        // テスト用のログ設定
        let _ = tracing_subscriber::fmt()
            .with_env_filter("task_backend=debug,tower_http=debug")
            .with_test_writer()
            .try_init();
    });
}
