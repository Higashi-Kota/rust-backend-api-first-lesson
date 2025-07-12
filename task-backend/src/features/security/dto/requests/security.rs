// task-backend/src/features/security/dto/requests/security.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// トークンクリーンアップリクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CleanupTokensRequest {
    #[validate(length(min = 1, message = "Cleanup type is required"))]
    pub cleanup_type: String, // "refresh_tokens", "password_reset_tokens", "all"
}

/// 全トークン無効化リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
#[allow(dead_code)] // Fields are used in service implementation
pub struct RevokeAllTokensRequest {
    pub user_id: Option<Uuid>, // 特定ユーザーのみ（Noneの場合は全ユーザー）

    #[validate(length(min = 1, message = "Reason is required"))]
    pub reason: String,

    pub exclude_current_user: bool, // 実行者のトークンを除外するか
}

/// 監査レポート生成リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
#[allow(dead_code)] // Fields are used in service implementation
pub struct AuditReportRequest {
    #[validate(length(min = 1, message = "Report type is required"))]
    pub report_type: String, // "security", "tokens", "sessions", "comprehensive"

    pub date_range: Option<DateRange>,
    pub include_details: Option<bool>,
}

/// 日付範囲
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DateRange {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cleanup_tokens_request_deserialization() {
        let json = json!({
            "cleanup_type": "refresh_tokens"
        });

        let request: CleanupTokensRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.cleanup_type, "refresh_tokens");
        assert!(request.validate().is_ok());

        // Test validation error
        let invalid_json = json!({
            "cleanup_type": ""
        });
        let invalid_request: CleanupTokensRequest = serde_json::from_value(invalid_json).unwrap();
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_revoke_all_tokens_request_deserialization() {
        let json = json!({
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "reason": "Security breach detected",
            "exclude_current_user": true
        });

        let request: RevokeAllTokensRequest = serde_json::from_value(json).unwrap();
        assert_eq!(
            request.user_id,
            Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap())
        );
        assert_eq!(request.reason, "Security breach detected");
        assert!(request.exclude_current_user);
        assert!(request.validate().is_ok());

        // Test with null user_id
        let json_no_user = json!({
            "user_id": null,
            "reason": "Emergency shutdown",
            "exclude_current_user": false
        });

        let request2: RevokeAllTokensRequest = serde_json::from_value(json_no_user).unwrap();
        assert_eq!(request2.user_id, None);
        assert_eq!(request2.reason, "Emergency shutdown");
        assert!(!request2.exclude_current_user);
        assert!(request2.validate().is_ok());

        // Test validation error
        let invalid_json = json!({
            "user_id": null,
            "reason": "",
            "exclude_current_user": false
        });
        let invalid_request: RevokeAllTokensRequest = serde_json::from_value(invalid_json).unwrap();
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_audit_report_request_deserialization() {
        // Test with all fields
        let json = json!({
            "report_type": "security",
            "date_range": {
                "start_date": "2023-01-01T00:00:00Z",
                "end_date": "2023-12-31T23:59:59Z"
            },
            "include_details": true
        });

        let request: AuditReportRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.report_type, "security");
        assert!(request.date_range.is_some());
        assert_eq!(request.include_details, Some(true));
        assert!(request.validate().is_ok());

        // Test with minimal fields
        let json_minimal = json!({
            "report_type": "comprehensive"
        });

        let request2: AuditReportRequest = serde_json::from_value(json_minimal).unwrap();
        assert_eq!(request2.report_type, "comprehensive");
        assert!(request2.date_range.is_none());
        assert!(request2.include_details.is_none());
        assert!(request2.validate().is_ok());

        // Test validation error
        let invalid_json = json!({
            "report_type": ""
        });
        let invalid_request: AuditReportRequest = serde_json::from_value(invalid_json).unwrap();
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_date_range_deserialization() {
        let json = json!({
            "start_date": "2023-01-01T00:00:00Z",
            "end_date": "2023-12-31T23:59:59Z"
        });

        let date_range: DateRange = serde_json::from_value(json).unwrap();
        assert_eq!(
            date_range.start_date.to_rfc3339(),
            "2023-01-01T00:00:00+00:00"
        );
        assert_eq!(
            date_range.end_date.to_rfc3339(),
            "2023-12-31T23:59:59+00:00"
        );
    }
}
