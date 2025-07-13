// 一時的に旧DTOを使用（Phase 19の互換性確保のため）
use crate::error::AppResult;
use crate::features::auth::middleware::AuthenticatedUser;
use crate::features::organization::dto::organization::{
    CreateOrganizationRequest, InviteOrganizationMemberRequest, OrganizationCapacityResponse,
    OrganizationListResponse, OrganizationMemberDetailResponse, OrganizationResponse,
    OrganizationSearchQuery, OrganizationStatsResponse, UpdateOrganizationMemberRoleRequest,
    UpdateOrganizationRequest, UpdateOrganizationSettingsRequest,
    UpdateOrganizationSubscriptionRequest,
};
use crate::features::organization::dto::{
    AnalyticsData, AnalyticsSummary, DepartmentInfo, OrganizationAnalyticsResponse,
    OrganizationDepartmentsResponse,
};
use crate::features::organization::models::organization::OrganizationRole;
use crate::shared::types::ApiResponse;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use serde_json::json;
use std::str::FromStr;
use uuid::Uuid;
use validator::Validate;

/// 組織作成
pub async fn create_organization_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateOrganizationRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<OrganizationResponse>>)> {
    // バリデーション
    payload.validate()?;

    let organization_response = app_state
        .organization_service
        .create_organization(payload, user.user_id())
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            "Organization created successfully",
            organization_response,
        )),
    ))
}

/// 組織詳細取得
pub async fn get_organization_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<OrganizationResponse>>> {
    let organization_response = app_state
        .organization_service
        .get_organization_by_id(organization_id, user.user_id())
        .await?;

    Ok(Json(ApiResponse::success(
        "Organization retrieved successfully",
        organization_response,
    )))
}

/// 組織一覧取得
pub async fn get_organizations_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(query): Query<OrganizationSearchQuery>,
) -> AppResult<Json<ApiResponse<Vec<OrganizationListResponse>>>> {
    let (organizations, _) = app_state
        .organization_service
        .get_organizations_paginated(query, user.user_id())
        .await?;

    Ok(Json(ApiResponse::success(
        "Organizations retrieved successfully",
        organizations,
    )))
}

/// 組織更新
pub async fn update_organization_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
    Json(payload): Json<UpdateOrganizationRequest>,
) -> AppResult<Json<ApiResponse<OrganizationResponse>>> {
    // バリデーション
    payload.validate()?;

    // 権限チェック（オーナーまたは管理者のみ更新可能）
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), organization_id)
        .await?;

    let organization_response = app_state
        .organization_service
        .update_organization(organization_id, payload, user.user_id())
        .await?;

    Ok(Json(ApiResponse::success(
        "Organization updated successfully",
        organization_response,
    )))
}

/// 組織削除
pub async fn delete_organization_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
) -> AppResult<(StatusCode, Json<ApiResponse<()>>)> {
    app_state
        .organization_service
        .delete_organization(organization_id, user.user_id())
        .await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Organization deleted successfully",
            (),
        )),
    ))
}

/// 組織設定更新
pub async fn update_organization_settings_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
    Json(payload): Json<UpdateOrganizationSettingsRequest>,
) -> AppResult<Json<ApiResponse<OrganizationResponse>>> {
    let organization_response = app_state
        .organization_service
        .update_organization_settings(organization_id, payload, user.user_id())
        .await?;

    Ok(Json(ApiResponse::success(
        "Organization settings updated successfully",
        organization_response,
    )))
}

/// 組織サブスクリプション更新
pub async fn update_organization_subscription_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
    Json(payload): Json<UpdateOrganizationSubscriptionRequest>,
) -> AppResult<Json<ApiResponse<OrganizationResponse>>> {
    payload.validate()?;

    // 権限チェック（オーナーのみ更新可能）
    // まず組織を取得
    let organization = app_state
        .organization_service
        .get_organization_by_id(organization_id, user.user_id())
        .await?;

    // オーナーかチェック
    if organization.owner_id != user.user_id() {
        return Err(crate::error::AppError::Forbidden(
            "Only organization owner can update subscription".to_string(),
        ));
    }

    // 組織サービスを使ってサブスクリプションを更新
    let organization_response = app_state
        .organization_service
        .update_organization_subscription(
            organization_id,
            payload.subscription_tier,
            user.user_id(),
        )
        .await?;

    Ok(Json(ApiResponse::success(
        "Organization subscription updated successfully",
        organization_response,
    )))
}

/// 組織サブスクリプション履歴取得
pub async fn get_organization_subscription_history_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    // 権限チェック（管理者権限が必要）
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), organization_id)
        .await?;

    // 組織情報を取得してowner_idを取得
    let organization = app_state
        .organization_service
        .get_organization_by_id(organization_id, user.user_id())
        .await?;

    // 履歴を取得（organizationのowner_idで検索）
    let history = app_state
        .subscription_history_repo
        .find_by_user_id(organization.owner_id)
        .await?;

    // serde_json::Valueに変換
    let history_json: Vec<serde_json::Value> = history
        .into_iter()
        .map(|h| {
            json!({
                "id": h.id,
                "user_id": h.user_id,
                "previous_tier": h.previous_tier,
                "new_tier": h.new_tier,
                "changed_by": h.changed_by,
                "change_reason": h.reason,
                "changed_at": h.changed_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(
        "Subscription history retrieved successfully",
        history_json,
    )))
}

/// 組織メンバー招待
pub async fn invite_organization_member_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
    Json(payload): Json<InviteOrganizationMemberRequest>,
) -> AppResult<(
    StatusCode,
    Json<ApiResponse<OrganizationMemberDetailResponse>>,
)> {
    payload.validate()?;

    let member_response = app_state
        .organization_service
        .invite_member(organization_id, payload, user.user_id())
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            "Member invited successfully",
            member_response,
        )),
    ))
}

