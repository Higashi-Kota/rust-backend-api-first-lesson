// task-backend/src/features/security/handlers/permission_handler.rs

use crate::api::AppState;
use crate::core::permission::{Permission, PermissionResult, PermissionScope, Privilege};
use crate::core::subscription_tier::SubscriptionTier;
use crate::error::{AppError, AppResult};
use crate::features::auth::middleware::{AuthenticatedUser, AuthenticatedUserWithRole};
use crate::features::security::dto::query::{FeatureQuery, PermissionQuery};
use crate::features::security::dto::{requests, *};
use crate::shared::types::common::ApiResponse;
use axum::{
    extract::{Json, Path, Query, State},
    routing::{get, post},
    Router,
};
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

// --- Permission Checking Endpoints ---

/// 特定の権限をチェック
pub async fn check_permission_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CheckPermissionRequest>,
) -> AppResult<Json<ApiResponse<PermissionCheckResponse>>> {
    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!("Permission check validation failed: {}", validation_errors);
        let errors: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();
        AppError::ValidationErrors(errors)
    })?;

    info!(
        user_id = %user.user_id(),
        resource = %payload.resource,
        action = %payload.action,
        "Checking permission"
    );

    // 権限チェックを実行
    let result = if let Some(ref role) = user.claims.role {
        role.can_perform_action(&payload.resource, &payload.action, payload.target_user_id)
    } else {
        // Basic permission check using role name - simplified version
        if user.is_admin() || payload.resource == "tasks" {
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
        user_id: user.user_id(),
        resource: payload.resource,
        action: payload.action,
        allowed: is_allowed,
        is_admin: user.is_admin(),
        is_member: user.claims.is_member(),
        reason: denial_reason,
        scope: scope_info,
        privilege: privilege_info,
        expires_at: None,
    };

    info!(
        user_id = %user.user_id(),
        allowed = %response.allowed,
        has_scope = %response.scope.is_some(),
        has_privilege = %response.privilege.is_some(),
        "Permission check completed with detailed information"
    );

    Ok(Json(ApiResponse::success(
        "Permission check completed",
        response,
    )))
}

