// src/error.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
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
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

// axum でエラーをHTTPレスポンスに変換するための実装
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            AppError::DbErr(db_err) => {
                eprintln!("Database error: {:?}", db_err); // サーバーログには詳細を出す

                // 具体的なDBエラーのタイプに基づいて適切なステータスコードを返す
                let status = match db_err {
                    sea_orm::DbErr::RecordNotFound(_) => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };

                // クライアントへのエラーメッセージをより具体的に
                let (message, details) = match &db_err {
                    sea_orm::DbErr::RecordNotFound(entity) => (
                        "The requested resource was not found".to_string(),
                        Some(json!({ "entity": entity })),
                    ),
                    sea_orm::DbErr::Exec(_msg) => (
                        "A database operation failed".to_string(),
                        Some(json!({ "operation": "exec", "hint": "Check database connection" })),
                    ),
                    sea_orm::DbErr::Query(_msg) => (
                        "A database query failed".to_string(),
                        Some(json!({ "operation": "query", "hint": "Check query syntax" })),
                    ),
                    _ => ("A database error occurred".to_string(), None),
                };

                (
                    status,
                    ErrorResponse {
                        success: false,
                        error: message.clone(),
                        message,
                        details,
                        validation_errors: None,
                        errors: None,
                        error_type: "database_error".to_string(),
                    },
                )
            }
            AppError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    success: false,
                    error: message.clone(),
                    message,
                    details: None,
                    validation_errors: None,
                    errors: None,
                    error_type: "not_found".to_string(),
                },
            ),
            AppError::ValidationError(message) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    success: false,
                    error: message.clone(),
                    message,
                    details: None,
                    validation_errors: None,
                    errors: None,
                    error_type: "validation_error".to_string(),
                },
            ),
            AppError::ValidationErrors(errors) => {
                let mut field_errors = HashMap::new();
                for error in &errors {
                    if let Some((field, message)) = error.split_once(": ") {
                        field_errors
                            .entry(field.to_string())
                            .or_insert_with(Vec::new)
                            .push(message.to_string());
                    }
                }
                let errors_array: Vec<serde_json::Value> =
                    errors.iter().map(|e| json!({"message": e})).collect();
                (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse {
                        success: false,
                        error: "Validation failed".to_string(),
                        message: "Validation failed".to_string(),
                        details: None,
                        validation_errors: Some(field_errors),
                        errors: Some(errors_array),
                        error_type: "validation_errors".to_string(),
                    },
                )
            }
            AppError::UuidError(err) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    success: false,
                    error: format!("Invalid UUID: {}", err),
                    message: format!("Invalid UUID: {}", err),
                    details: None,
                    validation_errors: None,
                    errors: None,
                    error_type: "invalid_uuid".to_string(),
                },
            ),
            AppError::ValidationFailure(errors) => {
                let field_errors: HashMap<String, Vec<String>> = errors
                    .field_errors()
                    .into_iter()
                    .map(|(field, errors)| {
                        let messages = errors
                            .iter()
                            .map(|e| {
                                e.message
                                    .as_ref()
                                    .map_or_else(|| "Invalid value".to_string(), |m| m.to_string())
                            })
                            .collect();
                        (field.to_string(), messages)
                    })
                    .collect();
                let errors_array: Vec<serde_json::Value> = field_errors
                    .iter()
                    .flat_map(|(field, messages)| {
                        messages
                            .iter()
                            .map(move |msg| json!({"message": format!("{}: {}", field, msg)}))
                    })
                    .collect();
                (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse {
                        success: false,
                        error: "Validation failed".to_string(),
                        message: "Validation failed".to_string(),
                        details: None,
                        validation_errors: Some(field_errors),
                        errors: Some(errors_array),
                        error_type: "validation_errors".to_string(),
                    },
                )
            }
            AppError::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    success: false,
                    error: message.clone(),
                    message,
                    details: None,
                    validation_errors: None,
                    errors: None,
                    error_type: "bad_request".to_string(),
                },
            ),
            AppError::Unauthorized(message) => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    success: false,
                    error: message.clone(),
                    message,
                    details: None,
                    validation_errors: None,
                    errors: None,
                    error_type: "unauthorized".to_string(),
                },
            ),
            AppError::Forbidden(message) => (
                StatusCode::FORBIDDEN,
                ErrorResponse {
                    success: false,
                    error: message.clone(),
                    message,
                    details: None,
                    validation_errors: None,
                    errors: None,
                    error_type: "forbidden".to_string(),
                },
            ),
            AppError::Conflict(message) => (
                StatusCode::CONFLICT,
                ErrorResponse {
                    success: false,
                    error: message.clone(),
                    message,
                    details: None,
                    validation_errors: None,
                    errors: None,
                    error_type: "conflict".to_string(),
                },
            ),
            AppError::InternalServerError(message) => {
                eprintln!("Internal server error: {}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        success: false,
                        error: "An internal server error occurred".to_string(),
                        message: "An internal server error occurred".to_string(),
                        details: None,
                        validation_errors: None,
                        errors: None,
                        error_type: "internal_server_error".to_string(),
                    },
                )
            }
            AppError::ExternalServiceError(message) => {
                eprintln!("External service error: {}", message);
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    ErrorResponse {
                        success: false,
                        error: "External service error".to_string(),
                        message: message.clone(),
                        details: None,
                        validation_errors: None,
                        errors: None,
                        error_type: "external_service_error".to_string(),
                    },
                )
            }
        };

        (status, Json(error_response)).into_response()
    }
}

// Result 型のエイリアス
pub type AppResult<T> = Result<T, AppError>;

/// 統一的なエラーレスポンス構造
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<serde_json::Value>>,
    pub error_type: String,
}