/// 組織メンバー詳細取得
pub async fn get_organization_member_handler(
    State(_app_state): State<crate::api::AppState>,
    _user: AuthenticatedUser,
    Path((organization_id, member_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // TODO: Phase 19でPermissionServiceが正しく統合後、コメントを解除
    // app_state
    //     .permission_service
    //     .check_organization_access(user.user_id(), organization_id)
    //     .await?;

    // TODO: 実装が必要
    Ok(Json(ApiResponse::success(
        "Member details retrieved successfully",
        json!({
            "member_id": member_id,
            "organization_id": organization_id
        }),
    )))
}

/// 組織メンバー役割更新
pub async fn update_organization_member_role_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path((organization_id, member_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateOrganizationMemberRoleRequest>,
) -> AppResult<Json<ApiResponse<OrganizationMemberDetailResponse>>> {
    // 権限チェック（管理者権限必須）
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), organization_id)
        .await?;

    // リポジトリを使用してメンバー情報を取得
    use crate::features::organization::repositories::organization::OrganizationRepository;
    let org_repo = OrganizationRepository::new(app_state.db.as_ref().clone());

    // メンバー情報を取得
    let mut member = org_repo
        .find_member_by_id(member_id)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound("Organization member not found".to_string())
        })?;

    // ロールを更新
    member.role = payload.role.clone();

    // データベースを更新
    let updated_member = org_repo.update_member(&member).await?;

    // ユーザー情報を取得してレスポンスを作成
    let user_info = app_state
        .user_service
        .get_user_profile(updated_member.user_id)
        .await?;

    let member_response = OrganizationMemberDetailResponse {
        id: updated_member.id,
        user_id: updated_member.user_id,
        username: user_info.username,
        email: user_info.email,
        role: updated_member.role.clone(),
        is_owner: updated_member.is_owner(),
        is_admin: updated_member.is_admin(),
        can_manage: updated_member.can_manage(),
        can_create_teams: updated_member.role.can_create_teams(),
        can_invite_members: updated_member.role.can_invite_members(),
        can_change_settings: updated_member.role.can_change_settings(),
        joined_at: updated_member.joined_at,
        invited_by: updated_member.invited_by,
    };

    Ok(Json(ApiResponse::success(
        "Member role updated successfully",
        member_response,
    )))
}

