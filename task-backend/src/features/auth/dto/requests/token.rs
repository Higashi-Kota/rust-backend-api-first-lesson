use crate::utils::validation::common;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// トークンリフレッシュリクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = common::required::MIN_LENGTH, message = "Refresh token is required"))]
    pub refresh_token: String,
}

/// トークン無効化リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RevokeTokenRequest {
    #[validate(length(min = common::required::MIN_LENGTH, message = "Refresh token is required"))]
    pub refresh_token: String,
}

/// メール認証実行リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EmailVerificationRequest {
    #[validate(length(min = common::required::MIN_LENGTH, message = "Verification token is required"))]
    pub token: String,
}

/// メール認証再送リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ResendVerificationEmailRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

/// アカウント削除リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct DeleteAccountRequest {
    #[validate(length(min = common::required::MIN_LENGTH, message = "Password is required for account deletion"))]
    pub password: String,

    #[validate(length(min = common::required::MIN_LENGTH, message = "Confirmation text is required"))]
    pub confirmation: String,
}

impl DeleteAccountRequest {
    pub fn validate_deletion(&self) -> Result<(), &'static str> {
        if self.confirmation != "DELETE MY ACCOUNT" {
            return Err("Confirmation text must be 'DELETE MY ACCOUNT'");
        }
        Ok(())
    }
}