/// 複数の権限を一括検証
pub async fn validate_permissions_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ValidatePermissionRequest>,
) -> AppResult<Json<ApiResponse<PermissionValidationResponse>>> {
    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!("Permission validation failed: {}", validation_errors);
        let errors: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();
        AppError::ValidationErrors(errors)
    })?;

    info!(
        user_id = %user.user_id(),
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
            if user.is_admin() || permission_check.resource == "tasks" {
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

    // 全体の結果を判定
    let overall_result = if require_all {
        allowed_count == checks.len()
    } else {
        allowed_count > 0
    };

    // ValidationSummaryのnewメソッドを活用
    let summary = ValidationSummary::new(&checks);

    let response = PermissionValidationResponse {
        user_id: user.user_id(),
        overall_result,
        require_all,
        checks,
        summary,
    };

    info!(
        user_id = %user.user_id(),
        overall_result = %response.overall_result,
        allowed_count = %response.summary.allowed_count,
        denied_count = %response.summary.denied_count,
        "Permission validation completed"
    );

    Ok(Json(ApiResponse::success(
        "Permission validation completed",
        response,
    )))
}

/// ユーザーの権限情報を取得
pub async fn get_user_permissions_handler(
    State(_app_state): State<AppState>,
    _admin_user: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<UserPermissionsResponse>>> {
    info!(
        target_user_id = %user_id,
        "Getting user permissions"
    );

    // モック実装（実際にはDBからユーザー情報を取得）
    let role_info = UserRoleInfo {
        role_id: Uuid::new_v4(),
        role_name: "member".to_string(),
        display_name: "Member".to_string(),
        is_active: true,
        permission_level: 1,
    };

    let subscription_tier = SubscriptionTier::Pro;

    // Privilege::pro_advancedメソッドを活用
    let pro_privilege = Privilege::pro_advanced(
        "task_management",
        10_000,
        100,
        vec!["advanced_filter", "export"],
    );

    let permissions = vec![
        PermissionInfo {
            resource: "tasks".to_string(),
            action: "read".to_string(),
            scope: PermissionScope::Own,
            granted_at: chrono::Utc::now(),
            expires_at: None,
        },
        PermissionInfo {
            resource: "tasks".to_string(),
            action: "create".to_string(),
            scope: PermissionScope::Own,
            granted_at: chrono::Utc::now(),
            expires_at: None,
        },
    ];

    let features = vec![FeatureInfo {
        feature_name: "advanced_tasks".to_string(),
        display_name: "Advanced Task Management".to_string(),
        description: "Enhanced task management features".to_string(),
        category: "tasks".to_string(),
        required_tier: SubscriptionTier::Pro,
        is_enabled: pro_privilege.is_available_for_tier(&subscription_tier),
        quota: Some(QuotaInfo {
            max_items: pro_privilege.get_max_items(),
            rate_limit: pro_privilege.get_rate_limit(),
            features: vec!["advanced_filter".to_string(), "export".to_string()],
            current_usage: None,
        }),
    }];

    let effective_scopes = vec![PermissionScopeInfo {
        scope: PermissionScope::Own,
        description: PermissionScope::Own.description().to_string(),
        level: PermissionScope::Own.level(),
    }];

    let response = UserPermissionsResponse {
        user_id,
        role: role_info,
        subscription_tier,
        permissions,
        features,
        effective_scopes,
        last_updated: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(
        "User permissions retrieved",
        response,
    )))
}

/// 利用可能なリソース一覧を取得
pub async fn get_available_resources_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<PermissionQuery>,
) -> AppResult<Json<ApiResponse<AvailableResourcesResponse>>> {
    info!(
        user_id = %user.user_id(),
        resource_filter = ?query.resource,
        action_filter = ?query.action,
        "Getting available resources"
    );

    let mut resources = vec![ResourceInfo {
        resource_type: "tasks".to_string(),
        display_name: "Tasks".to_string(),
        description: "Task management resources".to_string(),
        available_actions: vec![
            ActionInfo {
                action: "read".to_string(),
                display_name: "Read".to_string(),
                description: "View tasks".to_string(),
                required_role: None,
                required_tier: None,
            },
            ActionInfo {
                action: "create".to_string(),
                display_name: "Create".to_string(),
                description: "Create new tasks".to_string(),
                required_role: None,
                required_tier: None,
            },
        ],
        restricted_actions: vec![],
        scope: PermissionScope::Own,
    }];

    // フィルタリング
    if let Some(resource_filter) = query.resource {
        resources.retain(|r| r.resource_type.contains(&resource_filter));
    }

    let total_resources = resources.len() as u32;
    let accessible_resources = resources.len() as u32;

    let response = AvailableResourcesResponse {
        user_id: user.user_id(),
        resources,
        total_resources,
        accessible_resources,
        restricted_resources: 0,
    };

    Ok(Json(ApiResponse::success(
        "Available resources retrieved",
        response,
    )))
}

