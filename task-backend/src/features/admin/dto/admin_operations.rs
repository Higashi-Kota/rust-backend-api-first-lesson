// task-backend/src/features/admin/dto/admin_operations.rs

use crate::features::task::dto::{BatchUpdateTaskItemDto, CreateTaskDto};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

// --- Task Management DTOs ---

/// 管理者向けタスク一括作成リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AdminBulkCreateTasksRequest {
    #[validate(length(min = 1, max = 100, message = "Must provide 1-100 tasks"))]
    pub tasks: Vec<CreateTaskDto>,
    pub assign_to_user: Option<Uuid>,
}

/// 管理者向けタスク一括更新リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AdminBulkUpdateTasksRequest {
    #[validate(length(min = 1, max = 100, message = "Must provide 1-100 task updates"))]
    pub updates: Vec<BatchUpdateTaskItemDto>,
}

/// 管理者向けタスク一括削除リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AdminBulkDeleteTasksRequest {
    #[validate(length(min = 1, max = 100, message = "Must provide 1-100 task IDs"))]
    pub task_ids: Vec<Uuid>,
}

/// 管理者向けタスク統計レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminTaskStatsResponse {
    pub total_tasks: u32,
    pub tasks_by_status: Vec<TaskStatusStats>,
    pub tasks_by_user: Vec<UserTaskStats>,
    pub recent_activity: Vec<TaskActivityStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatusStats {
    pub status: String,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTaskStats {
    pub user_id: Uuid,
    pub task_count: u64,
    pub completed_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskActivityStats {
    pub date: String,
    pub created_count: u64,
    pub completed_count: u64,
}

/// 管理者向け一括操作レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminBulkOperationResponse {
    pub success_count: usize,
    pub failed_count: usize,
    pub total_requested: usize,
    pub errors: Vec<String>,
}

// --- Subscription Management DTOs ---

/// ユーザーのサブスクリプション変更リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ChangeUserSubscriptionRequest {
    #[validate(length(min = 1, message = "New tier must not be empty"))]
    pub new_tier: String,
    pub reason: Option<String>,
}

/// ユーザーのサブスクリプション変更レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeUserSubscriptionResponse {
    pub user_id: Uuid,
    pub previous_tier: String,
    pub new_tier: String,
    pub changed_at: DateTime<Utc>,
    pub history_id: Uuid,
}

// --- Data Cleanup/Maintenance DTOs ---

/// 一括操作履歴レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct BulkOperationHistoryResponse {
    pub id: Uuid,
    pub operation_type: String,
    pub performed_by: Uuid,
    pub performed_by_username: Option<String>,
    pub affected_count: i32,
    pub status: String,
    pub error_details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkOperationRecord {
    pub id: Uuid,
    pub operation_type: String,
    pub admin_user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub success_count: u32,
    pub failed_count: u32,
    pub details: serde_json::Value,
}

/// クリーンアップ結果レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupResultResponse {
    pub operation_type: String,
    pub deleted_count: u64,
    pub before_date: Option<DateTime<Utc>>,
    pub performed_at: DateTime<Utc>,
}

/// ユーザー機能メトリクスレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct UserFeatureMetricsResponse {
    pub user_id: Uuid,
    pub action_counts: HashMap<String, i64>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureUsageMetric {
    pub feature_name: String,
    pub current_usage: u64,
    pub limit: Option<u64>,
    pub usage_percentage: Option<f64>,
}
