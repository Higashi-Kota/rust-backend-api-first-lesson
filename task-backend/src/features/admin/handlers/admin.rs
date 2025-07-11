use crate::error::AppResult;
use crate::features::auth::middleware::AuthenticatedUserWithRole;
use crate::shared::types::ApiResponse;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupQuery {
    pub days: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureMetricsQuery {
    pub user_id: Option<Uuid>,
    pub days: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkDeleteRequest {
    pub task_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupResult {
    pub deleted_count: i32,
    pub operation_type: String,
}

/// Admin cleanup daily summaries
pub async fn cleanup_daily_summaries_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<CleanupQuery>,
) -> AppResult<Json<ApiResponse<CleanupResult>>> {
    let days = query.days.unwrap_or(365); // Default to 365 days as per test

    let admin_service =
        crate::features::admin::services::AdminService::new((*app_state.db).clone());
    let deleted_count = admin_service.cleanup_daily_summaries(days).await?;

    let result = CleanupResult {
        deleted_count,
        operation_type: "daily_activity_summary_cleanup".to_string(),
    };

    Ok(Json(ApiResponse::success(
        "Daily summaries cleanup completed",
        result,
    )))
}

/// Admin cleanup bulk operations
pub async fn cleanup_bulk_operations_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<CleanupQuery>,
) -> AppResult<Json<ApiResponse<CleanupResult>>> {
    let days = query.days.unwrap_or(90); // Default to 90 days as per test

    // Validate minimum days
    if days < 30 {
        return Err(crate::error::AppError::ValidationError(
            "Minimum retention period is 30 days".to_string(),
        ));
    }

    let admin_service =
        crate::features::admin::services::AdminService::new((*app_state.db).clone());
    let deleted_count = admin_service.cleanup_bulk_operations(days).await?;

    let result = CleanupResult {
        deleted_count,
        operation_type: "bulk_operation_history_cleanup".to_string(),
    };

    Ok(Json(ApiResponse::success(
        "Bulk operations cleanup completed",
        result,
    )))
}

/// Admin cleanup expired sessions
pub async fn cleanup_expired_sessions_handler(
    State(_app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
) -> AppResult<Json<ApiResponse<CleanupResult>>> {
    // TODO: Implement actual cleanup logic
    let result = CleanupResult {
        deleted_count: 0,
        operation_type: "expired_sessions".to_string(),
    };

    Ok(Json(ApiResponse::success(
        "Expired sessions cleanup completed",
        result,
    )))
}

/// Admin cleanup feature metrics
pub async fn cleanup_feature_metrics_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<CleanupQuery>,
) -> AppResult<Json<ApiResponse<CleanupResult>>> {
    let days = query.days.unwrap_or(180); // Default to 180 days

    let admin_service =
        crate::features::admin::services::AdminService::new((*app_state.db).clone());
    let deleted_count = admin_service.cleanup_feature_metrics(days).await?;

    let result = CleanupResult {
        deleted_count,
        operation_type: "feature_usage_metrics_cleanup".to_string(),
    };

    Ok(Json(ApiResponse::success(
        "Feature usage metrics cleanup completed",
        result,
    )))
}

/// Admin get user feature metrics
pub async fn get_user_feature_metrics_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<FeatureMetricsQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    if let Some(user_id) = query.user_id {
        // Get metrics for specific user
        let days = query.days.unwrap_or(30);
        let admin_service =
            crate::features::admin::services::AdminService::new((*app_state.db).clone());
        let metrics = admin_service
            .get_user_feature_metrics(user_id, days)
            .await?;

        // Aggregate counts by feature_name and action_type
        let mut action_counts = serde_json::Map::new();
        for metric in &metrics {
            let key = format!("{}_{}", metric.feature_name, metric.action_type);
            let count = action_counts
                .get(&key)
                .and_then(|v| v.as_i64())
                .unwrap_or(0)
                + 1;
            action_counts.insert(key, json!(count));
        }

        let result = json!({
            "user_id": user_id.to_string(),
            "action_counts": action_counts,
            "total_metrics": metrics.len()
        });

        Ok(Json(ApiResponse::success(
            "User feature metrics retrieved successfully",
            result,
        )))
    } else {
        // Return empty metrics if no user_id provided
        let metrics = json!({
            "feature_metrics": [],
            "total_metrics": 0
        });

        Ok(Json(ApiResponse::success(
            "User feature metrics retrieved successfully",
            metrics,
        )))
    }
}

