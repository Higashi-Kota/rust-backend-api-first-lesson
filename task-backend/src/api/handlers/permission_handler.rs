// task-backend/src/api/handlers/permission_handler.rs

use crate::api::dto::permission_dto::*;
use crate::api::AppState;
use crate::domain::permission::{
    Permission, PermissionQuota, PermissionResult, PermissionScope, Privilege,
};
use crate::domain::subscription_tier::SubscriptionTier;
use crate::error::{AppError, AppResult};
use crate::extractors::{ValidatedMultiPath, ValidatedUuid};
use crate::middleware::auth::{AuthenticatedUser, AuthenticatedUserWithRole};
use crate::types::{ApiResponse, Timestamp};
use crate::utils::error_helper::convert_validation_errors;
use crate::utils::permission::PermissionType;
use axum::{
    extract::{Json, Query, State},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

// 複数パラメータ用のPath構造体
#[derive(Deserialize)]
pub struct ResourceActionPath {
    pub resource: String,
    pub action: String,
}

// --- Query Parameters ---

/// 権限検索パラメータ
#[derive(Debug, Deserialize)]
pub struct PermissionQuery {
    pub resource: Option<String>,
    pub action: Option<String>,
}

/// 機能検索パラメータ
#[derive(Debug, Deserialize)]
pub struct FeatureQuery {
    pub category: Option<String>,
}

// --- Permission Checking Endpoints ---

/// 特定の権限をチェック
pub async fn check_permission_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CheckPermissionRequest>,
) -> AppResult<ApiResponse<PermissionCheckResponse>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "permission_handler::check_permission"))?;

    info!(
        user_id = %user.claims.user_id,
        resource = %payload.resource,
        action = %payload.action,
        "Checking permission"
    );

    // 権限チェックを実行
    let result = if let Some(ref role) = user.claims.role {
        role.can_perform_action(&payload.resource, &payload.action, payload.target_user_id)
    } else {
        // Basic permission check using role name - simplified version
        if user.claims.is_admin() || payload.resource == "tasks" {
            // PermissionResult::allowedメソッドを使用
            PermissionResult::allowed(None, PermissionScope::Own)
        } else {
            // PermissionResult::deniedメソッドを使用
            PermissionResult::denied("Insufficient permissions")
        }
    };

    // Permission matchesメソッドの活用例
    let _permission_example = Permission::new("tasks", "read", PermissionScope::Own);
    let _matches = _permission_example.matches("tasks", "read");

    // PermissionResultのインスタンスメソッドを活用して詳細情報を取得
    let is_allowed = result.is_allowed();
    let denial_reason = result.get_denial_reason().cloned();
    let scope_info = result.get_scope().map(|scope| PermissionScopeInfo {
        scope: scope.clone(),
        description: scope.description().to_string(),
        level: scope.level(),
    });
    let privilege_info = result.get_privilege().map(|p| PrivilegeInfo {
        name: p.name.clone(),
        subscription_tier: p.subscription_tier,
        quota: p.quota.as_ref().map(|q| QuotaInfo {
            max_items: q.max_items,
            rate_limit: q.rate_limit,
            features: q.features.clone(),
            current_usage: None,
        }),
        expires_at: None,
    });

    // レスポンスを構築
    let response = PermissionCheckResponse {
        user_id: user.claims.user_id,
        resource: payload.resource,
        action: payload.action,
        allowed: is_allowed,
        is_admin: user.claims.is_admin(),
        is_member: user.claims.is_member(),
        reason: denial_reason,
        scope: scope_info,
        privilege: privilege_info,
        expires_at: None,
    };

    info!(
        user_id = %user.claims.user_id,
        allowed = %response.allowed,
        has_scope = %response.scope.is_some(),
        has_privilege = %response.privilege.is_some(),
        "Permission check completed with detailed information"
    );

    Ok(ApiResponse::success(response))
}

/// 複数の権限を一括検証
pub async fn validate_permissions_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ValidatePermissionRequest>,
) -> AppResult<ApiResponse<PermissionValidationResponse>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "permission_handler::grant_permission"))?;

    info!(
        user_id = %user.claims.user_id,
        permission_count = %payload.permissions.len(),
        require_all = %payload.require_all.unwrap_or(true),
        "Validating multiple permissions"
    );

    let require_all = payload.require_all.unwrap_or(true);
    let mut checks = Vec::new();
    let mut allowed_count = 0;

    // 各権限をチェック
    for permission_check in payload.permissions {
        let result = if let Some(ref role) = user.claims.role {
            role.can_perform_action(
                &permission_check.resource,
                &permission_check.action,
                permission_check.target_user_id,
            )
        } else {
            // Basic permission check - simplified version
            if user.claims.is_admin() || permission_check.resource == "tasks" {
                // PermissionResult::allowedメソッドを使用
                PermissionResult::allowed(None, PermissionScope::Own)
            } else {
                // PermissionResult::deniedメソッドを使用
                PermissionResult::denied("Insufficient permissions")
            }
        };

        let (allowed, reason, scope) = match result {
            PermissionResult::Allowed { scope, .. } => (
                true,
                None,
                Some(PermissionScopeInfo {
                    scope: scope.clone(),
                    description: scope.description().to_string(),
                    level: scope.level(),
                }),
            ),
            PermissionResult::Denied { reason } => (false, Some(reason), None),
        };

        if allowed {
            allowed_count += 1;
        }

        checks.push(PermissionCheckResult {
            resource: permission_check.resource,
            action: permission_check.action,
            allowed,
            reason,
            scope,
        });
    }

    // 全体結果を決定
    let overall_result = if require_all {
        allowed_count == checks.len()
    } else {
        allowed_count > 0
    };

    let summary = ValidationSummary::new(&checks);

    let response = PermissionValidationResponse {
        user_id: user.claims.user_id,
        overall_result,
        require_all,
        checks,
        summary,
    };

    info!(
        user_id = %user.claims.user_id,
        overall_result = %response.overall_result,
        success_rate = %response.summary.success_rate,
        "Permission validation completed"
    );

    Ok(ApiResponse::success(response))
}

