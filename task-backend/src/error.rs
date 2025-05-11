// src/error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sea_orm::DbErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),

    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Failed to parse UUID: {0}")]
    UuidError(#[from] uuid::Error),

    #[allow(dead_code)] // この行を追加して警告を抑制
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    // 他にも必要なエラーバリアントを追加できます
}

// axum でエラーをHTTPレスポンスに変換するための実装
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DbErr(db_err) => {
                eprintln!("Database error: {:?}", db_err); // サーバーログには詳細を出す
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "A database error occurred".to_string(),
                )
            }
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            AppError::ValidationError(message) => (StatusCode::BAD_REQUEST, message),
            AppError::UuidError(err) => (StatusCode::BAD_REQUEST, format!("Invalid UUID: {}", err)),
            AppError::InternalServerError(message) => {
                eprintln!("Internal server error: {}", message);
                (StatusCode::INTERNAL_SERVER_ERROR, message)
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

// Result 型のエイリアス
pub type AppResult<T> = Result<T, AppError>;