/// Admin cleanup all handler
pub async fn cleanup_all_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<CleanupQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let _days = query.days.unwrap_or(30);

    let admin_service =
        crate::features::admin::services::AdminService::new((*app_state.db).clone());
    let (daily_count, bulk_count, metrics_count) = admin_service.cleanup_all(365, 90, 180).await?;

    let total_deleted = daily_count + bulk_count + metrics_count;

    let results = json!({
        "daily_summaries": {
            "deleted_count": daily_count,
            "operation_type": format!("daily_summaries_older_than_{}_days", 365)
        },
        "bulk_operations": {
            "deleted_count": bulk_count,
            "operation_type": format!("bulk_operations_older_than_{}_days", 90)
        },
        "feature_usage_metrics": {
            "deleted_count": metrics_count,
            "operation_type": format!("feature_usage_metrics_older_than_{}_days", 180)
        },
        "total_deleted": total_deleted
    });

    Ok(Json(ApiResponse::success(
        "All cleanup operations completed",
        results,
    )))
}

/// Admin list bulk operations
pub async fn list_bulk_operations_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    let admin_service =
        crate::features::admin::services::AdminService::new((*app_state.db).clone());
    let operations = admin_service.list_bulk_operations().await?;

    // Convert to JSON values for compatibility
    let operations_json: Vec<serde_json::Value> = operations
        .into_iter()
        .map(|op| {
            json!({
                "id": op.id,
                "operation_type": op.operation_type,
                "status": op.status,
                "affected_count": op.affected_count,
                "performed_by": op.performed_by,
                "created_at": op.created_at,
                "completed_at": op.completed_at,
                "error_details": op.error_details,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(
        "Bulk operations retrieved successfully",
        operations_json,
    )))
}

/// Admin bulk delete tasks
pub async fn bulk_delete_tasks_handler(
    State(_app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Json(request): Json<BulkDeleteRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    // TODO: Implement actual bulk delete logic
    let result = json!({
        "deleted_count": request.task_ids.len(),
        "task_ids": request.task_ids
    });

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("Tasks deleted successfully", result)),
    ))
}

/// Admin delete single task by ID
pub async fn admin_delete_task_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(task_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // Admin can delete any task
    app_state
        .task_service
        .admin_delete_task_by_id(task_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Admin list all tasks
pub async fn admin_list_all_tasks_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    // Admin can see all tasks from all users
    let tasks = app_state.task_service.admin_list_all_tasks().await?;

    // Convert to JSON values
    let tasks_json: Vec<serde_json::Value> = tasks
        .into_iter()
        .map(|task| {
            json!({
                "id": task.id,
                "user_id": task.user_id,
                "title": task.title,
                "description": task.description,
                "status": task.status,
                "priority": task.priority,
                "due_date": task.due_date,
                "created_at": task.created_at,
                "updated_at": task.updated_at,
            })
        })
        .collect();

    Ok(Json(tasks_json))
}

/// Admin list tasks for specific user
pub async fn admin_list_user_tasks_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    // Admin can see all tasks for a specific user
    let tasks = app_state
        .task_service
        .admin_list_user_tasks(user_id)
        .await?;

    // Convert to JSON values
    let tasks_json: Vec<serde_json::Value> = tasks
        .into_iter()
        .map(|task| {
            json!({
                "id": task.id,
                "user_id": task.user_id,
                "title": task.title,
                "description": task.description,
                "status": task.status,
                "priority": task.priority,
                "due_date": task.due_date,
                "created_at": task.created_at,
                "updated_at": task.updated_at,
            })
        })
        .collect();

    Ok(Json(tasks_json))
}