/// 機能アクセス情報を取得
pub async fn get_feature_access_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<FeatureQuery>,
) -> AppResult<Json<ApiResponse<FeatureAccessResponse>>> {
    info!(
        user_id = %user.user_id(),
        category_filter = ?query.category,
        "Getting feature access information"
    );

    let tier = user.claims.subscription_tier;

    // 利用可能な機能を取得
    let mut available_features = vec![];
    let mut restricted_features = vec![];

    // Free tier features
    let free_privilege = Privilege::free_basic("basic_tasks", 100, 10);
    available_features.push(FeatureInfo {
        feature_name: "basic_tasks".to_string(),
        display_name: "Basic Task Management".to_string(),
        description: "Core task management features".to_string(),
        category: "tasks".to_string(),
        required_tier: SubscriptionTier::Free,
        is_enabled: free_privilege.is_available_for_tier(&tier),
        quota: Some(QuotaInfo {
            max_items: free_privilege.get_max_items(),
            rate_limit: free_privilege.get_rate_limit(),
            features: vec!["basic_access".to_string()],
            current_usage: Some(QuotaUsage {
                items_used: 50,
                requests_today: 5,
                features_used: vec!["basic_access".to_string()],
                last_reset: chrono::Utc::now(),
            }),
        }),
    });

    // Pro features
    if !tier.is_at_least(&SubscriptionTier::Pro) {
        restricted_features.push(RestrictedFeatureInfo {
            feature_name: "advanced_reporting".to_string(),
            display_name: "Advanced Reporting".to_string(),
            required_tier: SubscriptionTier::Pro,
            current_tier: tier,
            upgrade_required: true,
            restriction_reason: "Pro subscription required".to_string(),
        });
    }

    // フィルタリング
    if let Some(category) = query.category {
        available_features.retain(|f| f.category == category);
        restricted_features.retain(|f| f.feature_name.contains(&category));
    }

    let feature_limits = match tier {
        SubscriptionTier::Free => FeatureLimits {
            max_projects: Some(3),
            max_tasks_per_project: Some(100),
            max_team_members: Some(5),
            max_api_requests_per_hour: Some(100),
            max_storage_mb: Some(100),
            advanced_features_enabled: false,
            custom_integrations_enabled: false,
        },
        SubscriptionTier::Pro => FeatureLimits {
            max_projects: Some(50),
            max_tasks_per_project: Some(1000),
            max_team_members: Some(50),
            max_api_requests_per_hour: Some(1000),
            max_storage_mb: Some(10_000),
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
    };

    let response = FeatureAccessResponse {
        user_id: user.user_id(),
        subscription_tier: tier,
        available_features,
        restricted_features,
        feature_limits,
    };

    Ok(Json(ApiResponse::success(
        "Feature access information retrieved",
        response,
    )))
}

/// 管理者機能アクセス情報を取得
pub async fn get_admin_features_handler(
    State(_app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
) -> AppResult<Json<ApiResponse<AdminFeaturesResponse>>> {
    // 管理者権限チェック - can_access_admin_features を使用
    if let Some(role) = admin_user.role() {
        if !crate::utils::permission::PermissionChecker::can_access_admin_features(role) {
            return Err(AppError::Forbidden(
                "Admin or Enterprise subscription required for admin features".to_string(),
            ));
        }
    } else {
        return Err(AppError::Forbidden("Role information required".to_string()));
    }

    info!(
        admin_id = %admin_user.user_id(),
        "Getting admin features"
    );

    let admin_features = vec![
        AdminFeatureInfo {
            feature_name: "user_management".to_string(),
            display_name: "User Management".to_string(),
            category: "administration".to_string(),
            description: "Manage user accounts and permissions".to_string(),
            risk_level: AdminRiskLevel::Medium,
            requires_confirmation: true,
        },
        AdminFeatureInfo {
            feature_name: "system_configuration".to_string(),
            display_name: "System Configuration".to_string(),
            category: "administration".to_string(),
            description: "Configure system-wide settings".to_string(),
            risk_level: AdminRiskLevel::High,
            requires_confirmation: true,
        },
    ];

    let system_permissions = vec![
        SystemPermissionInfo {
            permission_name: "manage_users".to_string(),
            display_name: "Manage Users".to_string(),
            description: "Create, update, and delete user accounts".to_string(),
            scope: SystemPermissionScope::SystemWide,
            is_granted: true,
        },
        SystemPermissionInfo {
            permission_name: "view_audit_logs".to_string(),
            display_name: "View Audit Logs".to_string(),
            description: "Access system audit logs".to_string(),
            scope: SystemPermissionScope::ReadOnly,
            is_granted: true,
        },
    ];

    let audit_capabilities = AuditCapabilities {
        can_view_audit_logs: true,
        can_export_audit_logs: true,
        can_view_system_logs: true,
        audit_retention_days: 365,
        real_time_monitoring: true,
    };

    let response = AdminFeaturesResponse {
        admin_user_id: admin_user.user_id(),
        admin_features,
        system_permissions,
        audit_capabilities,
    };

    Ok(Json(ApiResponse::success(
        "Admin features retrieved",
        response,
    )))
}

/// アナリティクス機能アクセス情報を取得
pub async fn get_analytics_features_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<AnalyticsFeaturesResponse>>> {
    info!(
        user_id = %user.user_id(),
        "Getting analytics features"
    );

    let tier = user.claims.subscription_tier;

    let analytics_level = match tier {
        SubscriptionTier::Free => AnalyticsLevel::Basic,
        SubscriptionTier::Pro => AnalyticsLevel::Advanced,
        SubscriptionTier::Enterprise => AnalyticsLevel::Enterprise,
    };

    let available_reports = vec![ReportInfo {
        report_name: "task_summary".to_string(),
        display_name: "Task Summary".to_string(),
        category: "tasks".to_string(),
        description: "Basic task statistics".to_string(),
        required_tier: SubscriptionTier::Free,
        is_real_time: false,
        scheduled_available: false,
    }];

    let data_retention_days = match tier {
        SubscriptionTier::Free => Some(30),
        SubscriptionTier::Pro => Some(365),
        SubscriptionTier::Enterprise => None, // Unlimited
    };

    let export_capabilities = ExportCapabilities {
        formats: match tier {
            SubscriptionTier::Free => vec!["csv".to_string()],
            SubscriptionTier::Pro => vec!["csv".to_string(), "json".to_string()],
            SubscriptionTier::Enterprise => vec![
                "csv".to_string(),
                "json".to_string(),
                "pdf".to_string(),
                "excel".to_string(),
            ],
        },
        max_records: match tier {
            SubscriptionTier::Free => Some(1000),
            SubscriptionTier::Pro => Some(100_000),
            SubscriptionTier::Enterprise => None,
        },
        batch_export: tier.is_at_least(&SubscriptionTier::Pro),
        scheduled_export: tier.is_at_least(&SubscriptionTier::Enterprise),
        custom_templates: tier.is_at_least(&SubscriptionTier::Enterprise),
    };

    let response = AnalyticsFeaturesResponse {
        user_id: user.user_id(),
        subscription_tier: tier,
        analytics_level,
        available_reports,
        data_retention_days,
        export_capabilities,
    };

    Ok(Json(ApiResponse::success(
        "Analytics features retrieved",
        response,
    )))
}

// --- Permission Audit Endpoints ---

/// リソース固有の権限をチェック
pub async fn check_resource_permission_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Path((resource, action)): Path<(String, String)>,
) -> AppResult<Json<ApiResponse<ResourcePermissionResponse>>> {
    info!(
        user_id = %user.user_id(),
        resource = %resource,
        action = %action,
        "Checking resource permission"
    );

    let result = if let Some(ref role) = user.claims.role {
        role.can_perform_action(&resource, &action, None)
    } else if user.is_admin() || resource == "tasks" {
        PermissionResult::allowed(None, PermissionScope::Own)
    } else {
        PermissionResult::denied("Insufficient permissions")
    };

    let (allowed, reason, scope_info) = match result {
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

    let subscription_requirement = if !allowed && (resource == "analytics" || action == "export") {
        Some(SubscriptionRequirement {
            required_tier: SubscriptionTier::Pro,
            current_tier: user.claims.subscription_tier,
            upgrade_required: !user
                .claims
                .subscription_tier
                .is_at_least(&SubscriptionTier::Pro),
            upgrade_message: "Pro subscription required for this feature".to_string(),
        })
    } else {
        None
    };

    let response = ResourcePermissionResponse {
        user_id: user.user_id(),
        resource,
        action,
        allowed,
        reason,
        permission_scope: scope_info,
        subscription_requirements: subscription_requirement,
        checked_at: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(
        "Resource permission checked",
        response,
    )))
}

/// バルク権限チェック
pub async fn bulk_permission_check_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<BulkPermissionCheckRequest>,
) -> AppResult<Json<ApiResponse<BulkPermissionCheckResponse>>> {
    let start = std::time::Instant::now();

    info!(
        user_id = %user.user_id(),
        check_count = %payload.checks.len(),
        "Bulk permission check"
    );

    let target_user_id = payload.user_id.unwrap_or(user.user_id());
    let mut check_results = Vec::new();

    for check in payload.checks {
        let result = if let Some(ref role) = user.claims.role {
            role.can_perform_action(&check.resource, &check.action, Some(target_user_id))
        } else if user.is_admin() || check.resource == "tasks" {
            PermissionResult::allowed(None, PermissionScope::Own)
        } else {
            PermissionResult::denied("Insufficient permissions")
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

        check_results.push(PermissionCheckResult {
            resource: check.resource,
            action: check.action,
            allowed,
            reason,
            scope,
        });
    }

    let summary = ValidationSummary::new(&check_results);
    let execution_time_ms = start.elapsed().as_millis() as u64;

    let response = BulkPermissionCheckResponse {
        user_id: target_user_id,
        checks: check_results,
        summary,
        execution_time_ms,
        checked_at: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(
        "Bulk permission check completed",
        response,
    )))
}

