// task-backend/src/api/dto/common.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    #[allow(dead_code)]
    pub fn success_message(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: true,
            message: message.into(),
            data: Some(()),
            metadata: None,
        }
    }

    /// メタデータ付き成功レスポンスを作成
    #[allow(dead_code)]
    pub fn success_with_metadata(
        message: impl Into<String>,
        data: T,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
            metadata: Some(metadata),
        }
    }
}

/// 統一APIエラーレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub success: bool,
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<HashMap<String, Vec<String>>>,
}

impl ApiError {
    /// 基本エラーレスポンスを作成
    #[allow(dead_code)]
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            error: error.into(),
            message: message.into(),
            details: None,
            validation_errors: None,
        }
    }

    /// 詳細付きエラーレスポンスを作成
    #[allow(dead_code)]
    pub fn with_details(
        error: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            success: false,
            error: error.into(),
            message: message.into(),
            details: Some(details),
            validation_errors: None,
        }
    }

    /// バリデーションエラーレスポンスを作成
    #[allow(dead_code)]
    pub fn validation_error(
        message: impl Into<String>,
        validation_errors: HashMap<String, Vec<String>>,
    ) -> Self {
        Self {
            success: false,
            error: "VALIDATION_ERROR".to_string(),
            message: message.into(),
            details: None,
            validation_errors: Some(validation_errors),
        }
    }

    /// 認証エラーレスポンスを作成
    #[allow(dead_code)]
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new("UNAUTHORIZED", message)
    }

    /// 権限不足エラーレスポンスを作成
    #[allow(dead_code)]
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new("FORBIDDEN", message)
    }

    /// 見つからないエラーレスポンスを作成
    #[allow(dead_code)]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message)
    }

    /// 競合エラーレスポンスを作成
    #[allow(dead_code)]
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new("CONFLICT", message)
    }

    /// 内部サーバーエラーレスポンスを作成
    #[allow(dead_code)]
    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_SERVER_ERROR", message)
    }

    /// 不正なリクエストエラーレスポンスを作成
    #[allow(dead_code)]
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("BAD_REQUEST", message)
    }
}

/// ページネーション情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
    pub total_count: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationMeta {
    pub fn new(page: i32, per_page: i32, total_count: i64) -> Self {
        let total_pages = ((total_count as f64) / (per_page as f64)).ceil() as i32;

        Self {
            page,
            per_page,
            total_pages,
            total_count,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

/// ページネーション付きレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, page: i32, per_page: i32, total_count: i64) -> Self {
        Self {
            items,
            pagination: PaginationMeta::new(page, per_page, total_count),
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

    #[allow(dead_code)]
    pub fn created(item: T) -> Self {
        Self::new(item, vec!["Created".to_string()])
    }

    pub fn updated(item: T, changes: Vec<String>) -> Self {
        Self::new(item, changes)
    }

    #[allow(dead_code)]
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
    fn test_api_error_basic() {
        let error = ApiError::new("VALIDATION_ERROR", "Invalid input");
        assert!(!error.success);
        assert_eq!(error.error, "VALIDATION_ERROR");
        assert_eq!(error.message, "Invalid input");
    }

    #[test]
    fn test_api_error_validation() {
        let mut validation_errors = HashMap::new();
        validation_errors.insert("email".to_string(), vec!["Invalid format".to_string()]);

        let error = ApiError::validation_error("Validation failed", validation_errors);
        assert!(!error.success);
        assert_eq!(error.error, "VALIDATION_ERROR");
        assert!(error.validation_errors.is_some());
    }

    #[test]
    fn test_pagination_meta() {
        let pagination = PaginationMeta::new(2, 10, 25);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 10);
        assert_eq!(pagination.total_pages, 3);
        assert_eq!(pagination.total_count, 25);
        assert!(pagination.has_next);
        assert!(pagination.has_prev);
    }

    #[test]
    fn test_paginated_response() {
        let items = vec![1, 2, 3];
        let response = PaginatedResponse::new(items, 1, 3, 10);
        assert_eq!(response.items.len(), 3);
        assert_eq!(response.pagination.total_count, 10);
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
