// src/api/dto/task_dto.rs
use crate::api::dto::common::PaginatedResponse;
use crate::domain::task_model;
use crate::domain::task_status::TaskStatus;
use crate::domain::task_visibility::TaskVisibility;
use crate::types::{optional_timestamp, Timestamp};
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
    #[serde(default, with = "optional_timestamp")]
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
    #[serde(default, with = "optional_timestamp")]
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
    #[serde(default, with = "optional_timestamp")]
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

#[derive(Serialize, Deserialize, Debug, Clone)] // Clone を追加
pub struct TaskDto {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: String,
    #[serde(default, with = "optional_timestamp")]
    pub due_date: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,

    // マルチテナント対応フィールド
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<Uuid>,
    pub visibility: TaskVisibility,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<Uuid>,

    // 表示用の追加情報
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_info: Option<String>,
}

// SeaORM の Model から TaskDto への変換
impl From<task_model::Model> for TaskDto {
    fn from(model: task_model::Model) -> Self {
        let owner_info = Some(model.get_owner_info());

        Self {
            id: model.id,
            title: model.title,
            description: model.description,
            status: TaskStatus::from_str(&model.status).unwrap_or(TaskStatus::Todo),
            priority: model.priority,
            due_date: model.due_date,
            user_id: model.user_id,
            created_at: Timestamp::from_datetime(model.created_at),
            updated_at: Timestamp::from_datetime(model.updated_at),
            team_id: model.team_id,
            organization_id: model.organization_id,
            visibility: model.visibility,
            assigned_to: model.assigned_to,
            owner_info,
        }
    }
}

// --- Batch Response DTOs (任意) ---
#[derive(Serialize, Deserialize, Debug)] // Deserialize を追加
pub struct BatchCreateResponseDto {
    pub created_tasks: Vec<TaskDto>,
    pub created_count: usize,
}

#[derive(Serialize, Deserialize, Debug)] // Deserialize を追加
pub struct BatchUpdateResponseDto {
    pub updated_count: usize,
}

#[derive(Serialize, Deserialize, Debug)] // Deserialize を追加
pub struct BatchDeleteResponseDto {
    pub deleted_count: usize,
}

// --- ページネーション用DTO ---
/// ページネーション付きタスクレスポンス (統一構造体使用)
pub type PaginatedTasksDto = PaginatedResponse<TaskDto>;

// --- 動的権限システム用レスポンス ---

/// サブスクリプション階層別タスクレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskResponse {
    /// 基本レスポンス（デフォルト）
    Limited(PaginatedTasksDto),
    /// 拡張レスポンス（Pro機能付き）
    Enhanced(PaginatedTasksDto),
    /// 無制限レスポンス（Enterprise）
    Unlimited(PaginatedTasksDto),
    /// Free階層レスポンス
    Free {
        tasks: Vec<TaskDto>,
        limit_reached: bool,
        quota_info: String,
    },
    /// Pro階層レスポンス
    Pro {
        tasks: Vec<TaskDto>,
        features: Vec<String>,
        export_available: bool,
    },
    /// Enterprise階層レスポンス
    Enterprise {
        tasks: Vec<TaskDto>,
        bulk_operations: bool,
        unlimited_access: bool,
    },
}

// TaskResponseの実装は削除（未使用のため）