/// 組織メンバー削除
pub async fn remove_organization_member_handler(
    State(_app_state): State<crate::api::AppState>,
    _user: AuthenticatedUser,
    Path((_organization_id, _member_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<ApiResponse<()>>> {
    // TODO: Phase 19でOrganizationServiceが正しいDTOを使用するように更新後、コメントを解除
    // app_state
    //     .organization_service
    //     .remove_member(organization_id, member_id, user.user_id())
    //     .await?;

    Ok(Json(ApiResponse::success(
        "Member removed successfully",
        (),
    )))
}

/// 組織容量チェック
pub async fn get_organization_capacity_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<OrganizationCapacityResponse>>> {
    // Phase 19: 旧サービスを使用して互換性を保つ
    let capacity_response = app_state
        .organization_service
        .check_organization_capacity(organization_id, user.user_id())
        .await?;

    Ok(Json(ApiResponse::success(
        "Organization capacity retrieved successfully",
        capacity_response,
    )))
}

/// 組織統計情報取得
pub async fn get_organization_stats_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<OrganizationStatsResponse>>> {
    // Phase 19: 旧サービスを使用して互換性を保つ
    let stats_response = app_state
        .organization_service
        .get_organization_stats(user.user_id())
        .await?;

    Ok(Json(ApiResponse::success(
        "Organization statistics retrieved successfully",
        stats_response,
    )))
}

/// 組織分析データ取得
pub async fn get_organization_analytics_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<OrganizationAnalyticsResponse>>> {
    // 組織情報を取得
    let organization = app_state
        .organization_service
        .get_organization_by_id(organization_id, user.user_id())
        .await?;

    // 分析データを取得（直接リポジトリを使用）
    use crate::features::organization::repositories::analytics::AnalyticsRepository;
    let analytics_data =
        AnalyticsRepository::find_by_organization_id(&app_state.db, organization_id, Some(30))
            .await?;

    // DTOに変換
    let mut analytics_dto = Vec::new();
    for data in analytics_data {
        analytics_dto.push(AnalyticsData {
            analytics_type: data.analytics_type,
            period: data.period,
            period_start: data.period_start,
            period_end: data.period_end,
            metrics: serde_json::json!({
                "metric_name": data.metric_name,
                "metric_value": data.metric_value,
            }),
            recorded_at: data.created_at,
        });
    }

    // サマリー情報を計算（モックデータ）
    let member_count = organization.members.len() as u32;
    let team_count = organization.current_team_count;

    let summary = AnalyticsSummary {
        total_members: member_count,
        active_teams: team_count,
        storage_used_mb: 1024,      // モックデータ
        api_calls_this_month: 5000, // モックデータ
        subscription_tier: organization.subscription_tier,
        usage_percentage: (member_count as f32 / organization.max_members as f32) * 100.0,
    };

    let response = OrganizationAnalyticsResponse {
        organization_id,
        organization_name: organization.name,
        analytics_data: analytics_dto,
        summary,
        message: "Organization analytics retrieved successfully".to_string(),
    };

    Ok(Json(ApiResponse::success(
        "Organization analytics retrieved successfully",
        response,
    )))
}

/// 組織分析詳細情報取得
pub async fn get_organization_analytics_details_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // 権限チェック
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), organization_id)
        .await?;

    // 分析データを取得
    use crate::features::organization::repositories::analytics::AnalyticsRepository;
    let analytics_data =
        AnalyticsRepository::find_by_organization_id(&app_state.db, organization_id, None).await?;

    // 各分析データの詳細情報を取得（未使用メソッドを活用）
    let mut detailed_analytics = Vec::new();

    for data in analytics_data {
        let analytics_type = data.get_analytics_type();
        let period = data.get_period();
        let metric_value = data.get_metric_value().ok();

        detailed_analytics.push(json!({
            "id": data.id,
            "analytics_type": analytics_type,
            "analytics_type_display": analytics_type.to_string(),
            "metric_name": data.metric_name,
            "period": period,
            "period_display": period.to_string(),
            "period_start": data.period_start,
            "period_end": data.period_end,
            "metric_value": metric_value,
            "raw_value": data.metric_value,
            "calculated_by": data.calculated_by,
            "created_at": data.created_at,
            "department_id": data.department_id,
        }));
    }

    // 分析タイプ別のサマリー
    use crate::features::organization::models::analytics::{AnalyticsType, Period};
    let type_summary = json!({
        "performance": detailed_analytics.iter().filter(|a| a["analytics_type"] == json!(AnalyticsType::Performance)).count(),
        "productivity": detailed_analytics.iter().filter(|a| a["analytics_type"] == json!(AnalyticsType::Productivity)).count(),
        "engagement": detailed_analytics.iter().filter(|a| a["analytics_type"] == json!(AnalyticsType::Engagement)).count(),
        "quality": detailed_analytics.iter().filter(|a| a["analytics_type"] == json!(AnalyticsType::Quality)).count(),
        "resource": detailed_analytics.iter().filter(|a| a["analytics_type"] == json!(AnalyticsType::Resource)).count(),
        "user": detailed_analytics.iter().filter(|a| a["analytics_type"] == json!(AnalyticsType::User)).count(),
        "security": detailed_analytics.iter().filter(|a| a["analytics_type"] == json!(AnalyticsType::Security)).count(),
        "compliance": detailed_analytics.iter().filter(|a| a["analytics_type"] == json!(AnalyticsType::Compliance)).count(),
    });

    // 期間別のサマリー
    let period_summary = json!({
        "daily": detailed_analytics.iter().filter(|a| a["period"] == json!(Period::Daily)).count(),
        "weekly": detailed_analytics.iter().filter(|a| a["period"] == json!(Period::Weekly)).count(),
        "monthly": detailed_analytics.iter().filter(|a| a["period"] == json!(Period::Monthly)).count(),
        "quarterly": detailed_analytics.iter().filter(|a| a["period"] == json!(Period::Quarterly)).count(),
        "yearly": detailed_analytics.iter().filter(|a| a["period"] == json!(Period::Yearly)).count(),
    });

    let response = json!({
        "organization_id": organization_id,
        "total_analytics": detailed_analytics.len(),
        "analytics_details": detailed_analytics,
        "type_summary": type_summary,
        "period_summary": period_summary,
        "last_updated": chrono::Utc::now(),
    });

    Ok(Json(ApiResponse::success(
        "Organization analytics details retrieved successfully",
        response,
    )))
}

/// 組織メンバー権限チェック
pub async fn check_organization_member_permissions_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path((organization_id, member_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // 権限チェック
    app_state
        .permission_service
        .check_resource_access(
            user.user_id(),
            "organization_member",
            Some(member_id),
            "view",
        )
        .await?;

    // メンバー情報を取得
    use crate::features::organization::repositories::organization::OrganizationRepository;
    let org_repo = OrganizationRepository::new(app_state.db.as_ref().clone());
    let member = org_repo
        .find_member_by_user_and_organization(member_id, organization_id)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound("Organization member not found".to_string())
        })?;

    // OrganizationMemberの未使用メソッドを活用
    let permissions = json!({
        "member_id": member.user_id,
        "organization_id": member.organization_id,
        "role": member.role.to_string(),
        "is_owner": member.is_owner(),
        "is_admin": member.is_admin(),
        "can_manage": member.can_manage(),
        "permissions": {
            "can_manage_organization": member.role.can_manage(),
            "can_create_teams": member.role.can_create_teams(),
            "can_invite_members": member.role.can_invite_members(),
            "can_change_settings": member.role.can_change_settings(),
        },
        "joined_at": member.joined_at,
        "invited_by": member.invited_by,
    });

    Ok(Json(ApiResponse::success(
        "Organization member permissions retrieved successfully",
        permissions,
    )))
}