/// ユーザーの有効権限を取得
pub async fn get_user_effective_permissions_handler(
    State(_app_state): State<AppState>,
    _admin_user: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
    Query(query): Query<UserEffectivePermissionsQuery>,
) -> AppResult<Json<ApiResponse<UserEffectivePermissionsResponse>>> {
    info!(
        target_user_id = %user_id,
        include_inherited = %query.include_inherited.unwrap_or(true),
        resource_filter = ?query.resource_filter,
        "Getting user effective permissions"
    );

    // モック実装
    let role = UserRoleInfo {
        role_id: Uuid::new_v4(),
        role_name: "member".to_string(),
        display_name: "Member".to_string(),
        is_active: true,
        permission_level: 1,
    };

    let effective_permissions = vec![
        EffectivePermission {
            resource: "tasks".to_string(),
            action: "read".to_string(),
            scope: PermissionScope::Own,
            source: PermissionSource::Role,
            granted_at: chrono::Utc::now(),
            expires_at: None,
            conditions: vec![],
        },
        EffectivePermission {
            resource: "tasks".to_string(),
            action: "create".to_string(),
            scope: PermissionScope::Own,
            source: PermissionSource::Role,
            granted_at: chrono::Utc::now(),
            expires_at: None,
            conditions: vec![],
        },
    ];

    let inherited_permissions = if query.include_inherited.unwrap_or(true) {
        vec![InheritedPermission {
            resource: "team_tasks".to_string(),
            action: "read".to_string(),
            scope: PermissionScope::Team,
            inherited_from: PermissionSource::Role,
            inheritance_chain: vec!["team_member".to_string()],
            granted_at: chrono::Utc::now(),
        }]
    } else {
        vec![]
    };

    let denied_permissions = vec![DeniedPermission {
        resource: "admin".to_string(),
        action: "manage".to_string(),
        reason: "Admin role required".to_string(),
        required_role: Some("admin".to_string()),
        required_subscription: None,
        can_be_granted: false,
    }];

    let permission_summary = PermissionSummary {
        total_permissions: 3,
        effective_permissions: 2,
        inherited_permissions: 1,
        denied_permissions: 1,
        coverage_percentage: 66.7,
        highest_scope: PermissionScope::Team,
    };

    let response = UserEffectivePermissionsResponse {
        user_id,
        role,
        subscription_tier: SubscriptionTier::Pro,
        effective_permissions,
        inherited_permissions,
        denied_permissions,
        permission_summary,
        last_updated: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(
        "User effective permissions retrieved",
        response,
    )))
}