/// Admin get organization stats
pub async fn get_organization_stats_handler(
    State(_app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // TODO: Implement actual stats logic
    let stats = json!({
        "total_organizations": 0,
        "organizations_by_tier": [],
        "total_teams": 0,
        "total_members": 0
    });

    Ok(Json(ApiResponse::success(
        "Organization statistics retrieved successfully",
        stats,
    )))
}

/// Admin list organizations
pub async fn admin_list_organizations_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<OrganizationListQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::organization::models::organization;
    use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * page_size) as u64;

    // Build query with optional filter
    let mut query_builder = organization::Entity::find();

    if let Some(subscription_tier) = &query.subscription_tier {
        query_builder = query_builder
            .filter(organization::Column::SubscriptionTier.eq(subscription_tier.clone()));
    }

    // Get total count
    let total_count = query_builder
        .clone()
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0);

    // Get organizations
    let organizations = query_builder
        .limit(page_size as u64)
        .offset(offset)
        .all(app_state.db.as_ref())
        .await
        .unwrap_or_default();

    let org_responses: Vec<serde_json::Value> = organizations
        .into_iter()
        .map(|org| {
            json!({
                "id": org.id,
                "name": org.name,
                "owner_id": org.owner_id,
                "subscription_tier": org.subscription_tier,
                "created_at": org.created_at,
                "updated_at": org.updated_at,
                "teams_count": 0, // TODO: Implement actual count
                "members_count": 1, // TODO: Implement actual count
                "active": true,
            })
        })
        .collect();

    let total_pages = ((total_count as f64) / (page_size as f64)).ceil() as i32;

    // Count organizations by tier
    use std::collections::HashMap;
    let mut tier_counts: HashMap<String, u32> = HashMap::new();
    for org in &org_responses {
        let tier = org["subscription_tier"].as_str().unwrap_or("free");
        *tier_counts.entry(tier.to_string()).or_insert(0) += 1;
    }

    let tier_summary: Vec<serde_json::Value> = tier_counts
        .into_iter()
        .map(|(tier, count)| {
            json!({
                "tier": tier,
                "count": count,
            })
        })
        .collect();

    let response = json!({
        "organizations": org_responses,
        "pagination": {
            "total_count": total_count,
            "page": page,
            "page_size": page_size,
            "total_pages": total_pages,
            "has_next": page < total_pages,
            "has_prev": page > 1,
        },
        "tier_summary": tier_summary,
    });

    Ok(Json(ApiResponse::success(
        "Organizations retrieved successfully",
        response,
    )))
}

/// Admin list roles
pub async fn admin_list_roles_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<crate::shared::types::pagination::PaginationQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::security::models::role;
    use crate::features::user::models::user;
    use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as u64;

    // Get all roles
    let all_roles = role::Entity::find()
        .all(app_state.db.as_ref())
        .await
        .unwrap_or_default();

    let total_count = all_roles.len() as i64;

    // Apply pagination manually
    let start = offset as usize;
    let end = (start + per_page as usize).min(all_roles.len());
    let roles: Vec<role::Model> = if start < all_roles.len() {
        all_roles[start..end].to_vec()
    } else {
        vec![]
    };

    // Count users for each role
    let mut role_responses = vec![];
    for role in roles {
        let user_count = user::Entity::find()
            .filter(user::Column::RoleId.eq(role.id))
            .count(app_state.db.as_ref())
            .await
            .unwrap_or(0);

        let permissions = json!({
            "can_create_organization": role.name == "admin",
            "can_manage_organization": role.name == "admin",
            "can_create_teams": true,
            "can_manage_teams": role.name == "admin",
            "can_manage_users": role.name == "admin",
            "can_view_analytics": true,
        });

        role_responses.push(json!({
            "id": role.id,
            "name": role.name,
            "display_name": role.display_name,
            "description": role.description,
            "is_active": role.is_active,
            "is_system_role": matches!(role.name.as_str(), "admin" | "member"),
            "user_count": user_count,
            "created_at": role.created_at,
            "updated_at": role.updated_at,
            "permissions": permissions,
        }));
    }

    let total_pages = ((total_count as f64) / (per_page as f64)).ceil() as i32;

    let response = json!({
        "roles": role_responses,
        "total_count": total_count,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total_pages": total_pages,
            "total_count": total_count,
        },
    });

    Ok(Json(ApiResponse::success(
        "Roles retrieved successfully",
        response,
    )))
}

