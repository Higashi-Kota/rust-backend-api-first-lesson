// tests/common/mod.rs
pub mod app_helper;
pub mod auth_helper;
pub mod db;
pub mod mock_storage;
pub mod permission_helpers;
pub mod request;
pub mod stripe_helper;
pub mod test_data;

use chrono::Utc;
use std::sync::Once;
use task_backend::api::dto::task_dto::{CreateTaskDto, TaskDto, UpdateTaskDto};
use task_backend::domain::task_status::TaskStatus;
use uuid::Uuid;

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

// テストデータジェネレーター
pub fn create_test_task() -> CreateTaskDto {
    CreateTaskDto {
        title: "Test Task".to_string(),
        description: Some("Test Description".to_string()),
        status: Some(TaskStatus::Todo),
        priority: None,
        due_date: Some(Utc::now()),
    }
}

pub fn create_test_task_with_title(title: &str) -> CreateTaskDto {
    CreateTaskDto {
        title: title.to_string(),
        description: Some("Test Description".to_string()),
        status: Some(TaskStatus::Todo),
        priority: None,
        due_date: Some(Utc::now()),
    }
}

pub fn create_update_task() -> UpdateTaskDto {
    UpdateTaskDto {
        title: Some("Updated Task".to_string()),
        description: Some("Updated Description".to_string()),
        status: Some(TaskStatus::InProgress),
        priority: None,
        due_date: Some(Utc::now()),
    }
}

// タスクIDの検証用ヘルパー
pub fn is_valid_uuid(task: &TaskDto) -> bool {
    task.id != Uuid::nil()
}

// UUID検証ヘルパー関数
#[allow(dead_code)]
pub mod uuid_helper {
    use uuid::Uuid;

    /// 文字列がValidなUUIDかを検証
    pub fn validate_uuid_str(uuid_str: &str) -> Result<Uuid, String> {
        Uuid::parse_str(uuid_str).map_err(|_| format!("Invalid UUID format: '{}'", uuid_str))
    }

    /// URLパスからUUIDを抽出して検証
    pub fn extract_and_validate_uuid(path: &str, param_name: &str) -> Result<Uuid, String> {
        let parts: Vec<&str> = path.split('/').collect();

        // パラメータ名から対応する位置を見つける
        let position = match param_name {
            "team_id" => parts.iter().position(|&p| p == "teams").map(|i| i + 1),
            "member_id" | "invitation_id" => {
                parts.iter().rposition(|&p| !p.is_empty() && p != "teams")
            }
            "task_id" => parts.iter().position(|&p| p == "tasks").map(|i| i + 1),
            "user_id" => parts.iter().position(|&p| p == "users").map(|i| i + 1),
            "organization_id" => parts
                .iter()
                .position(|&p| p == "organizations")
                .map(|i| i + 1),
            _ => None,
        };

        match position {
            Some(pos) if pos < parts.len() => validate_uuid_str(parts[pos])
                .map_err(|_| format!("Invalid UUID format for {}: '{}'", param_name, parts[pos])),
            _ => Err(format!(
                "Parameter '{}' not found in path: {}",
                param_name, path
            )),
        }
    }

    /// テスト用のランダムUUID生成
    pub fn generate_test_uuid() -> Uuid {
        Uuid::new_v4()
    }

    /// テスト用の固定UUID（再現可能なテスト用）
    pub fn test_uuid_1() -> Uuid {
        Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap()
    }

    pub fn test_uuid_2() -> Uuid {
        Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap()
    }
}
