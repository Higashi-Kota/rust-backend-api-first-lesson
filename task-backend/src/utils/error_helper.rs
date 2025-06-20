// task-backend/src/utils/error_helper.rs

//! エラーハンドリングの統一化ヘルパー
//!
//! 全てのサービス層とハンドラー層で共通して使用するエラー処理パターンを提供します。

use crate::error::AppError;
use tracing::{error, warn};
use validator::ValidationErrors;

// =============================================================================
// バリデーションエラー処理の統一
// =============================================================================

/// validatorのValidationErrorsをAppErrorに変換する統一処理
///
/// # Arguments
/// * `validation_errors` - validator crate からのバリデーションエラー
/// * `context` - エラーが発生したコンテキスト（ログ用）
///
/// # Returns
/// * `AppError::ValidationErrors` - 統一された形式のバリデーションエラー
pub fn convert_validation_errors(validation_errors: ValidationErrors, context: &str) -> AppError {
    warn!(
        context = %context,
        error_count = validation_errors.field_errors().len(),
        "Validation failed"
    );

    let errors: Vec<String> = validation_errors
        .field_errors()
        .into_iter()
        .flat_map(|(field, errors)| {
            errors.iter().map(move |error| {
                let message = error
                    .message
                    .as_ref()
                    .map_or_else(|| "Invalid value".to_string(), |cow| cow.to_string());
                format!("{}: {}", field, message)
            })
        })
        .collect();

    AppError::ValidationErrors(errors)
}

// =============================================================================
// ログ付きエラー変換パターン
// =============================================================================

/// 内部サーバーエラーをログ付きで生成
///
/// # Arguments
/// * `error` - 元のエラー
/// * `context` - エラーが発生したコンテキスト
/// * `user_message` - ユーザーに表示するメッセージ
pub fn internal_server_error<E: std::fmt::Display>(
    error: E,
    context: &str,
    user_message: &str,
) -> AppError {
    error!(
        error = %error,
        context = %context,
        "Internal server error occurred"
    );
    AppError::InternalServerError(user_message.to_string())
}

/// リソース未発見エラーをログ付きで生成
pub fn not_found_error(resource: &str, identifier: &str, context: &str) -> AppError {
    warn!(
        context = %context,
        resource = %resource,
        identifier = %identifier,
        "Resource not found"
    );
    AppError::NotFound(format!(
        "{} with identifier {} not found",
        resource, identifier
    ))
}

/// 競合エラーをログ付きで生成
pub fn conflict_error(message: &str, context: &str) -> AppError {
    warn!(
        context = %context,
        message = %message,
        "Resource conflict occurred"
    );
    AppError::Conflict(message.to_string())
}

// =============================================================================
// テスト
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[derive(Validate)]
    struct TestStruct {
        #[validate(length(min = 3, max = 10))]
        name: String,
        #[validate(email)]
        email: String,
    }

    #[test]
    fn test_convert_validation_errors() {
        let test_data = TestStruct {
            name: "ab".to_string(),             // too short
            email: "invalid-email".to_string(), // invalid format
        };

        let validation_errors = test_data.validate().unwrap_err();
        let app_error = convert_validation_errors(validation_errors, "test");

        match app_error {
            AppError::ValidationErrors(errors) => {
                assert!(!errors.is_empty());
                assert!(errors.iter().any(|e| e.contains("name")));
                assert!(errors.iter().any(|e| e.contains("email")));
            }
            _ => panic!("Expected ValidationErrors"),
        }
    }

    #[test]
    fn test_not_found_error() {
        let error = not_found_error("User", "123", "user service");
        match error {
            AppError::NotFound(message) => {
                assert_eq!(message, "User with identifier 123 not found");
            }
            _ => panic!("Expected NotFound error"),
        }
    }
}
