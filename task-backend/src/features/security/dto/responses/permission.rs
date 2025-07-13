// task-backend/src/features/security/dto/responses/permission.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::permission::PermissionScope;
use crate::core::subscription_tier::SubscriptionTier;

/// 権限チェック結果レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionCheckResponse {
    pub user_id: Uuid,
    pub resource: String,
    pub action: String,
    pub allowed: bool,
    pub is_admin: bool,
    pub is_member: bool,
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

// 追加のレスポンス型（監査関連）

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
    pub summary: PermissionAuditSummary,
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

/// 権限監査サマリー
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionAuditSummary {
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

/// 複雑な操作の権限チェック結果
#[derive(Debug, Serialize, Deserialize)]
pub struct ComplexOperationPermissionResponse {
    pub user_id: Uuid,
    pub operation: String,
    pub operation_allowed: bool,
    pub permission_details: Vec<PermissionCheckDetail>,
    pub checked_at: DateTime<Utc>,
}

/// 権限チェックの詳細
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionCheckDetail {
    pub permission_type: String,
    pub allowed: bool,
    pub description: String,
}