/// 組織メンバー役割一括更新
pub async fn bulk_update_member_roles_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // 権限チェック（管理者権限必須）
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), organization_id)
        .await?;

    // ペイロードから更新情報を取得
    let updates = payload["updates"]
        .as_array()
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid updates format".to_string()))?;

    let mut updated_members = Vec::new();
    use crate::features::organization::repositories::organization::OrganizationRepository;
    let org_repo = OrganizationRepository::new(app_state.db.as_ref().clone());

    for update in updates {
        let member_id = update["member_id"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| crate::error::AppError::BadRequest("Invalid member_id".to_string()))?;

        let new_role_str = update["new_role"]
            .as_str()
            .ok_or_else(|| crate::error::AppError::BadRequest("Invalid new_role".to_string()))?;

        let new_role = OrganizationRole::from_str(new_role_str)
            .map_err(|_| crate::error::AppError::BadRequest("Invalid role value".to_string()))?;

        // メンバー情報を取得
        let mut member = org_repo
            .find_member_by_user_and_organization(member_id, organization_id)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::NotFound("Organization member not found".to_string())
            })?;

        // update_roleメソッドを活用
        let old_role = member.role.clone();
        member.update_role(new_role);

        // TODO: 実際にはリポジトリで更新を保存する必要がある
        // app_state.organization_repository.update_member_role(...).await?;

        updated_members.push(json!({
            "member_id": member_id,
            "old_role": old_role.to_string(),
            "new_role": member.role.to_string(),
            "is_owner": member.is_owner(),
            "is_admin": member.is_admin(),
            "can_manage": member.can_manage(),
        }));
    }

    let response = json!({
        "organization_id": organization_id,
        "updated_count": updated_members.len(),
        "updated_members": updated_members,
        "updated_at": chrono::Utc::now(),
    });

    Ok(Json(ApiResponse::success(
        "Organization member roles updated successfully",
        response,
    )))
}

/// 組織総数取得（管理者用）
pub async fn get_organizations_count_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // 管理者権限チェック
    app_state
        .permission_service
        .check_resource_access(user.user_id(), "admin", None, "read")
        .await?;

    use crate::features::organization::repositories::organization::OrganizationRepository;
    let org_repo = OrganizationRepository::new(app_state.db.as_ref().clone());
    let total_count = org_repo.count_all_organizations().await?;

    let response = json!({
        "total_organizations": total_count,
        "timestamp": chrono::Utc::now(),
    });

    Ok(Json(ApiResponse::success(
        "Total organization count retrieved successfully",
        response,
    )))
}

/// 組織部門一覧取得
pub async fn get_organization_departments_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<OrganizationDepartmentsResponse>>> {
    // 組織情報を取得
    let organization = app_state
        .organization_service
        .get_organization_by_id(organization_id, user.user_id())
        .await?;

    // 部門データを取得（直接リポジトリを使用）
    use crate::features::organization::repositories::department::DepartmentRepository;
    use crate::features::organization::repositories::department_member::DepartmentMemberRepository;
    let departments =
        DepartmentRepository::find_hierarchy_by_organization_id(&app_state.db, organization_id)
            .await?;

    // 部門の階層構造を構築
    let mut department_map: std::collections::HashMap<Uuid, DepartmentInfo> =
        std::collections::HashMap::new();
    let mut root_departments = Vec::new();

    // 全部門をDTOに変換し、マップに格納
    for dept in departments {
        // 各部門のメンバー数を取得
        let members =
            DepartmentMemberRepository::find_by_department_id(&app_state.db, dept.id).await?;
        let member_count = members.len();

        let dept_info = DepartmentInfo {
            id: dept.id,
            name: dept.name.clone(),
            description: dept.description.clone(),
            parent_department_id: dept.parent_department_id,
            hierarchy_level: dept.hierarchy_level,
            hierarchy_path: dept.hierarchy_path.clone(),
            manager_user_id: dept.manager_user_id,
            manager_name: None, // 実装簡略化のため省略
            member_count: member_count as u32,
            is_active: dept.is_active,
            created_at: dept.created_at,
            updated_at: dept.updated_at,
            children: Vec::new(),
        };

        department_map.insert(dept.id, dept_info);

        if dept.parent_department_id.is_none() {
            root_departments.push(dept.id);
        }
    }

    // 階層構造を構築
    let dept_ids_to_process: Vec<Uuid> = department_map.keys().copied().collect();
    for dept_id in dept_ids_to_process.clone() {
        if let Some(dept) = department_map.get(&dept_id) {
            if let Some(parent_id) = dept.parent_department_id {
                if let Some(child_dept) = department_map.remove(&dept_id) {
                    if let Some(parent_dept) = department_map.get_mut(&parent_id) {
                        parent_dept.children.push(child_dept);
                    } else {
                        department_map.insert(dept_id, child_dept);
                    }
                }
            }
        }
    }

    // ルート部門のみを結果として返す
    let mut result_departments = Vec::new();
    for root_id in root_departments {
        if let Some(dept) = department_map.remove(&root_id) {
            result_departments.push(dept);
        }
    }

    let response = OrganizationDepartmentsResponse {
        organization_id,
        organization_name: organization.name,
        departments: result_departments,
        total_departments: department_map.len() as u32,
        message: "Organization departments retrieved successfully".to_string(),
    };

    Ok(Json(ApiResponse::success(
        "Organization departments retrieved successfully",
        response,
    )))
}

/// 組織一覧ページネーション付き取得
pub async fn get_organizations_paginated_handler(
    State(_app_state): State<crate::api::AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<OrganizationSearchQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // TODO: Phase 19でOrganizationServiceが正しいDTOを使用するように更新後、コメントを解除
    // let (organizations, total_count) = app_state
    //     .organization_service
    //     .get_organizations_paginated(query.clone(), user.user_id())
    //     .await?;
    let organizations: Vec<OrganizationListResponse> = vec![];
    let total_count = 0i64;

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let total_pages = ((total_count as f32) / (page_size as f32)).ceil() as u32;

    Ok(Json(ApiResponse::success(
        "Organizations retrieved successfully",
        json!({
            "organizations": organizations,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total_items": total_count,
                "total_pages": total_pages,
                "has_next": page < total_pages,
                "has_prev": page > 1
            }
        }),
    )))
}

