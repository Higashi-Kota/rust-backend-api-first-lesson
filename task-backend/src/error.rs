// src/error.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use validator::ValidationErrors;

use crate::types::ApiResponse;

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
        let status = match &self {
            AppError::DbErr(db_err) => {
                eprintln!("Database error: {:?}", db_err);
                match db_err {
                    sea_orm::DbErr::RecordNotFound(_) => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                }
            }
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::ValidationErrors(_) => StatusCode::BAD_REQUEST,
            AppError::UuidError(_) => StatusCode::BAD_REQUEST,
            AppError::ValidationFailure(_) => StatusCode::BAD_REQUEST,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::InternalServerError(message) => {
                eprintln!("Internal server error: {}", message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::ExternalServiceError(message) => {
                eprintln!("External service error: {}", message);
                StatusCode::SERVICE_UNAVAILABLE
            }
        };

        let error_response = ApiResponse::<()>::error(self);
        (status, error_response).into_response()
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

/// エラー詳細情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

impl From<AppError> for ErrorDetail {
    fn from(error: AppError) -> Self {
        match &error {
            AppError::DbErr(db_err) => {
                let message = match db_err {
                    sea_orm::DbErr::RecordNotFound(_) => "The requested resource was not found",
                    sea_orm::DbErr::Exec(_) => "A database operation failed",
                    sea_orm::DbErr::Query(_) => "A database query failed",
                    _ => "A database error occurred",
                };
                ErrorDetail {
                    code: "DATABASE_ERROR".to_string(),
                    message: message.to_string(),
                    field: None,
                }
            }
            AppError::NotFound(msg) => ErrorDetail {
                code: "NOT_FOUND".to_string(),
                message: msg.clone(),
                field: None,
            },
            AppError::ValidationError(msg) => ErrorDetail {
                code: "VALIDATION_ERROR".to_string(),
                message: msg.clone(),
                field: None,
            },
            AppError::ValidationErrors(errors) => ErrorDetail {
                code: "VALIDATION_ERRORS".to_string(),
                message: format!("Multiple validation errors: {}", errors.join(", ")),
                field: None,
            },
            AppError::UuidError(err) => ErrorDetail {
                code: "UUID_ERROR".to_string(),
                message: format!("Invalid UUID: {}", err),
                field: None,
            },
            AppError::ValidationFailure(errors) => {
                let fields: Vec<String> = errors
                    .field_errors()
                    .keys()
                    .map(|k| (*k).to_string())
                    .collect();
                ErrorDetail {
                    code: "VALIDATION_FAILURE".to_string(),
                    message: format!("Validation failed for fields: {}", fields.join(", ")),
                    field: None,
                }
            }
            AppError::BadRequest(msg) => ErrorDetail {
                code: "BAD_REQUEST".to_string(),
                message: msg.clone(),
                field: None,
            },
            AppError::Unauthorized(msg) => ErrorDetail {
                code: "UNAUTHORIZED".to_string(),
                message: msg.clone(),
                field: None,
            },
            AppError::Forbidden(msg) => ErrorDetail {
                code: "FORBIDDEN".to_string(),
                message: msg.clone(),
                field: None,
            },
            AppError::Conflict(msg) => ErrorDetail {
                code: "CONFLICT".to_string(),
                message: msg.clone(),
                field: None,
            },
            AppError::InternalServerError(_) => ErrorDetail {
                code: "INTERNAL_SERVER_ERROR".to_string(),
                message: "An internal server error occurred".to_string(),
                field: None,
            },
            AppError::ExternalServiceError(msg) => ErrorDetail {
                code: "EXTERNAL_SERVICE_ERROR".to_string(),
                message: msg.clone(),
                field: None,
            },
        }
    }
}
