// src/api/dto/task_dto.rs
use crate::api::dto::PaginatedResponse;
use crate::domain::task_model;
use crate::utils::validation::common;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// --- Request DTOs ---

#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct CreateTaskDto {
    #[validate(
        length(
            min = common::task::TITLE_MIN_LENGTH,
            max = common::task::TITLE_MAX_LENGTH,
            message = "Task title must be between 1 and 200 characters"
        ),
        custom(function = common::validate_task_title)
    )]
    pub title: String,

    #[validate(length(
        max = common::task::DESCRIPTION_MAX_LENGTH,
        message = "Task description must not exceed 2000 characters"
    ))]
    pub description: Option<String>,

    pub status: Option<String>, // 省略時はデフォルト値を使いたい場合
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct UpdateTaskDto {
    #[validate(
        length(
            min = common::task::TITLE_MIN_LENGTH,
            max = common::task::TITLE_MAX_LENGTH,
            message = "Task title must be between 1 and 200 characters"
        ),
        custom(function = common::validate_task_title)
    )]
    pub title: Option<String>,

    #[validate(length(
        max = common::task::DESCRIPTION_MAX_LENGTH,
        message = "Task description must not exceed 2000 characters"
    ))]
    pub description: Option<String>,

    pub status: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
}

// --- Batch Request DTOs ---

#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct BatchCreateTaskDto {
    #[validate(nested)]
    pub tasks: Vec<CreateTaskDto>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Validate)]
pub struct BatchUpdateTaskItemDto {
    pub id: Uuid,

    #[validate(
        length(
            min = common::task::TITLE_MIN_LENGTH,
            max = common::task::TITLE_MAX_LENGTH,
            message = "Task title must be between 1 and 200 characters"
        ),
        custom(function = common::validate_task_title)
    )]
    pub title: Option<String>,

    #[validate(length(
        max = common::task::DESCRIPTION_MAX_LENGTH,
        message = "Task description must not exceed 2000 characters"
    ))]
    pub description: Option<String>,

    pub status: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct BatchUpdateTaskDto {
    #[validate(nested)]
    pub tasks: Vec<BatchUpdateTaskItemDto>,
}

#[derive(Deserialize, Serialize, Debug)] // Serialize を追加
pub struct BatchDeleteTaskDto {
    pub ids: Vec<Uuid>,
}

// --- Response DTO ---

#[derive(Serialize, Deserialize, Debug)] // Deserialize を追加
pub struct TaskDto {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub due_date: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>,
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
            user_id: model.user_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

// --- Batch Response DTOs (任意) ---
#[derive(Serialize, Deserialize, Debug)] // Deserialize を追加
pub struct BatchCreateResponseDto {
    pub created_tasks: Vec<TaskDto>,
    pub created_count: usize,
    // pub errors: Vec<String>, // エラーがあった場合の詳細など
}

#[derive(Serialize, Deserialize, Debug)] // Deserialize を追加
pub struct BatchUpdateResponseDto {
    pub updated_count: usize,
    // pub updated_tasks: Vec<TaskDto>, // 更新後のタスクを返す場合
    // pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)] // Deserialize を追加
pub struct BatchDeleteResponseDto {
    pub deleted_count: usize,
    // pub errors: Vec<String>,
}

// --- フィルタリング用DTO ---
#[derive(Deserialize, Serialize, Debug, Default)] // Serialize を追加
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
/// ページネーション付きタスクレスポンス (統一構造体使用)
pub type PaginatedTasksDto = PaginatedResponse<TaskDto>;
