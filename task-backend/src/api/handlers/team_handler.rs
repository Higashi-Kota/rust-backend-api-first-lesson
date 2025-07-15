// task-backend/src/api/handlers/team_handler.rs

use crate::api::dto::team_dto::*;
use crate::api::handlers::team_invitation_handler;
use crate::api::AppState;
use crate::error::AppResult;
use crate::middleware::auth::AuthenticatedUser;
use crate::types::ApiResponse;
use crate::utils::error_helper::convert_validation_errors;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};
use uuid::Uuid;
use validator::Validate;

/// チーム作成
pub async fn create_team_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateTeamRequest>,
) -> AppResult<(StatusCode, ApiResponse<TeamResponse>)> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "team"))?;

    // PermissionServiceを使用してチーム作成権限をチェック
    app_state
        .permission_service
        .check_resource_access(user.user_id(), "team", None, "create")
        .await?;

    // サブスクリプション制限はTeamServiceで処理されるため、ここでは重複チェックしない

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
    Path(team_id): Path<Uuid>,
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
) -> AppResult<ApiResponse<Vec<TeamListResponse>>> {
    let teams = app_state
        .team_service
        .get_teams(query, user.user_id())
        .await?;

    Ok(ApiResponse::success(teams))
}

/// チーム更新
pub async fn update_team_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(team_id): Path<Uuid>,
    Json(payload): Json<UpdateTeamRequest>,
) -> AppResult<ApiResponse<TeamResponse>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "team"))?;

    // PermissionServiceを使用してチーム管理権限をチェック
    app_state
        .permission_service
        .check_team_management_permission(user.user_id(), team_id)
        .await?;

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
    Path(team_id): Path<Uuid>,
) -> AppResult<(StatusCode, ApiResponse<()>)> {
    // PermissionServiceを使用してチーム管理権限をチェック
    app_state
        .permission_service
        .check_team_management_permission(user.user_id(), team_id)
        .await?;

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
    Path(team_id): Path<Uuid>,
    Json(payload): Json<InviteTeamMemberRequest>,
) -> AppResult<(StatusCode, ApiResponse<TeamMemberResponse>)> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "team"))?;

    // PermissionServiceを使用してチーム管理権限をチェック
    app_state
        .permission_service
        .check_team_management_permission(user.user_id(), team_id)
        .await?;

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
    Path((team_id, member_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateTeamMemberRoleRequest>,
) -> AppResult<ApiResponse<TeamMemberResponse>> {
    let member_response = app_state
        .team_service
        .update_team_member_role(team_id, member_id, payload, user.user_id())
        .await?;

    Ok(ApiResponse::success(member_response))
}

/// チームメンバー削除
pub async fn remove_team_member_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path((team_id, member_id)): Path<(Uuid, Uuid)>,
) -> AppResult<(StatusCode, ApiResponse<()>)> {
    app_state
        .team_service
        .remove_team_member(team_id, member_id, user.user_id())
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
    Query(query): Query<TeamPaginationQuery>,
) -> AppResult<ApiResponse<TeamPaginationResponse>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    let (teams, total_count) = app_state
        .team_service
        .get_teams_with_pagination(page, page_size, query.organization_id, user.user_id())
        .await?;

    let response = TeamPaginationResponse::new(teams, total_count, page, page_size);

    Ok(ApiResponse::success(response))
}

/// チームメンバーの詳細情報を取得（権限情報付き）
pub async fn get_team_member_details_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path((team_id, member_id)): Path<(Uuid, Uuid)>,
) -> AppResult<ApiResponse<TeamMemberDetailResponse>> {
    let member_detail = app_state
        .team_service
        .get_team_member_detail(team_id, member_id, user.user_id())
        .await?;

    Ok(ApiResponse::success(member_detail))
}

// --- ルーター ---

/// チームルーターを作成
pub fn team_router(app_state: AppState) -> Router {
    Router::new()
        // チーム管理
        .route("/teams", post(create_team_handler))
        .route("/teams", get(list_teams_handler))
        .route("/teams/{id}", get(get_team_handler))
        .route("/teams/{id}", patch(update_team_handler))
        .route("/teams/{id}", delete(delete_team_handler))
        // チームメンバー管理
        .route("/teams/{id}/members", post(invite_team_member_handler))
        .route(
            "/teams/{team_id}/members/{member_id}",
            get(get_team_member_details_handler),
        )
        .route(
            "/teams/{team_id}/members/{member_id}/role",
            patch(update_team_member_role_handler),
        )
        .route(
            "/teams/{team_id}/members/{member_id}",
            delete(remove_team_member_handler),
        )
        // Phase 2.2: チーム招待・権限管理 API
        .route(
            "/teams/{id}/bulk-member-invite",
            post(team_invitation_handler::bulk_member_invite),
        )
        .route(
            "/teams/{id}/invitations",
            get(team_invitation_handler::get_team_invitations),
        )
        .route(
            "/teams/{id}/invitations/{invite_id}/decline",
            patch(team_invitation_handler::decline_invitation),
        )
        // 新規招待API
        .route(
            "/teams/{id}/invitations/single",
            post(team_invitation_handler::create_single_invitation),
        )
        .route(
            "/teams/{id}/invitations/paginated",
            get(team_invitation_handler::get_invitations_with_pagination),
        )
        .route(
            "/teams/{team_id}/invitations/check/{email}",
            get(team_invitation_handler::check_team_invitation),
        )
        // ユーザー招待関連API
        .route(
            "/invitations/by-email",
            get(team_invitation_handler::get_invitations_by_email),
        )
        .route(
            "/invitations/stats",
            get(team_invitation_handler::count_user_invitations),
        )
        .route(
            "/invitations/bulk-update",
            post(team_invitation_handler::bulk_update_invitation_status),
        )
        // 管理者向け招待API
        .route(
            "/admin/invitations/expired/cleanup",
            post(team_invitation_handler::cleanup_expired_invitations),
        )
        .route(
            "/admin/invitations/old/delete",
            delete(team_invitation_handler::delete_old_invitations),
        )
        // 招待受諾・キャンセル・再送API
        .route(
            "/invitations/{id}/accept",
            post(team_invitation_handler::accept_invitation),
        )
        .route(
            "/teams/{team_id}/invitations/{invite_id}/cancel",
            delete(team_invitation_handler::cancel_invitation),
        )
        .route(
            "/invitations/{id}/resend",
            post(team_invitation_handler::resend_invitation),
        )
        // 追加の統計・管理API
        .route(
            "/teams/{id}/invitations/statistics",
            get(team_invitation_handler::get_invitation_statistics),
        )
        .route(
            "/invitations/by-creator",
            get(team_invitation_handler::get_invitations_by_creator),
        )
        .route(
            "/users/invitations",
            get(team_invitation_handler::get_user_invitations),
        )
        // チーム統計
        .route("/teams/stats", get(get_team_stats_handler))
        // チーム一覧（ページング付き）
        .route("/teams/paginated", get(get_teams_with_pagination_handler))
        .with_state(app_state)
}

/// チームルーターをAppStateから作成
pub fn team_router_with_state(app_state: AppState) -> Router {
    team_router(app_state)
}
