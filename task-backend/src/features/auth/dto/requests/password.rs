use crate::utils::validation::common;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// パスワードリセット要求リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PasswordResetRequestRequest {
    #[validate(
        email(message = "Invalid email format"),
        custom(function = common::validate_email_format)
    )]
    pub email: String,
}

/// パスワードリセット実行リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PasswordResetRequest {
    #[validate(length(min = common::required::MIN_LENGTH, message = "Reset token is required"))]
    pub token: String,

    #[validate(
        length(min = common::password::MIN_LENGTH, message = "New password must be at least 8 characters"),
        custom(function = common::validate_password_strength)
    )]
    pub new_password: String,
}

/// パスワード変更リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PasswordChangeRequest {
    #[validate(length(min = common::required::MIN_LENGTH, message = "Current password is required"))]
    pub current_password: String,

    #[validate(
        length(min = common::password::MIN_LENGTH, message = "New password must be at least 8 characters"),
        custom(function = common::validate_password_strength)
    )]
    pub new_password: String,

    #[validate(must_match(other = "new_password", message = "Password confirmation must match"))]
    pub new_password_confirmation: String,
}

impl PasswordChangeRequest {
    pub fn validate_password_change(&self) -> Result<(), &'static str> {
        if self.new_password != self.new_password_confirmation {
            return Err("Password confirmation does not match");
        }
        Ok(())
    }
}