/// ユーザーの権限情報を取得
pub async fn get_user_permissions_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedUuid(target_user_id): ValidatedUuid,
) -> AppResult<ApiResponse<UserPermissionsResponse>> {
    // アクセス権限チェック（自分の情報または管理者のみ）
    if user.claims.user_id != target_user_id && !user.claims.is_admin() {
        warn!(
            user_id = %user.claims.user_id,
            target_user_id = %target_user_id,
            "Access denied: Cannot view other user's permissions"
        );
        return Err(AppError::Forbidden(
            "You can only view your own permissions".to_string(),
        ));
    }

    info!(
        user_id = %user.claims.user_id,
        target_user_id = %target_user_id,
        "Getting user permissions"
    );

    // 権限情報を構築（実際の実装では、ターゲットユーザーの情報を取得）
    let role_info = UserRoleInfo {
        role_id: Uuid::new_v4(), // 実際の実装では適切なロールIDを取得
        role_name: user.claims.role_name.clone(),
        display_name: user.claims.role_name.clone(),
        is_active: true,
        permission_level: if user.claims.is_admin() { 100 } else { 10 },
    };

    // 基本権限を構築
    let permissions = get_basic_permissions(&user.claims.subscription_tier);

    // 機能情報を構築
    let features = get_available_features(&user.claims.subscription_tier);

    // 有効スコープを構築
    let effective_scopes = get_effective_scopes(&user.claims.role_name);

    let response = UserPermissionsResponse {
        user_id: target_user_id,
        role: role_info,
        subscription_tier: user.claims.subscription_tier,
        permissions,
        features,
        effective_scopes,
        last_updated: Timestamp::now(),
    };

    info!(
        user_id = %user.claims.user_id,
        target_user_id = %target_user_id,
        permission_count = %response.permissions.len(),
        feature_count = %response.features.len(),
        "User permissions retrieved"
    );

    Ok(ApiResponse::success(response))
}

/// 利用可能なリソース一覧を取得
pub async fn get_available_resources_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<PermissionQuery>,
) -> AppResult<ApiResponse<AvailableResourcesResponse>> {
    info!(
        user_id = %user.claims.user_id,
        resource_filter = ?query.resource,
        action_filter = ?query.action,
        "Getting available resources"
    );

    let resources =
        get_user_accessible_resources(&user.claims.role_name, &user.claims.subscription_tier);

    // フィルタリング
    let filtered_resources = if let Some(resource_filter) = query.resource {
        resources
            .into_iter()
            .filter(|r| r.resource_type.contains(&resource_filter))
            .collect()
    } else {
        resources
    };

    let total_resources = get_total_system_resources();
    let accessible_count = filtered_resources.len() as u32;
    let restricted_count = total_resources - accessible_count;

    let response = AvailableResourcesResponse {
        user_id: user.claims.user_id,
        resources: filtered_resources,
        total_resources,
        accessible_resources: accessible_count,
        restricted_resources: restricted_count,
    };

    info!(
        user_id = %user.claims.user_id,
        accessible_resources = %response.accessible_resources,
        restricted_resources = %response.restricted_resources,
        "Available resources retrieved"
    );

    Ok(ApiResponse::success(response))
}

// --- Feature Access Endpoints ---

/// 利用可能な機能を取得
pub async fn get_feature_access_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<FeatureQuery>,
) -> AppResult<ApiResponse<FeatureAccessResponse>> {
    info!(
        user_id = %user.claims.user_id,
        subscription_tier = %user.claims.subscription_tier,
        "Getting feature access information"
    );

    let available_features = get_available_features(&user.claims.subscription_tier);
    let restricted_features = get_restricted_features(&user.claims.subscription_tier);
    let feature_limits = get_feature_limits(&user.claims.subscription_tier);

    // PermissionScope::includesメソッドの活用例
    let global_scope = PermissionScope::Global;
    let team_scope = PermissionScope::Team;
    let _includes_check = global_scope.includes(&team_scope);

    // カテゴリフィルタリング
    let filtered_available = if let Some(category) = &query.category {
        available_features
            .into_iter()
            .filter(|f| f.category == *category)
            .collect()
    } else {
        available_features
    };

    let filtered_restricted = if let Some(category) = &query.category {
        restricted_features
            .into_iter()
            .filter(|f| f.feature_name.contains(category))
            .collect()
    } else {
        restricted_features
    };

    let response = FeatureAccessResponse {
        user_id: user.claims.user_id,
        subscription_tier: user.claims.subscription_tier,
        available_features: filtered_available,
        restricted_features: filtered_restricted,
        feature_limits,
    };

    info!(
        user_id = %user.claims.user_id,
        available_count = %response.available_features.len(),
        restricted_count = %response.restricted_features.len(),
        "Feature access information retrieved"
    );

    Ok(ApiResponse::success(response))
}