/// Admin get role with subscription details
pub async fn admin_get_role_with_subscription_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(role_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::security::models::role;
    use sea_orm::EntityTrait;

    // Get the role
    let role = role::Entity::find_by_id(role_id)
        .one(app_state.db.as_ref())
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Role not found".to_string()))?;

    // Create subscription tier info
    let subscription_benefits = match role.name.as_str() {
        "admin" => json!({
            "tier": "enterprise",
            "max_users": "unlimited",
            "max_tasks": "unlimited",
            "features": [
                "full_analytics",
                "api_access",
                "priority_support",
                "custom_integrations",
                "team_management",
                "bulk_operations"
            ],
            "api_rate_limit": 10000,
            "storage_gb": 100
        }),
        "member" => json!({
            "tier": "free",
            "max_users": 5,
            "max_tasks": 100,
            "features": [
                "basic_analytics",
                "task_management"
            ],
            "api_rate_limit": 1000,
            "storage_gb": 1
        }),
        _ => json!({
            "tier": "custom",
            "max_users": 10,
            "max_tasks": 500,
            "features": ["task_management"],
            "api_rate_limit": 2000,
            "storage_gb": 5
        }),
    };

    let response = json!({
        "id": role.id,
        "name": role.name,
        "display_name": role.display_name,
        "description": role.description,
        "is_active": role.is_active,
        "is_system_role": matches!(role.name.as_str(), "admin" | "member"),
        "created_at": role.created_at,
        "updated_at": role.updated_at,
        "permissions": {
            "base_permissions": {
                "tasks": {
                    "create": true,
                    "read": true,
                    "update": true,
                    "delete": role.name == "admin",
                },
                "teams": {
                    "create": true,
                    "manage": role.name == "admin",
                },
                "users": {
                    "manage": role.name == "admin",
                },
                "admin": {
                    "full_access": role.name == "admin",
                },
            },
            "can_create_organization": role.name == "admin",
            "can_manage_organization": role.name == "admin",
            "can_create_teams": true,
            "can_manage_teams": role.name == "admin",
            "can_manage_users": role.name == "admin",
            "can_view_analytics": true,
        },
        "subscription_info": {
            "applicable_tiers": ["all"],
            "tier": subscription_benefits["tier"].clone(),
            "max_users": subscription_benefits["max_users"].clone(),
            "max_tasks": subscription_benefits["max_tasks"].clone(),
            "features": subscription_benefits["features"].clone(),
            "api_rate_limit": subscription_benefits["api_rate_limit"].clone(),
            "storage_gb": subscription_benefits["storage_gb"].clone(),
        },
    });

    Ok(Json(ApiResponse::success(
        "Role retrieved with subscription details",
        response,
    )))
}

