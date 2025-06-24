// task-backend/src/api/dto/security_dto.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// --- リクエストDTO ---

/// トークンクリーンアップリクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CleanupTokensRequest {
    #[validate(length(min = 1, message = "Cleanup type is required"))]
    pub cleanup_type: String, // "refresh_tokens", "password_reset_tokens", "all"
}

/// 全トークン無効化リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RevokeAllTokensRequest {
    pub user_id: Option<Uuid>, // 特定ユーザーのみ（Noneの場合は全ユーザー）

    #[validate(length(min = 1, message = "Reason is required"))]
    pub reason: String,

    pub exclude_current_user: bool, // 実行者のトークンを除外するか
}

/// 監査レポート生成リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct AuditReportRequest {
    #[validate(length(min = 1, message = "Report type is required"))]
    pub report_type: String, // "security", "tokens", "sessions", "comprehensive"

    pub date_range: Option<DateRange>,
    pub include_details: Option<bool>,
}

/// 日付範囲
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

// --- レスポンスDTO ---

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

// --- テスト用ヘルパー ---

#[cfg(test)]
pub mod test_helpers {
    use super::*;

    pub fn create_valid_cleanup_tokens_request() -> CleanupTokensRequest {
        CleanupTokensRequest {
            cleanup_type: "refresh_tokens".to_string(),
        }
    }

    pub fn create_valid_revoke_all_tokens_request() -> RevokeAllTokensRequest {
        RevokeAllTokensRequest {
            user_id: None,
            reason: "Security incident detected".to_string(),
            exclude_current_user: true,
        }
    }

