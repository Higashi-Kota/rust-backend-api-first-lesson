// task-backend/src/api/dto/common.rs

use serde::{Deserialize, Serialize};

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

    /// メタデータ付き成功レスポンスを作成
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

/// ページネーションクエリパラメータ
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

impl PaginationQuery {
    /// デフォルト値を適用してページとper_pageを取得
    pub fn get_pagination(&self) -> (i32, i32) {
        let page = self.page.unwrap_or(1).max(1);
        let per_page = self.per_page.unwrap_or(20).clamp(1, 100);
        (page, per_page)
    }

    /// オフセットを計算
    pub fn get_offset(&self) -> i32 {
        let (page, per_page) = self.get_pagination();
        (page - 1) * per_page
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