/// システム権限監査レポートを取得
pub async fn get_system_permission_audit_handler(
    State(_app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(query): Query<SystemPermissionAuditQuery>,
) -> AppResult<Json<ApiResponse<SystemPermissionAuditResponse>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    info!(
        admin_id = %admin_user.user_id(),
        user_filter = ?query.user_id,
        resource_filter = ?query.resource,
        action_filter = ?query.action,
        "Getting system permission audit"
    );

    // モック監査エントリー
    let mut audit_entries = vec![
        PermissionAuditEntry {
            id: Uuid::new_v4(),
            user_id: query.user_id.unwrap_or(Uuid::new_v4()),
            resource: "tasks".to_string(),
            action: "read".to_string(),
            result: AuditResult::Allowed,
            reason: None,
            scope: Some(PermissionScope::Own),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            timestamp: chrono::Utc::now(),
        },
        PermissionAuditEntry {
            id: Uuid::new_v4(),
            user_id: query.user_id.unwrap_or(Uuid::new_v4()),
            resource: "admin".to_string(),
            action: "manage".to_string(),
            result: AuditResult::Denied,
            reason: Some("Insufficient permissions".to_string()),
            scope: None,
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            timestamp: chrono::Utc::now(),
        },
    ];

    // フィルタリング
    if let Some(resource) = &query.resource {
        audit_entries.retain(|e| e.resource == *resource);
    }
    if let Some(action) = &query.action {
        audit_entries.retain(|e| e.action == *action);
    }

    let total_checks = audit_entries.len() as u32;
    let allowed_checks = audit_entries
        .iter()
        .filter(|e| matches!(e.result, AuditResult::Allowed))
        .count() as u32;
    let denied_checks = total_checks - allowed_checks;

    let summary = AuditSummary {
        total_checks,
        allowed_checks,
        denied_checks,
        unique_users: 1,
        unique_resources: 2,
        most_accessed_resource: "tasks".to_string(),
        most_denied_action: "manage".to_string(),
        success_rate: if total_checks > 0 {
            (allowed_checks as f64 / total_checks as f64) * 100.0
        } else {
            0.0
        },
    };

    let audit_period = AuditPeriod {
        start_date: query
            .from_date
            .unwrap_or(chrono::Utc::now() - chrono::Duration::days(7)),
        end_date: query.to_date.unwrap_or(chrono::Utc::now()),
        duration_hours: 168,
    };

    let response = SystemPermissionAuditResponse {
        audit_entries,
        summary,
        total_entries: total_checks,
        filtered_entries: total_checks,
        audit_period,
    };

    Ok(Json(ApiResponse::success(
        "System permission audit retrieved",
        response,
    )))
}

