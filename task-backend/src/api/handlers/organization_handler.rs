// task-backend/src/api/handlers/organization_handler.rs

use crate::api::dto::organization_dto::*;
use crate::error::AppResult;
use crate::features::auth::middleware::AuthenticatedUser;
use crate::shared::types::ApiResponse;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
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

    // 組織管理権限チェック（PermissionServiceを使用）
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
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    // 組織管理権限チェック（PermissionServiceを使用）
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), organization_id)
        .await?;

    app_state
        .organization_service
        .delete_organization(organization_id, user.user_id())
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        Json(json!({
            "success": true,
            "message": "Organization deleted successfully"
        })),
    ))
}

/// 組織メンバー招待
pub async fn invite_organization_member_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
    Json(payload): Json<InviteOrganizationMemberRequest>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    // バリデーション
    payload.validate()?;

    let member_response = app_state
        .organization_service
        .invite_organization_member(organization_id, payload, user.user_id())
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": member_response,
            "message": "Organization member invited successfully"
        })),
    ))
}

/// 組織メンバー役割更新
pub async fn update_organization_member_role_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path((organization_id, member_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateOrganizationMemberRoleRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let member_response = app_state
        .organization_service
        .update_organization_member_role(organization_id, member_id, payload, user.user_id())
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": member_response,
        "message": "Organization member role updated successfully"
    })))
}

/// 組織メンバー削除
pub async fn remove_organization_member_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path((organization_id, member_id)): Path<(Uuid, Uuid)>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    app_state
        .organization_service
        .remove_organization_member(organization_id, member_id, user.user_id())
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        Json(json!({
            "success": true,
            "message": "Organization member removed successfully"
        })),
    ))
}

/// 組織設定更新
pub async fn update_organization_settings_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
    Json(payload): Json<UpdateOrganizationSettingsRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // 組織管理権限チェック（PermissionServiceを使用）
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), organization_id)
        .await?;

    let organization_response = app_state
        .organization_service
        .update_organization_settings(organization_id, payload, user.user_id())
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": organization_response,
        "message": "Organization settings updated successfully"
    })))
}

/// 組織統計取得
pub async fn get_organization_stats_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<serde_json::Value>> {
    let stats = app_state
        .organization_service
        .get_organization_stats(user.user_id())
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": stats,
        "message": "Organization statistics retrieved successfully"
    })))
}

/// 組織メンバー詳細取得（権限情報付き）
pub async fn get_organization_member_details_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path((organization_id, member_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<ApiResponse<OrganizationMemberDetailResponse>>> {
    let member_detail = app_state
        .organization_service
        .get_organization_member_detail(organization_id, member_id, user.user_id())
        .await?;

    Ok(Json(ApiResponse::success(
        "Organization member details retrieved successfully",
        member_detail,
    )))
}

/// 組織容量チェック
pub async fn check_organization_capacity_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<OrganizationCapacityResponse>>> {
    let capacity = app_state
        .organization_service
        .check_organization_capacity(organization_id, user.user_id())
        .await?;

    Ok(Json(ApiResponse::success(
        "Organization capacity retrieved successfully",
        capacity,
    )))
}

/// 組織一覧をページネーション付きで取得
pub async fn get_organizations_paginated_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(query): Query<OrganizationSearchQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let (organizations, total_count) = app_state
        .organization_service
        .get_organizations_paginated(query, user.user_id())
        .await?;

    Ok(Json(ApiResponse::success(
        "Organizations retrieved successfully",
        json!({
            "organizations": organizations,
            "total_count": total_count,
            "page": 1,
            "per_page": 20
        }),
    )))
}

/// 組織のサブスクリプション階層を更新
pub async fn update_organization_subscription_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
    Json(payload): Json<UpdateOrganizationSubscriptionRequest>,
) -> AppResult<Json<ApiResponse<OrganizationResponse>>> {
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

/// 組織のサブスクリプション履歴を取得
pub async fn get_organization_subscription_history_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(organization_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    // 組織へのアクセス権限をチェック
    app_state
        .permission_service
        .check_organization_management_permission(user.user_id(), organization_id)
        .await?;

    // 組織を取得してオーナーIDを確認
    let organization = app_state
        .organization_service
        .get_organization_by_id(organization_id, user.user_id())
        .await?;

    // オーナーのサブスクリプション履歴を取得
    let history = app_state
        .subscription_history_repo
        .find_by_user_id(organization.owner_id)
        .await?;

    // レスポンス形式に変換
    let history_response: Vec<serde_json::Value> = history
        .into_iter()
        .map(|h| {
            json!({
                "id": h.id,
                "user_id": h.user_id,
                "old_tier": h.previous_tier,
                "new_tier": h.new_tier,
                "changed_at": h.changed_at,
                "changed_by": h.changed_by,
                "reason": h.reason,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(
        "Subscription history retrieved successfully",
        history_response,
    )))
}

/// 組織ルーターを構築
pub fn organization_router_with_state(app_state: crate::api::AppState) -> axum::Router {
    use axum::{
        routing::{delete, get, patch, post, put},
        Router,
    };

    Router::new()
        // 静的ルートを先に定義
        .route("/organizations/stats", get(get_organization_stats_handler))
        .route(
            "/organizations/paginated",
            get(get_organizations_paginated_handler),
        )
        // リソースルート
        .route("/organizations", post(create_organization_handler))
        .route("/organizations", get(get_organizations_handler))
        .route("/organizations/{id}", get(get_organization_handler))
        .route("/organizations/{id}", patch(update_organization_handler))
        .route("/organizations/{id}", delete(delete_organization_handler))
        .route(
            "/organizations/{id}/capacity",
            get(check_organization_capacity_handler),
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
            get(get_organization_member_details_handler),
        )
        .route(
            "/organizations/{id}/members/{member_id}",
            patch(update_organization_member_role_handler),
        )
        .route(
            "/organizations/{id}/members/{member_id}",
            delete(remove_organization_member_handler),
        )
        .with_state(app_state)
}