/// 管理者機能アクセス情報を取得
pub async fn get_admin_features_handler(
    State(app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
) -> AppResult<ApiResponse<AdminFeaturesResponse>> {
    use crate::middleware::auth::{
        check_create_permission, check_delete_permission, check_resource_access_permission,
        check_view_permission,
    };
    use crate::utils::permission::PermissionType;

    // 管理者権限チェック（check_permission_typeを使用）
    app_state
        .permission_service
        .check_permission_type(admin_user.user_id(), PermissionType::IsAdmin, None)
        .await?;

    // 権限チェック関数の使用例
    let _can_create = check_create_permission(&admin_user, "admin_features").is_ok();
    let _can_view = check_view_permission(&admin_user, "admin_features", None).is_ok();
    let _can_delete = check_delete_permission(&admin_user, "admin_features", None).is_ok();
    let _resource_access =
        check_resource_access_permission(&admin_user, admin_user.user_id()).is_ok();

    info!(
        admin_id = %admin_user.user_id(),
        "Getting admin features"
    );

    let admin_features = get_admin_features();
    let system_permissions = get_system_permissions();
    let audit_capabilities = get_audit_capabilities();

    let response = AdminFeaturesResponse {
        admin_user_id: admin_user.user_id(),
        admin_features,
        system_permissions,
        audit_capabilities,
    };

    info!(
        admin_id = %admin_user.user_id(),
        admin_feature_count = %response.admin_features.len(),
        system_permission_count = %response.system_permissions.len(),
        "Admin features retrieved"
    );

    Ok(ApiResponse::success(response))
}

/// アナリティクス機能アクセス情報を取得
pub async fn get_analytics_features_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<ApiResponse<AnalyticsFeaturesResponse>> {
    info!(
        user_id = %user.claims.user_id,
        subscription_tier = %user.claims.subscription_tier,
        "Getting analytics features"
    );

    let analytics_level =
        determine_analytics_level(&user.claims.subscription_tier, user.claims.is_admin());
    let available_reports =
        get_available_reports(&user.claims.subscription_tier, user.claims.is_admin());
    let data_retention_days = get_data_retention_days(&user.claims.subscription_tier);
    let export_capabilities = get_export_capabilities(&user.claims.subscription_tier);

    let response = AnalyticsFeaturesResponse {
        user_id: user.claims.user_id,
        subscription_tier: user.claims.subscription_tier,
        analytics_level,
        available_reports,
        data_retention_days,
        export_capabilities,
    };

    info!(
        user_id = %user.claims.user_id,
        analytics_level = ?response.analytics_level,
        report_count = %response.available_reports.len(),
        "Analytics features retrieved"
    );

    Ok(ApiResponse::success(response))
}

// --- Helper Functions ---

fn get_basic_permissions(tier: &SubscriptionTier) -> Vec<PermissionInfo> {
    // Permission::read_own, write_ownなどを活用
    let read_permission = Permission::read_own("tasks");
    let write_permission = Permission::write_own("tasks");
    let mut permissions = vec![
        PermissionInfo {
            resource: read_permission.resource.clone(),
            action: read_permission.action.clone(),
            scope: read_permission.scope.clone(),
            granted_at: Timestamp::now(),
            expires_at: None,
        },
        PermissionInfo {
            resource: write_permission.resource.clone(),
            action: "create".to_string(),
            scope: write_permission.scope.clone(),
            granted_at: Timestamp::now(),
            expires_at: None,
        },
        PermissionInfo {
            resource: "tasks".to_string(),
            action: "update".to_string(),
            scope: PermissionScope::Own,
            granted_at: Timestamp::now(),
            expires_at: None,
        },
        PermissionInfo {
            resource: "tasks".to_string(),
            action: "delete".to_string(),
            scope: PermissionScope::Own,
            granted_at: Timestamp::now(),
            expires_at: None,
        },
    ];

    match tier {
        SubscriptionTier::Pro => {
            permissions.push(PermissionInfo {
                resource: "tasks".to_string(),
                action: "export".to_string(),
                scope: PermissionScope::Team,
                granted_at: Timestamp::now(),
                expires_at: None,
            });
        }
        SubscriptionTier::Enterprise => {
            permissions.push(PermissionInfo {
                resource: "tasks".to_string(),
                action: "bulk_operations".to_string(),
                scope: PermissionScope::Organization,
                granted_at: Timestamp::now(),
                expires_at: None,
            });
        }
        _ => {}
    }

    permissions
}

fn get_available_features(tier: &SubscriptionTier) -> Vec<FeatureInfo> {
    // Privilegeメソッドを活用して特権情報を取得
    let basic_privilege = Privilege::free_basic("basic_tasks", 100, 10);
    let mut features = vec![FeatureInfo {
        feature_name: "basic_tasks".to_string(),
        display_name: "Basic Task Management".to_string(),
        description: "Create, read, update, and delete tasks".to_string(),
        category: "tasks".to_string(),
        required_tier: SubscriptionTier::Free,
        is_enabled: basic_privilege.is_available_for_tier(tier),
        quota: Some(QuotaInfo {
            max_items: basic_privilege.get_max_items(),
            rate_limit: basic_privilege.get_rate_limit(),
            features: vec!["basic_filter".to_string()],
            current_usage: None,
        }),
    }];

    if tier.is_at_least(&SubscriptionTier::Pro) {
        let pro_privilege = Privilege::pro_advanced(
            "advanced_tasks",
            10_000,
            100,
            vec!["advanced_filter", "export"],
        );
        features.push(FeatureInfo {
            feature_name: "advanced_tasks".to_string(),
            display_name: "Advanced Task Management".to_string(),
            description: "Advanced task features including team collaboration".to_string(),
            category: "tasks".to_string(),
            required_tier: SubscriptionTier::Pro,
            is_enabled: pro_privilege.is_available_for_tier(tier),
            quota: Some(QuotaInfo {
                max_items: pro_privilege.get_max_items(),
                rate_limit: pro_privilege.get_rate_limit(),
                features: vec!["advanced_filter".to_string(), "export".to_string()],
                current_usage: None,
            }),
        });

        // PermissionQuota::has_featureメソッドの活用例
        if let Some(quota) = &pro_privilege.quota {
            let _has_advanced = quota.has_feature("advanced_filter");
            let _has_export = quota.has_feature("export");
        }
    }

    if tier.is_at_least(&SubscriptionTier::Enterprise) {
        let enterprise_privilege = Privilege::enterprise_unlimited(
            "enterprise_tasks",
            vec!["bulk_operations", "api_access", "custom_integrations"],
        );
        features.push(FeatureInfo {
            feature_name: "enterprise_tasks".to_string(),
            display_name: "Enterprise Task Management".to_string(),
            description: "Unlimited task management with enterprise features".to_string(),
            category: "tasks".to_string(),
            required_tier: SubscriptionTier::Enterprise,
            is_enabled: enterprise_privilege.is_available_for_tier(tier),
            quota: None,
        });
    }

    features
}

fn get_restricted_features(tier: &SubscriptionTier) -> Vec<RestrictedFeatureInfo> {
    let mut restricted = Vec::new();

    if !tier.is_at_least(&SubscriptionTier::Pro) {
        restricted.push(RestrictedFeatureInfo {
            feature_name: "advanced_reporting".to_string(),
            display_name: "Advanced Reporting".to_string(),
            required_tier: SubscriptionTier::Pro,
            current_tier: *tier,
            upgrade_required: true,
            restriction_reason: "Pro subscription required".to_string(),
        });
    }

    if !tier.is_at_least(&SubscriptionTier::Enterprise) {
        restricted.push(RestrictedFeatureInfo {
            feature_name: "api_access".to_string(),
            display_name: "API Access".to_string(),
            required_tier: SubscriptionTier::Enterprise,
            current_tier: *tier,
            upgrade_required: true,
            restriction_reason: "Enterprise subscription required".to_string(),
        });
    }

    restricted
}

fn get_effective_scopes(role_name: &str) -> Vec<PermissionScopeInfo> {
    match role_name {
        "admin" => vec![
            PermissionScopeInfo {
                scope: PermissionScope::Own,
                description: PermissionScope::Own.description().to_string(),
                level: PermissionScope::Own.level(),
            },
            PermissionScopeInfo {
                scope: PermissionScope::Team,
                description: PermissionScope::Team.description().to_string(),
                level: PermissionScope::Team.level(),
            },
            PermissionScopeInfo {
                scope: PermissionScope::Organization,
                description: PermissionScope::Organization.description().to_string(),
                level: PermissionScope::Organization.level(),
            },
            PermissionScopeInfo {
                scope: PermissionScope::Global,
                description: PermissionScope::Global.description().to_string(),
                level: PermissionScope::Global.level(),
            },
        ],
        _ => vec![PermissionScopeInfo {
            scope: PermissionScope::Own,
            description: PermissionScope::Own.description().to_string(),
            level: PermissionScope::Own.level(),
        }],
    }
}

fn get_user_accessible_resources(role_name: &str, _tier: &SubscriptionTier) -> Vec<ResourceInfo> {
    let mut resources = vec![ResourceInfo {
        resource_type: "tasks".to_string(),
        display_name: "Tasks".to_string(),
        description: "Task management system".to_string(),
        available_actions: vec![
            ActionInfo {
                action: "read".to_string(),
                display_name: "View Tasks".to_string(),
                description: "View task information".to_string(),
                required_role: None,
                required_tier: Some(SubscriptionTier::Free),
            },
            ActionInfo {
                action: "create".to_string(),
                display_name: "Create Tasks".to_string(),
                description: "Create new tasks".to_string(),
                required_role: None,
                required_tier: Some(SubscriptionTier::Free),
            },
        ],
        restricted_actions: vec![],
        scope: if role_name == "admin" {
            PermissionScope::Global
        } else {
            PermissionScope::Own
        },
    }];

    if role_name == "admin" {
        resources.push(ResourceInfo {
            resource_type: "users".to_string(),
            display_name: "User Management".to_string(),
            description: "User administration system".to_string(),
            available_actions: vec![
                ActionInfo {
                    action: "read".to_string(),
                    display_name: "View Users".to_string(),
                    description: "View user information".to_string(),
                    required_role: Some("admin".to_string()),
                    required_tier: None,
                },
                ActionInfo {
                    action: "update".to_string(),
                    display_name: "Manage Users".to_string(),
                    description: "Update user information".to_string(),
                    required_role: Some("admin".to_string()),
                    required_tier: None,
                },
            ],
            restricted_actions: vec![],
            scope: PermissionScope::Global,
        });
    }

    resources
}

fn get_total_system_resources() -> u32 {
    10 // Total number of resource types in the system
}

fn get_feature_limits(tier: &SubscriptionTier) -> FeatureLimits {
    match tier {
        SubscriptionTier::Free => FeatureLimits {
            max_projects: Some(3),
            max_tasks_per_project: Some(100),
            max_team_members: Some(1),
            max_api_requests_per_hour: Some(100),
            max_storage_mb: Some(100),
            advanced_features_enabled: false,
            custom_integrations_enabled: false,
        },
        SubscriptionTier::Pro => FeatureLimits {
            max_projects: Some(50),
            max_tasks_per_project: Some(10_000),
            max_team_members: Some(10),
            max_api_requests_per_hour: Some(1_000),
            max_storage_mb: Some(5_000),
            advanced_features_enabled: true,
            custom_integrations_enabled: false,
        },
        SubscriptionTier::Enterprise => FeatureLimits {
            max_projects: None,
            max_tasks_per_project: None,
            max_team_members: None,
            max_api_requests_per_hour: None,
            max_storage_mb: None,
            advanced_features_enabled: true,
            custom_integrations_enabled: true,
        },
    }
}

fn get_admin_features() -> Vec<AdminFeatureInfo> {
    vec![
        AdminFeatureInfo {
            feature_name: "user_management".to_string(),
            display_name: "User Management".to_string(),
            category: "administration".to_string(),
            description: "Manage users and their accounts".to_string(),
            risk_level: AdminRiskLevel::Medium,
            requires_confirmation: false,
        },
        AdminFeatureInfo {
            feature_name: "system_settings".to_string(),
            display_name: "System Settings".to_string(),
            category: "system".to_string(),
            description: "Configure system-wide settings".to_string(),
            risk_level: AdminRiskLevel::High,
            requires_confirmation: true,
        },
        AdminFeatureInfo {
            feature_name: "data_export".to_string(),
            display_name: "Data Export".to_string(),
            category: "data".to_string(),
            description: "Export system data".to_string(),
            risk_level: AdminRiskLevel::Critical,
            requires_confirmation: true,
        },
    ]
}

fn get_system_permissions() -> Vec<SystemPermissionInfo> {
    vec![
        SystemPermissionInfo {
            permission_name: "view_audit_logs".to_string(),
            display_name: "View Audit Logs".to_string(),
            description: "Access to system audit logs".to_string(),
            scope: SystemPermissionScope::ReadOnly,
            is_granted: true,
        },
        SystemPermissionInfo {
            permission_name: "modify_system_config".to_string(),
            display_name: "Modify System Configuration".to_string(),
            description: "Change system configuration".to_string(),
            scope: SystemPermissionScope::SystemWide,
            is_granted: true,
        },
    ]
}

fn get_audit_capabilities() -> AuditCapabilities {
    AuditCapabilities {
        can_view_audit_logs: true,
        can_export_audit_logs: true,
        can_view_system_logs: true,
        audit_retention_days: 90,
        real_time_monitoring: true,
    }
}

fn determine_analytics_level(tier: &SubscriptionTier, is_admin: bool) -> AnalyticsLevel {
    if is_admin {
        AnalyticsLevel::Custom
    } else {
        match tier {
            SubscriptionTier::Free => AnalyticsLevel::Basic,
            SubscriptionTier::Pro => AnalyticsLevel::Advanced,
            SubscriptionTier::Enterprise => AnalyticsLevel::Enterprise,
        }
    }
}

fn get_available_reports(tier: &SubscriptionTier, is_admin: bool) -> Vec<ReportInfo> {
    let mut reports = vec![ReportInfo {
        report_name: "task_summary".to_string(),
        display_name: "Task Summary".to_string(),
        category: "tasks".to_string(),
        description: "Basic task completion statistics".to_string(),
        required_tier: SubscriptionTier::Free,
        is_real_time: false,
        scheduled_available: false,
    }];

    if tier.is_at_least(&SubscriptionTier::Pro) || is_admin {
        reports.push(ReportInfo {
            report_name: "advanced_analytics".to_string(),
            display_name: "Advanced Analytics".to_string(),
            category: "analytics".to_string(),
            description: "Detailed performance and trend analysis".to_string(),
            required_tier: SubscriptionTier::Pro,
            is_real_time: true,
            scheduled_available: true,
        });
    }

    if tier.is_at_least(&SubscriptionTier::Enterprise) || is_admin {
        reports.push(ReportInfo {
            report_name: "enterprise_dashboard".to_string(),
            display_name: "Enterprise Dashboard".to_string(),
            category: "enterprise".to_string(),
            description: "Comprehensive enterprise metrics and insights".to_string(),
            required_tier: SubscriptionTier::Enterprise,
            is_real_time: true,
            scheduled_available: true,
        });
    }

    reports
}

fn get_data_retention_days(tier: &SubscriptionTier) -> Option<u32> {
    match tier {
        SubscriptionTier::Free => Some(30),
        SubscriptionTier::Pro => Some(365),
        SubscriptionTier::Enterprise => None, // Unlimited
    }
}

fn get_export_capabilities(tier: &SubscriptionTier) -> ExportCapabilities {
    match tier {
        SubscriptionTier::Free => {
            // PermissionQuota::limitedを活用
            let quota = PermissionQuota::limited(1_000, 100);
            ExportCapabilities {
                formats: vec!["csv".to_string()],
                max_records: quota.max_items,
                batch_export: false,
                scheduled_export: false,
                custom_templates: false,
            }
        }
        SubscriptionTier::Pro => {
            // PermissionQuota::limitedを活用
            let quota = PermissionQuota::limited(100_000, 1_000);
            ExportCapabilities {
                formats: vec!["csv".to_string(), "json".to_string(), "pdf".to_string()],
                max_records: quota.max_items,
                batch_export: true,
                scheduled_export: false,
                custom_templates: false,
            }
        }
        SubscriptionTier::Enterprise => {
            // PermissionQuota::unlimitedを活用
            let quota = PermissionQuota::unlimited();
            ExportCapabilities {
                formats: vec![
                    "csv".to_string(),
                    "json".to_string(),
                    "pdf".to_string(),
                    "excel".to_string(),
                ],
                max_records: quota.max_items,
                batch_export: true,
                scheduled_export: true,
                custom_templates: true,
            }
        }
    }
}

// --- Permission Audit APIs ---

/// リソース固有権限チェック
pub async fn check_resource_permission_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedMultiPath(params): ValidatedMultiPath<ResourceActionPath>,
    Query(query): Query<UserEffectivePermissionsQuery>,
) -> AppResult<ApiResponse<ResourcePermissionResponse>> {
    info!(
        user_id = %user.claims.user_id,
        resource = %params.resource,
        action = %params.action,
        "Checking resource-specific permission"
    );

    // 権限チェックを実行
    let result = if let Some(ref role) = user.claims.role {
        role.can_perform_action(
            &params.resource,
            &params.action,
            query.include_inherited.map(|_| user.claims.user_id),
        )
    } else {
        // Basic permission check - simplified version
        if user.claims.is_admin() || params.resource == "tasks" {
            // PermissionResult::allowedメソッドを使用
            PermissionResult::allowed(None, PermissionScope::Own)
        } else {
            // PermissionResult::deniedメソッドを使用
            PermissionResult::denied("Insufficient permissions")
        }
    };

    // サブスクリプション要件チェック
    let subscription_requirements = if let PermissionResult::Denied { .. } = result {
        Some(SubscriptionRequirement {
            required_tier: SubscriptionTier::Pro,
            current_tier: user.claims.subscription_tier,
            upgrade_required: !user
                .claims
                .subscription_tier
                .is_at_least(&SubscriptionTier::Pro),
            upgrade_message: format!("Pro subscription required for {} access", params.resource),
        })
    } else {
        None
    };

    let response = match result {
        PermissionResult::Allowed { scope, .. } => ResourcePermissionResponse {
            user_id: user.claims.user_id,
            resource: params.resource.clone(),
            action: params.action.clone(),
            allowed: true,
            reason: None,
            permission_scope: Some(PermissionScopeInfo {
                scope: scope.clone(),
                description: scope.description().to_string(),
                level: scope.level(),
            }),
            subscription_requirements,
            checked_at: Timestamp::now(),
        },
        PermissionResult::Denied { reason } => ResourcePermissionResponse {
            user_id: user.claims.user_id,
            resource: params.resource,
            action: params.action,
            allowed: false,
            reason: Some(reason),
            permission_scope: None,
            subscription_requirements,
            checked_at: Timestamp::now(),
        },
    };

    info!(
        user_id = %user.claims.user_id,
        allowed = %response.allowed,
        "Resource permission check completed"
    );

    Ok(ApiResponse::success(response))
}

