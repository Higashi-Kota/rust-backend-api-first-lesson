// task-backend/src/features/user/dto/responses/mod.rs

use crate::features::user::models::user::SafeUser;
use crate::features::user::services::user_service::UserStats;
use crate::shared::types::common::{ApiResponse, OperationResult};
use crate::shared::types::pagination::PaginatedResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

impl From<crate::features::user::models::user_settings::Model> for UserSettingsDto {
    fn from(settings: crate::features::user::models::user_settings::Model) -> Self {
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

#[cfg(test)]
mod tests {
    use crate::shared::types::pagination::PaginationMeta;

    #[test]
    fn test_pagination_meta() {
        let pagination = PaginationMeta::new(2, 10, 25);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 10);
        assert_eq!(pagination.total_pages, 3);
        assert!(pagination.has_next);
        assert!(pagination.has_prev);
    }
}
