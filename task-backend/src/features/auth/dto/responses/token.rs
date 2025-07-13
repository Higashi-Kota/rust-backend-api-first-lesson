use crate::features::user::models::user::SafeUser;
use crate::infrastructure::jwt::TokenPair;
use serde::Serialize;

/// トークンリフレッシュレスポンス
#[derive(Debug, Clone, Serialize)]
pub struct TokenRefreshResponse {
    pub user: SafeUser,
    pub tokens: TokenPair,
}

/// アカウント削除レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct AccountDeletionResponse {
    pub message: String,
}

/// メール認証レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct EmailVerificationResponse {
    pub message: String,
    pub email_verified: bool,
}

/// メール認証再送レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct ResendVerificationEmailResponse {
    pub message: String,
}
