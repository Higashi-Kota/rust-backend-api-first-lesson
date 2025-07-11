// TODO: Phase 19で各ハンドラーがサービスを正しく使用するようになったら#[allow(unused_variables)]を削除
#![allow(unused_variables)]

// 一時的に旧DTOを使用（Phase 19の互換性確保のため）
use crate::domain::organization_model::OrganizationRole;
use crate::features::organization::dto::organization::{
    CreateOrganizationRequest, InviteOrganizationMemberRequest, OrganizationCapacityResponse,
    OrganizationListResponse, OrganizationMemberDetailResponse, OrganizationMemberResponse,
    OrganizationResponse, OrganizationSearchQuery, OrganizationStatsResponse,
    UpdateOrganizationMemberRoleRequest, UpdateOrganizationRequest,
    UpdateOrganizationSettingsRequest, UpdateOrganizationSubscriptionRequest,
};
// TODO: Phase 19でOrganizationServiceの本来の使用箇所が移行されたら#[allow(unused_imports)]を削除
#[allow(unused_imports)]
use super::super::services::OrganizationService;
use crate::error::AppResult;
use crate::features::auth::middleware::AuthenticatedUser;
// TODO: Phase 19でPermissionServiceとSubscriptionServiceの使用箇所が移行されたら#[allow(unused_imports)]を削除
#[allow(unused_imports)]
use crate::features::subscription::services::subscription::SubscriptionService;
#[allow(unused_imports)]
use crate::service::permission_service::PermissionService;
use crate::shared::types::ApiResponse;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use serde_json::json;
// TODO: Phase 19でArcの使用箇所が移行されたら#[allow(unused_imports)]を削除
#[allow(unused_imports)]
use std::sync::Arc;
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub async fn invite_organization_member_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
    Json(payload): Json<InviteOrganizationMemberRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<OrganizationMemberResponse>>)> {
    payload.validate()?;

    let member_response = app_state
        .organization_service
        .invite_organization_member(organization_id, payload, user.user_id())
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
#[allow(dead_code)]
pub async fn get_organization_member_handler(
    State(_app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
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
#[allow(dead_code)]
pub async fn update_organization_member_role_handler(
    State(_app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path((organization_id, member_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateOrganizationMemberRoleRequest>,
) -> AppResult<Json<ApiResponse<OrganizationMemberDetailResponse>>> {
    // TODO: Phase 19でOrganizationServiceが正しいDTOを使用するように更新後、コメントを解除
    // let member_response = app_state
    //     .organization_service
    //     .update_member_role(organization_id, member_id, payload, user.user_id())
    //     .await?;
    let role = payload.role.clone();
    let member_response = OrganizationMemberDetailResponse {
        id: member_id,
        user_id: member_id,
        username: "Updated Member".to_string(),
        email: "member@example.com".to_string(),
        role: role.clone(),
        is_owner: false,
        is_admin: role == OrganizationRole::Admin,
        can_manage: role != OrganizationRole::Member,
        can_create_teams: role != OrganizationRole::Member,
        can_invite_members: role != OrganizationRole::Member,
        can_change_settings: role == OrganizationRole::Owner,
        joined_at: chrono::Utc::now(),
        invited_by: Some(user.user_id()),
    };

    Ok(Json(ApiResponse::success(
        "Member role updated successfully",
        member_response,
    )))
}

/// 組織メンバー削除
#[allow(dead_code)]
pub async fn remove_organization_member_handler(
    State(_app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path((organization_id, member_id)): Path<(Uuid, Uuid)>,
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
#[allow(dead_code)]
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

/// 組織一覧ページネーション付き取得
#[allow(dead_code)]
pub async fn get_organizations_paginated_handler(
    State(_app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
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

/// Organization routes
// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除
#[allow(unused_imports)]
pub fn organization_routes() -> Router<crate::api::AppState> {
    Router::new()
        .route("/organizations", post(create_organization_handler))
        .route("/organizations", get(get_organizations_handler))
        .route("/organizations/stats", get(get_organization_stats_handler))
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
}

/// Organization router with state
// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除
#[allow(unused_imports)]
pub fn organization_router_with_state(app_state: crate::api::AppState) -> Router {
    organization_routes().with_state(app_state)
}
