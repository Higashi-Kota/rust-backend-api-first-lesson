// task-backend/src/api/dto/permission_dto.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::permission::{PermissionResult, PermissionScope};
use crate::domain::subscription_tier::SubscriptionTier;

// --- Request DTOs ---

/// 権限チェックリクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CheckPermissionRequest {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Resource must be between 1 and 50 characters"
    ))]
    pub resource: String,

    #[validate(length(
        min = 1,
        max = 50,
        message = "Action must be between 1 and 50 characters"
    ))]
    pub action: String,

    pub target_user_id: Option<Uuid>,
    pub context: Option<PermissionContext>,
}

/// 権限検証リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ValidatePermissionRequest {
    pub permissions: Vec<PermissionCheck>,
    pub require_all: Option<bool>, // true: AND logic, false: OR logic
}

/// 個別権限チェック
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PermissionCheck {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Resource must be between 1 and 50 characters"
    ))]
    pub resource: String,

    #[validate(length(
        min = 1,
        max = 50,
        message = "Action must be between 1 and 50 characters"
    ))]
    pub action: String,

    pub target_user_id: Option<Uuid>,
}

/// 権限コンテキスト
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionContext {
    pub organization_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub additional_context: Option<serde_json::Value>,
}

/// 機能アクセスチェックリクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct FeatureAccessRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Feature name must be between 1 and 100 characters"
    ))]
    pub feature_name: String,

    pub required_tier: Option<SubscriptionTier>,
    pub context: Option<PermissionContext>,
}

// --- Response DTOs ---

/// 権限チェック結果レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionCheckResponse {
    pub user_id: Uuid,
    pub resource: String,
    pub action: String,
    pub allowed: bool,
    pub reason: Option<String>,
    pub scope: Option<PermissionScopeInfo>,
    pub privilege: Option<PrivilegeInfo>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// 権限検証結果レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionValidationResponse {
    pub user_id: Uuid,
    pub overall_result: bool,
    pub require_all: bool,
    pub checks: Vec<PermissionCheckResult>,
    pub summary: ValidationSummary,
}

/// 個別権限チェック結果
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionCheckResult {
    pub resource: String,
    pub action: String,
    pub allowed: bool,
    pub reason: Option<String>,
    pub scope: Option<PermissionScopeInfo>,
}

/// ユーザー権限情報レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct UserPermissionsResponse {
    pub user_id: Uuid,
    pub role: UserRoleInfo,
    pub subscription_tier: SubscriptionTier,
    pub permissions: Vec<PermissionInfo>,
    pub features: Vec<FeatureInfo>,
    pub effective_scopes: Vec<PermissionScopeInfo>,
    pub last_updated: DateTime<Utc>,
}

/// 利用可能リソース一覧レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableResourcesResponse {
    pub user_id: Uuid,
    pub resources: Vec<ResourceInfo>,
    pub total_resources: u32,
    pub accessible_resources: u32,
    pub restricted_resources: u32,
}

/// 機能アクセス情報レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureAccessResponse {
    pub user_id: Uuid,
    pub subscription_tier: SubscriptionTier,
    pub available_features: Vec<FeatureInfo>,
    pub restricted_features: Vec<RestrictedFeatureInfo>,
    pub feature_limits: FeatureLimits,
}

/// 管理者機能アクセスレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminFeaturesResponse {
    pub admin_user_id: Uuid,
    pub admin_features: Vec<AdminFeatureInfo>,
    pub system_permissions: Vec<SystemPermissionInfo>,
    pub audit_capabilities: AuditCapabilities,
}

/// アナリティクス機能アクセスレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsFeaturesResponse {
    pub user_id: Uuid,
    pub subscription_tier: SubscriptionTier,
    pub analytics_level: AnalyticsLevel,
    pub available_reports: Vec<ReportInfo>,
    pub data_retention_days: Option<u32>,
    pub export_capabilities: ExportCapabilities,
}

// --- Supporting Structures ---

/// 権限スコープ情報
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionScopeInfo {
    pub scope: PermissionScope,
    pub description: String,
    pub level: u8,
}

/// 特権情報
#[derive(Debug, Serialize, Deserialize)]
pub struct PrivilegeInfo {
    pub name: String,
    pub subscription_tier: SubscriptionTier,
    pub quota: Option<QuotaInfo>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// クォータ情報
#[derive(Debug, Serialize, Deserialize)]
pub struct QuotaInfo {
    pub max_items: Option<u32>,
    pub rate_limit: Option<u32>,
    pub features: Vec<String>,
    pub current_usage: Option<QuotaUsage>,
}

/// クォータ使用状況
#[derive(Debug, Serialize, Deserialize)]
pub struct QuotaUsage {
    pub items_used: u32,
    pub requests_today: u32,
    pub features_used: Vec<String>,
    pub last_reset: DateTime<Utc>,
}

/// 検証サマリー
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub total_checks: u32,
    pub allowed_count: u32,
    pub denied_count: u32,
    pub success_rate: f64,
}

