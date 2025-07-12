use crate::features::user::models::user::SafeUser;
use crate::infrastructure::jwt::TokenPair;
use serde::Serialize;

/// 認証レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub user: SafeUser,
    pub tokens: TokenPair,
    pub message: String,
}

/// ログアウトレスポンス
#[derive(Debug, Clone, Serialize)]
pub struct LogoutResponse {
    pub message: String,
}

/// 現在のユーザー情報レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct CurrentUserResponse {
    pub user: SafeUser,
}

/// 認証ステータスレスポンス
#[derive(Debug, Clone, Serialize)]
pub struct AuthStatusResponse {
    pub is_authenticated: bool,
    pub user: Option<SafeUser>,
}