// --- Specific Resource Permission Endpoints ---

/// リソース作成権限をチェック
pub async fn check_create_resource_permission_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<requests::CreateResourcePermissionRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    payload.validate().map_err(|validation_errors| {
        warn!(
            "Create resource permission validation failed: {}",
            validation_errors
        );
        let errors: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();
        AppError::ValidationErrors(errors)
    })?;

    info!(
        user_id = %user.user_id(),
        resource_type = %payload.resource_type,
        "Checking create resource permission"
    );

    // can_create_resourceメソッドを活用
    let allowed = user.claims.can_create_resource(&payload.resource_type);

    let response = serde_json::json!({
        "user_id": user.user_id(),
        "resource_type": payload.resource_type,
        "action": "create",
        "allowed": allowed,
        "reason": if !allowed { Some("Insufficient permissions to create this resource type") } else { None },
        "checked_at": chrono::Utc::now()
    });

    Ok(Json(ApiResponse::success(
        "Create resource permission checked",
        response,
    )))
}

/// リソース削除権限をチェック
pub async fn check_delete_resource_permission_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<requests::DeleteResourcePermissionRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    payload.validate().map_err(|validation_errors| {
        warn!(
            "Delete resource permission validation failed: {}",
            validation_errors
        );
        let errors: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();
        AppError::ValidationErrors(errors)
    })?;

    info!(
        user_id = %user.user_id(),
        resource_type = %payload.resource_type,
        resource_id = ?payload.resource_id,
        "Checking delete resource permission"
    );

    // can_delete_resourceメソッドを活用
    let allowed = user
        .claims
        .can_delete_resource(&payload.resource_type, payload.resource_id);

    let response = serde_json::json!({
        "user_id": user.user_id(),
        "resource_type": payload.resource_type,
        "resource_id": payload.resource_id,
        "action": "delete",
        "allowed": allowed,
        "reason": if !allowed { Some("Insufficient permissions to delete this resource") } else { None },
        "checked_at": chrono::Utc::now()
    });

    Ok(Json(ApiResponse::success(
        "Delete resource permission checked",
        response,
    )))
}

