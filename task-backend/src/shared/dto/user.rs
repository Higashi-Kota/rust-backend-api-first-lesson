// task-backend/src/shared/dto/user.rs

use crate::domain::user_model::SafeUser;
use crate::features::user::services::user_service::UserStats;
use crate::shared::types::common::{ApiResponse, OperationResult};
use crate::shared::types::pagination::PaginatedResponse;
use crate::utils::validation::common;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// --- リクエストDTO ---

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

// --- レスポンスDTO ---

/// ユーザープロフィールレスポンス
#[derive(Debug, Clone, Serialize)]
pub struct UserProfileResponse {
    pub user: SafeUser,
}

/// ユーザー統計レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct UserStatsResponse {
    pub stats: UserStats,
    pub additional_info: UserAdditionalInfo,
}

/// ユーザー追加情報
#[derive(Debug, Clone, Serialize)]
pub struct UserAdditionalInfo {
    pub days_since_registration: i64,
    pub last_activity: Option<DateTime<Utc>>,
    pub account_status: AccountStatus,
}

/// アカウント状態
#[derive(Debug, Clone, Serialize)]
pub struct AccountStatus {
    pub is_active: bool,
    pub email_verified: bool,
    pub verification_required: bool,
    pub restrictions: Vec<AccountRestriction>,
}

/// アカウント制限
#[derive(Debug, Clone, Serialize)]
pub struct AccountRestriction {
    pub restriction_type: RestrictionType,
    pub reason: String,
    pub expires_at: Option<DateTime<Utc>>,
}

/// 制限タイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestrictionType {
    EmailVerificationRequired,
    TemporarySuspension,
    PasswordResetRequired,
    TwoFactorAuthRequired,
}

/// ユーザー設定レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsResponse {
    pub user_id: Uuid,
    pub preferences: UserPreferences,
    pub security: SecuritySettings,
    pub notifications: NotificationSettings,
}

/// ユーザー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub language: String,
    pub timezone: String,
    pub theme: String,
    pub date_format: String,
    pub time_format: String,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            timezone: "UTC".to_string(),
            theme: "light".to_string(),
            date_format: "YYYY-MM-DD".to_string(),
            time_format: "24h".to_string(),
        }
    }
}

/// セキュリティ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub two_factor_enabled: bool,
    pub login_notifications: bool,
    pub session_timeout_minutes: i32,
    pub allowed_ip_ranges: Vec<String>,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            two_factor_enabled: false,
            login_notifications: true,
            session_timeout_minutes: 30,
            allowed_ip_ranges: vec![],
        }
    }
}

/// 通知設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_notifications: bool,
    pub security_alerts: bool,
    pub task_reminders: bool,
    pub newsletter: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            email_notifications: true,
            security_alerts: true,
            task_reminders: true,
            newsletter: false,
        }
    }
}

/// ユーザー一覧レスポンス（管理者用）
pub type UserListResponse = PaginatedResponse<UserSummary>;

/// ユーザー概要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub task_count: i64,
}

// PaginationInfo は common.rs の PaginationMeta に統一

/// プロフィール更新レスポンス
pub type ProfileUpdateResponse = ApiResponse<OperationResult<SafeUser>>;

/// メール認証レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct EmailVerificationResponse {
    pub message: String,
    pub verified: bool,
    pub user: Option<SafeUser>,
}

/// アカウント状態更新レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct AccountStatusUpdateResponse {
    pub user: SafeUser,
    pub message: String,
    pub previous_status: bool,
    pub new_status: bool,
}

// --- クエリパラメータ ---

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

/// ユーザー権限チェックレスポンス
#[derive(Debug, Serialize)]
pub struct UserPermissionsResponse {
    pub user_id: Uuid,
    pub is_member: bool,
    pub is_admin: bool,
    pub is_active: bool,
    pub email_verified: bool,
    pub subscription_tier: String,
    pub can_create_teams: bool,
    pub can_access_analytics: bool,
}

/// メール認証履歴レスポンス
#[derive(Debug, Serialize)]
pub struct EmailVerificationHistoryResponse {
    pub user_id: Uuid,
    pub verification_history: Vec<EmailVerificationHistoryItem>,
    pub total_verifications: u32,
    pub last_verification: Option<DateTime<Utc>>,
}

/// メール認証履歴アイテム
#[derive(Debug, Serialize)]
pub struct EmailVerificationHistoryItem {
    pub token_id: Uuid,
    pub verified_at: DateTime<Utc>,
    pub days_since_verification: i64,
    pub verification_status: String,
}

/// メール認証保留状態レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct PendingEmailVerificationResponse {
    pub user_id: Uuid,
    pub has_pending_verification: bool,
    pub latest_token_sent_at: Option<DateTime<Utc>>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub attempts_count: u32,
}

/// トークン状態レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenStatusResponse {
    pub exists: bool,
    pub is_valid: bool,
    pub is_used: bool,
    pub is_expired: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub used_at: Option<DateTime<Utc>>,
}

// --- ヘルパー関数 ---

impl UserAdditionalInfo {
    pub fn from_user_stats(stats: &UserStats) -> Self {
        let days_since_registration = Utc::now()
            .signed_duration_since(stats.created_at)
            .num_days();

        let account_status = AccountStatus {
            is_active: stats.is_active,
            email_verified: stats.email_verified,
            verification_required: !stats.email_verified,
            restrictions: Self::get_restrictions(stats),
        };

        Self {
            days_since_registration,
            last_activity: stats.last_login_at,
            account_status,
        }
    }

