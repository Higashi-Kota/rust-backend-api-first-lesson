// src/error.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::{ValidationError, ValidationErrors};

use crate::types::ApiResponse;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DbErr(#[from] DbErr),

    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("Failed to parse UUID: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("Validation failed")]
    ValidationFailure(#[from] ValidationErrors),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

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
            AppError::UuidError(_) => StatusCode::BAD_REQUEST,
            AppError::ValidationFailure(_) => StatusCode::BAD_REQUEST,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
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

/// エラー詳細情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

impl AppError {
    fn to_error_detail(&self) -> ErrorDetail {
        match self {
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
            AppError::UuidError(err) => ErrorDetail {
                code: "UUID_ERROR".to_string(),
                message: format!("Invalid UUID: {}", err),
                field: None,
            },
            AppError::ValidationFailure(errors) => {
                let messages: Vec<String> = errors
                    .field_errors()
                    .into_iter()
                    .flat_map(|(field, errors)| {
                        errors.iter().map(move |error| {
                            format!(
                                "{}: {}",
                                field,
                                error.message.as_ref().unwrap_or(&"Invalid value".into())
                            )
                        })
                    })
                    .collect();
                ErrorDetail {
                    code: "VALIDATION_ERROR".to_string(),
                    message: messages.join(", "),
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
            AppError::InternalServerError(_) => ErrorDetail::internal(),
            AppError::ExternalServiceError(msg) => ErrorDetail {
                code: "EXTERNAL_SERVICE_ERROR".to_string(),
                message: msg.clone(),
                field: None,
            },
            AppError::Validation(err) => ErrorDetail {
                code: "VALIDATION_ERROR".to_string(),
                message: err.to_string(),
                field: None,
            },
        }
    }
}

impl ErrorDetail {
    pub fn internal() -> Self {
        ErrorDetail {
            code: "INTERNAL_SERVER_ERROR".to_string(),
            message: "An internal server error occurred".to_string(),
            field: None,
        }
    }
}

impl From<AppError> for ErrorDetail {
    fn from(error: AppError) -> Self {
        error.to_error_detail()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[derive(Debug, Deserialize, Validate)]
    struct TestRequest {
        #[validate(length(min = 1, max = 100))]
        name: String,
        #[validate(range(min = 1, max = 5))]
        priority: i32,
    }

    #[test]
    fn test_error_to_detail_conversion() {
        // Test existing error variants
        let not_found_err = AppError::NotFound("User not found".to_string());
        let detail = not_found_err.to_error_detail();
        assert_eq!(detail.code, "NOT_FOUND");
        assert_eq!(detail.message, "User not found");

        let unauthorized_err = AppError::Unauthorized("Invalid token".to_string());
        let detail = unauthorized_err.to_error_detail();
        assert_eq!(detail.code, "UNAUTHORIZED");
        assert_eq!(detail.message, "Invalid token");

        let forbidden_err = AppError::Forbidden("Access denied".to_string());
        let detail = forbidden_err.to_error_detail();
        assert_eq!(detail.code, "FORBIDDEN");
        assert_eq!(detail.message, "Access denied");

        let internal_err = AppError::InternalServerError("System failure".to_string());
        let detail = internal_err.to_error_detail();
        assert_eq!(detail.code, "INTERNAL_SERVER_ERROR");
        assert_eq!(detail.message, "An internal server error occurred");

        // Test new unified variants
        let validation_err = AppError::Validation(ValidationError::new("Invalid field"));
        let detail = validation_err.to_error_detail();
        assert_eq!(detail.code, "VALIDATION_ERROR");
        // ValidationError includes additional formatting
        assert!(detail.message.contains("Invalid field"));
    }

    #[test]
    fn test_validation_with_validator_crate() {
        let test_data = TestRequest {
            name: "".to_string(), // Invalid: too short
            priority: 10,         // Invalid: out of range
        };

        let result = test_data.validate();
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("name"));
        assert!(errors.field_errors().contains_key("priority"));
    }
}
