use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ErrorDetail;
use crate::shared::types::PaginationMeta;
use crate::types::Timestamp;

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseMeta {
    pub request_id: String,
    pub timestamp: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<ResponsePaginationMeta>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ResponsePaginationMeta {
    pub current_page: i32,
    pub page_size: i32,
    pub total_pages: i32,
    pub total_items: i64,
}

impl From<PaginationMeta> for ResponsePaginationMeta {
    fn from(meta: PaginationMeta) -> Self {
        Self {
            current_page: meta.page,
            page_size: meta.per_page,
            total_pages: meta.total_pages,
            total_items: meta.total_count,
        }
    }
}

impl ResponseMeta {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            timestamp: Timestamp::now(),
            pagination: None,
        }
    }
}

impl Default for ResponseMeta {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(ResponseMeta::new()),
        }
    }
}

impl ApiResponse<()> {
    pub fn error(error: impl Into<ErrorDetail>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
            meta: Some(ResponseMeta::new()),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_success_response() {
        let data = vec!["item1", "item2"];
        let response = ApiResponse::success(data.clone());

        assert!(response.success);
        assert_eq!(response.data, Some(data));
        assert!(response.error.is_none());
        assert!(response.meta.is_some());
    }

    #[test]
    fn test_error_response() {
        let error = ErrorDetail {
            code: "TEST_ERROR".to_string(),
            message: "Test error message".to_string(),
            field: None,
        };
        let response = ApiResponse::<()>::error(error.clone());

        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error.as_ref().unwrap().code, error.code);
        assert!(response.meta.is_some());
    }

    #[test]
    fn test_response_meta_timestamp_is_unix_timestamp() {
        let response = ApiResponse::success("test");
        let meta = response.meta.unwrap();

        // timestampがUnix timestamp形式でシリアライズされることを確認
        let serialized = serde_json::to_value(&meta).unwrap();
        assert!(serialized["timestamp"].is_i64());

        // 現在時刻に近い値であることを確認（±5秒の範囲）
        let timestamp_value = serialized["timestamp"].as_i64().unwrap();
        let now = Utc::now().timestamp();
        assert!((timestamp_value - now).abs() <= 5);
    }
}