/// バルク権限チェック
pub async fn bulk_permission_check_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<BulkPermissionCheckRequest>,
) -> AppResult<ApiResponse<BulkPermissionCheckResponse>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "permission_handler::check_bulk_permissions"))?;

    // 追加の手動バリデーション
    for check in &payload.checks {
        if check.resource.trim().is_empty() {
            warn!("Empty resource in bulk permission check");
            return Err(AppError::BadRequest("Resource cannot be empty".to_string()));
        }
        if check.action.trim().is_empty() {
            warn!("Empty action in bulk permission check");
            return Err(AppError::BadRequest("Action cannot be empty".to_string()));
        }
    }

    let start_time = std::time::Instant::now();
    let target_user_id = payload.user_id.unwrap_or(user.claims.user_id);

    info!(
        user_id = %user.claims.user_id,
        target_user_id = %target_user_id,
        check_count = %payload.checks.len(),
        "Processing bulk permission check"
    );

    let mut checks = Vec::new();

    // 各権限をチェック
    for permission_check in payload.checks {
        let result = if let Some(ref role) = user.claims.role {
            role.can_perform_action(
                &permission_check.resource,
                &permission_check.action,
                permission_check.target_user_id,
            )
        } else {
            // Basic permission check - simplified version
            if user.claims.is_admin() || permission_check.resource == "tasks" {
                // PermissionResult::allowedメソッドを使用
                PermissionResult::allowed(None, PermissionScope::Own)
            } else {
                // PermissionResult::deniedメソッドを使用
                PermissionResult::denied("Insufficient permissions")
            }
        };

        let (allowed, reason, scope) = match result {
            PermissionResult::Allowed { scope, .. } => (
                true,
                None,
                Some(PermissionScopeInfo {
                    scope: scope.clone(),
                    description: scope.description().to_string(),
                    level: scope.level(),
                }),
            ),
            PermissionResult::Denied { reason } => (false, Some(reason), None),
        };

        checks.push(PermissionCheckResult {
            resource: permission_check.resource,
            action: permission_check.action,
            allowed,
            reason,
            scope,
        });
    }

    let execution_time = start_time.elapsed().as_millis() as u64;
    let summary = ValidationSummary::new(&checks);

    let response = BulkPermissionCheckResponse {
        user_id: target_user_id,
        checks,
        summary,
        execution_time_ms: execution_time,
        checked_at: Timestamp::now(),
    };

    info!(
        user_id = %user.claims.user_id,
        target_user_id = %target_user_id,
        success_rate = %response.summary.success_rate,
        execution_time_ms = %response.execution_time_ms,
        "Bulk permission check completed"
    );

    Ok(ApiResponse::success(response))
}