/// ユーザーロール情報
#[derive(Debug, Serialize, Deserialize)]
pub struct UserRoleInfo {
    pub role_id: Uuid,
    pub role_name: String,
    pub display_name: String,
    pub is_active: bool,
    pub permission_level: u8,
}

/// 権限情報
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionInfo {
    pub resource: String,
    pub action: String,
    pub scope: PermissionScope,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// 機能情報
#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureInfo {
    pub feature_name: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub required_tier: SubscriptionTier,
    pub is_enabled: bool,
    pub quota: Option<QuotaInfo>,
}

/// 制限された機能情報
#[derive(Debug, Serialize, Deserialize)]
pub struct RestrictedFeatureInfo {
    pub feature_name: String,
    pub display_name: String,
    pub required_tier: SubscriptionTier,
    pub current_tier: SubscriptionTier,
    pub upgrade_required: bool,
    pub restriction_reason: String,
}

/// リソース情報
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub resource_type: String,
    pub display_name: String,
    pub description: String,
    pub available_actions: Vec<ActionInfo>,
    pub restricted_actions: Vec<RestrictedActionInfo>,
    pub scope: PermissionScope,
}

/// アクション情報
#[derive(Debug, Serialize, Deserialize)]
pub struct ActionInfo {
    pub action: String,
    pub display_name: String,
    pub description: String,
    pub required_role: Option<String>,
    pub required_tier: Option<SubscriptionTier>,
}

/// 制限されたアクション情報
#[derive(Debug, Serialize, Deserialize)]
pub struct RestrictedActionInfo {
    pub action: String,
    pub display_name: String,
    pub restriction_reason: String,
    pub required_role: Option<String>,
    pub required_tier: Option<SubscriptionTier>,
}

/// 機能制限
#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureLimits {
    pub max_projects: Option<u32>,
    pub max_tasks_per_project: Option<u32>,
    pub max_team_members: Option<u32>,
    pub max_api_requests_per_hour: Option<u32>,
    pub max_storage_mb: Option<u32>,
    pub advanced_features_enabled: bool,
    pub custom_integrations_enabled: bool,
}

/// 管理者機能情報
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminFeatureInfo {
    pub feature_name: String,
    pub display_name: String,
    pub category: String,
    pub description: String,
    pub risk_level: AdminRiskLevel,
    pub requires_confirmation: bool,
}

/// システム権限情報
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemPermissionInfo {
    pub permission_name: String,
    pub display_name: String,
    pub description: String,
    pub scope: SystemPermissionScope,
    pub is_granted: bool,
}

/// 監査機能
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditCapabilities {
    pub can_view_audit_logs: bool,
    pub can_export_audit_logs: bool,
    pub can_view_system_logs: bool,
    pub audit_retention_days: u32,
    pub real_time_monitoring: bool,
}

/// アナリティクスレベル
#[derive(Debug, Serialize, Deserialize)]
pub enum AnalyticsLevel {
    Basic,
    Advanced,
    Enterprise,
    Custom,
}

/// レポート情報
#[derive(Debug, Serialize, Deserialize)]
pub struct ReportInfo {
    pub report_name: String,
    pub display_name: String,
    pub category: String,
    pub description: String,
    pub required_tier: SubscriptionTier,
    pub is_real_time: bool,
    pub scheduled_available: bool,
}

/// エクスポート機能
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportCapabilities {
    pub formats: Vec<String>, // ["csv", "json", "pdf", "excel"]
    pub max_records: Option<u32>,
    pub batch_export: bool,
    pub scheduled_export: bool,
    pub custom_templates: bool,
}

/// 管理者リスクレベル
#[derive(Debug, Serialize, Deserialize)]
pub enum AdminRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// システム権限スコープ
#[derive(Debug, Serialize, Deserialize)]
pub enum SystemPermissionScope {
    ReadOnly,
    Modify,
    Delete,
    SystemWide,
}

// --- Helper Implementations ---

