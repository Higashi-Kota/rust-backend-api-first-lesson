// task-backend/src/features/security/dto/responses/security.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 日付範囲（リクエストと共通）
use crate::features::security::dto::requests::security::DateRange;

/// トークン統計レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct TokenStatsResponse {
    pub refresh_tokens: RefreshTokenStats,
    pub password_reset_tokens: PasswordResetTokenStats,
    pub message: String,
}

/// リフレッシュトークン統計
#[derive(Debug, Clone, Serialize)]
pub struct RefreshTokenStats {
    pub total_active: u64,
    pub total_expired: u64,
    pub users_with_tokens: u64,
    pub average_tokens_per_user: f64,
    pub oldest_token_age_days: i64,
    pub newest_token_age_hours: i64,
}

/// パスワードリセットトークン統計
#[derive(Debug, Clone, Serialize)]
pub struct PasswordResetTokenStats {
    pub total_active: u64,
    pub total_used: u64,
    pub total_expired: u64,
    pub requests_today: u64,
    pub requests_this_week: u64,
    pub average_usage_time_minutes: f64,
}

/// リフレッシュトークン監視レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct RefreshTokenMonitorResponse {
    pub active_tokens: Vec<ActiveTokenSummary>,
    pub message: String,
}

/// アクティブトークン概要
#[derive(Debug, Clone, Serialize)]
pub struct ActiveTokenSummary {
    pub user_id: Uuid,
    pub username: String,
    pub token_count: u64,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// トークンクリーンアップレスポンス
#[derive(Debug, Clone, Serialize)]
pub struct CleanupTokensResponse {
    pub result: CleanupResult,
    pub message: String,
}

/// クリーンアップ結果
#[derive(Debug, Clone, Serialize)]
pub struct CleanupResult {
    pub deleted_count: u64,
    pub cleanup_type: String,
}

/// パスワードリセット監視レスポンス
#[derive(Debug, Clone, Serialize)]
pub struct PasswordResetMonitorResponse {
    pub recent_activity: Vec<PasswordResetActivity>,
    pub message: String,
}

/// パスワードリセット活動
#[derive(Debug, Clone, Serialize)]
pub struct PasswordResetActivity {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub requested_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub status: String, // "pending", "used", "expired"
}

/// 全トークン無効化レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeAllTokensResponse {
    pub result: RevokeResult,
    pub message: String,
}

/// 無効化結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeResult {
    pub revoked_count: u64,
    pub affected_users: u64,
    pub revocation_reason: String,
    pub revoked_at: DateTime<Utc>,
}

/// セッション分析レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalyticsResponse {
    pub analytics: SessionAnalytics,
    pub message: String,
}

/// セッション分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalytics {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub unique_users_today: u64,
    pub unique_users_this_week: u64,
    pub average_session_duration_minutes: f64,
    pub peak_concurrent_sessions: u64,
    pub suspicious_activity_count: u64,
    pub geographic_distribution: Vec<GeographicSession>,
    pub device_distribution: Vec<DeviceSession>,
}

/// 地理的セッション分布
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicSession {
    pub country: String,
    pub session_count: u64,
    pub unique_users: u64,
}

/// デバイス別セッション分布
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSession {
    pub device_type: String, // "desktop", "mobile", "tablet", "unknown"
    pub session_count: u64,
    pub unique_users: u64,
}

/// 監査レポートレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReportResponse {
    pub report: AuditReport,
    pub message: String,
}

/// 監査レポート
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub report_id: Uuid,
    pub report_type: String,
    pub generated_at: DateTime<Utc>,
    pub generated_by: Uuid,
    pub date_range: Option<DateRange>,
    pub summary: AuditSummary,
    pub findings: Vec<AuditFinding>,
    pub recommendations: Vec<String>,
}

/// 監査概要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub total_events: u64,
    pub security_incidents: u64,
    pub failed_logins: u64,
    pub token_irregularities: u64,
    pub suspicious_activities: u64,
    pub risk_level: String, // "low", "medium", "high", "critical"
}

/// 監査発見事項
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    pub finding_id: Uuid,
    pub category: String, // "authentication", "authorization", "token_management", "session"
    pub severity: String, // "info", "warning", "error", "critical"
    pub description: String,
    pub affected_users: Vec<Uuid>,
    pub first_occurrence: DateTime<Utc>,
    pub last_occurrence: DateTime<Utc>,
    pub count: u64,
    pub details: Option<serde_json::Value>,
}
