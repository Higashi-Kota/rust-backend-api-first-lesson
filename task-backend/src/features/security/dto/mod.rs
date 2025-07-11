pub mod legacy;
pub mod permission;
pub mod query;
pub mod requests;
pub mod responses;
pub mod role;
pub mod security;

// Re-export specific types to avoid conflicts
// Permission module exports (旧構造)
pub use permission::{
    BulkPermissionCheckRequest, CheckPermissionRequest, ComplexOperationRequest,
    FeatureAccessRequest, SystemPermissionAuditQuery, UserEffectivePermissionsQuery,
    ValidatePermissionRequest,
};

// Security module exports (旧構造 - 重複を避けるため個別にエクスポート)
// 一時的にすべてエクスポート（Phase 19で整理）
pub use security::*;

// Query module exports
pub use query::*;

// Requests module exports
// pub use requests::permission::*;  // TODO: Remove if not needed
// TODO: Phase 19 - 旧security::*との重複を解消後、以下をアンコメント
// pub use requests::security::{
//     CleanupTokensRequest, RevokeAllTokensRequest, AuditReportRequest,
//     DateRange,
// };

// Responses module exports
pub use responses::permission::{
    ActionInfo, AdminFeatureInfo, AdminFeaturesResponse, AdminRiskLevel, AnalyticsFeaturesResponse,
    AnalyticsLevel, AuditCapabilities, AuditPeriod, AuditResult, AvailableResourcesResponse,
    BulkPermissionCheckResponse, ComplexOperationPermissionResponse, DeniedPermission,
    EffectivePermission, ExportCapabilities, FeatureAccessResponse, FeatureInfo, FeatureLimits,
    InheritedPermission, PermissionAuditEntry, PermissionAuditSummary, PermissionCheckDetail,
    PermissionCheckResponse, PermissionCheckResult, PermissionCondition, PermissionInfo,
    PermissionScopeInfo, PermissionSource, PermissionSummary, PermissionValidationResponse,
    PrivilegeInfo, QuotaInfo, ReportInfo, ResourceInfo, ResourcePermissionResponse,
    RestrictedFeatureInfo, SubscriptionRequirement, SystemPermissionAuditResponse,
    SystemPermissionInfo, SystemPermissionScope, UserEffectivePermissionsResponse,
    UserPermissionsResponse, UserRoleInfo, ValidationSummary,
};
// TODO: Phase 19 - 旧security::*との重複を解消後、以下をアンコメント
// pub use responses::security::{
//     TokenStatsResponse, CleanupTokensResponse, RevokeAllTokensResponse,
//     RefreshTokenMonitorResponse, PasswordResetMonitorResponse,
//     SessionAnalyticsResponse, AuditReportResponse,
// };

// Role module exports
pub use role::*;
