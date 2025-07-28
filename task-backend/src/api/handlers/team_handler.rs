// task-backend/src/api/handlers/team_handler.rs

use crate::api::dto::team_dto::*;
use crate::api::dto::team_query_dto::TeamSearchQuery;
use crate::api::AppState;
use crate::error::AppResult;
use crate::extractors::{deserialize_uuid, ValidatedMultiPath, ValidatedUuid};
use crate::middleware::auth::AuthenticatedUser;
use crate::middleware::authorization::{resources, Action};
use crate::require_permission;
use crate::shared::types::PaginatedResponse;
use crate::types::ApiResponse;
use crate::utils::error_helper::convert_validation_errors;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

// 複数パラメータ用のPath構造体
#[derive(Deserialize)]
pub struct TeamMemberPath {
    #[serde(deserialize_with = "deserialize_uuid")]
    pub team_id: Uuid,
    #[serde(deserialize_with = "deserialize_uuid")]
    pub member_id: Uuid,
}

/// チーム作成
pub async fn create_team_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateTeamRequest>,
) -> AppResult<(StatusCode, ApiResponse<TeamResponse>)> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "team_handler::create_team"))?;

    // 権限チェックはミドルウェアで実施済み

    let team_response = app_state
        .team_service
        .create_team(payload, user.user_id())
        .await?;

    Ok((StatusCode::CREATED, ApiResponse::success(team_response)))
}

/// チーム詳細取得
pub async fn get_team_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedUuid(team_id): ValidatedUuid,
) -> AppResult<ApiResponse<TeamResponse>> {
    let team_response = app_state
        .team_service
        .get_team_by_id(team_id, user.user_id())
        .await?;

    Ok(ApiResponse::success(team_response))
}

/// チーム一覧取得
pub async fn list_teams_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<TeamSearchQuery>,
) -> AppResult<ApiResponse<PaginatedResponse<TeamListResponse>>> {
    let (teams, total) = app_state
        .team_service
        .search_teams(&query, user.user_id())
        .await?;

    let (page, per_page) = query.pagination.get_pagination();
    let response = PaginatedResponse::new(teams, page, per_page, total as i64);

    Ok(ApiResponse::success(response))
}

/// チーム更新
pub async fn update_team_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedUuid(team_id): ValidatedUuid,
    Json(payload): Json<UpdateTeamRequest>,
) -> AppResult<ApiResponse<TeamResponse>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "team_handler::update_team"))?;

    // 権限チェックはミドルウェアで実施済み

    let team_response = app_state
        .team_service
        .update_team(team_id, payload, user.user_id())
        .await?;

    Ok(ApiResponse::success(team_response))
}

/// チーム削除
pub async fn delete_team_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedUuid(team_id): ValidatedUuid,
) -> AppResult<(StatusCode, ApiResponse<()>)> {
    // 権限チェックはミドルウェアで実施済み

    app_state
        .team_service
        .delete_team(team_id, user.user_id())
        .await?;

    Ok((StatusCode::NO_CONTENT, ApiResponse::success(())))
}

/// チームメンバー招待
pub async fn invite_team_member_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedUuid(team_id): ValidatedUuid,
    Json(payload): Json<InviteTeamMemberRequest>,
) -> AppResult<(StatusCode, ApiResponse<TeamMemberResponse>)> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "team_handler::invite_team_member"))?;

    // 権限チェックはミドルウェアで実施済み

    let member_response = app_state
        .team_service
        .invite_team_member(team_id, payload, user.user_id())
        .await?;

    Ok((StatusCode::CREATED, ApiResponse::success(member_response)))
}

/// チームメンバー役割更新
pub async fn update_team_member_role_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedMultiPath(params): ValidatedMultiPath<TeamMemberPath>,
    Json(payload): Json<UpdateTeamMemberRoleRequest>,
) -> AppResult<ApiResponse<TeamMemberResponse>> {
    let member_response = app_state
        .team_service
        .update_team_member_role(params.team_id, params.member_id, payload, user.user_id())
        .await?;

    Ok(ApiResponse::success(member_response))
}

/// チームメンバー削除
pub async fn remove_team_member_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedMultiPath(params): ValidatedMultiPath<TeamMemberPath>,
) -> AppResult<(StatusCode, ApiResponse<()>)> {
    app_state
        .team_service
        .remove_team_member(params.team_id, params.member_id, user.user_id())
        .await?;

    Ok((StatusCode::NO_CONTENT, ApiResponse::success(())))
}

/// チーム統計取得
pub async fn get_team_stats_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<ApiResponse<TeamStatsResponse>> {
    let stats = app_state
        .team_service
        .get_team_stats(user.user_id())
        .await?;

    Ok(ApiResponse::success(stats))
}

/// チーム一覧をページング付きで取得
pub async fn get_teams_with_pagination_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<TeamSearchQuery>,
) -> AppResult<ApiResponse<TeamPaginationResponse>> {
    let (page, per_page) = query.pagination.get_pagination();
    let page = page as u64;
    let per_page = per_page as u64;

    let (teams, total_count) = app_state
        .team_service
        .get_teams_with_pagination(page, per_page, query.organization_id, user.user_id())
        .await?;

    let response =
        TeamPaginationResponse::new(teams, page as i32, per_page as i32, total_count as i64);

    Ok(ApiResponse::success(response))
}

/// チームメンバーの詳細情報を取得（権限情報付き）
pub async fn get_team_member_details_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedMultiPath(params): ValidatedMultiPath<TeamMemberPath>,
) -> AppResult<ApiResponse<TeamMemberDetailResponse>> {
    let member_detail = app_state
        .team_service
        .get_team_member_detail(params.team_id, params.member_id, user.user_id())
        .await?;

    Ok(ApiResponse::success(member_detail))
}