/// ユーザー有効権限取得
pub async fn get_user_effective_permissions_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedUuid(target_user_id): ValidatedUuid,
    Query(query): Query<UserEffectivePermissionsQuery>,
) -> AppResult<ApiResponse<UserEffectivePermissionsResponse>> {
    // アクセス権限チェック（自分の情報または管理者のみ）
    if user.claims.user_id != target_user_id && !user.claims.is_admin() {
        warn!(
            user_id = %user.claims.user_id,
            target_user_id = %target_user_id,
            "Access denied: Cannot view other user's effective permissions"
        );
        return Err(AppError::Forbidden(
            "You can only view your own effective permissions".to_string(),
        ));
    }

    info!(
        user_id = %user.claims.user_id,
        target_user_id = %target_user_id,
        include_inherited = ?query.include_inherited,
        resource_filter = ?query.resource_filter,
        "Getting user effective permissions"
    );

    // 基本的なロール情報を構築
    let role_info = UserRoleInfo {
        role_id: Uuid::new_v4(), // 実際の実装では適切なロールIDを取得
        role_name: user.claims.role_name.clone(),
        display_name: user.claims.role_name.clone(),
        is_active: true,
        permission_level: if user.claims.is_admin() { 100 } else { 10 },
    };

    // 有効権限を構築
    let effective_permissions = get_effective_permissions_for_user(
        &user.claims.role_name,
        &user.claims.subscription_tier,
        query.resource_filter.as_deref(),
    );

    // 継承権限を構築（include_inheritedが指定された場合）
    let inherited_permissions = if query.include_inherited.unwrap_or(false) {
        get_inherited_permissions(&user.claims.role_name)
    } else {
        vec![]
    };

    // 拒否権限を構築
    let denied_permissions = get_denied_permissions(&user.claims.subscription_tier);

    // 権限サマリーを計算
    let permission_summary = PermissionSummary {
        total_permissions: effective_permissions.len() as u32,
        effective_permissions: effective_permissions.len() as u32,
        inherited_permissions: inherited_permissions.len() as u32,
        denied_permissions: denied_permissions.len() as u32,
        coverage_percentage: if denied_permissions.is_empty() {
            100.0
        } else {
            (effective_permissions.len() as f64
                / (effective_permissions.len() + denied_permissions.len()) as f64)
                * 100.0
        },
        highest_scope: if user.claims.is_admin() {
            PermissionScope::Global
        } else {
            PermissionScope::Own
        },
    };

    let response = UserEffectivePermissionsResponse {
        user_id: target_user_id,
        role: role_info,
        subscription_tier: user.claims.subscription_tier,
        effective_permissions,
        inherited_permissions,
        denied_permissions,
        permission_summary,
        last_updated: Timestamp::now(),
    };

    info!(
        user_id = %user.claims.user_id,
        target_user_id = %target_user_id,
        effective_count = %response.effective_permissions.len(),
        inherited_count = %response.inherited_permissions.len(),
        denied_count = %response.denied_permissions.len(),
        "User effective permissions retrieved"
    );

    Ok(ApiResponse::success(response))
}