    fn get_restrictions(stats: &UserStats) -> Vec<AccountRestriction> {
        let mut restrictions = Vec::new();

        if !stats.email_verified {
            restrictions.push(AccountRestriction {
                restriction_type: RestrictionType::EmailVerificationRequired,
                reason: "Email address has not been verified".to_string(),
                expires_at: None,
            });
        }

        if !stats.is_active {
            restrictions.push(AccountRestriction {
                restriction_type: RestrictionType::TemporarySuspension,
                reason: "Account has been deactivated".to_string(),
                expires_at: None,
            });
        }

        restrictions
    }
}

// PaginationInfo の実装は common.rs の PaginationMeta に移行

// --- バリデーション用の正規表現 ---

// --- 新規API用のDTO ---

/// サブスクリプション分析クエリ
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SubscriptionQuery {
    pub tier: Option<String>,
}

/// ユーザー分析レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct UserAnalyticsResponse {
    pub stats: UserStats,
    pub role_stats: Vec<RoleUserStats>,
    pub message: String,
}

/// ロール情報付きユーザーレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWithRoleResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub subscription_tier: String,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub role: crate::features::security::dto::legacy::role_dto::RoleResponse,
}

/// ロール別ユーザー統計
#[derive(Debug, Clone, Serialize)]
pub struct RoleUserStats {
    pub role_name: String,
    pub role_display_name: String,
    pub total_users: u64,
    pub active_users: u64,
    pub verified_users: u64,
}

/// サブスクリプション分析レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionAnalyticsResponse {
    pub tier: String,
    pub analytics: SubscriptionAnalytics,
    pub message: String,
}

/// サブスクリプション分析データ
#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionAnalytics {
    pub total_users: u64,
    pub free_users: u64,
    pub pro_users: u64,
    pub enterprise_users: u64,
    pub conversion_rate: f64,
}

/// ユーザーアクティビティ統計レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct UserActivityStatsResponse {
    pub stats: UserActivityStats,
    pub message: String,
}

/// ユーザーアクティビティ統計
#[derive(Debug, Clone, Serialize)]
pub struct UserActivityStats {
    pub total_logins_today: u64,
    pub total_logins_week: u64,
    pub total_logins_month: u64,
    pub active_users_today: u64,
    pub active_users_week: u64,
    pub active_users_month: u64,
    pub average_session_duration: f64,
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

/// 一括操作レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct BulkOperationResponse {
    pub operation_id: String,
    pub operation: String,
    pub total_users: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub errors: Vec<String>,
    pub message: String,
    pub results: Option<serde_json::Value>,
    pub execution_time_ms: u64,
    pub executed_at: String,
}

/// 一括操作結果
#[derive(Debug, Clone)]
pub struct BulkOperationResult {
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<String>,
    pub results: Option<serde_json::Value>,
}

// --- ユーザー設定関連DTO ---

/// ユーザー設定レスポンス（管理者API用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsDto {
    pub user_id: Uuid,
    pub language: String,
    pub timezone: String,
    pub notifications_enabled: bool,
    pub email_notifications: serde_json::Value,
    pub ui_preferences: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::domain::user_settings_model::Model> for UserSettingsDto {
    fn from(settings: crate::domain::user_settings_model::Model) -> Self {
        let email_notifications = serde_json::to_value(settings.get_email_notifications())
            .unwrap_or(serde_json::json!({}));
        let ui_preferences =
            serde_json::to_value(settings.get_ui_preferences()).unwrap_or(serde_json::json!({}));

        UserSettingsDto {
            user_id: settings.user_id,
            language: settings.language,
            timezone: settings.timezone,
            notifications_enabled: settings.notifications_enabled,
            email_notifications,
            ui_preferences,
            created_at: settings.created_at,
            updated_at: settings.updated_at,
        }
    }
}

impl From<UserSettingsResponse> for UserSettingsDto {
    fn from(settings: UserSettingsResponse) -> Self {
        UserSettingsDto {
            user_id: settings.user_id,
            language: settings.preferences.language,
            timezone: settings.preferences.timezone,
            notifications_enabled: settings.notifications.email_notifications,
            email_notifications: serde_json::json!({
                "security_alerts": settings.notifications.security_alerts,
                "task_reminders": settings.notifications.task_reminders,
                "newsletter": settings.notifications.newsletter,
            }),
            ui_preferences: serde_json::json!({
                "theme": settings.preferences.theme,
                "date_format": settings.preferences.date_format,
                "time_format": settings.preferences.time_format,
            }),
            created_at: chrono::Utc::now(), // これらは実際にはDBから取得すべき
            updated_at: chrono::Utc::now(),
        }
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

/// 言語別ユーザー統計
#[derive(Debug, Clone, Serialize)]
pub struct UsersByLanguageResponse {
    pub language: String,
    pub user_count: usize,
    pub user_ids: Vec<Uuid>,
}

/// 通知有効ユーザーレスポンス
#[derive(Debug, Clone, Serialize)]
pub struct UsersWithNotificationResponse {
    pub notification_type: String,
    pub user_count: usize,
    pub user_ids: Vec<Uuid>,
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

    #[test]
    fn test_pagination_meta() {
        use crate::shared::types::pagination::PaginationMeta;
        let pagination = PaginationMeta::new(2, 10, 25);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 10);
        assert_eq!(pagination.total_pages, 3);
        assert!(pagination.has_next);
        assert!(pagination.has_prev);
    }
}
