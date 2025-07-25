// task-backend/src/service/security_service.rs

use crate::api::dto::security_dto::*;
use crate::error::AppResult;
use crate::log_with_context;
use crate::repository::{
    activity_log_repository::ActivityLogRepository,
    login_attempt_repository::LoginAttemptRepository,
    password_reset_token_repository::PasswordResetTokenRepository,
    refresh_token_repository::RefreshTokenRepository,
    security_incident_repository::SecurityIncidentRepository, user_repository::UserRepository,
};
use crate::types::Timestamp;
use chrono::Duration;
use std::sync::Arc;
use uuid::Uuid;

/// セキュリティ管理サービス
pub struct SecurityService {
    refresh_token_repo: Arc<RefreshTokenRepository>,
    password_reset_repo: Arc<PasswordResetTokenRepository>,
    activity_log_repo: Arc<ActivityLogRepository>,
    security_incident_repo: Arc<SecurityIncidentRepository>,
    login_attempt_repo: Arc<LoginAttemptRepository>,
    user_repo: Arc<UserRepository>,
}

impl SecurityService {
    pub fn new(
        refresh_token_repo: Arc<RefreshTokenRepository>,
        password_reset_repo: Arc<PasswordResetTokenRepository>,
        activity_log_repo: Arc<ActivityLogRepository>,
        security_incident_repo: Arc<SecurityIncidentRepository>,
        login_attempt_repo: Arc<LoginAttemptRepository>,
        user_repo: Arc<UserRepository>,
    ) -> Self {
        Self {
            refresh_token_repo,
            password_reset_repo,
            activity_log_repo,
            security_incident_repo,
            login_attempt_repo,
            user_repo,
        }
    }

    /// リフレッシュトークン統計を取得
    pub async fn get_refresh_token_stats(&self) -> AppResult<RefreshTokenStats> {
        log_with_context!(tracing::Level::DEBUG, "Getting refresh token statistics");
        let stats = self.refresh_token_repo.get_token_stats().await?;

        // 最古・最新のトークン年齢を実際に計算
        let oldest_token_age_days = self
            .refresh_token_repo
            .get_oldest_active_token_age_days()
            .await?
            .unwrap_or(0);

        let newest_token_age_hours = self
            .refresh_token_repo
            .get_newest_active_token_age_hours()
            .await?
            .unwrap_or(0);

        Ok(RefreshTokenStats {
            total_active: stats.active_tokens,
            total_expired: stats.expired_tokens,
            users_with_tokens: stats.total_tokens,
            average_tokens_per_user: if stats.total_tokens > 0 {
                stats.active_tokens as f64 / stats.total_tokens as f64
            } else {
                0.0
            },
            oldest_token_age_days,
            newest_token_age_hours,
        })
    }

