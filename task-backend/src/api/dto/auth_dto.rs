// task-backend/src/api/dto/auth_dto.rs
#![allow(dead_code)]

use crate::domain::user_model::SafeUser;
use crate::utils::jwt::TokenPair;
use serde::{Deserialize, Serialize};
use validator::Validate;

// --- リクエストDTO ---

/// ユーザー登録リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(
        min = 3,
        max = 30,
        message = "Username must be between 3 and 30 characters"
    ))]
    pub username: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

/// ログインリクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SigninRequest {
    #[validate(length(min = 1, message = "Email or username is required"))]
    pub identifier: String, // email or username

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

/// パスワードリセット要求リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PasswordResetRequestRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

/// パスワードリセット実行リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PasswordResetRequest {
    #[validate(length(min = 1, message = "Reset token is required"))]
    pub token: String,

    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
    pub new_password: String,
}

/// パスワード変更リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PasswordChangeRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,

    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
    pub new_password: String,

    #[validate(must_match(
        other = "new_password",
        message = "Password confirmation does not match"
    ))]
    pub new_password_confirmation: String,
}

/// トークンリフレッシュリクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

/// アカウント削除リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct DeleteAccountRequest {
    #[validate(length(min = 1, message = "Password is required for account deletion"))]
    pub password: String,

    #[validate(length(min = 1, message = "Confirmation text is required"))]
    pub confirmation: String,
}

// --- レスポンスDTO ---

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

/// トークンリフレッシュレスポンス
#[derive(Debug, Clone, Serialize)]
pub struct TokenRefreshResponse {
    pub user: SafeUser,
    pub tokens: TokenPair,
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_in: i64,  // 秒
    pub refresh_token_expires_in: i64, // 秒
    pub token_type: String,
}

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

/// アカウント削除レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct AccountDeletionResponse {
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
    pub authenticated: bool,
    pub user: Option<SafeUser>,
    pub access_token_expires_in: Option<i64>, // 秒
}

/// エラーレスポンス
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// 成功レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct SuccessResponse {
    pub message: String,
    pub data: Option<serde_json::Value>,
}

// --- バリデーション ---

/// カスタムバリデーション関数
impl PasswordChangeRequest {
    /// パスワード変更のカスタムバリデーション
    pub fn validate_password_change(&self) -> Result<(), String> {
        // 現在のパスワードと新しいパスワードが同じでないかチェック
        if self.current_password == self.new_password {
            return Err("New password must be different from current password".to_string());
        }

        // パスワード確認が一致するかチェック
        if self.new_password != self.new_password_confirmation {
            return Err("Password confirmation does not match".to_string());
        }

        Ok(())
    }
}

impl DeleteAccountRequest {
    /// アカウント削除のカスタムバリデーション
    pub fn validate_deletion(&self) -> Result<(), String> {
        if self.confirmation != "CONFIRM_DELETE" {
            return Err("Confirmation text must be 'CONFIRM_DELETE'".to_string());
        }
        Ok(())
    }
}

// --- 認証フロー支援 ---

/// 認証フローのステップ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthFlowStep {
    SignupPending,
    EmailVerificationRequired,
    SignupComplete,
    SigninComplete,
    PasswordResetRequested,
    PasswordResetComplete,
    AccountDeleted,
}

/// 認証フローレスポンス
#[derive(Debug, Clone, Serialize)]
pub struct AuthFlowResponse {
    pub step: AuthFlowStep,
    pub message: String,
    pub next_action: Option<String>,
    pub data: Option<serde_json::Value>,
}

// --- Cookie設定 ---

/// Cookie設定
#[derive(Debug, Clone)]
pub struct CookieConfig {
    pub access_token_name: String,
    pub refresh_token_name: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: String,
    pub domain: Option<String>,
    pub path: String,
}

impl Default for CookieConfig {
    fn default() -> Self {
        Self {
            access_token_name: "access_token".to_string(),
            refresh_token_name: "refresh_token".to_string(),
            secure: true,                    // HTTPS必須
            http_only: true,                 // XSS防止
            same_site: "Strict".to_string(), // CSRF防止
            domain: None,
            path: "/".to_string(),
        }
    }
}

// --- セキュリティヘッダー ---

/// セキュリティヘッダー設定
#[derive(Debug, Clone)]
pub struct SecurityHeaders {
    pub content_security_policy: String,
    pub x_frame_options: String,
    pub x_content_type_options: String,
    pub referrer_policy: String,
    pub permissions_policy: String,
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self {
            content_security_policy: "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none';".to_string(),
            x_frame_options: "DENY".to_string(),
            x_content_type_options: "nosniff".to_string(),
            referrer_policy: "strict-origin-when-cross-origin".to_string(),
            permissions_policy: "camera=(), microphone=(), geolocation=()".to_string(),
        }
    }
}

// --- バリデーション用の正規表現と定数 ---

// --- テスト用ヘルパー ---

#[cfg(test)]
pub mod test_helpers {
    use super::*;

    pub fn create_valid_signup_request() -> SignupRequest {
        SignupRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            password: "securepassword123".to_string(),
        }
    }

    pub fn create_valid_signin_request() -> SigninRequest {
        SigninRequest {
            identifier: "testuser".to_string(),
            password: "securepassword123".to_string(),
        }
    }

    pub fn create_valid_password_change_request() -> PasswordChangeRequest {
        PasswordChangeRequest {
            current_password: "currentpassword123".to_string(),
            new_password: "newpassword123".to_string(),
            new_password_confirmation: "newpassword123".to_string(),
        }
    }

    pub fn create_valid_delete_account_request() -> DeleteAccountRequest {
        DeleteAccountRequest {
            password: "password123".to_string(),
            confirmation: "CONFIRM_DELETE".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_signup_request_validation() {
        let mut request = test_helpers::create_valid_signup_request();
        assert!(request.validate().is_ok());

        // 無効なメールアドレス
        request.email = "invalid-email".to_string();
        assert!(request.validate().is_err());

        // 短すぎるユーザー名
        request.email = "test@example.com".to_string();
        request.username = "ab".to_string();
        assert!(request.validate().is_err());

        // 短すぎるパスワード
        request.username = "testuser".to_string();
        request.password = "short".to_string();
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_signin_request_validation() {
        let mut request = test_helpers::create_valid_signin_request();
        assert!(request.validate().is_ok());

        // 空の識別子
        request.identifier = "".to_string();
        assert!(request.validate().is_err());

        // 空のパスワード
        request.identifier = "testuser".to_string();
        request.password = "".to_string();
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_password_change_validation() {
        let mut request = test_helpers::create_valid_password_change_request();
        assert!(request.validate().is_ok());
        assert!(request.validate_password_change().is_ok());

        // パスワード確認が一致しない
        request.new_password_confirmation = "different".to_string();
        assert!(request.validate_password_change().is_err());

        // 現在のパスワードと新しいパスワードが同じ
        request.new_password_confirmation = "newpassword123".to_string();
        request.current_password = "newpassword123".to_string();
        assert!(request.validate_password_change().is_err());
    }

    #[test]
    fn test_delete_account_validation() {
        let mut request = test_helpers::create_valid_delete_account_request();
        assert!(request.validate().is_ok());
        assert!(request.validate_deletion().is_ok());

        // 間違った確認テキスト
        request.confirmation = "WRONG_CONFIRMATION".to_string();
        assert!(request.validate_deletion().is_err());
    }
}
