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
#[allow(dead_code)] // DTO fields used in deserialization
pub struct RevokeAllTokensRequest {
    pub user_id: Option<Uuid>, // 特定ユーザーのみ（Noneの場合は全ユーザー）

    #[validate(length(min = 1, message = "Reason is required"))]
    pub reason: String,

    pub exclude_current_user: bool, // 実行者のトークンを除外するか
}

/// 監査レポート生成リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
#[allow(dead_code)] // DTO fields used in deserialization
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