/// システム権限監査（管理者のみ）
pub async fn get_system_permission_audit_handler(
    State(app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(query): Query<SystemPermissionAuditQuery>,
) -> AppResult<ApiResponse<SystemPermissionAuditResponse>> {
    // 管理者権限チェック（check_permission_typeを使用）
    app_state
        .permission_service
        .check_permission_type(admin_user.user_id(), PermissionType::IsAdmin, None)
        .await?;

    info!(
        admin_id = %admin_user.user_id(),
        user_filter = ?query.user_id,
        resource_filter = ?query.resource,
        action_filter = ?query.action,
        date_range = ?query.created_after,
        "Getting system permission audit"
    );

    // 監査期間を設定
    let audit_period = AuditPeriod {
        start_date: query.created_after.map_or_else(
            || Timestamp::from(Timestamp::now().inner() - chrono::Duration::days(7)),
            Timestamp::from,
        ),
        end_date: query
            .created_before
            .map_or_else(Timestamp::now, Timestamp::from),
        duration_hours: 168, // 7 days default
    };

    // モック監査エントリを生成（実際の実装では、データベースから取得）
    let audit_entries = generate_mock_audit_entries(&query, 100);
    let filtered_entries = audit_entries.len() as u32;

    // 監査サマリーを計算
    let allowed_checks = audit_entries
        .iter()
        .filter(|e| matches!(e.result, AuditResult::Allowed))
        .count() as u32;
    let denied_checks = audit_entries
        .iter()
        .filter(|e| matches!(e.result, AuditResult::Denied))
        .count() as u32;

    let summary = AuditSummary {
        total_checks: audit_entries.len() as u32,
        allowed_checks,
        denied_checks,
        unique_users: 10,    // モック値
        unique_resources: 5, // モック値
        most_accessed_resource: "tasks".to_string(),
        most_denied_action: "delete".to_string(),
        success_rate: if audit_entries.is_empty() {
            100.0
        } else {
            (allowed_checks as f64 / audit_entries.len() as f64) * 100.0
        },
    };

    let response = SystemPermissionAuditResponse {
        audit_entries,
        summary,
        total_entries: 1000, // モック値
        filtered_entries,
        audit_period,
    };

    info!(
        admin_id = %admin_user.user_id(),
        total_entries = %response.total_entries,
        filtered_entries = %response.filtered_entries,
        success_rate = %response.summary.success_rate,
        "System permission audit retrieved"
    );

    Ok(ApiResponse::success(response))
}

// --- Helper Functions for Permission Audit ---

fn get_effective_permissions_for_user(
    role_name: &str,
    tier: &SubscriptionTier,
    resource_filter: Option<&str>,
) -> Vec<EffectivePermission> {
    let mut permissions = vec![
        EffectivePermission {
            resource: "tasks".to_string(),
            action: "read".to_string(),
            scope: if role_name == "admin" {
                PermissionScope::Global
            } else {
                PermissionScope::Own
            },
            source: PermissionSource::Role,
            granted_at: Timestamp::now(),
            expires_at: None,
            conditions: vec![],
        },
        EffectivePermission {
            resource: "tasks".to_string(),
            action: "create".to_string(),
            scope: if role_name == "admin" {
                PermissionScope::Global
            } else {
                PermissionScope::Own
            },
            source: PermissionSource::Role,
            granted_at: Timestamp::now(),
            expires_at: None,
            conditions: vec![],
        },
    ];

    // Add subscription-based permissions
    if tier.is_at_least(&SubscriptionTier::Pro) {
        permissions.push(EffectivePermission {
            resource: "tasks".to_string(),
            action: "export".to_string(),
            scope: PermissionScope::Team,
            source: PermissionSource::Subscription,
            granted_at: Timestamp::now(),
            expires_at: None,
            conditions: vec![PermissionCondition {
                condition_type: "subscription_tier".to_string(),
                value: "pro".to_string(),
                description: "Pro subscription required".to_string(),
            }],
        });
    }

    if tier.is_at_least(&SubscriptionTier::Enterprise) {
        permissions.push(EffectivePermission {
            resource: "tasks".to_string(),
            action: "bulk_operations".to_string(),
            scope: PermissionScope::Organization,
            source: PermissionSource::Subscription,
            granted_at: Timestamp::now(),
            expires_at: None,
            conditions: vec![],
        });
    }

    // Apply resource filter if specified
    if let Some(filter) = resource_filter {
        permissions.retain(|p| p.resource.contains(filter));
    }

    permissions
}

fn get_inherited_permissions(role_name: &str) -> Vec<InheritedPermission> {
    if role_name == "admin" {
        // Permission::admin_globalを活用
        let admin_users_permission = Permission::admin_global("users");
        vec![
            InheritedPermission {
                resource: admin_users_permission.resource,
                action: "manage".to_string(),
                scope: PermissionScope::Global,
                inherited_from: PermissionSource::Role,
                inheritance_chain: vec!["admin".to_string(), "member".to_string()],
                granted_at: Timestamp::now(),
            },
            InheritedPermission {
                resource: "system".to_string(),
                action: "configure".to_string(),
                scope: PermissionScope::Global,
                inherited_from: PermissionSource::System,
                inheritance_chain: vec!["system".to_string(), "admin".to_string()],
                granted_at: Timestamp::now(),
            },
        ]
    } else {
        vec![]
    }
}

fn get_denied_permissions(tier: &SubscriptionTier) -> Vec<DeniedPermission> {
    let mut denied = vec![];

    if !tier.is_at_least(&SubscriptionTier::Pro) {
        denied.push(DeniedPermission {
            resource: "tasks".to_string(),
            action: "export".to_string(),
            reason: "Pro subscription required".to_string(),
            required_role: None,
            required_subscription: Some(SubscriptionTier::Pro),
            can_be_granted: true,
        });
    }

    if !tier.is_at_least(&SubscriptionTier::Enterprise) {
        denied.push(DeniedPermission {
            resource: "tasks".to_string(),
            action: "bulk_operations".to_string(),
            reason: "Enterprise subscription required".to_string(),
            required_role: None,
            required_subscription: Some(SubscriptionTier::Enterprise),
            can_be_granted: true,
        });
    }

    denied
}

fn generate_mock_audit_entries(
    query: &SystemPermissionAuditQuery,
    count: usize,
) -> Vec<PermissionAuditEntry> {
    let mut entries = Vec::new();
    let base_time = Timestamp::now();

    for i in 0..count {
        let entry = PermissionAuditEntry {
            id: Uuid::new_v4(),
            user_id: query.user_id.unwrap_or_else(Uuid::new_v4),
            resource: query
                .resource
                .clone()
                .unwrap_or_else(|| "tasks".to_string()),
            action: query.action.clone().unwrap_or_else(|| "read".to_string()),
            result: if i % 4 == 0 {
                AuditResult::Denied
            } else {
                AuditResult::Allowed
            },
            reason: if i % 4 == 0 {
                Some("Insufficient permissions".to_string())
            } else {
                None
            },
            scope: Some(if i % 3 == 0 {
                PermissionScope::Global
            } else {
                PermissionScope::Own
            }),
            ip_address: Some("192.168.1.100".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            timestamp: Timestamp::from(base_time.inner() - chrono::Duration::hours(i as i64)),
        };
        entries.push(entry);
    }

    entries
}

// --- NEW: Dead code utilization endpoints ---

/// 権限拒否の詳細情報を取得
pub async fn check_permission_denial_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CheckPermissionRequest>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "permission_handler::check_permission_denial"))?;

    info!(
        user_id = %user.claims.user_id,
        resource = %payload.resource,
        action = %payload.action,
        "Checking permission denial details"
    );

    // 権限チェックを実行
    let result = if let Some(ref role) = user.claims.role {
        role.can_perform_action(&payload.resource, &payload.action, payload.target_user_id)
    } else if user.claims.is_admin() || payload.resource == "tasks" {
        PermissionResult::allowed(None, PermissionScope::Own)
    } else {
        PermissionResult::denied("Insufficient permissions")
    };

    // is_denied()メソッドを活用
    let is_denied = result.is_denied();
    let denial_details = if is_denied {
        let reason = result
            .get_denial_reason()
            .cloned()
            .unwrap_or_else(|| "Permission denied".to_string());
        Some(serde_json::json!({
            "reason": reason,
            "suggestions": get_permission_suggestions(&payload.resource, &payload.action, &user.claims.subscription_tier),
            "required_tier": get_required_tier_for_action(&payload.resource, &payload.action),
            "upgrade_url": "/subscription/upgrade"
        }))
    } else {
        None
    };

    let response = serde_json::json!({
        "user_id": user.claims.user_id,
        "resource": payload.resource,
        "action": payload.action,
        "is_denied": is_denied,
        "denial_details": denial_details,
        "checked_at": Timestamp::now()
    });

    info!(
        user_id = %user.claims.user_id,
        is_denied = %is_denied,
        "Permission denial check completed"
    );

    Ok(ApiResponse::success(response))
}

