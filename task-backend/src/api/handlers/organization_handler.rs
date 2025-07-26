// task-backend/src/api/handlers/organization_handler.rs

use crate::api::dto::organization_dto::*;
use crate::api::dto::organization_query_dto::OrganizationSearchQuery;
use crate::domain::organization_model::Organization;
use crate::error::AppResult;
use crate::extractors::{deserialize_uuid, ValidatedMultiPath, ValidatedUuid};
use crate::middleware::auth::AuthenticatedUser;
use crate::middleware::authorization::{resources, Action};
use crate::require_permission;
use crate::shared::types::PaginatedResponse;
use crate::types::ApiResponse;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

// 複数パラメータ用のPath構造体
#[derive(Deserialize)]
pub struct OrganizationMemberPath {
    #[serde(deserialize_with = "deserialize_uuid")]
    pub organization_id: Uuid,
    #[serde(deserialize_with = "deserialize_uuid")]
    pub member_id: Uuid,
}

/// 組織作成
pub async fn create_organization_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateOrganizationRequest>,
) -> AppResult<(StatusCode, ApiResponse<OrganizationResponse>)> {
    // バリデーション
    payload.validate()?;

    let organization_response = app_state
        .organization_service
        .create_organization(payload, user.user_id())
        .await?;

    Ok((
        StatusCode::CREATED,
        ApiResponse::success(organization_response),
    ))
}

/// 組織詳細取得
pub async fn get_organization_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    ValidatedUuid(organization_id): ValidatedUuid,
) -> AppResult<ApiResponse<OrganizationResponse>> {
    let organization_response = app_state
        .organization_service
        .get_organization_by_id(organization_id, user.user_id())
        .await?;

    Ok(ApiResponse::success(organization_response))
}

/// 組織一覧取得
pub async fn get_organizations_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(query): Query<OrganizationSearchQuery>,
) -> AppResult<ApiResponse<Vec<OrganizationListResponse>>> {
    let (organizations, _) = app_state
        .organization_service
        .get_organizations_paginated(query, user.user_id())
        .await?;

    Ok(ApiResponse::success(organizations))
}

/// 組織更新
pub async fn update_organization_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    ValidatedUuid(organization_id): ValidatedUuid,
    Json(payload): Json<UpdateOrganizationRequest>,
) -> AppResult<ApiResponse<OrganizationResponse>> {
    // バリデーション
    payload.validate()?;

    // 権限チェックはミドルウェアで実施済み

    let organization_response = app_state
        .organization_service
        .update_organization(organization_id, payload, user.user_id())
        .await?;

    Ok(ApiResponse::success(organization_response))
}

/// 組織削除
pub async fn delete_organization_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    ValidatedUuid(organization_id): ValidatedUuid,
) -> AppResult<(StatusCode, ApiResponse<serde_json::Value>)> {
    // 権限チェックはミドルウェアで実施済み

    app_state
        .organization_service
        .delete_organization(organization_id, user.user_id())
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        ApiResponse::success(json!({
            "message": "Organization deleted successfully"
        })),
    ))
}

/// 組織メンバー招待
pub async fn invite_organization_member_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    ValidatedUuid(organization_id): ValidatedUuid,
    Json(payload): Json<InviteOrganizationMemberRequest>,
) -> AppResult<(StatusCode, ApiResponse<serde_json::Value>)> {
    // バリデーション
    payload.validate()?;

    let member_response = app_state
        .organization_service
        .invite_organization_member(organization_id, payload, user.user_id())
        .await?;

    Ok((
        StatusCode::CREATED,
        ApiResponse::success(json!({
            "member": member_response,
            "message": "Organization member invited successfully"
        })),
    ))
}

/// 組織メンバー役割更新
pub async fn update_organization_member_role_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    ValidatedMultiPath(params): ValidatedMultiPath<OrganizationMemberPath>,
    Json(payload): Json<UpdateOrganizationMemberRoleRequest>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    let member_response = app_state
        .organization_service
        .update_organization_member_role(
            params.organization_id,
            params.member_id,
            payload,
            user.user_id(),
        )
        .await?;

    Ok(ApiResponse::success(json!({
        "member": member_response,
        "message": "Organization member role updated successfully"
    })))
}

/// 組織メンバー削除
pub async fn remove_organization_member_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    ValidatedMultiPath(params): ValidatedMultiPath<OrganizationMemberPath>,
) -> AppResult<(StatusCode, ApiResponse<serde_json::Value>)> {
    app_state
        .organization_service
        .remove_organization_member(params.organization_id, params.member_id, user.user_id())
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        ApiResponse::success(json!({
            "message": "Organization member removed successfully"
        })),
    ))
}

/// 組織設定更新
pub async fn update_organization_settings_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    ValidatedUuid(organization_id): ValidatedUuid,
    Json(payload): Json<UpdateOrganizationSettingsRequest>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    // 権限チェックはミドルウェアで実施済み

    let organization_response = app_state
        .organization_service
        .update_organization_settings(organization_id, payload, user.user_id())
        .await?;

    Ok(ApiResponse::success(json!({
        "organization": organization_response,
        "message": "Organization settings updated successfully"
    })))
}

/// 組織統計取得
pub async fn get_organization_stats_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
) -> AppResult<ApiResponse<OrganizationStatsResponse>> {
    let stats = app_state
        .organization_service
        .get_organization_stats(user.user_id())
        .await?;

    Ok(ApiResponse::success(stats))
}