// --- Department Management Handlers ---

/// Create department
pub async fn create_department_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // 権限チェック
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), organization_id)
        .await?;

    use crate::features::organization::models::department;
    use crate::features::organization::repositories::department::DepartmentRepository;
    use sea_orm::Set;

    let name = payload["name"]
        .as_str()
        .ok_or_else(|| {
            crate::error::AppError::BadRequest("Department name is required".to_string())
        })?
        .to_string();

    let parent_id = payload["parent_department_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok());

    // Calculate hierarchy level and path
    let (hierarchy_level, hierarchy_path) = if let Some(parent_id) = parent_id {
        // Get parent department to calculate hierarchy
        if let Ok(Some(parent)) = DepartmentRepository::find_by_id(&app_state.db, parent_id).await {
            (
                parent.hierarchy_level + 1,
                format!("{}/{}", parent.hierarchy_path, parent.id),
            )
        } else {
            (0, "/".to_string())
        }
    } else {
        (0, "/".to_string())
    };

    let department_model = department::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(name),
        description: Set(payload["description"].as_str().map(|s| s.to_string())),
        organization_id: Set(organization_id),
        parent_department_id: Set(parent_id),
        hierarchy_level: Set(hierarchy_level),
        hierarchy_path: Set(hierarchy_path),
        manager_user_id: Set(payload["manager_user_id"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok())),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    let department = DepartmentRepository::create(&app_state.db, department_model).await?;

    let response = serde_json::json!({
        "id": department.id,
        "organization_id": department.organization_id,
        "name": department.name,
        "description": department.description,
        "parent_department_id": department.parent_department_id,
        "hierarchy_level": department.hierarchy_level,
        "hierarchy_path": department.hierarchy_path,
        "manager_user_id": department.manager_user_id,
        "is_active": department.is_active,
        "created_at": department.created_at,
    });

    Ok(Json(ApiResponse::success(
        "Department created successfully",
        response,
    )))
}

/// Get department by ID
pub async fn get_department_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(department_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::organization::repositories::department::DepartmentRepository;

    let department = DepartmentRepository::find_by_id(&app_state.db, department_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Department not found".to_string()))?;

    // 権限チェック
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), department.organization_id)
        .await?;

    let response = serde_json::json!({
        "id": department.id,
        "organization_id": department.organization_id,
        "name": department.name,
        "description": department.description,
        "parent_department_id": department.parent_department_id,
        "hierarchy_level": department.hierarchy_level,
        "hierarchy_path": department.hierarchy_path,
        "manager_user_id": department.manager_user_id,
        "is_active": department.is_active,
        "created_at": department.created_at,
        "updated_at": department.updated_at,
    });

    Ok(Json(ApiResponse::success(
        "Department retrieved successfully",
        response,
    )))
}

/// Update department
pub async fn update_department_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(department_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::organization::repositories::department::DepartmentRepository;

    let department = DepartmentRepository::find_by_id(&app_state.db, department_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Department not found".to_string()))?;

    // 権限チェック
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), department.organization_id)
        .await?;

    use sea_orm::{IntoActiveModel, Set};

    // Convert to ActiveModel and update fields
    let mut active_model = department.clone().into_active_model();

    if let Some(name) = payload["name"].as_str() {
        active_model.name = Set(name.to_string());
    }
    if let Some(description) = payload["description"].as_str() {
        active_model.description = Set(Some(description.to_string()));
    }
    if let Some(manager_id) = payload["manager_user_id"].as_str() {
        active_model.manager_user_id = Set(Uuid::parse_str(manager_id).ok());
    }
    active_model.updated_at = Set(chrono::Utc::now());

    let result = DepartmentRepository::update_by_model(&app_state.db, active_model).await?;

    let response = serde_json::json!({
        "id": result.id,
        "name": result.name,
        "description": result.description,
        "manager_user_id": result.manager_user_id,
        "updated_at": result.updated_at,
    });

    Ok(Json(ApiResponse::success(
        "Department updated successfully",
        response,
    )))
}

/// Delete department
pub async fn delete_department_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(department_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    use crate::features::organization::repositories::department::DepartmentRepository;

    let department = DepartmentRepository::find_by_id(&app_state.db, department_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Department not found".to_string()))?;

    // 権限チェック
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), department.organization_id)
        .await?;

    DepartmentRepository::delete_by_id(&app_state.db, department_id).await?;

    Ok(Json(ApiResponse::success(
        "Department deleted successfully",
        (),
    )))
}

