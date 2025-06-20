// src/error.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),

    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Multiple validation errors")]
    ValidationErrors(Vec<String>),

    #[error("Failed to parse UUID: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("Validation failed")]
    ValidationFailure(#[from] ValidationErrors),

    #[error("Bad request: {0}")]
    #[allow(dead_code)]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

// axum でエラーをHTTPレスポンスに変換するための実装
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_json) = match self {
            AppError::DbErr(db_err) => {
                eprintln!("Database error: {:?}", db_err); // サーバーログには詳細を出す

                // 具体的なDBエラーのタイプに基づいて適切なステータスコードを返す
                let status = match db_err {
                    sea_orm::DbErr::RecordNotFound(_) => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };

                // クライアントへのエラーメッセージをより具体的に
                let message = match db_err {
                    sea_orm::DbErr::RecordNotFound(_) => {
                        "The requested resource was not found".to_string()
                    }
                    _ => "A database error occurred".to_string(),
                };

                (
                    status,
                    json!({
                        "error": message,
                        "error_type": "database_error"
                    }),
                )
            }
            AppError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                json!({
                    "error": message,
                    "error_type": "not_found"
                }),
            ),
            AppError::ValidationError(message) => (
                StatusCode::BAD_REQUEST,
                json!({
                    "error": message,
                    "error_type": "validation_error"
                }),
            ),
            AppError::ValidationErrors(errors) => (
                StatusCode::BAD_REQUEST,
                json!({
                    "errors": errors.iter().map(|e| json!({"message": e})).collect::<Vec<_>>(),
                    "error_type": "validation_errors"
                }),
            ),
            AppError::UuidError(err) => (
                StatusCode::BAD_REQUEST,
                json!({
                    "error": format!("Invalid UUID: {}", err),
                    "error_type": "invalid_uuid"
                }),
            ),
            AppError::ValidationFailure(errors) => (
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "Validation failed",
                    "details": errors.field_errors(),
                    "error_type": "validation_failure"
                }),
            ),
            AppError::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                json!({
                    "error": message,
                    "error_type": "bad_request"
                }),
            ),
            AppError::Unauthorized(message) => (
                StatusCode::UNAUTHORIZED,
                json!({
                    "error": message,
                    "error_type": "unauthorized"
                }),
            ),
            AppError::Forbidden(message) => (
                StatusCode::FORBIDDEN,
                json!({
                    "error": message,
                    "error_type": "forbidden"
                }),
            ),
            AppError::Conflict(message) => (
                StatusCode::CONFLICT,
                json!({
                    "error": message,
                    "error_type": "conflict"
                }),
            ),
            AppError::InternalServerError(message) => {
                eprintln!("Internal server error: {}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({
                        "error": "An internal server error occurred",
                        "error_type": "internal_server_error"
                    }),
                )
            }
        };

        (status, Json(error_json)).into_response()
    }
}

// Result 型のエイリアス
pub type AppResult<T> = Result<T, AppError>;