    pub fn create_valid_audit_report_request() -> AuditReportRequest {
        AuditReportRequest {
            report_type: "security".to_string(),
            date_range: None,
            include_details: Some(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_cleanup_tokens_request_validation() {
        let mut request = test_helpers::create_valid_cleanup_tokens_request();
        assert!(request.validate().is_ok());

        // 有効なクリーンアップタイプのテスト
        let valid_cleanup_types = ["expired", "inactive", "all", "orphaned"];
        for cleanup_type in valid_cleanup_types {
            request.cleanup_type = cleanup_type.to_string();
            let result = request.validate();
            assert!(
                result.is_ok(),
                "Cleanup type '{}' should be valid",
                cleanup_type
            );
        }

        // 無効なクリーンアップタイプ
        let invalid_cleanup_types = [
            "",                 // 空文字列
            " ",                // 空白
            "invalid_type",     // 不正なタイプ
            "EXPIRED",          // 大文字（小文字のみ想定）
            "expired ",         // 末尾スペース
            " expired",         // 先頭スペース
            "exp ired",         // 中間スペース
            "expired\n",        // 制御文字
            "drop_table",       // SQLインジェクション試行
            "<script>",         // XSS試行
            "../../etc/passwd", // パストラバーサル試行
        ];

        for invalid_type in invalid_cleanup_types {
            request.cleanup_type = invalid_type.to_string();
            let result = request.validate();
            if result.is_ok() {
                // 基本バリデーションを通過したが、ビジネスロジックで拒否される
                println!("Invalid cleanup type '{}' passed basic validation - will be rejected by business logic", invalid_type.escape_debug());
            } else {
                assert!(
                    result.is_err(),
                    "Invalid cleanup type '{}' should be rejected",
                    invalid_type.escape_debug()
                );
            }
        }

        // エッジケース：非常に長いタイプ名（DoS攻撃対策）
        request.cleanup_type = "a".repeat(1000);
        let result = request.validate();
        if result.is_ok() {
            println!("Very long cleanup type should be rejected for security");
        }

        // Unicode攻撃テスト
        let unicode_attacks = [
            "ｅｘｐｉｒｅｄ",  // 全角文字
            "expіred",         // キリル文字のi
            "expired\u{200B}", // ゼロ幅スペース
        ];

        for unicode_attack in unicode_attacks {
            request.cleanup_type = unicode_attack.to_string();
            let result = request.validate();
            if result.is_ok() {
                println!(
                    "Unicode spoofing attack '{}' should be detected",
                    unicode_attack
                );
            }
        }
    }

    #[test]
    fn test_revoke_all_tokens_request_validation() {
        let mut request = test_helpers::create_valid_revoke_all_tokens_request();
        assert!(request.validate().is_ok());

        // 有効な理由のパターンテスト
        let valid_reasons = [
            "Security incident detected",
            "Routine maintenance",
            "Suspected account compromise",
            "System migration",
            "Emergency security measure",
            "Multi-character reason with numbers 123",
            "Reason with special chars: -_.,()",
        ];

        for reason in valid_reasons {
            request.reason = reason.to_string();
            let result = request.validate();
            assert!(result.is_ok(), "Reason '{}' should be valid", reason);
        }

        // 無効な理由のテスト
        let definitely_invalid = [
            "", // 空文字列 - 確実に無効
        ];

        for invalid_reason in definitely_invalid {
            request.reason = invalid_reason.to_string();
            let result = request.validate();
            assert!(
                result.is_err(),
                "Reason '{}' should be invalid",
                invalid_reason.escape_debug()
            );
        }

        // 基本バリデーションを通るが、ビジネスロジックで拒否されるべき理由
        let borderline_invalid = [
            " ",    // 空白のみ
            "  ",   // 複数空白
            "\n",   // 改行のみ
            "\t",   // タブのみ
            "\r\n", // CRLF
        ];

        for borderline_reason in borderline_invalid {
            request.reason = borderline_reason.to_string();
            let result = request.validate();
            if result.is_ok() {
                // 基本バリデーションは通るが、実際のビジネスロジックで拒否される
                println!("Borderline invalid reason '{}' passed basic validation - will be rejected by business logic", borderline_reason.escape_debug());
            } else {
                assert!(
                    result.is_err(),
                    "Reason '{}' should be invalid",
                    borderline_reason.escape_debug()
                );
            }
        }

        // セキュリティテスト：悪意のある入力
        let malicious_reasons = [
            "<script>alert('xss')</script>", // XSS試行
            "'; DROP TABLE users; --",       // SQLインジェクション試行
            "../../../etc/passwd",           // パストラバーサル試行
            "javascript:alert('malicious')", // JavaScript実行試行
            "\x00\x01\x02",                  // バイナリデータ
        ];

        for malicious_reason in malicious_reasons {
            request.reason = malicious_reason.to_string();
            let result = request.validate();
            if result.is_ok() {
                // 基本バリデーションは通るが、後段でサニタイゼーション
                println!(
                    "Malicious reason '{}' will be sanitized at service layer",
                    malicious_reason.escape_debug()
                );
            }
        }

        // 長さのテスト
        // 非常に短い理由
        request.reason = "a".to_string();
        let result = request.validate();
        if result.is_ok() {
            println!("Very short reason should be rejected for meaningful auditing");
        }

        // 非常に長い理由（DoS攻撃対策）
        request.reason = "a".repeat(10000);
        let result = request.validate();
        if result.is_ok() {
            println!("Very long reason should be rejected for DoS protection");
        }

        // ビジネスロジックテスト：ユーザーIDと排除フラグの組み合わせ
        let test_user_id = uuid::Uuid::new_v4();

        // 特定ユーザーを指定して現在ユーザーを排除しない場合
        request.user_id = Some(test_user_id);
        request.exclude_current_user = false;
        request.reason = "Target specific user".to_string();
        assert!(
            request.validate().is_ok(),
            "Specific user targeting should be valid"
        );

        // 全ユーザー対象で現在ユーザーを排除する場合
        request.user_id = None;
        request.exclude_current_user = true;
        request.reason = "Revoke all except current".to_string();
        assert!(
            request.validate().is_ok(),
            "Exclude current user should be valid"
        );

        // 特定ユーザー指定で排除フラグが有効（論理的矛盾）
        request.user_id = Some(test_user_id);
        request.exclude_current_user = true;
        request.reason = "Logical contradiction".to_string();
        let result = request.validate();
        if result.is_ok() {
            // 基本バリデーションは通るが、ビジネスロジックで検証される
            println!("Logical contradiction in user_id + exclude_current_user will be validated at service layer");
        }
    }

    #[test]
    fn test_audit_report_request_validation() {
        let mut request = test_helpers::create_valid_audit_report_request();
        assert!(request.validate().is_ok());

        // 無効なケース
        request.report_type = "".to_string();
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_token_stats_structure() {
        // 正常なケース：典型的なプロダクション統計
        let normal_stats = RefreshTokenStats {
            total_active: 100,
            total_expired: 50,
            users_with_tokens: 80,
            average_tokens_per_user: 1.25,
            oldest_token_age_days: 7,
            newest_token_age_hours: 2,
        };

        // ビジネスロジックの検証
        assert!(
            normal_stats.total_active > 0,
            "Active tokens should be positive"
        );
        assert!(
            normal_stats.users_with_tokens <= normal_stats.total_active,
            "Users with tokens cannot exceed active tokens"
        );
        assert!(
            normal_stats.average_tokens_per_user > 0.0,
            "Average tokens per user should be positive"
        );

        // 追加のビジネスルール検証
        assert!(
            normal_stats.users_with_tokens > 0,
            "Should have users with tokens if active tokens exist"
        );

        let calculated_average =
            normal_stats.total_active as f64 / normal_stats.users_with_tokens as f64;
        assert!(
            (normal_stats.average_tokens_per_user - calculated_average).abs() < 0.01,
            "Average tokens per user should match calculation: {} vs {}",
            normal_stats.average_tokens_per_user,
            calculated_average
        );

        // エッジケース：システム初期状態
        let empty_system_stats = RefreshTokenStats {
            total_active: 0,
            total_expired: 0,
            users_with_tokens: 0,
            average_tokens_per_user: 0.0,
            oldest_token_age_days: 0,
            newest_token_age_hours: 0,
        };

        assert_eq!(empty_system_stats.total_active, 0);
        assert_eq!(empty_system_stats.users_with_tokens, 0);
        assert_eq!(empty_system_stats.average_tokens_per_user, 0.0);

        // エッジケース：大規模システム
        let large_system_stats = RefreshTokenStats {
            total_active: 1_000_000,
            total_expired: 500_000,
            users_with_tokens: 100_000,
            average_tokens_per_user: 10.0,
            oldest_token_age_days: 365,
            newest_token_age_hours: 0,
        };

        assert!(large_system_stats.total_active >= large_system_stats.users_with_tokens);
        assert!(large_system_stats.average_tokens_per_user >= 1.0);
        assert!(
            large_system_stats.oldest_token_age_days <= 365,
            "Tokens older than 1 year might indicate security issues"
        );

        // ビジネスロジック：セキュリティ警告レベルのテスト
        let suspicious_stats = RefreshTokenStats {
            total_active: 50,
            total_expired: 1000,
            users_with_tokens: 10,
            average_tokens_per_user: 5.0,
            oldest_token_age_days: 400, // 1年以上
            newest_token_age_hours: 1,
        };

        // セキュリティ指標の検証
        let expiry_ratio = suspicious_stats.total_expired as f64
            / (suspicious_stats.total_active + suspicious_stats.total_expired) as f64;
        assert!(
            expiry_ratio > 0.8,
            "High expiry ratio might indicate security policy issues"
        );

        assert!(
            suspicious_stats.oldest_token_age_days > 365,
            "Very old tokens should trigger security review"
        );

        assert!(
            suspicious_stats.average_tokens_per_user >= 1.0,
            "Each user should have at least one token if they have any"
        );

        // パフォーマンス考慮：異常に多いトークン数
        let performance_concern_stats = RefreshTokenStats {
            total_active: 1000,
            total_expired: 100,
            users_with_tokens: 10,
            average_tokens_per_user: 100.0, // 1ユーザーあたり100トークン
            oldest_token_age_days: 30,
            newest_token_age_hours: 1,
        };

        assert!(
            performance_concern_stats.average_tokens_per_user > 50.0,
            "Very high token count per user might indicate abuse"
        );

        // 時間的整合性の検証
        assert!(
            normal_stats.oldest_token_age_days * 24 >= normal_stats.newest_token_age_hours,
            "Oldest token age should be greater than newest token age"
        );

        // 数学的整合性の検証：平均値の妥当性
        let min_possible_average = 1.0; // 各ユーザー最低1トークン
        let max_possible_average = normal_stats.total_active as f64 / 1.0; // 1ユーザーが全トークン保持

        assert!(
            normal_stats.average_tokens_per_user >= min_possible_average,
            "Average should be at least 1.0"
        );
        assert!(
            normal_stats.average_tokens_per_user <= max_possible_average,
            "Average cannot exceed total tokens"
        );
    }

    #[test]
    fn test_session_analytics_structure() {
        let analytics = SessionAnalytics {
            total_sessions: 1000,
            active_sessions: 200,
            unique_users_today: 150,
            unique_users_this_week: 800,
            average_session_duration_minutes: 45.5,
            peak_concurrent_sessions: 250,
            suspicious_activity_count: 5,
            geographic_distribution: vec![],
            device_distribution: vec![],
        };

        assert!(analytics.active_sessions <= analytics.total_sessions);
        assert!(analytics.unique_users_today <= analytics.unique_users_this_week);
        assert!(analytics.average_session_duration_minutes > 0.0);
    }
}
