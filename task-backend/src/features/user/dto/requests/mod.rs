// task-backend/src/features/user/dto/requests/mod.rs

use crate::utils::validation::common;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// ユーザー名更新リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateUsernameRequest {
    #[validate(
        length(
            min = common::username::MIN_LENGTH,
            max = common::username::MAX_LENGTH,
            message = "Username must be between 3 and 30 characters"
        ),
        custom(function = common::validate_username)
    )]
    pub username: String,
}

/// メールアドレス更新リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateEmailRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

/// プロフィール更新リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(
        length(
            min = common::username::MIN_LENGTH,
            max = common::username::MAX_LENGTH,
            message = "Username must be between 3 and 30 characters"
        ),
        custom(function = common::validate_username)
    )]
    pub username: Option<String>,

    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
}

/// アカウント状態更新リクエスト（管理者用）
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateAccountStatusRequest {
    pub is_active: bool,
}

/// メール認証確認リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct VerifyEmailRequest {
    #[validate(length(min = common::required::MIN_LENGTH, message = "Verification token is required"))]
    pub token: String,
}

/// メール認証再送信リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ResendVerificationEmailRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

/// ユーザー検索クエリ
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UserSearchQuery {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Search term must be between 1 and 100 characters"
    ))]
    pub q: Option<String>,

    pub is_active: Option<bool>,
    pub email_verified: Option<bool>,

    #[validate(range(min = 1, max = 100, message = "Page must be between 1 and 100"))]
    pub page: Option<i32>,

    #[validate(range(min = 1, max = 100, message = "Per page must be between 1 and 100"))]
    pub per_page: Option<i32>,

    pub sort_by: Option<UserSortField>,
    pub sort_order: Option<SortOrder>,
}

/// ユーザーソートフィールド
#[derive(Debug, Clone, Deserialize)]
pub enum UserSortField {
    #[serde(rename = "username")]
    Username,
    #[serde(rename = "email")]
    Email,
    #[serde(rename = "created_at")]
    CreatedAt,
    #[serde(rename = "last_login_at")]
    LastLoginAt,
    #[serde(rename = "task_count")]
    TaskCount,
}

/// ソート順序
#[derive(Debug, Clone, Deserialize)]
pub enum SortOrder {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

/// サブスクリプション分析クエリ
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SubscriptionQuery {
    pub tier: Option<String>,
}

/// 一括ユーザー操作リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct BulkUserOperationsRequest {
    #[validate(length(min = 1, message = "At least one user ID is required"))]
    pub user_ids: Vec<Uuid>,

    pub operation: BulkUserOperation,
    pub parameters: Option<serde_json::Value>,
    pub notify_users: Option<bool>,
}

/// 一括操作の種類
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BulkUserOperation {
    // 基本操作
    Activate,
    Deactivate,
    UpdateRole,
    UpdateSubscription,
    // 高度な操作
    SendNotification,
    ResetPasswords,
    ExportUserData,
    BulkDelete,
    BulkInvite,
}

impl std::fmt::Display for BulkUserOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BulkUserOperation::Activate => "activate",
            BulkUserOperation::Deactivate => "deactivate",
            BulkUserOperation::UpdateRole => "update_role",
            BulkUserOperation::UpdateSubscription => "update_subscription",
            BulkUserOperation::SendNotification => "send_notification",
            BulkUserOperation::ResetPasswords => "reset_passwords",
            BulkUserOperation::ExportUserData => "export_user_data",
            BulkUserOperation::BulkDelete => "bulk_delete",
            BulkUserOperation::BulkInvite => "bulk_invite",
        };
        write!(f, "{}", s)
    }
}

/// ユーザー設定更新リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUserSettingsRequest {
    pub language: Option<String>,
    pub timezone: Option<String>,
    pub notifications_enabled: Option<bool>,
    pub email_notifications: Option<serde_json::Value>,
    pub ui_preferences: Option<serde_json::Value>,
}

// --- バリデーション ---

impl UpdateProfileRequest {
    /// プロフィール更新のカスタムバリデーション
    pub fn validate_update(&self) -> Result<(), String> {
        if self.username.is_none() && self.email.is_none() {
            return Err("At least one field must be provided for update".to_string());
        }
        Ok(())
    }

    /// 更新された項目のリストを取得
    pub fn get_updated_fields(&self) -> Vec<String> {
        let mut fields = Vec::new();
        if self.username.is_some() {
            fields.push("username".to_string());
        }
        if self.email.is_some() {
            fields.push("email".to_string());
        }
        fields
    }
}

impl UserSearchQuery {
    /// デフォルト値を適用
    pub fn with_defaults(self) -> Self {
        Self {
            q: self.q,
            is_active: self.is_active,
            email_verified: self.email_verified,
            page: Some(self.page.unwrap_or(1)),
            per_page: Some(self.per_page.unwrap_or(20)),
            sort_by: self.sort_by.or(Some(UserSortField::CreatedAt)),
            sort_order: self.sort_order.or(Some(SortOrder::Descending)),
        }
    }
}

// --- テスト用ヘルパー ---

#[cfg(test)]
pub mod test_helpers {
    use super::*;

    pub fn create_valid_update_username_request() -> UpdateUsernameRequest {
        UpdateUsernameRequest {
            username: "newusername".to_string(),
        }
    }

    pub fn create_valid_update_email_request() -> UpdateEmailRequest {
        UpdateEmailRequest {
            email: "newemail@example.com".to_string(),
        }
    }

    pub fn create_valid_update_profile_request() -> UpdateProfileRequest {
        UpdateProfileRequest {
            username: Some("newusername".to_string()),
            email: Some("newemail@example.com".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_update_username_request_validation() {
        let mut request = test_helpers::create_valid_update_username_request();
        assert!(request.validate().is_ok());

        // 短すぎるユーザー名
        request.username = "ab".to_string();
        assert!(request.validate().is_err());

        // 長すぎるユーザー名
        request.username = "a".repeat(31);
        assert!(request.validate().is_err());

        // 無効な文字
        request.username = "invalid-username!".to_string();
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_update_email_request_validation() {
        let mut request = test_helpers::create_valid_update_email_request();
        assert!(request.validate().is_ok());

        // 無効なメールアドレス
        request.email = "invalid-email".to_string();
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_update_profile_request_validation() {
        let mut request = test_helpers::create_valid_update_profile_request();
        assert!(request.validate().is_ok());
        assert!(request.validate_update().is_ok());

        // 両方ともNone
        request.username = None;
        request.email = None;
        assert!(request.validate_update().is_err());

        // 更新フィールドのテスト
        request.username = Some("testuser".to_string());
        request.email = None;
        let fields = request.get_updated_fields();
        assert_eq!(fields, vec!["username"]);
    }

    #[test]
    fn test_user_search_query_defaults() {
        let query = UserSearchQuery {
            q: None,
            is_active: None,
            email_verified: None,
            page: None,
            per_page: None,
            sort_by: None,
            sort_order: None,
        };

        let query_with_defaults = query.with_defaults();
        assert_eq!(query_with_defaults.page, Some(1));
        assert_eq!(query_with_defaults.per_page, Some(20));
        assert!(matches!(
            query_with_defaults.sort_by,
            Some(UserSortField::CreatedAt)
        ));
        assert!(matches!(
            query_with_defaults.sort_order,
            Some(SortOrder::Descending)
        ));
    }
}