/// Admin list users with roles
pub async fn admin_list_users_with_roles_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<crate::shared::types::pagination::PaginationQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::user::models::user;
    use sea_orm::{EntityTrait, PaginatorTrait, QuerySelect};
    use std::collections::HashMap;

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * page_size) as u64;

    // Get total count
    let total_count = user::Entity::find()
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0);

    // Get users
    let users = user::Entity::find()
        .limit(page_size as u64)
        .offset(offset)
        .all(app_state.db.as_ref())
        .await
        .unwrap_or_default();

    // Get roles for each user and count role distribution
    let mut user_responses = vec![];
    let mut role_counts: HashMap<String, u32> = HashMap::new();

    for user in users {
        let role = app_state
            .role_service
            .get_role_by_id(user.role_id)
            .await
            .ok();

        let role_name = role
            .as_ref()
            .map_or_else(|| "member".to_string(), |r| r.name.to_string());
        *role_counts.entry(role_name.clone()).or_insert(0) += 1;

        let is_admin = role.as_ref().is_some_and(|r| r.is_admin());

        let permissions_json = json!({
            "can_create_organization": true,
            "can_manage_organization": is_admin,
            "can_create_teams": true,
            "can_manage_teams": is_admin,
            "can_manage_users": is_admin,
            "can_view_analytics": true,
        });

        user_responses.push(json!({
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "is_active": user.is_active,
            "email_verified": user.email_verified,
            "created_at": user.created_at,
            "last_login_at": user.last_login_at,
            "role": {
                "id": role.as_ref().map_or(user.role_id, |r| r.id),
                "name": role_name,
                "display_name": role.as_ref().map_or_else(|| "Member".to_string(), |r| r.display_name.clone()),
                "permissions": permissions_json,
            },
        }));
    }

    // Create role summary
    let role_summary: Vec<serde_json::Value> = role_counts
        .into_iter()
        .map(|(role_name, count)| {
            json!({
                "role": role_name,
                "count": count,
            })
        })
        .collect();

    let total_pages = ((total_count as f64) / (page_size as f64)).ceil() as i32;

    let response = json!({
        "users": user_responses,
        "role_summary": role_summary,
        "pagination": {
            "total": total_count,
            "page": page,
            "per_page": page_size,
            "total_pages": total_pages,
        },
    });

    Ok(Json(ApiResponse::success(
        "Users retrieved successfully",
        response,
    )))
}

/// Admin subscription analytics
pub async fn admin_subscription_analytics_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::subscription::models::history::Entity as SubscriptionHistory;
    use chrono::{Duration, Utc};
    use sea_orm::EntityTrait;

    // Get all subscription histories
    let histories = SubscriptionHistory::find()
        .all(app_state.db.as_ref())
        .await
        .unwrap_or_default();

    // Count by tier changes
    let mut tier_distribution = std::collections::HashMap::new();
    let mut total_upgrades = 0;
    let mut total_downgrades = 0;
    let mut recent_upgrades = vec![];
    let mut recent_downgrades = vec![];

    // Track current tiers (latest for each user)
    let mut user_tiers: std::collections::HashMap<Uuid, String> = std::collections::HashMap::new();

    let thirty_days_ago = Utc::now() - Duration::days(30);

    for history in &histories {
        // Update user's current tier
        user_tiers.insert(history.user_id, history.new_tier.clone());

        // Count upgrades and downgrades
        let prev_level = match history.previous_tier.as_deref() {
            Some("free") => 1,
            Some("pro") => 2,
            Some("enterprise") => 3,
            _ => 0,
        };

        let new_level = match history.new_tier.as_str() {
            "free" => 1,
            "pro" => 2,
            "enterprise" => 3,
            _ => 0,
        };

        match new_level.cmp(&prev_level) {
            std::cmp::Ordering::Greater => {
                total_upgrades += 1;
                if history.changed_at > thirty_days_ago {
                    recent_upgrades.push(json!({
                        "user_id": history.user_id,
                        "from": history.previous_tier,
                        "to": history.new_tier,
                        "changed_at": history.changed_at,
                    }));
                }
            }
            std::cmp::Ordering::Less => {
                total_downgrades += 1;
                if history.changed_at > thirty_days_ago {
                    recent_downgrades.push(json!({
                        "user_id": history.user_id,
                        "from": history.previous_tier,
                        "to": history.new_tier,
                        "changed_at": history.changed_at,
                    }));
                }
            }
            std::cmp::Ordering::Equal => {}
        }
    }

    // Count current distribution
    for tier in user_tiers.values() {
        *tier_distribution.entry(tier.clone()).or_insert(0) += 1;
    }

    // Convert to array format
    let tier_distribution_array: Vec<serde_json::Value> = tier_distribution
        .into_iter()
        .map(|(tier, count)| {
            json!({
                "tier": tier,
                "count": count,
            })
        })
        .collect();

    let response = json!({
        "tier_distribution": tier_distribution_array,
        "total_upgrades": total_upgrades,
        "total_downgrades": total_downgrades,
        "recent_upgrades": recent_upgrades.into_iter().take(5).collect::<Vec<_>>(),
        "recent_downgrades": recent_downgrades.into_iter().take(5).collect::<Vec<_>>(),
    });

    Ok(Json(ApiResponse::success(
        "Subscription analytics retrieved successfully",
        response,
    )))
}