/// 統一検索クエリを使用したチーム検索
pub async fn search_teams_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<TeamSearchQuery>,
) -> AppResult<ApiResponse<PaginatedResponse<TeamListResponse>>> {
    info!(
        user_id = %user.claims.user_id,
        search = ?query.search,
        "Searching teams with unified query"
    );

    let (teams, total_count) = app_state
        .team_service
        .search_teams(&query, user.claims.user_id)
        .await?;

    let (page, per_page) = query.pagination.get_pagination();
    let paginated_response = PaginatedResponse::new(teams, page, per_page, total_count as i64);

    Ok(ApiResponse::success(paginated_response))
}

// --- ルーター ---

/// チームルーターを作成（統一権限チェックミドルウェアを使用）
pub fn team_router(app_state: AppState) -> Router {
    use crate::api::handlers::team_invitation_handler::{
        accept_invitation, bulk_member_invite, bulk_update_invitation_status, cancel_invitation,
        check_team_invitation, cleanup_expired_invitations, count_user_invitations,
        create_single_invitation, decline_invitation, delete_old_invitations,
        get_invitation_statistics, get_invitations_by_creator, get_invitations_by_email,
        get_invitations_with_pagination, get_team_invitations, get_user_invitations,
        resend_invitation,
    };

    Router::new()
        // === 基本的なチーム操作 ===
        .route(
            "/teams",
            post(create_team_handler), // 権限チェックはハンドラー内で実施（organization_idの有無により異なるため）
        )
        .route(
            "/teams",
            get(list_teams_handler), // チーム一覧は認証のみで閲覧可能
        )
        .route(
            "/teams/search",
            get(search_teams_handler), // 検索も認証のみで可能
        )
        .route(
            "/teams/{id}",
            get(get_team_handler).route_layer(require_permission!(resources::TEAM, Action::View)),
        )
        .route(
            "/teams/{id}",
            patch(update_team_handler)
                .route_layer(require_permission!(resources::TEAM, Action::Update)),
        )
        .route(
            "/teams/{id}",
            delete(delete_team_handler)
                .route_layer(require_permission!(resources::TEAM, Action::Delete)),
        )
        // === チームメンバー管理 ===
        .route(
            "/teams/{id}/members",
            post(invite_team_member_handler)
                .route_layer(require_permission!(resources::TEAM, Action::Update)),
        )
        .route(
            "/teams/{team_id}/members/{member_id}",
            get(get_team_member_details_handler)
                .route_layer(require_permission!(resources::TEAM, Action::View)),
        )
        .route(
            "/teams/{team_id}/members/{member_id}/role",
            patch(update_team_member_role_handler)
                .route_layer(require_permission!(resources::TEAM, Action::Update)),
        )
        .route(
            "/teams/{team_id}/members/{member_id}",
            delete(remove_team_member_handler)
                .route_layer(require_permission!(resources::TEAM, Action::Update)),
        )
        // === チーム招待管理 ===
        .route(
            "/teams/{id}/bulk-member-invite",
            post(bulk_member_invite)
                .route_layer(require_permission!(resources::TEAM, Action::Update)),
        )
        .route(
            "/teams/{id}/invitations",
            get(get_team_invitations)
                .route_layer(require_permission!(resources::TEAM, Action::View)),
        )
        .route(
            "/teams/{team_id}/invitations/{invitation_id}/decline",
            patch(decline_invitation), // 招待を受けた本人のみが拒否可能（特殊な権限チェック）
        )
        .route(
            "/teams/{id}/invitations/single",
            post(create_single_invitation)
                .route_layer(require_permission!(resources::TEAM, Action::Update)),
        )
        .route(
            "/teams/{id}/invitations/paginated",
            get(get_invitations_with_pagination)
                .route_layer(require_permission!(resources::TEAM, Action::View)),
        )
        .route(
            "/teams/{team_id}/invitations/check/{email}",
            get(check_team_invitation)
                .route_layer(require_permission!(resources::TEAM, Action::View)),
        )
        // === ユーザー招待関連API ===
        .route("/invitations/by-email", get(get_invitations_by_email))
        .route("/invitations/stats", get(count_user_invitations))
        .route(
            "/invitations/bulk-update",
            post(bulk_update_invitation_status),
        )
        .route("/users/invitations", get(get_user_invitations))
        .route("/invitations/by-creator", get(get_invitations_by_creator))
        // === 管理者向け招待API ===
        .route(
            "/admin/invitations/expired/cleanup",
            post(cleanup_expired_invitations)
                .route_layer(require_permission!(resources::INVITATION, Action::Admin)),
        )
        .route(
            "/admin/invitations/old/delete",
            delete(delete_old_invitations)
                .route_layer(require_permission!(resources::INVITATION, Action::Admin)),
        )
        // === 招待受諾・キャンセル・再送API ===
        .route("/invitations/{id}/accept", post(accept_invitation))
        .route(
            "/teams/{team_id}/invitations/{invitation_id}/cancel",
            delete(cancel_invitation)
                .route_layer(require_permission!(resources::TEAM, Action::Update)),
        )
        .route("/invitations/{id}/resend", post(resend_invitation))
        // === 追加の統計・管理API ===
        .route(
            "/teams/{id}/invitations/statistics",
            get(get_invitation_statistics)
                .route_layer(require_permission!(resources::TEAM, Action::View)),
        )
        // === チーム統計 ===
        .route("/teams/stats", get(get_team_stats_handler))
        .route("/teams/paginated", get(get_teams_with_pagination_handler))
        .with_state(app_state)
}

/// チームルーターをAppStateから作成
pub fn team_router_with_state(app_state: AppState) -> Router {
    team_router(app_state)
}