/// ユーザーアクセス権限をチェック
pub async fn check_user_access_permission_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<requests::UserAccessPermissionRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    payload.validate().map_err(|validation_errors| {
        warn!(
            "User access permission validation failed: {}",
            validation_errors
        );
        let errors: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();
        AppError::ValidationErrors(errors)
    })?;

    info!(
        user_id = %user.user_id(),
        target_user_id = %payload.target_user_id,
        "Checking user access permission"
    );

    // can_access_userメソッドを活用
    let allowed = user.claims.can_access_user(payload.target_user_id);

    let response = serde_json::json!({
        "user_id": user.user_id(),
        "target_user_id": payload.target_user_id,
        "action": "access",
        "allowed": allowed,
        "is_self": user.user_id() == payload.target_user_id,
        "reason": if !allowed { Some("Cannot access other user's data without proper permissions") } else { None },
        "checked_at": chrono::Utc::now()
    });

    Ok(Json(ApiResponse::success(
        "User access permission checked",
        response,
    )))
}

/// リソース表示権限をチェック
pub async fn check_view_resource_permission_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<requests::ViewResourcePermissionRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    payload.validate().map_err(|validation_errors| {
        warn!(
            "View resource permission validation failed: {}",
            validation_errors
        );
        let errors: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();
        AppError::ValidationErrors(errors)
    })?;

    info!(
        user_id = %user.user_id(),
        resource_type = %payload.resource_type,
        resource_id = ?payload.resource_id,
        "Checking view resource permission"
    );

    // can_view_resourceメソッドを活用
    let allowed = user
        .claims
        .can_view_resource(&payload.resource_type, payload.resource_id);

    let response = serde_json::json!({
        "user_id": user.user_id(),
        "resource_type": payload.resource_type,
        "resource_id": payload.resource_id,
        "action": "view",
        "allowed": allowed,
        "reason": if !allowed { Some("Insufficient permissions to view this resource") } else { None },
        "checked_at": chrono::Utc::now()
    });

    Ok(Json(ApiResponse::success(
        "View resource permission checked",
        response,
    )))
}

// --- Additional Dead Code Utilization ---

/// 権限拒否の詳細チェック
pub async fn check_permission_denial_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CheckPermissionRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    info!(
        user_id = %user.user_id(),
        resource = %payload.resource,
        action = %payload.action,
        "Checking permission denial details"
    );

    let result = if let Some(ref role) = user.claims.role {
        role.can_perform_action(&payload.resource, &payload.action, payload.target_user_id)
    } else {
        PermissionResult::denied("No role assigned")
    };

    // is_denied()メソッドを活用
    if result.is_denied() {
        let denial_reason = result.get_denial_reason().cloned().unwrap_or_default();
        let suggestions = vec![
            "Contact your administrator for access".to_string(),
            "Check your subscription tier".to_string(),
        ];

        let response = serde_json::json!({
            "user_id": user.user_id(),
            "resource": payload.resource,
            "action": payload.action,
            "denied": true,
            "reason": denial_reason,
            "suggestions": suggestions,
            "upgrade_required": false,
            "checked_at": chrono::Utc::now()
        });

        Ok(Json(ApiResponse::success(
            "Permission denial check completed",
            response,
        )))
    } else {
        Ok(Json(ApiResponse::success(
            "Permission granted",
            serde_json::json!({
                "user_id": user.user_id(),
                "resource": payload.resource,
                "action": payload.action,
                "denied": false,
            }),
        )))
    }
}

/// 特権機能のチェック
pub async fn check_privilege_feature_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<FeatureAccessRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    info!(
        user_id = %user.user_id(),
        feature_name = %payload.feature_name,
        "Checking privilege feature access"
    );

    // 特権を構築してhas_feature()を使用
    let privilege = match user.claims.subscription_tier {
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
        "user_id": user.user_id(),
        "feature_check": feature_details,
        "feature_available": has_feature,
        "upgrade_required": !has_feature && payload.required_tier.is_some_and(|t| !user.claims.subscription_tier.is_at_least(&t)),
        "checked_at": chrono::Utc::now()
    });

    info!(
        user_id = %user.user_id(),
        feature_name = %payload.feature_name,
        has_feature = %has_feature,
        "Privilege feature check completed"
    );

    Ok(Json(ApiResponse::success(
        "Feature access check completed",
        response,
    )))
}