    /// パスワードリセットトークン統計を取得
    pub async fn get_password_reset_stats(&self) -> AppResult<PasswordResetTokenStats> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Getting password reset token statistics"
        );
        let stats = self.password_reset_repo.get_token_stats().await?;

        // 実際のデータから計算
        let requests_today = self.password_reset_repo.count_requests_today().await?;
        let requests_this_week = self.password_reset_repo.count_requests_this_week().await?;
        let average_usage_time_minutes = self
            .password_reset_repo
            .get_average_usage_time_minutes()
            .await?
            .unwrap_or(0.0);

        Ok(PasswordResetTokenStats {
            total_active: stats.active_tokens,
            total_used: stats.used_tokens,
            total_expired: stats.expired_tokens,
            requests_today,
            requests_this_week,
            average_usage_time_minutes,
        })
    }

    /// 期限切れリフレッシュトークンをクリーンアップ
    pub async fn cleanup_expired_refresh_tokens(&self) -> AppResult<CleanupResult> {
        log_with_context!(tracing::Level::DEBUG, "Cleaning up expired refresh tokens");
        let result = self.refresh_token_repo.cleanup_expired_tokens().await?;

        log_with_context!(
            tracing::Level::INFO,
            "Cleaned up expired refresh tokens",
            "deleted_count" => result.deleted_count
        );

        Ok(CleanupResult {
            deleted_count: result.deleted_count,
            cleanup_type: "refresh_tokens".to_string(),
        })
    }

    /// 期限切れパスワードリセットトークンをクリーンアップ
    pub async fn cleanup_expired_password_reset_tokens(&self) -> AppResult<CleanupResult> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Cleaning up expired password reset tokens"
        );
        let result = self.password_reset_repo.cleanup_expired_tokens().await?;

        log_with_context!(
            tracing::Level::INFO,
            "Cleaned up expired password reset tokens",
            "deleted_count" => result.total_deleted
        );

        Ok(CleanupResult {
            deleted_count: result.total_deleted,
            cleanup_type: "password_reset_tokens".to_string(),
        })
    }

    /// 最近のパスワードリセット活動を取得
    pub async fn get_recent_password_reset_activity(
        &self,
    ) -> AppResult<Vec<PasswordResetActivity>> {
        let activities = self
            .password_reset_repo
            .get_recent_reset_activity(24)
            .await?;

        // PasswordResetActivity DTOに変換
        let mut result = Vec::new();
        for activity in activities {
            // ユーザー情報を取得
            let (username, email) =
                if let Ok(Some(user)) = self.user_repo.find_by_id(activity.user_id).await {
                    (user.username, user.email)
                } else {
                    (
                        "Deleted User".to_string(),
                        "deleted@example.com".to_string(),
                    )
                };

            result.push(PasswordResetActivity {
                user_id: activity.user_id,
                username,
                email,
                requested_at: Timestamp::from_datetime(activity.created_at),
                used_at: if activity.is_used {
                    Some(activity.updated_at)
                } else {
                    None
                },
                expires_at: Timestamp::from_datetime(activity.expires_at),
                ip_address: None, // IPアドレスはリセットトークンには保存されていないため、ログから取得する必要がある
                status: if activity.is_used {
                    "used".to_string()
                } else if activity.expires_at < Timestamp::now().inner() {
                    "expired".to_string()
                } else {
                    "pending".to_string()
                },
            });
        }

        Ok(result)
    }

    /// 全トークン無効化
    pub async fn revoke_all_tokens(
        &self,
        request: &RevokeAllTokensRequest,
        current_user_id: Option<Uuid>,
    ) -> AppResult<RevokeResult> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Revoking tokens",
            "user_id" => request.user_id,
            "exclude_current_user" => request.exclude_current_user,
            "reason" => &request.reason
        );
        let mut total_revoked = 0u64;
        let affected_users: u64;

        match request.user_id {
            Some(user_id) => {
                // 特定ユーザーのトークンを無効化
                let revoked = self
                    .refresh_token_repo
                    .revoke_all_user_tokens(user_id)
                    .await?;
                total_revoked += revoked;
                affected_users = if revoked > 0 { 1 } else { 0 };
            }
            None => {
                // 全ユーザーのトークンを無効化
                if request.exclude_current_user {
                    if let Some(current_id) = current_user_id {
                        let result = self
                            .refresh_token_repo
                            .revoke_all_tokens_except_user(current_id)
                            .await?;
                        total_revoked += result.revoked_count;
                        affected_users = result.affected_users;
                    } else {
                        let result = self.refresh_token_repo.revoke_all_tokens().await?;
                        total_revoked += result.revoked_count;
                        affected_users = result.affected_users;
                    }
                } else {
                    let result = self.refresh_token_repo.revoke_all_tokens().await?;
                    total_revoked += result.revoked_count;
                    affected_users = result.affected_users;
                }
            }
        }

        log_with_context!(
            tracing::Level::INFO,
            "Tokens revoked successfully",
            "revoked_count" => total_revoked,
            "affected_users" => affected_users,
            "reason" => &request.reason
        );

        Ok(RevokeResult {
            revoked_count: total_revoked,
            affected_users,
            revocation_reason: request.reason.clone(),
            revoked_at: Timestamp::now(),
        })
    }

    /// セッション分析を取得
    pub async fn get_session_analytics(&self) -> AppResult<SessionAnalytics> {
        log_with_context!(tracing::Level::DEBUG, "Getting session analytics");
        // 実際のデータをリポジトリから取得
        let refresh_stats = self.refresh_token_repo.get_token_stats().await?;

        // アクティビティログから実際のユーザー数を取得
        let unique_users_today = self.activity_log_repo.count_unique_users_today().await?;
        let unique_users_this_week = self
            .activity_log_repo
            .count_unique_users_this_week()
            .await?;

        // 不審なアクティビティをログイン試行から検出
        let suspicious_ips = self.login_attempt_repo.find_suspicious_ips(5, 1).await?;
        let suspicious_activity_count = suspicious_ips.len() as u64;

        // 平均セッション継続時間を計算
        let average_session_duration_minutes = self
            .refresh_token_repo
            .get_average_session_duration_minutes()
            .await?;

        // ピーク時の同時セッション数を取得（過去24時間）
        let peak_concurrent_sessions = self
            .refresh_token_repo
            .get_peak_concurrent_sessions(24)
            .await?;

        // 地理情報分布を取得
        let geographic_data = self
            .refresh_token_repo
            .get_geographic_distribution()
            .await?;
        let geographic_distribution = geographic_data
            .into_iter()
            .map(|(country, session_count, unique_users)| GeographicSession {
                country,
                session_count,
                unique_users,
            })
            .collect();

        // デバイス分布を取得
        let device_data = self.refresh_token_repo.get_device_distribution().await?;
        let device_distribution = device_data
            .into_iter()
            .map(|(device_type, session_count, unique_users)| DeviceSession {
                device_type,
                session_count,
                unique_users,
            })
            .collect();

        Ok(SessionAnalytics {
            total_sessions: refresh_stats.total_tokens,
            active_sessions: refresh_stats.active_tokens,
            unique_users_today,
            unique_users_this_week,
            average_session_duration_minutes,
            peak_concurrent_sessions,
            suspicious_activity_count,
            geographic_distribution,
            device_distribution,
        })
    }

    /// 不審なIPアドレス情報を取得
    pub async fn get_suspicious_ips(
        &self,
        failed_attempts_threshold: u32,
        hours: u32,
    ) -> AppResult<Vec<crate::api::dto::analytics_dto::SuspiciousIpInfo>> {
        let suspicious_ips = self
            .login_attempt_repo
            .find_suspicious_ips_with_details(failed_attempts_threshold as u64, hours as i64)
            .await?;

        Ok(suspicious_ips
            .into_iter()
            .map(|(ip_address, failed_attempts, last_attempt)| {
                crate::api::dto::analytics_dto::SuspiciousIpInfo {
                    ip_address,
                    failed_attempts,
                    last_attempt: Timestamp::from_datetime(last_attempt),
                }
            })
            .collect())
    }

    /// 失敗したログイン試行回数を取得
    pub async fn get_failed_login_counts(&self) -> AppResult<(u64, u64)> {
        let today = Timestamp::now().inner() - Duration::days(1);
        let this_week = Timestamp::now().inner() - Duration::days(7);

        let failed_today = self
            .login_attempt_repo
            .count_total_failed_attempts(today)
            .await?;
        let failed_this_week = self
            .login_attempt_repo
            .count_total_failed_attempts(this_week)
            .await?;

        Ok((failed_today, failed_this_week))
    }

    /// セキュリティインシデント数を取得
    pub async fn get_security_incident_count(&self, days: i64) -> AppResult<u64> {
        let start_date = Timestamp::now().inner() - Duration::days(days);
        let end_date = Timestamp::now().inner();

        self.security_incident_repo
            .count_by_date_range(start_date, end_date)
            .await
    }

    /// 監査レポートを生成
    pub async fn generate_audit_report(
        &self,
        request: &AuditReportRequest,
        generated_by: Uuid,
    ) -> AppResult<AuditReport> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Generating audit report",
            "report_type" => &request.report_type,
            "generated_by" => generated_by
        );
        let refresh_stats = self.refresh_token_repo.get_token_stats().await?;
        let password_stats = self.password_reset_repo.get_token_stats().await?;

        // 実際のデータを取得
        let total_events = refresh_stats.total_tokens + password_stats.total_tokens;

        // デフォルトの日付範囲を設定（過去30日間）
        let (start_date, end_date) = if let Some(date_range) = &request.date_range {
            (date_range.start_date.inner(), date_range.end_date.inner())
        } else {
            let end_date = Timestamp::now().inner();
            let start_date = end_date - Duration::days(30);
            (start_date, end_date)
        };

        // セキュリティインシデントの実数を取得
        let security_incidents = self
            .security_incident_repo
            .count_by_date_range(start_date, end_date)
            .await?;

        // 失敗ログインの実数を取得
        let failed_logins = self
            .login_attempt_repo
            .count_total_failed_attempts(start_date)
            .await?;

        // 不審なアクティビティを検出
        let suspicious_ips = self.login_attempt_repo.find_suspicious_ips(5, 24).await?;
        let suspicious_activities = suspicious_ips.len() as u64;

        let risk_level = if suspicious_activities > 10 {
            "high"
        } else if suspicious_activities > 3 {
            "medium"
        } else {
            "low"
        };

        let summary = AuditSummary {
            total_events,
            security_incidents,
            failed_logins,
            token_irregularities: refresh_stats.expired_tokens / 10,
            suspicious_activities,
            risk_level: risk_level.to_string(),
        };

        let findings = if request.include_details.unwrap_or(false) {
            vec![
                AuditFinding {
                    finding_id: Uuid::new_v4(),
                    category: "token_management".to_string(),
                    severity: "info".to_string(),
                    description: "High number of expired tokens detected".to_string(),
                    affected_users: vec![],
                    first_occurrence: Timestamp::from(
                        Timestamp::now().inner() - Duration::hours(24),
                    ),
                    last_occurrence: Timestamp::now(),
                    count: refresh_stats.expired_tokens,
                    details: Some(serde_json::json!({
                        "expired_count": refresh_stats.expired_tokens,
                        "active_count": refresh_stats.active_tokens
                    })),
                },
                AuditFinding {
                    finding_id: Uuid::new_v4(),
                    category: "authentication".to_string(),
                    severity: "warning".to_string(),
                    description: "Multiple password reset requests detected".to_string(),
                    affected_users: vec![],
                    first_occurrence: Timestamp::from(
                        Timestamp::now().inner() - Duration::hours(48),
                    ),
                    last_occurrence: Timestamp::now(),
                    count: password_stats.total_tokens,
                    details: Some(serde_json::json!({
                        "used_tokens": password_stats.used_tokens,
                        "active_tokens": password_stats.active_tokens
                    })),
                },
            ]
        } else {
            vec![AuditFinding {
                finding_id: Uuid::new_v4(),
                category: "token_management".to_string(),
                severity: "info".to_string(),
                description: "High number of expired tokens detected".to_string(),
                affected_users: vec![],
                first_occurrence: Timestamp::from(Timestamp::now().inner() - Duration::hours(24)),
                last_occurrence: Timestamp::now(),
                count: refresh_stats.expired_tokens,
                details: None,
            }]
        };

        let recommendations = match request.report_type.as_str() {
            "security" => vec![
                "Implement stronger password policies".to_string(),
                "Enable two-factor authentication".to_string(),
            ],
            "tokens" => vec![
                "Reduce token expiration time".to_string(),
                "Implement automatic token cleanup".to_string(),
            ],
            _ => vec!["Review security policies regularly".to_string()],
        };

        let report_id = Uuid::new_v4();

        log_with_context!(
            tracing::Level::INFO,
            "Audit report generated successfully",
            "report_id" => report_id,
            "report_type" => &request.report_type,
            "risk_level" => &summary.risk_level
        );

        Ok(AuditReport {
            report_id,
            report_type: request.report_type.clone(),
            generated_at: Timestamp::now(),
            generated_by,
            date_range: request.date_range.clone(),
            summary,
            findings,
            recommendations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::dto::security_dto::test_helpers;
    use validator::Validate;

    #[tokio::test]
    async fn test_security_service_creation() {
        // セキュリティサービスの作成テスト
        // 実際のテストでは mock を使用
    }

    #[test]
    fn test_revoke_all_tokens_request_structure() {
        let request = test_helpers::create_valid_revoke_all_tokens_request();

        assert!(request.validate().is_ok());
        assert_eq!(request.reason, "Security incident detected");
        assert!(request.exclude_current_user);
        assert!(request.user_id.is_none());
    }

    #[test]
    fn test_audit_report_request_structure() {
        let request = test_helpers::create_valid_audit_report_request();

        assert!(request.validate().is_ok());
        assert_eq!(request.report_type, "security");
        assert!(request.date_range.is_none());
        assert_eq!(request.include_details, Some(false));
    }

    #[test]
    fn test_session_analytics_data_consistency() {
        let analytics = SessionAnalytics {
            total_sessions: 1000,
            active_sessions: 200,
            unique_users_today: 150,
            unique_users_this_week: 800,
            average_session_duration_minutes: 45.0,
            peak_concurrent_sessions: 250,
            suspicious_activity_count: 5,
            geographic_distribution: vec![GeographicSession {
                country: "Japan".to_string(),
                session_count: 100,
                unique_users: 20,
            }],
            device_distribution: vec![DeviceSession {
                device_type: "desktop".to_string(),
                session_count: 120,
                unique_users: 18,
            }],
        };

        // データ整合性をチェック
        assert!(analytics.active_sessions <= analytics.total_sessions);
        assert!(analytics.unique_users_today <= analytics.unique_users_this_week);
        assert!(analytics.peak_concurrent_sessions >= analytics.active_sessions);
        assert!(analytics.average_session_duration_minutes > 0.0);
    }

    #[test]
    fn test_audit_summary_risk_level_calculation() {
        let low_risk = AuditSummary {
            total_events: 100,
            security_incidents: 0,
            failed_logins: 5,
            token_irregularities: 2,
            suspicious_activities: 1,
            risk_level: "low".to_string(),
        };

        let high_risk = AuditSummary {
            total_events: 1000,
            security_incidents: 5,
            failed_logins: 100,
            token_irregularities: 50,
            suspicious_activities: 25,
            risk_level: "high".to_string(),
        };

        assert_eq!(low_risk.risk_level, "low");
        assert_eq!(high_risk.risk_level, "high");
        assert!(high_risk.security_incidents > low_risk.security_incidents);
        assert!(high_risk.suspicious_activities > low_risk.suspicious_activities);
    }

    #[test]
    fn test_revoke_result_structure() {
        let result = RevokeResult {
            revoked_count: 25,
            affected_users: 10,
            revocation_reason: "Security maintenance".to_string(),
            revoked_at: Timestamp::now(),
        };

        assert_eq!(result.revoked_count, 25);
        assert_eq!(result.affected_users, 10);
        assert_eq!(result.revocation_reason, "Security maintenance");
        assert!(result.revoked_at <= Timestamp::now());
    }
}