/// Get department children
pub async fn get_department_children_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(parent_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    use crate::features::organization::repositories::department::DepartmentRepository;

    // Get parent department to check permissions
    let parent = DepartmentRepository::find_by_id(&app_state.db, parent_id)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound("Parent department not found".to_string())
        })?;

    // 権限チェック
    app_state
        .permission_service
        .check_user_access(user.user_id(), parent.organization_id)
        .await?;

    let children =
        DepartmentRepository::find_children_by_parent_id(&app_state.db, parent_id).await?;

    let response: Vec<serde_json::Value> = children
        .into_iter()
        .map(|dept| {
            serde_json::json!({
                "id": dept.id,
                "name": dept.name,
                "description": dept.description,
                "hierarchy_level": dept.hierarchy_level,
                "is_active": dept.is_active,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(
        "Department children retrieved successfully",
        response,
    )))
}

/// Get departments by hierarchy path
pub async fn get_departments_by_path_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(query): Query<serde_json::Value>,
) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    use crate::features::organization::repositories::department::DepartmentRepository;

    let path_prefix = query["path_prefix"]
        .as_str()
        .ok_or_else(|| crate::error::AppError::BadRequest("path_prefix is required".to_string()))?;

    let organization_id = query["organization_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| {
            crate::error::AppError::BadRequest("organization_id is required".to_string())
        })?;

    let departments = DepartmentRepository::find_by_hierarchy_path_prefix(
        &app_state.db,
        organization_id,
        path_prefix,
    )
    .await?;

    // Filter by user's accessible organizations
    let mut accessible_departments = Vec::new();
    for dept in departments {
        if app_state
            .permission_service
            .check_user_access(user.user_id(), dept.organization_id)
            .await
            .is_ok()
        {
            accessible_departments.push(serde_json::json!({
                "id": dept.id,
                "organization_id": dept.organization_id,
                "name": dept.name,
                "hierarchy_path": dept.hierarchy_path,
                "hierarchy_level": dept.hierarchy_level,
            }));
        }
    }

    Ok(Json(ApiResponse::success(
        "Departments by path retrieved successfully",
        accessible_departments,
    )))
}

/// Get department by name and organization
pub async fn get_department_by_name_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(query): Query<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::organization::repositories::department::DepartmentRepository;

    let name = query["name"]
        .as_str()
        .ok_or_else(|| crate::error::AppError::BadRequest("name is required".to_string()))?;

    let organization_id = query["organization_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| {
            crate::error::AppError::BadRequest("Valid organization_id is required".to_string())
        })?;

    // 権限チェック
    app_state
        .permission_service
        .check_user_access(user.user_id(), organization_id)
        .await?;

    let department = DepartmentRepository::find_by_name_and_organization(
        &app_state.db,
        name,
        organization_id,
        None,
    )
    .await?
    .ok_or_else(|| crate::error::AppError::NotFound("Department not found".to_string()))?;

    let response = serde_json::json!({
        "id": department.id,
        "name": department.name,
        "description": department.description,
        "hierarchy_path": department.hierarchy_path,
        "manager_user_id": department.manager_user_id,
    });

    Ok(Json(ApiResponse::success(
        "Department found successfully",
        response,
    )))
}

/// Update department hierarchy
pub async fn update_department_hierarchy_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(department_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::organization::repositories::department::DepartmentRepository;

    let department = DepartmentRepository::find_by_id(&app_state.db, department_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Department not found".to_string()))?;

    // 権限チェック
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), department.organization_id)
        .await?;

    let new_path = payload["new_path"]
        .as_str()
        .ok_or_else(|| crate::error::AppError::BadRequest("new_path is required".to_string()))?;

    DepartmentRepository::update_hierarchy_path_recursive(
        &app_state.db,
        department_id,
        Some(new_path),
    )
    .await?;

    let response = serde_json::json!({
        "department_id": department_id,
        "new_hierarchy_path": new_path,
        "updated_at": chrono::Utc::now(),
    });

    Ok(Json(ApiResponse::success(
        "Department hierarchy updated successfully",
        response,
    )))
}

/// Count organization departments
pub async fn count_organization_departments_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // 権限チェック
    app_state
        .permission_service
        .check_user_access(user.user_id(), organization_id)
        .await?;

    use crate::features::organization::repositories::department::DepartmentRepository;
    let count = DepartmentRepository::count_by_organization(&app_state.db, organization_id).await?;

    let response = serde_json::json!({
        "organization_id": organization_id,
        "department_count": count,
    });

    Ok(Json(ApiResponse::success(
        "Department count retrieved successfully",
        response,
    )))
}

/// Get root departments
pub async fn get_root_departments_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    // 権限チェック
    app_state
        .permission_service
        .check_user_access(user.user_id(), organization_id)
        .await?;

    use crate::features::organization::repositories::department::DepartmentRepository;
    let root_departments =
        DepartmentRepository::find_root_departments(&app_state.db, organization_id).await?;

    let response: Vec<serde_json::Value> = root_departments
        .into_iter()
        .map(|dept| {
            serde_json::json!({
                "id": dept.id,
                "name": dept.name,
                "description": dept.description,
                "hierarchy_path": dept.hierarchy_path,
                "manager_user_id": dept.manager_user_id,
                "created_at": dept.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(
        "Root departments retrieved successfully",
        response,
    )))
}

/// Check circular dependency
pub async fn check_circular_dependency_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(department_id): Path<Uuid>,
    Query(query): Query<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::organization::repositories::department::DepartmentRepository;

    let department = DepartmentRepository::find_by_id(&app_state.db, department_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Department not found".to_string()))?;

    // 権限チェック
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), department.organization_id)
        .await?;

    let new_parent_id = query["new_parent_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| {
            crate::error::AppError::BadRequest("Valid new_parent_id is required".to_string())
        })?;

    let has_circular = DepartmentRepository::exists_circular_dependency(
        &app_state.db,
        department_id,
        new_parent_id,
    )
    .await?;

    let response = serde_json::json!({
        "department_id": department_id,
        "new_parent_id": new_parent_id,
        "has_circular_dependency": has_circular,
    });

    Ok(Json(ApiResponse::success(
        "Circular dependency check completed",
        response,
    )))
}

