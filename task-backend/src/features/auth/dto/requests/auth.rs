use crate::utils::validation::common;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// ユーザー登録リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(
        email(message = "Invalid email format"),
        custom(function = common::validate_email_format)
    )]
    pub email: String,

    #[validate(
        length(
            min = common::username::MIN_LENGTH,
            max = common::username::MAX_LENGTH,
            message = "Username must be between 3 and 30 characters"
        ),
        custom(function = common::validate_username)
    )]
    pub username: String,

    #[validate(
        length(min = common::password::MIN_LENGTH, message = "Password must be at least 8 characters"),
        custom(function = common::validate_password_strength)
    )]
    pub password: String,
}

/// ログインリクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SigninRequest {
    #[validate(length(min = common::required::MIN_LENGTH, message = "Email or username is required"))]
    pub identifier: String, // email or username

    #[validate(length(min = common::required::MIN_LENGTH, message = "Password is required"))]
    pub password: String,
}
