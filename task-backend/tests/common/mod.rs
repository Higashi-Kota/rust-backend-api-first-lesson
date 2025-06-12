// tests/common/mod.rs
pub mod app_helper;
pub mod auth_helper;
pub mod db;
pub mod test_data;

use chrono::Utc;
use task_backend::api::dto::task_dto::{CreateTaskDto, TaskDto, UpdateTaskDto};
use uuid::Uuid;

// テストデータジェネレーター
pub fn create_test_task() -> CreateTaskDto {
    CreateTaskDto {
        title: "Test Task".to_string(),
        description: Some("Test Description".to_string()),
        status: Some("todo".to_string()),
        due_date: Some(Utc::now()),
    }
}

pub fn create_test_task_with_title(title: &str) -> CreateTaskDto {
    CreateTaskDto {
        title: title.to_string(),
        description: Some("Test Description".to_string()),
        status: Some("todo".to_string()),
        due_date: Some(Utc::now()),
    }
}

pub fn create_update_task() -> UpdateTaskDto {
    UpdateTaskDto {
        title: Some("Updated Task".to_string()),
        description: Some("Updated Description".to_string()),
        status: Some("in_progress".to_string()),
        due_date: Some(Utc::now()),
    }
}

// タスクIDの検証用ヘルパー
pub fn is_valid_uuid(task: &TaskDto) -> bool {
    task.id != Uuid::nil()
}
