// src/features/task/dto/requests/mod.rs
use crate::core::task_status::TaskStatus;
use crate::utils::validation::common;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// --- Request DTOs ---

#[derive(Deserialize, Serialize, Debug, Validate, Clone)]
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

    pub status: Option<TaskStatus>, // 省略時はデフォルト値を使いたい場合
    pub priority: Option<String>,   // 'low', 'medium', 'high'、省略時は'medium'
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

    pub status: Option<TaskStatus>,
    pub priority: Option<String>, // 'low', 'medium', 'high'
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

    pub status: Option<TaskStatus>,
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

// --- フィルタリング用DTO ---
#[derive(Deserialize, Serialize, Debug, Default)] // Serialize を追加
pub struct TaskFilterDto {
    pub status: Option<TaskStatus>,
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