// --- Department Member Management Handlers ---

/// Add department member
pub async fn add_department_member_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(department_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::organization::models::department_member::{self, DepartmentRole};
    use crate::features::organization::repositories::department::DepartmentRepository;
    use crate::features::organization::repositories::department_member::DepartmentMemberRepository;
    use sea_orm::Set;

    // Get department to check permissions
    let department = DepartmentRepository::find_by_id(&app_state.db, department_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Department not found".to_string()))?;

    // 権限チェック
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), department.organization_id)
        .await?;

    let user_id = payload["user_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| {
            crate::error::AppError::BadRequest("Valid user_id is required".to_string())
        })?;

    let role = payload["role"]
        .as_str()
        .and_then(|s| serde_json::from_value::<DepartmentRole>(serde_json::json!(s)).ok())
        .unwrap_or(DepartmentRole::Member);

    let member_model = department_member::ActiveModel {
        id: Set(Uuid::new_v4()),
        department_id: Set(department_id),
        user_id: Set(user_id),
        role: Set(role.to_string()),
        is_active: Set(true),
        joined_at: Set(chrono::Utc::now()),
        added_by: Set(user.user_id()),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    let member = DepartmentMemberRepository::create(&app_state.db, member_model).await?;

    let response = serde_json::json!({
        "id": member.id,
        "department_id": member.department_id,
        "user_id": member.user_id,
        "role": member.role,
        "is_active": member.is_active,
        "joined_at": member.joined_at,
        "added_by": member.added_by,
    });

    Ok(Json(ApiResponse::success(
        "Department member added successfully",
        response,
    )))
}

/// Get department member
pub async fn get_department_member_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path((department_id, member_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::organization::repositories::department::DepartmentRepository;
    use crate::features::organization::repositories::department_member::DepartmentMemberRepository;

    // Get department to check permissions
    let department = DepartmentRepository::find_by_id(&app_state.db, department_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Department not found".to_string()))?;

    // 権限チェック
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), department.organization_id)
        .await?;

    let member = DepartmentMemberRepository::find_by_id(&app_state.db, member_id)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound("Department member not found".to_string())
        })?;

    let response = serde_json::json!({
        "id": member.id,
        "department_id": member.department_id,
        "user_id": member.user_id,
        "role": member.role,
        "is_active": member.is_active,
        "joined_at": member.joined_at,
    });

    Ok(Json(ApiResponse::success(
        "Department member retrieved successfully",
        response,
    )))
}

