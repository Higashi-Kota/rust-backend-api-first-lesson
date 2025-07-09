// task-backend/src/api/dto/common.rs

use serde::{Deserialize, Serialize};

// ページネーション関連の型を shared::types::pagination から再エクスポート
pub use crate::shared::types::pagination::{PaginatedResponse, PaginationMeta, PaginationQuery};

/// 統一API成功レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl<T> ApiResponse<T> {
    /// 成功レスポンスを作成
    pub fn success(message: impl Into<String>, data: T) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
            metadata: None,
        }
    }

    /// メッセージのみの成功レスポンスを作成
    pub fn success_message(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: true,
            message: message.into(),
            data: Some(()),
            metadata: None,
        }
    }
}

/// 操作結果を表すレスポンス（作成・更新・削除用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult<T> {
    pub item: T,
    pub changes: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> OperationResult<T> {
    pub fn new(item: T, changes: Vec<String>) -> Self {
        Self {
            item,
            changes,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn created(item: T) -> Self {
        Self::new(item, vec!["Created".to_string()])
    }

    pub fn updated(item: T, changes: Vec<String>) -> Self {
        Self::new(item, changes)
    }

    pub fn deleted(item: T) -> Self {
        Self::new(item, vec!["Deleted".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("User created successfully", json!({"id": 1}));
        assert!(response.success);
        assert_eq!(response.message, "User created successfully");
        assert!(response.data.is_some());
    }

    #[test]
    fn test_api_response_success_message() {
        let response = ApiResponse::<()>::success_message("Operation completed");
        assert!(response.success);
        assert_eq!(response.message, "Operation completed");
        assert!(response.data.is_some());
    }

    #[test]
    fn test_operation_result() {
        let result = OperationResult::created(json!({"id": 1}));
        assert_eq!(result.changes, vec!["Created"]);
        assert!(result.timestamp <= chrono::Utc::now());

        let changes = vec!["Updated name".to_string(), "Updated email".to_string()];
        let result = OperationResult::updated(json!({"id": 1}), changes.clone());
        assert_eq!(result.changes, changes);
    }
}