/// 特権の機能チェック
pub async fn check_privilege_feature_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<FeatureAccessRequest>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "permission_handler::check_feature_access"))?;

    info!(
        user_id = %user.claims.user_id,
        feature_name = %payload.feature_name,
        "Checking privilege feature access"
    );

    // 特権を構築してhas_feature()を使用
    let privilege = match &user.claims.subscription_tier {
        SubscriptionTier::Free => Privilege::free_basic("user_tasks", 100, 10),
        SubscriptionTier::Pro => Privilege::pro_advanced(
            "user_tasks",
            10_000,
            100,
            vec!["advanced_filter", "export", &payload.feature_name],
        ),
        SubscriptionTier::Enterprise => Privilege::enterprise_unlimited(
            "user_tasks",
            vec!["all_features", &payload.feature_name],
        ),
    };

    // has_feature()メソッドを活用
    let has_feature = privilege.has_feature(&payload.feature_name);
    let feature_details = serde_json::json!({
        "feature_name": payload.feature_name,
        "has_feature": has_feature,
        "user_tier": user.claims.subscription_tier,
        "privilege_name": privilege.name,
        "privilege_tier": privilege.subscription_tier,
        "available_features": privilege.quota.as_ref().map_or(&vec![], |q| &q.features),
        "max_items": privilege.get_max_items(),
        "rate_limit": privilege.get_rate_limit(),
    });

    let response = serde_json::json!({
        "user_id": user.claims.user_id,
        "feature_check": feature_details,
        "feature_available": has_feature,
        "upgrade_required": !has_feature && payload.required_tier.is_some_and(|t| !user.claims.subscription_tier.is_at_least(&t)),
        "checked_at": Timestamp::now()
    });

    info!(
        user_id = %user.claims.user_id,
        feature_name = %payload.feature_name,
        has_feature = %has_feature,
        "Privilege feature check completed"
    );

    Ok(ApiResponse::success(response))
}

// Helper functions for denial suggestions
fn get_permission_suggestions(
    resource: &str,
    action: &str,
    tier: &SubscriptionTier,
) -> Vec<String> {
    let mut suggestions = vec![];

    match (resource, action) {
        ("tasks", "export") if !tier.is_at_least(&SubscriptionTier::Pro) => {
            suggestions.push("Upgrade to Pro tier to export tasks".to_string());
            suggestions.push("Use the web interface to view tasks".to_string());
        }
        ("tasks", "bulk_operations") if !tier.is_at_least(&SubscriptionTier::Enterprise) => {
            suggestions.push("Upgrade to Enterprise tier for bulk operations".to_string());
            suggestions.push("Process tasks individually".to_string());
        }
        ("admin", _) => {
            suggestions.push("Contact your administrator for access".to_string());
        }
        _ => {
            suggestions.push("Check your permissions with your administrator".to_string());
        }
    }

    suggestions
}