impl From<PermissionResult> for PermissionCheckResponse {
    fn from(result: PermissionResult) -> Self {
        match result {
            PermissionResult::Allowed { privilege, scope } => {
                let scope_info = Some(PermissionScopeInfo {
                    scope: scope.clone(),
                    description: scope.description(),
                    level: scope.level(),
                });

                let privilege_info = privilege.map(|p| PrivilegeInfo {
                    name: p.name,
                    subscription_tier: p.subscription_tier,
                    quota: p.quota.map(|q| QuotaInfo {
                        max_items: q.max_items,
                        rate_limit: q.rate_limit,
                        features: q.features,
                        current_usage: None, // Would be populated from actual usage tracking
                    }),
                    expires_at: None,
                });

                Self {
                    user_id: Uuid::new_v4(),  // This would be set by the handler
                    resource: "".to_string(), // This would be set by the handler
                    action: "".to_string(),   // This would be set by the handler
                    allowed: true,
                    reason: None,
                    scope: scope_info,
                    privilege: privilege_info,
                    expires_at: None,
                }
            }
            PermissionResult::Denied { reason } => {
                Self {
                    user_id: Uuid::new_v4(),  // This would be set by the handler
                    resource: "".to_string(), // This would be set by the handler
                    action: "".to_string(),   // This would be set by the handler
                    allowed: false,
                    reason: Some(reason),
                    scope: None,
                    privilege: None,
                    expires_at: None,
                }
            }
        }
    }
}

impl PermissionScope {
    pub fn description(&self) -> String {
        match self {
            PermissionScope::Own => "Access to own resources only".to_string(),
            PermissionScope::Team => "Access to team resources".to_string(),
            PermissionScope::Organization => "Access to organization resources".to_string(),
            PermissionScope::Global => "Access to all resources".to_string(),
        }
    }
}

impl ValidationSummary {
    pub fn new(checks: &[PermissionCheckResult]) -> Self {
        let total = checks.len() as u32;
        let allowed = checks.iter().filter(|c| c.allowed).count() as u32;
        let denied = total - allowed;
        let success_rate = if total > 0 {
            (allowed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total_checks: total,
            allowed_count: allowed,
            denied_count: denied,
            success_rate,
        }
    }
}

// --- Permission Audit API DTOs ---

/// リソース固有権限チェックリクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ResourcePermissionRequest {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Resource must be between 1 and 50 characters"
    ))]
    pub resource: String,

    #[validate(length(
        min = 1,
        max = 50,
        message = "Action must be between 1 and 50 characters"
    ))]
    pub action: String,

    pub target_user_id: Option<Uuid>,
    pub context: Option<PermissionContext>,
}

/// バルク権限チェックリクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BulkPermissionCheckRequest {
    pub checks: Vec<PermissionCheck>,
    pub user_id: Option<Uuid>, // 対象ユーザー（省略時は実行者）
}

/// ユーザー有効権限取得リクエスト
#[derive(Debug, Serialize, Deserialize)]
pub struct UserEffectivePermissionsQuery {
    pub include_inherited: Option<bool>,
    pub resource_filter: Option<String>,
}

/// システム権限監査リクエスト
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemPermissionAuditQuery {
    pub user_id: Option<Uuid>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

/// リソース固有権限チェックレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourcePermissionResponse {
    pub user_id: Uuid,
    pub resource: String,
    pub action: String,
    pub allowed: bool,
    pub reason: Option<String>,
    pub permission_scope: Option<PermissionScopeInfo>,
    pub subscription_requirements: Option<SubscriptionRequirement>,
    pub checked_at: DateTime<Utc>,
}

/// バルク権限チェックレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct BulkPermissionCheckResponse {
    pub user_id: Uuid,
    pub checks: Vec<PermissionCheckResult>,
    pub summary: ValidationSummary,
    pub execution_time_ms: u64,
    pub checked_at: DateTime<Utc>,
}

/// ユーザー有効権限レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct UserEffectivePermissionsResponse {
    pub user_id: Uuid,
    pub role: UserRoleInfo,
    pub subscription_tier: SubscriptionTier,
    pub effective_permissions: Vec<EffectivePermission>,
    pub inherited_permissions: Vec<InheritedPermission>,
    pub denied_permissions: Vec<DeniedPermission>,
    pub permission_summary: PermissionSummary,
    pub last_updated: DateTime<Utc>,
}

/// システム権限監査レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemPermissionAuditResponse {
    pub audit_entries: Vec<PermissionAuditEntry>,
    pub summary: AuditSummary,
    pub total_entries: u32,
    pub filtered_entries: u32,
    pub audit_period: AuditPeriod,
}

/// サブスクリプション要件
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionRequirement {
    pub required_tier: SubscriptionTier,
    pub current_tier: SubscriptionTier,
    pub upgrade_required: bool,
    pub upgrade_message: String,
}

