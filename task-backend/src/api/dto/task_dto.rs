// src/api/dto/task_dto.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::domain::task_model; // task_model を参照

// --- Request DTOs ---

#[derive(Deserialize, Debug)]
pub struct CreateTaskDto {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>, // 省略時はデフォルト値を使いたい場合
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateTaskDto {
    pub title: Option<String>,
    pub description: Option<String>, // Option<Option<String>> で明示的な null 設定も可能
    pub status: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
}

// --- Batch Request DTOs ---

#[derive(Deserialize, Debug)]
pub struct BatchCreateTaskDto {
    pub tasks: Vec<CreateTaskDto>,
}

#[derive(Deserialize, Debug, Clone)] // Clone を追加
pub struct BatchUpdateTaskItemDto {
    pub id: Uuid,
    // UpdateTaskDto と同じフィールドを持つか、必要なフィールドだけにするか選択
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug)]
pub struct BatchUpdateTaskDto {
    pub tasks: Vec<BatchUpdateTaskItemDto>,
}

#[derive(Deserialize, Debug)]
pub struct BatchDeleteTaskDto {
    pub ids: Vec<Uuid>,
}


// --- Response DTO ---

#[derive(Serialize, Debug)]
pub struct TaskDto {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// SeaORM の Model から TaskDto への変換
impl From<task_model::Model> for TaskDto {
    fn from(model: task_model::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            description: model.description,
            status: model.status,
            due_date: model.due_date,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

// --- Batch Response DTOs (任意) ---
#[derive(Serialize, Debug)]
pub struct BatchCreateResponseDto {
    pub created_tasks: Vec<TaskDto>,
    // pub errors: Vec<String>, // エラーがあった場合の詳細など
}

#[derive(Serialize, Debug)]
pub struct BatchUpdateResponseDto {
    pub updated_count: usize,
    // pub updated_tasks: Vec<TaskDto>, // 更新後のタスクを返す場合
    // pub errors: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct BatchDeleteResponseDto {
    pub deleted_count: usize,
    // pub errors: Vec<String>,
}

// --- フィルタリング用DTO ---
#[derive(Deserialize, Debug, Default)]
pub struct TaskFilterDto {
    pub status: Option<String>,
    pub title_contains: Option<String>,
    pub description_contains: Option<String>,
    pub due_date_before: Option<DateTime<Utc>>,
    pub due_date_after: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub sort_by: Option<String>, // "title", "due_date", "created_at", "status"
    pub sort_order: Option<String>, // "asc" or "desc"
}

// --- ページネーション用DTO ---
#[derive(Serialize, Debug)]
pub struct PaginatedTasksDto {
    pub tasks: Vec<TaskDto>,
    pub pagination: PaginationDto,
}

#[derive(Serialize, Debug)]
pub struct PaginationDto {
    pub current_page: u64,
    pub page_size: u64,
    pub total_items: u64,
    pub total_pages: u64,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}