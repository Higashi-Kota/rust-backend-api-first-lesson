// src/api/dto/task_dto.rs
use crate::api::dto::PaginatedResponse;
use crate::domain::task_model;
use crate::domain::task_status::TaskStatus;
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

// --- Response DTO ---

#[derive(Serialize, Deserialize, Debug, Clone)] // Clone を追加
pub struct TaskDto {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
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
            status: TaskStatus::from_str(&model.status).unwrap_or(TaskStatus::Todo),
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
}

#[derive(Serialize, Deserialize, Debug)] // Deserialize を追加
pub struct BatchUpdateResponseDto {
    pub updated_count: usize,
}

#[derive(Serialize, Deserialize, Debug)] // Deserialize を追加
pub struct BatchDeleteResponseDto {
    pub deleted_count: usize,
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

impl TaskResponse {
    /// タスク一覧を取得
    pub fn tasks(&self) -> &Vec<TaskDto> {
        match self {
            TaskResponse::Limited(paginated) => &paginated.items,
            TaskResponse::Enhanced(paginated) => &paginated.items,
            TaskResponse::Unlimited(paginated) => &paginated.items,
            TaskResponse::Free { tasks, .. } => tasks,
            TaskResponse::Pro { tasks, .. } => tasks,
            TaskResponse::Enterprise { tasks, .. } => tasks,
        }
    }

    /// 総件数を取得
    pub fn task_count(&self) -> usize {
        self.tasks().len()
    }

    /// 利用可能機能一覧を取得
    pub fn features(&self) -> Vec<String> {
        match self {
            TaskResponse::Limited(_) => vec!["basic_access".to_string()],
            TaskResponse::Enhanced(_) => vec![
                "basic_access".to_string(),
                "advanced_filter".to_string(),
                "export".to_string(),
            ],
            TaskResponse::Unlimited(_) => vec![
                "unlimited_access".to_string(),
                "bulk_operations".to_string(),
                "enterprise_features".to_string(),
            ],
            TaskResponse::Free { .. } => vec!["basic_access".to_string()],
            TaskResponse::Pro { features, .. } => features.clone(),
            TaskResponse::Enterprise { .. } => vec![
                "unlimited_access".to_string(),
                "bulk_operations".to_string(),
                "enterprise_features".to_string(),
            ],
        }
    }

    /// 総件数を取得（互換性のため）
    pub fn total_count(&self) -> usize {
        self.task_count()
    }
}