/// 有効権限
#[derive(Debug, Serialize, Deserialize)]
pub struct EffectivePermission {
    pub resource: String,
    pub action: String,
    pub scope: PermissionScope,
    pub source: PermissionSource,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Vec<PermissionCondition>,
}

/// 継承権限
#[derive(Debug, Serialize, Deserialize)]
pub struct InheritedPermission {
    pub resource: String,
    pub action: String,
    pub scope: PermissionScope,
    pub inherited_from: PermissionSource,
    pub inheritance_chain: Vec<String>,
    pub granted_at: DateTime<Utc>,
}

/// 拒否権限
#[derive(Debug, Serialize, Deserialize)]
pub struct DeniedPermission {
    pub resource: String,
    pub action: String,
    pub reason: String,
    pub required_role: Option<String>,
    pub required_subscription: Option<SubscriptionTier>,
    pub can_be_granted: bool,
}

/// 権限サマリー
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionSummary {
    pub total_permissions: u32,
    pub effective_permissions: u32,
    pub inherited_permissions: u32,
    pub denied_permissions: u32,
    pub coverage_percentage: f64,
    pub highest_scope: PermissionScope,
}

/// 権限監査エントリ
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionAuditEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub resource: String,
    pub action: String,
    pub result: AuditResult,
    pub reason: Option<String>,
    pub scope: Option<PermissionScope>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// 監査サマリー
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditSummary {
    pub total_checks: u32,
    pub allowed_checks: u32,
    pub denied_checks: u32,
    pub unique_users: u32,
    pub unique_resources: u32,
    pub most_accessed_resource: String,
    pub most_denied_action: String,
    pub success_rate: f64,
}

/// 監査期間
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditPeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub duration_hours: u32,
}

/// 権限ソース
#[derive(Debug, Serialize, Deserialize)]
pub enum PermissionSource {
    Role,
    Subscription,
    Direct,
    Inherited,
    System,
}

/// 権限条件
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionCondition {
    pub condition_type: String,
    pub value: String,
    pub description: String,
}

/// 監査結果
#[derive(Debug, Serialize, Deserialize)]
pub enum AuditResult {
    Allowed,
    Denied,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_check_response_allowed() {
        use crate::domain::permission::{PermissionResult, PermissionScope};

        let result = PermissionResult::Allowed {
            privilege: None,
            scope: PermissionScope::Own,
        };
        let mut response = PermissionCheckResponse::from(result);

        // Update the response with specific values
        let user_id = Uuid::new_v4();
        response.user_id = user_id;
        response.resource = "tasks".to_string();
        response.action = "read".to_string();

        assert_eq!(response.user_id, user_id);
        assert_eq!(response.resource, "tasks");
        assert_eq!(response.action, "read");
        assert!(response.allowed);
        assert!(response.reason.is_none());
    }

    #[test]
    fn test_permission_check_response_denied() {
        use crate::domain::permission::PermissionResult;

        let result = PermissionResult::Denied {
            reason: "Insufficient permissions".to_string(),
        };
        let mut response = PermissionCheckResponse::from(result);

        // Update the response with specific values
        let user_id = Uuid::new_v4();
        response.user_id = user_id;
        response.resource = "admin".to_string();
        response.action = "delete_user".to_string();

        assert_eq!(response.user_id, user_id);
        assert_eq!(response.resource, "admin");
        assert_eq!(response.action, "delete_user");
        assert!(!response.allowed);
        assert_eq!(
            response.reason,
            Some("Insufficient permissions".to_string())
        );
    }

    #[test]
    fn test_permission_scope_level() {
        use crate::domain::permission::PermissionScope;
        assert_eq!(PermissionScope::Own.level(), 1);
        assert_eq!(PermissionScope::Team.level(), 2);
        assert_eq!(PermissionScope::Organization.level(), 3);
        assert_eq!(PermissionScope::Global.level(), 4);
    }

    #[test]
    fn test_validation_summary() {
        let checks = vec![
            PermissionCheckResult {
                resource: "tasks".to_string(),
                action: "read".to_string(),
                allowed: true,
                reason: None,
                scope: None,
            },
            PermissionCheckResult {
                resource: "tasks".to_string(),
                action: "delete".to_string(),
                allowed: false,
                reason: Some("Insufficient permissions".to_string()),
                scope: None,
            },
        ];

        let summary = ValidationSummary::new(&checks);
        assert_eq!(summary.total_checks, 2);
        assert_eq!(summary.allowed_count, 1);
        assert_eq!(summary.denied_count, 1);
        assert_eq!(summary.success_rate, 50.0);
    }
}