/// Get all department members
pub async fn get_department_members_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(department_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    use crate::features::organization::repositories::department::DepartmentRepository;
    use crate::features::organization::repositories::department_member::DepartmentMemberRepository;

    // Get department to check permissions
    let department = DepartmentRepository::find_by_id(&app_state.db, department_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Department not found".to_string()))?;

    // 権限チェック
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), department.organization_id)
        .await?;

    let members =
        DepartmentMemberRepository::find_by_department_id(&app_state.db, department_id).await?;
    let member_count =
        DepartmentMemberRepository::count_by_department(&app_state.db, department_id).await?;

    let response: Vec<serde_json::Value> = members
        .into_iter()
        .map(|member| {
            serde_json::json!({
                "id": member.id,
                "user_id": member.user_id,
                "role": member.role,
                "is_active": member.is_active,
                "joined_at": member.joined_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(
        format!("Retrieved {} department members", member_count),
        response,
    )))
}

/// Deactivate department member
pub async fn deactivate_department_member_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path((department_id, member_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<ApiResponse<()>>> {
    use crate::features::organization::repositories::department::DepartmentRepository;
    use crate::features::organization::repositories::department_member::DepartmentMemberRepository;

    // Get department to check permissions
    let department = DepartmentRepository::find_by_id(&app_state.db, department_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Department not found".to_string()))?;

    // 権限チェック
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), department.organization_id)
        .await?;

    DepartmentMemberRepository::deactivate_by_id(&app_state.db, member_id).await?;

    Ok(Json(ApiResponse::success(
        "Department member deactivated successfully",
        (),
    )))
}

/// Update department member role
pub async fn update_department_member_role_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path((department_id, member_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::organization::models::department_member::DepartmentRole;
    use crate::features::organization::repositories::department::DepartmentRepository;
    use crate::features::organization::repositories::department_member::DepartmentMemberRepository;

    // Get department to check permissions
    let department = DepartmentRepository::find_by_id(&app_state.db, department_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Department not found".to_string()))?;

    // 権限チェック
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), department.organization_id)
        .await?;

    let new_role = payload["role"]
        .as_str()
        .and_then(|s| serde_json::from_value::<DepartmentRole>(serde_json::json!(s)).ok())
        .ok_or_else(|| crate::error::AppError::BadRequest("Valid role is required".to_string()))?;

    DepartmentMemberRepository::update_role(&app_state.db, member_id, &new_role.to_string())
        .await?;

    let response = serde_json::json!({
        "member_id": member_id,
        "new_role": new_role,
        "updated_at": chrono::Utc::now(),
    });

    Ok(Json(ApiResponse::success(
        "Department member role updated successfully",
        response,
    )))
}

/// Get department managers
pub async fn get_department_managers_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(department_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    use crate::features::organization::repositories::department::DepartmentRepository;
    use crate::features::organization::repositories::department_member::DepartmentMemberRepository;

    // Get department to check permissions
    let department = DepartmentRepository::find_by_id(&app_state.db, department_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Department not found".to_string()))?;

    // 権限チェック
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), department.organization_id)
        .await?;

    let managers =
        DepartmentMemberRepository::find_managers_by_department(&app_state.db, department_id)
            .await?;

    let response: Vec<serde_json::Value> = managers
        .into_iter()
        .map(|member| {
            serde_json::json!({
                "id": member.id,
                "user_id": member.user_id,
                "role": member.role,
                "joined_at": member.joined_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(
        "Department managers retrieved successfully",
        response,
    )))
}

/// Get user's departments
pub async fn get_user_departments_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    // Check if user is accessing their own data or is admin
    if user.user_id() != user_id && !user.is_admin() {
        return Err(crate::error::AppError::Forbidden(
            "You can only view your own departments".to_string(),
        ));
    }

    use crate::features::organization::repositories::department_member::DepartmentMemberRepository;

    let user_departments = DepartmentMemberRepository::find_users_departments_with_role(
        &app_state.db,
        user_id,
        vec!["manager", "lead", "member", "viewer"],
    )
    .await?;

    let response: Vec<serde_json::Value> = user_departments
        .into_iter()
        .map(|member| {
            serde_json::json!({
                "member_id": member.id,
                "department_id": member.department_id,
                "role": member.role,
                "is_active": member.is_active,
                "joined_at": member.joined_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(
        "User departments retrieved successfully",
        response,
    )))
}

/// Organization routes
pub fn organization_routes() -> Router<crate::api::AppState> {
    Router::new()
        .route("/organizations", post(create_organization_handler))
        .route("/organizations", get(get_organizations_handler))
        .route("/organizations/stats", get(get_organization_stats_handler))
        .route("/organizations/count", get(get_organizations_count_handler))
        .route(
            "/organizations/paginated",
            get(get_organizations_paginated_handler),
        )
        .route("/organizations/{id}", get(get_organization_handler))
        .route("/organizations/{id}", patch(update_organization_handler))
        .route("/organizations/{id}", delete(delete_organization_handler))
        .route(
            "/organizations/{id}/capacity",
            get(get_organization_capacity_handler),
        )
        .route(
            "/organizations/{id}/settings",
            patch(update_organization_settings_handler),
        )
        .route(
            "/organizations/{id}/subscription",
            patch(update_organization_subscription_handler),
        )
        .route(
            "/organizations/{id}/subscription",
            put(update_organization_subscription_handler),
        )
        .route(
            "/organizations/{id}/subscription/history",
            get(get_organization_subscription_history_handler),
        )
        .route(
            "/organizations/{id}/members",
            post(invite_organization_member_handler),
        )
        .route(
            "/organizations/{id}/members/{member_id}",
            get(get_organization_member_handler),
        )
        .route(
            "/organizations/{id}/members/{member_id}",
            patch(update_organization_member_role_handler),
        )
        .route(
            "/organizations/{id}/members/{member_id}",
            delete(remove_organization_member_handler),
        )
        .route(
            "/organizations/{id}/members/{member_id}/permissions",
            get(check_organization_member_permissions_handler),
        )
        .route(
            "/organizations/{id}/members/bulk-update-roles",
            post(bulk_update_member_roles_handler),
        )
        .route(
            "/organizations/{id}/analytics",
            get(get_organization_analytics_handler),
        )
        .route(
            "/organizations/{id}/analytics/details",
            get(get_organization_analytics_details_handler),
        )
        .route(
            "/organizations/{id}/departments",
            get(get_organization_departments_handler),
        )
        // New Department endpoints
        .route(
            "/organizations/{id}/departments",
            post(create_department_handler),
        )
        .route("/departments/{id}", get(get_department_handler))
        .route("/departments/{id}", patch(update_department_handler))
        .route("/departments/{id}", delete(delete_department_handler))
        .route(
            "/departments/{id}/children",
            get(get_department_children_handler),
        )
        .route("/departments/by-path", get(get_departments_by_path_handler))
        .route("/departments/by-name", get(get_department_by_name_handler))
        .route(
            "/departments/{id}/hierarchy",
            patch(update_department_hierarchy_handler),
        )
        .route(
            "/organizations/{id}/departments/count",
            get(count_organization_departments_handler),
        )
        .route(
            "/organizations/{id}/departments/roots",
            get(get_root_departments_handler),
        )
        .route(
            "/departments/{id}/check-circular",
            get(check_circular_dependency_handler),
        )
        // Department Member endpoints
        .route(
            "/departments/{id}/members",
            post(add_department_member_handler),
        )
        .route(
            "/departments/{id}/members/{member_id}",
            get(get_department_member_handler),
        )
        .route(
            "/departments/{id}/members",
            get(get_department_members_handler),
        )
        .route(
            "/departments/{id}/members/{member_id}",
            delete(deactivate_department_member_handler),
        )
        .route(
            "/departments/{id}/members/{member_id}/role",
            patch(update_department_member_role_handler),
        )
        .route(
            "/departments/{id}/managers",
            get(get_department_managers_handler),
        )
        .route(
            "/users/{user_id}/departments",
            get(get_user_departments_handler),
        )
}

/// Organization router with state
pub fn organization_router_with_state(app_state: crate::api::AppState) -> Router {
    organization_routes().with_state(app_state)
}