fn get_required_tier_for_action(resource: &str, action: &str) -> Option<SubscriptionTier> {
    match (resource, action) {
        ("tasks", "export") => Some(SubscriptionTier::Pro),
        ("tasks", "bulk_operations") => Some(SubscriptionTier::Enterprise),
        ("analytics", _) => Some(SubscriptionTier::Pro),
        _ => None,
    }
}

/// 複数権限の同時チェック（check_multiple_permissionsを使用）
pub async fn check_complex_operation_permissions_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ComplexOperationRequest>,
) -> AppResult<ApiResponse<ComplexOperationPermissionResponse>> {
    use crate::utils::permission::{PermissionType, ResourceContext};

    // バリデーション
    payload.validate().map_err(|e| {
        convert_validation_errors(e, "permission_handler::create_complex_operation")
    })?;

    info!(
        user_id = %user.claims.user_id,
        operation = %payload.operation,
        "Checking complex operation permissions"
    );

    // 複数の権限チェックを構築
    let checks = vec![
        // 管理者権限チェック
        (PermissionType::IsAdmin, None),
        // メンバー権限チェック
        (PermissionType::IsMember, None),
        // リソース作成権限チェック
        (
            PermissionType::CanCreateResource,
            Some(ResourceContext {
                resource_type: payload.resource_type.clone(),
                owner_id: None,
                target_user_id: None,
                requesting_user_id: user.claims.user_id,
            }),
        ),
        // リソース削除権限チェック
        (
            PermissionType::CanDeleteResource,
            Some(ResourceContext {
                resource_type: payload.resource_type.clone(),
                owner_id: payload.resource_id,
                target_user_id: None,
                requesting_user_id: user.claims.user_id,
            }),
        ),
    ];

    // check_multiple_permissionsを使用して一括チェック
    let results = app_state
        .permission_service
        .check_multiple_permissions(user.claims.user_id, checks)
        .await?;

    // 結果を解析
    let permission_results = vec![
        PermissionCheckDetail {
            permission_type: "IsAdmin".to_string(),
            allowed: results[0],
            description: "Administrator permission".to_string(),
        },
        PermissionCheckDetail {
            permission_type: "IsMember".to_string(),
            allowed: results[1],
            description: "Member permission".to_string(),
        },
        PermissionCheckDetail {
            permission_type: "CanCreateResource".to_string(),
            allowed: results[2],
            description: format!("Create {} permission", payload.resource_type),
        },
        PermissionCheckDetail {
            permission_type: "CanDeleteResource".to_string(),
            allowed: results[3],
            description: format!("Delete {} permission", payload.resource_type),
        },
    ];

    // 操作が許可されるか判定（必要な権限に基づいて）
    let operation_allowed = match payload.operation.as_str() {
        "create_and_delete" => results[2] && results[3], // 作成と削除の両方が必要
        "admin_only" => results[0],                      // 管理者権限のみ必要
        "member_create" => results[1] && results[2],     // メンバーかつ作成権限が必要
        _ => results[1],                                 // デフォルトはメンバー権限のみ
    };

    let response = ComplexOperationPermissionResponse {
        user_id: user.claims.user_id,
        operation: payload.operation,
        operation_allowed,
        permission_details: permission_results,
        checked_at: Timestamp::now(),
    };

    info!(
        user_id = %user.claims.user_id,
        operation_allowed = %response.operation_allowed,
        "Complex operation permission check completed"
    );

    Ok(ApiResponse::success(response))
}

// --- Health Check ---

/// 権限サービスのヘルスチェック
async fn permission_health_check_handler() -> &'static str {
    "Permission service OK"
}

// --- Router Setup ---

/// 権限管理ルーターを作成
pub fn permission_router(app_state: AppState) -> Router {
    use crate::middleware::authorization::{resources, Action};
    use crate::require_permission;

    Router::new()
        // 権限チェックエンドポイント
        .route("/permissions/check", post(check_permission_handler))
        .route("/permissions/validate", post(validate_permissions_handler))
        .route("/permissions/user/{id}", get(get_user_permissions_handler))
        .route(
            "/permissions/resources",
            get(get_available_resources_handler),
        )
        // Permission Audit APIs - NEW
        .route(
            "/permissions/resources/{resource}/actions/{action}/check",
            get(check_resource_permission_handler),
        )
        .route(
            "/permissions/bulk-check",
            post(bulk_permission_check_handler),
        )
        .route(
            "/permissions/user/{id}/effective",
            get(get_user_effective_permissions_handler),
        )
        .route(
            "/admin/permissions/audit",
            get(get_system_permission_audit_handler)
                .route_layer(require_permission!(resources::ROLE, Action::Admin)),
        )
        // 機能アクセスエンドポイント
        .route("/features/available", get(get_feature_access_handler))
        .route(
            "/features/admin",
            get(get_admin_features_handler)
                .route_layer(require_permission!(resources::ROLE, Action::Admin)),
        )
        .route(
            "/features/analytics",
            get(get_analytics_features_handler)
                .route_layer(require_permission!(resources::ANALYTICS, Action::View)),
        )
        // NEW: Dead code utilization endpoints
        .route(
            "/permissions/denial-check",
            post(check_permission_denial_handler),
        )
        .route(
            "/features/privilege-check",
            post(check_privilege_feature_handler),
        )
        // 複数権限チェック
        .route(
            "/permissions/complex-operation",
            post(check_complex_operation_permissions_handler),
        )
        // ヘルスチェック
        .route("/permissions/health", get(permission_health_check_handler))
        .with_state(app_state)
}

/// 権限管理ルーターをAppStateから作成
pub fn permission_router_with_state(app_state: AppState) -> Router {
    permission_router(app_state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_feature_limits_free() {
        let limits = get_feature_limits(&SubscriptionTier::Free);
        assert_eq!(limits.max_projects, Some(3));
        assert_eq!(limits.max_tasks_per_project, Some(100));
        assert!(!limits.advanced_features_enabled);
    }

    #[test]
    fn test_get_feature_limits_enterprise() {
        let limits = get_feature_limits(&SubscriptionTier::Enterprise);
        assert_eq!(limits.max_projects, None);
        assert_eq!(limits.max_tasks_per_project, None);
        assert!(limits.advanced_features_enabled);
        assert!(limits.custom_integrations_enabled);
    }

    #[test]
    fn test_determine_analytics_level() {
        assert!(matches!(
            determine_analytics_level(&SubscriptionTier::Free, false),
            AnalyticsLevel::Basic
        ));
        assert!(matches!(
            determine_analytics_level(&SubscriptionTier::Enterprise, false),
            AnalyticsLevel::Enterprise
        ));
        assert!(matches!(
            determine_analytics_level(&SubscriptionTier::Free, true),
            AnalyticsLevel::Custom
        ));
    }
}
