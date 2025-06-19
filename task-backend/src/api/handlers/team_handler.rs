// task-backend/src/api/handlers/team_handler.rs

use crate::api::dto::common::ApiResponse;
use crate::api::dto::team_dto::*;
use crate::api::AppState;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthenticatedUser;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

// Helper function to handle validation errors
fn handle_validation_error(err: validator::ValidationErrors) -> AppError {
    let messages: Vec<String> = err
        .field_errors()
        .iter()
        .flat_map(|(_, errors)| {
            errors
                .iter()
                .filter_map(|e| e.message.clone().map(|m| m.to_string()))
        })
        .collect();

    if messages.is_empty() {
        AppError::ValidationError("Validation failed".to_string())
    } else {
        AppError::ValidationErrors(messages)
    }
}

/// チーム作成
#[allow(dead_code)]
pub async fn create_team_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateTeamRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<TeamResponse>>)> {
    // バリデーション
    payload.validate().map_err(handle_validation_error)?;

    let team_response = app_state
        .team_service
        .create_team(payload, user.user_id())
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            "Team created successfully",
            team_response,
        )),
    ))
}

/// チーム詳細取得
#[allow(dead_code)]
pub async fn get_team_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(team_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<TeamResponse>>> {
    let team_response = app_state
        .team_service
        .get_team_by_id(team_id, user.user_id())
        .await?;

    Ok(Json(ApiResponse::success(
        "Team retrieved successfully",
        team_response,
    )))
}

/// チーム一覧取得
#[allow(dead_code)]
pub async fn list_teams_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<TeamSearchQuery>,
) -> AppResult<Json<ApiResponse<Vec<TeamListResponse>>>> {
    let teams = app_state
        .team_service
        .get_teams(query, user.user_id())
        .await?;

    Ok(Json(ApiResponse::success(
        "Teams retrieved successfully",
        teams,
    )))
}

/// チーム更新
#[allow(dead_code)]
pub async fn update_team_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(team_id): Path<Uuid>,
    Json(payload): Json<UpdateTeamRequest>,
) -> AppResult<Json<ApiResponse<TeamResponse>>> {
    // バリデーション
    payload.validate().map_err(handle_validation_error)?;

    let team_response = app_state
        .team_service
        .update_team(team_id, payload, user.user_id())
        .await?;

    Ok(Json(ApiResponse::success(
        "Team updated successfully",
        team_response,
    )))
}

/// チーム削除
#[allow(dead_code)]
pub async fn delete_team_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(team_id): Path<Uuid>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    app_state
        .team_service
        .delete_team(team_id, user.user_id())
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        Json(json!({
            "success": true,
            "message": "Team deleted successfully"
        })),
    ))
}

/// チームメンバー招待
#[allow(dead_code)]
pub async fn invite_team_member_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(team_id): Path<Uuid>,
    Json(payload): Json<InviteTeamMemberRequest>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    // バリデーション
    payload.validate().map_err(handle_validation_error)?;

    let member_response = app_state
        .team_service
        .invite_team_member(team_id, payload, user.user_id())
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": member_response,
            "message": "Team member invited successfully"
        })),
    ))
}

/// チームメンバー役割更新
#[allow(dead_code)]
pub async fn update_team_member_role_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path((team_id, member_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateTeamMemberRoleRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let member_response = app_state
        .team_service
        .update_team_member_role(team_id, member_id, payload, user.user_id())
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": member_response,
        "message": "Team member role updated successfully"
    })))
}

/// チームメンバー削除
#[allow(dead_code)]
pub async fn remove_team_member_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path((team_id, member_id)): Path<(Uuid, Uuid)>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    app_state
        .team_service
        .remove_team_member(team_id, member_id, user.user_id())
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        Json(json!({
            "success": true,
            "message": "Team member removed successfully"
        })),
    ))
}

/// チーム統計取得
#[allow(dead_code)]
pub async fn get_team_stats_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<TeamStatsResponse>>> {
    let stats = app_state
        .team_service
        .get_team_stats(user.user_id())
        .await?;

    Ok(Json(ApiResponse::success(
        "Team stats retrieved successfully",
        stats,
    )))
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
            "/teams/{team_id}/members/{member_id}/role",
            patch(update_team_member_role_handler),
        )
        .route(
            "/teams/{team_id}/members/{member_id}",
            delete(remove_team_member_handler),
        )
        // チーム統計
        .route("/teams/stats", get(get_team_stats_handler))
        .with_state(app_state)
}

/// チームルーターをAppStateから作成
pub fn team_router_with_state(app_state: AppState) -> Router {
    team_router(app_state)
}
