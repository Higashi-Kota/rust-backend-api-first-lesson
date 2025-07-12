use serde::Serialize;

/// パスワードリセット要求レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct PasswordResetRequestResponse {
    pub message: String,
}

/// パスワードリセットレスポンス
#[derive(Debug, Clone, Serialize)]
pub struct PasswordResetResponse {
    pub message: String,
}

/// パスワード変更レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct PasswordChangeResponse {
    pub message: String,
}