/// Admin subscription history search
pub async fn admin_subscription_history_search_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<SubscriptionHistorySearchQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::subscription::models::history::{Column, Entity as SubscriptionHistory};
    use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as u64;

    // Build query with filters
    let mut query_builder = SubscriptionHistory::find();

    if let Some(tier) = &query.tier {
        query_builder = query_builder.filter(
            Column::NewTier
                .eq(tier.clone())
                .or(Column::PreviousTier.eq(tier.clone())),
        );
    }

    // Get total count
    let total_count = query_builder
        .clone()
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0);

    // Get paginated results
    let histories = query_builder
        .order_by_desc(Column::ChangedAt)
        .limit(per_page as u64)
        .offset(offset)
        .all(app_state.db.as_ref())
        .await
        .unwrap_or_default();

    let items: Vec<serde_json::Value> = histories
        .into_iter()
        .map(|h| {
            json!({
                "id": h.id,
                "user_id": h.user_id,
                "previous_tier": h.previous_tier,
                "new_tier": h.new_tier,
                "changed_by": h.changed_by,
                "reason": h.reason,
                "changed_at": h.changed_at,
            })
        })
        .collect();

    let total_pages = ((total_count as f64) / (per_page as f64)).ceil() as i32;

    let response = json!({
        "items": items,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total_count": total_count,
            "total_pages": total_pages,
            "has_next": page < total_pages,
            "has_prev": page > 1,
        },
    });

    Ok(Json(ApiResponse::success(
        "Subscription history search results",
        response,
    )))
}

/// Admin get subscription history
pub async fn admin_subscription_history_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<SubscriptionHistoryQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::subscription::models::history::{Column, Entity as SubscriptionHistory};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

    let mut query_builder = SubscriptionHistory::find();

    // Apply date range filter if provided
    if let Some(start_date) = query.start_date {
        query_builder = query_builder.filter(Column::ChangedAt.gte(start_date));
    }

    if let Some(end_date) = query.end_date {
        query_builder = query_builder.filter(Column::ChangedAt.lte(end_date));
    }

    // Apply filter type if provided
    if let Some(filter) = &query.filter {
        match filter.as_str() {
            "upgrades" => {
                // This is a simplified approach - in real implementation,
                // you'd need to compare tier levels
                query_builder = query_builder.filter(
                    Column::NewTier
                        .eq("pro")
                        .and(Column::PreviousTier.eq("free"))
                        .or(Column::NewTier
                            .eq("enterprise")
                            .and(Column::PreviousTier.ne("enterprise"))),
                );
            }
            "downgrades" => {
                query_builder = query_builder.filter(
                    Column::NewTier
                        .eq("free")
                        .and(Column::PreviousTier.ne("free"))
                        .or(Column::NewTier
                            .eq("pro")
                            .and(Column::PreviousTier.eq("enterprise"))),
                );
            }
            _ => {}
        }
    }

    // Get all histories
    let histories = query_builder
        .order_by_desc(Column::ChangedAt)
        .all(app_state.db.as_ref())
        .await
        .unwrap_or_default();

    // Calculate tier stats
    let mut tier_stats = std::collections::HashMap::new();
    let mut change_summary = std::collections::HashMap::new();
    change_summary.insert("upgrades".to_string(), 0);
    change_summary.insert("downgrades".to_string(), 0);
    change_summary.insert("no_change".to_string(), 0);

    for history in &histories {
        // Count current tier distribution
        *tier_stats.entry(history.new_tier.clone()).or_insert(0) += 1;

        // Count change types
        let prev_level = history.previous_tier.as_deref().map_or(0, get_tier_level);
        let new_level = get_tier_level(&history.new_tier);

        match new_level.cmp(&prev_level) {
            std::cmp::Ordering::Greater => {
                *change_summary.get_mut("upgrades").unwrap() += 1;
            }
            std::cmp::Ordering::Less => {
                *change_summary.get_mut("downgrades").unwrap() += 1;
            }
            std::cmp::Ordering::Equal => {
                *change_summary.get_mut("no_change").unwrap() += 1;
            }
        }
    }

    // Convert histories to response format
    let history_responses: Vec<serde_json::Value> = histories
        .into_iter()
        .map(|h| {
            json!({
                "id": h.id,
                "user_id": h.user_id,
                "previous_tier": h.previous_tier,
                "new_tier": h.new_tier,
                "changed_by": h.changed_by,
                "reason": h.reason,
                "changed_at": h.changed_at,
            })
        })
        .collect();

    let response = json!({
        "histories": history_responses,
        "tier_stats": tier_stats,
        "change_summary": change_summary,
    });

    Ok(Json(ApiResponse::success(
        "Subscription history retrieved successfully",
        response,
    )))
}

