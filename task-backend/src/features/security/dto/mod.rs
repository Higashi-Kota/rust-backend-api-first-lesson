pub mod legacy;
pub mod permission;
pub mod query;
pub mod requests;
pub mod responses;
pub mod role;
pub mod security;

// Re-export all permission DTOs
pub use permission::*;

// Security module exports (旧構造 - 重複を避けるため個別にエクスポート)
// Exclude AuditSummary to avoid conflict
pub use security::{
    // Use permission::AuditSummary instead
    AuditReportRequest,
    AuditReportResponse,
    CleanupTokensRequest,
    CleanupTokensResponse,
    PasswordResetMonitorResponse,
    RefreshTokenMonitorResponse,
    RevokeAllTokensRequest,
    RevokeAllTokensResponse,
    SessionAnalyticsResponse,
    TokenStatsResponse,
};

// Query module exports

// Requests module exports

// Responses module exports
// (permission module でエクスポート済みのため省略)

// Role module exports
pub use role::*;
