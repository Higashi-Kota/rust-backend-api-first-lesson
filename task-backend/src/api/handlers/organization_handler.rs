// task-backend/src/api/handlers/organization_handler.rs

use crate::api::dto::common::ApiResponse;
use crate::api::dto::organization_dto::*;
use crate::error::AppResult;
use crate::middleware::auth::AuthenticatedUser;
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
    let organizations = app_state
        .organization_service
        .get_organizations(query, user.user_id())
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

/// 組織ルーターを構築
pub fn organization_router_with_state(app_state: crate::api::AppState) -> axum::Router {
    use axum::{
        routing::{delete, get, patch, post},
        Router,
    };

    Router::new()
        .route("/organizations", post(create_organization_handler))
        .route("/organizations", get(get_organizations_handler))
        .route("/organizations/{id}", get(get_organization_handler))
        .route("/organizations/{id}", patch(update_organization_handler))
        .route("/organizations/{id}", delete(delete_organization_handler))
        .route(
            "/organizations/{id}/settings",
            patch(update_organization_settings_handler),
        )
        .route(
            "/organizations/{id}/members",
            post(invite_organization_member_handler),
        )
        .route(
            "/organizations/{id}/members/{member_id}",
            patch(update_organization_member_role_handler),
        )
        .route(
            "/organizations/{id}/members/{member_id}",
            delete(remove_organization_member_handler),
        )
        .route("/organizations/stats", get(get_organization_stats_handler))
        .with_state(app_state)
}