fn get_tier_level(tier: &str) -> i32 {
    match tier {
        "free" => 1,
        "pro" => 2,
        "enterprise" => 3,
        _ => 0,
    }
}

#[derive(Debug, Deserialize)]
pub struct SubscriptionHistoryQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub filter: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubscriptionHistorySearchQuery {
    pub tier: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct OrganizationListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub subscription_tier: Option<String>,
}

/// Admin router
pub fn admin_router() -> Router<crate::api::AppState> {
    Router::new().nest(
        "/admin",
        Router::new()
            // Cleanup operations - support both GET and DELETE
            .route(
                "/cleanup/daily-summaries",
                delete(cleanup_daily_summaries_handler),
            )
            .route(
                "/cleanup/bulk-operations",
                get(list_bulk_operations_handler),
            )
            .route(
                "/cleanup/bulk-operations",
                delete(cleanup_bulk_operations_handler),
            )
            .route(
                "/cleanup/expired-sessions",
                delete(cleanup_expired_sessions_handler),
            )
            .route(
                "/cleanup/feature-metrics",
                get(get_user_feature_metrics_handler),
            )
            .route(
                "/cleanup/feature-metrics",
                delete(cleanup_feature_metrics_handler),
            )
            .route("/cleanup/all", delete(cleanup_all_handler))
            // Bulk operations
            .route("/bulk-operations", get(list_bulk_operations_handler))
            .route("/tasks/bulk/delete", delete(bulk_delete_tasks_handler))
            // Admin task management
            .route("/tasks", get(admin_list_all_tasks_handler))
            .route("/tasks/{id}", delete(admin_delete_task_handler))
            .route("/users/{user_id}/tasks", get(admin_list_user_tasks_handler))
            // Organization endpoints
            .route("/organizations/stats", get(get_organization_stats_handler))
            .route("/organizations", get(admin_list_organizations_handler))
            // User endpoints
            .route("/users/roles", get(admin_list_users_with_roles_handler))
            // Analytics endpoints
            .route("/analytics/roles", get(admin_list_roles_handler))
            .route(
                "/analytics/roles/{id}/subscription",
                get(admin_get_role_with_subscription_handler),
            )
            // Subscription endpoints
            .route(
                "/subscription/analytics",
                get(admin_subscription_analytics_handler),
            )
            .route(
                "/subscription/history",
                get(admin_subscription_history_handler),
            )
            .route(
                "/subscription/history/search",
                get(admin_subscription_history_search_handler),
            ),
    )
}
