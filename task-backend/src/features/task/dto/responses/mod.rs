// src/features/task/dto/responses/mod.rs
use crate::core::task_status::TaskStatus;
use crate::shared::types::pagination::PaginatedResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Response DTO ---

#[derive(Serialize, Deserialize, Debug, Clone)] // Clone を追加
pub struct TaskDto {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: String,
    pub due_date: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// SeaORM の Model から TaskDto への変換
impl From<crate::features::task::models::task_model::Model> for TaskDto {
    fn from(model: crate::features::task::models::task_model::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            description: model.description,
            status: TaskStatus::from_str(&model.status).unwrap_or(TaskStatus::Todo),
            priority: model.priority,
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