/// 組織メンバー詳細取得（権限情報付き）
pub async fn get_organization_member_details_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    ValidatedMultiPath(params): ValidatedMultiPath<OrganizationMemberPath>,
) -> AppResult<ApiResponse<OrganizationMemberDetailResponse>> {
    let member_detail = app_state
        .organization_service
        .get_organization_member_detail(params.organization_id, params.member_id, user.user_id())
        .await?;

    Ok(ApiResponse::success(member_detail))
}

/// 組織容量チェック
pub async fn check_organization_capacity_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    ValidatedUuid(organization_id): ValidatedUuid,
) -> AppResult<ApiResponse<OrganizationCapacityResponse>> {
    let capacity = app_state
        .organization_service
        .check_organization_capacity(organization_id, user.user_id())
        .await?;

    Ok(ApiResponse::success(capacity))
}

/// 組織一覧をページネーション付きで取得
pub async fn get_organizations_paginated_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(query): Query<OrganizationSearchQuery>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    let (organizations, total_count) = app_state
        .organization_service
        .get_organizations_paginated(query, user.user_id())
        .await?;

    Ok(ApiResponse::success(json!({
        "organizations": organizations,
        "total_count": total_count,
        "page": 1,
        "per_page": 20
    })))
}

/// 組織のサブスクリプション階層を更新
pub async fn update_organization_subscription_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    ValidatedUuid(organization_id): ValidatedUuid,
    Json(payload): Json<UpdateOrganizationSubscriptionRequest>,
) -> AppResult<ApiResponse<OrganizationResponse>> {
    let organization_response = app_state
        .organization_service
        .update_organization_subscription(
            organization_id,
            payload.subscription_tier,
            user.user_id(),
        )
        .await?;

    Ok(ApiResponse::success(organization_response))
}

/// 組織のサブスクリプション履歴を取得
pub async fn get_organization_subscription_history_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    ValidatedUuid(organization_id): ValidatedUuid,
) -> AppResult<ApiResponse<Vec<serde_json::Value>>> {
    // 権限チェックはミドルウェアで実施済み

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
                "changed_at": h.changed_at.timestamp(),
                "changed_by": h.changed_by,
                "reason": h.reason,
            })
        })
        .collect();

    Ok(ApiResponse::success(history_response))
}

/// 統一検索クエリを使用した組織検索
pub async fn search_organizations_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(query): Query<OrganizationSearchQuery>,
) -> AppResult<ApiResponse<PaginatedResponse<Organization>>> {
    info!(
        user_id = %user.claims.user_id,
        search = ?query.search,
        "Searching organizations with unified query"
    );

    let (organizations, total_count) = app_state
        .organization_service
        .search_organizations(&query, user.claims.user_id)
        .await?;

    let (page, per_page) = query.pagination.get_pagination();
    let paginated_response =
        PaginatedResponse::new(organizations, page, per_page, total_count as i64);

    Ok(ApiResponse::success(paginated_response))
}

/// 組織ルーターを構築（統一権限チェックミドルウェアを使用）
pub fn organization_router_with_state(app_state: crate::api::AppState) -> axum::Router {
    use crate::api::handlers::organization_hierarchy_handler::get_organization_hierarchy;

    Router::new()
        // === 基本的な組織操作 ===
        .route(
            "/organizations",
            post(create_organization_handler)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::Create)),
        )
        .route(
            "/organizations",
            get(get_organizations_handler), // 組織一覧は認証のみで閲覧可能
        )
        .route(
            "/organizations/search",
            get(search_organizations_handler), // 検索も認証のみで可能
        )
        .route(
            "/organizations/stats",
            get(get_organization_stats_handler), // 統計情報は認証のみで可能
        )
        .route(
            "/organizations/paginated",
            get(get_organizations_paginated_handler), // ページング付き一覧も認証のみで可能
        )
        .route(
            "/organizations/{id}",
            get(get_organization_handler)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::View)),
        )
        .route(
            "/organizations/{id}",
            patch(update_organization_handler)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::Update)),
        )
        .route(
            "/organizations/{id}",
            delete(delete_organization_handler)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::Delete)),
        )
        // === 組織メンバー管理 ===
        .route(
            "/organizations/{id}/members",
            post(invite_organization_member_handler)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::Update)),
        )
        .route(
            "/organizations/{organization_id}/members/{member_id}",
            get(get_organization_member_details_handler)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::View)),
        )
        .route(
            "/organizations/{organization_id}/members/{member_id}/role",
            patch(update_organization_member_role_handler)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::Update)),
        )
        .route(
            "/organizations/{organization_id}/members/{member_id}",
            delete(remove_organization_member_handler)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::Update)),
        )
        // === 組織階層管理 ===
        .route(
            "/organizations/{id}/hierarchy",
            get(get_organization_hierarchy)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::View)),
        )
        // === 組織設定・容量管理 ===
        .route(
            "/organizations/{id}/capacity",
            get(check_organization_capacity_handler)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::View)),
        )
        .route(
            "/organizations/{id}/settings",
            patch(update_organization_settings_handler)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::Update)),
        )
        // === サブスクリプション管理 ===
        .route(
            "/organizations/{id}/subscription",
            patch(update_organization_subscription_handler)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::Update)),
        )
        .route(
            "/organizations/{id}/subscription",
            put(update_organization_subscription_handler)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::Update)),
        )
        .route(
            "/organizations/{id}/subscription/history",
            get(get_organization_subscription_history_handler)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::View)),
        )
        .with_state(app_state)
}