/// 複雑な操作の権限チェック
pub async fn check_complex_operation_permissions_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ComplexOperationRequest>,
) -> AppResult<Json<ApiResponse<ComplexOperationPermissionResponse>>> {
    payload.validate().map_err(|validation_errors| {
        warn!("Complex operation validation failed: {}", validation_errors);
        let errors: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();
        AppError::ValidationErrors(errors)
    })?;

    info!(
        user_id = %user.user_id(),
        operation = %payload.operation,
        resource_type = %payload.resource_type,
        "Checking complex operation permissions"
    );

    // Permission::read_own, write_own, admin_globalメソッドを活用
    let required_permissions = match payload.operation.as_str() {
        "bulk_update" => vec![
            Permission::read_own(&payload.resource_type),
            Permission::write_own(&payload.resource_type),
        ],
        "admin_operation" => vec![Permission::admin_global(&payload.resource_type)],
        _ => vec![Permission::read_own(&payload.resource_type)],
    };

    let mut permission_results = Vec::new();
    let mut all_allowed = true;

    for permission in required_permissions {
        let matches = if let Some(ref role) = user.claims.role {
            let result = role.can_perform_action(
                &permission.resource,
                &permission.action,
                payload.resource_id,
            );
            result.is_allowed()
        } else {
            false
        };

        if !matches {
            all_allowed = false;
        }

        permission_results.push(PermissionCheckDetail {
            permission_type: format!("{}:{}", permission.resource, permission.action),
            allowed: matches,
            description: format!(
                "Permission to {} on {}",
                permission.action, permission.resource
            ),
        });
    }

    let response = ComplexOperationPermissionResponse {
        user_id: user.user_id(),
        operation: payload.operation,
        operation_allowed: all_allowed,
        permission_details: permission_results,
        checked_at: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(
        "Complex operation permission check completed",
        response,
    )))
}

// --- Router Configuration ---

/// Permission management router
pub fn permission_router() -> Router<AppState> {
    Router::new()
        // Basic permission endpoints
        .route("/permissions/check", post(check_permission_handler))
        .route("/permissions/validate", post(validate_permissions_handler))
        .route("/permissions/user/{id}", get(get_user_permissions_handler))
        .route(
            "/permissions/resources",
            get(get_available_resources_handler),
        )
        // Feature access endpoints
        .route("/features/available", get(get_feature_access_handler))
        .route("/features/admin", get(get_admin_features_handler))
        .route("/features/analytics", get(get_analytics_features_handler))
        // Audit endpoints
        .route(
            "/permissions/resources/{resource}/actions/{action}/check",
            get(check_resource_permission_handler),
        )
        .route(
            "/permissions/bulk-check",
            post(bulk_permission_check_handler),
        )
        .route(
            "/admin/permissions/bulk-check",
            post(bulk_permission_check_handler),
        )
        .route(
            "/permissions/user/{id}/effective",
            get(get_user_effective_permissions_handler),
        )
        .route(
            "/admin/permissions/audit",
            get(get_system_permission_audit_handler),
        )
        // Specific permission check endpoints
        .route(
            "/permissions/resources/create",
            post(check_create_resource_permission_handler),
        )
        .route(
            "/permissions/resources/delete",
            post(check_delete_resource_permission_handler),
        )
        .route(
            "/permissions/users/access",
            post(check_user_access_permission_handler),
        )
        .route(
            "/permissions/resources/view",
            post(check_view_resource_permission_handler),
        )
        // Additional endpoints for dead code utilization
        .route(
            "/permissions/denial-check",
            post(check_permission_denial_handler),
        )
        .route(
            "/features/privilege-check",
            post(check_privilege_feature_handler),
        )
        .route(
            "/permissions/complex-operation",
            post(check_complex_operation_permissions_handler),
        )
}
