// task-backend/src/api/dto/auth_dto.rs

// 統一レスポンス構造体は必要に応じてインポート
use crate::domain::user_model::SafeUser;
use crate::utils::jwt::TokenPair;
use crate::utils::validation::common;
use serde::{Deserialize, Serialize};
use validator::Validate;

// --- リクエストDTO ---

/// ユーザー登録リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(email(message = "Invalid email format"))]
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

/// パスワードリセット要求リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PasswordResetRequestRequest {
    #[validate(email(message = "Invalid email format"))]
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

    #[validate(must_match(
        other = "new_password",
        message = "Password confirmation does not match"
    ))]
    pub new_password_confirmation: String,
}

/// トークンリフレッシュリクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = common::required::MIN_LENGTH, message = "Refresh token is required"))]
    pub refresh_token: String,
}

/// アカウント削除リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct DeleteAccountRequest {
    #[validate(length(min = common::required::MIN_LENGTH, message = "Password is required for account deletion"))]
    pub password: String,

    #[validate(length(min = common::required::MIN_LENGTH, message = "Confirmation text is required"))]
    pub confirmation: String,
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

/// 認証ステータスレスポンス
#[derive(Debug, Clone, Serialize)]
pub struct AuthStatusResponse {
    pub authenticated: bool,
    pub user: Option<SafeUser>,
    pub access_token_expires_in: Option<i64>, // 秒
}

// 統一レスポンス構造体を使用 (common.rs から import)

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

// Cookie設定とセキュリティヘッダーは crate::api::CookieConfig と crate::api::SecurityHeaders を使用

// --- バリデーション用の正規表現と定数 ---

// --- テスト用ヘルパー ---

#[cfg(test)]
pub mod test_helpers {
    use super::*;

    pub fn create_valid_signup_request() -> SignupRequest {
        SignupRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            password: "SecurePassword123".to_string(),
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
            current_password: "CurrentPassword123".to_string(),
            new_password: "NewPassword123".to_string(),
            new_password_confirmation: "NewPassword123".to_string(),
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
        request.new_password_confirmation = "NewPassword123".to_string();
        request.current_password = "NewPassword123".to_string();
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

    #[test]
    fn test_auth_response_serialization() {
        use crate::domain::user_model::SafeUser;
        use crate::utils::jwt::TokenPair;
        use uuid::Uuid;

        let safe_user = SafeUser {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            is_active: true,
            email_verified: false,
            role_id: Uuid::new_v4(),
            subscription_tier: "free".to_string(),
            last_login_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let token_pair = TokenPair::new(
            "test_access_token".to_string(),
            "test_refresh_token".to_string(),
            15, // 15分
            7,  // 7日
            "2024-01-01T12:15:00Z".to_string(),
            "2024-01-01T12:12:00Z".to_string(),
        );

        let auth_response = AuthResponse {
            user: safe_user,
            tokens: token_pair,
            message: "Test message".to_string(),
        };

        // シリアライゼーションが成功することを確認
        let serialized = serde_json::to_string(&auth_response);
        assert!(serialized.is_ok());

        let json_str = serialized.unwrap();
        // tokensオブジェクト内にタイムスタンプフィールドがあることを確認
        assert!(json_str.contains("tokens"));
        assert!(json_str.contains("access_token_expires_at"));
        assert!(json_str.contains("should_refresh_at"));
    }
}
