// src/api/handlers/audit_log_handler.rs
use crate::api::AppState;
use crate::error::AppResult;
use crate::middleware::auth::AuthenticatedUser;
use crate::middleware::authorization::{resources, Action};
use crate::require_permission;
use crate::service::audit_log_service::PaginatedAuditLogs;
use crate::types::ApiResponse;
use crate::utils::error_helper::{forbidden_error, internal_server_error};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_page() -> u64 {
    1
}

fn default_per_page() -> u64 {
    20
}

/// ユーザー自身の監査ログを取得
pub async fn get_my_audit_logs(
    State(app_state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(query): Query<AuditLogQuery>,
) -> AppResult<impl IntoResponse> {
    info!(
        user_id = %auth_user.claims.user_id,
        page = %query.page,
        per_page = %query.per_page,
        "Getting user's own audit logs"
    );

    let logs = app_state
        .audit_log_service
        .get_user_audit_logs(auth_user.claims.user_id, query.page, query.per_page)
        .await?;

    Ok(ApiResponse::<PaginatedAuditLogs>::success(logs))
}

/// 特定ユーザーの監査ログを取得（管理者のみ）
pub async fn get_user_audit_logs(
    State(app_state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
    Query(query): Query<AuditLogQuery>,
) -> AppResult<impl IntoResponse> {
    // 権限チェックはミドルウェアで実施済み

    info!(
        admin_id = %auth_user.claims.user_id,
        target_user_id = %user_id,
        page = %query.page,
        per_page = %query.per_page,
        "Admin getting user audit logs"
    );

    let logs = app_state
        .audit_log_service
        .get_user_audit_logs(user_id, query.page, query.per_page)
        .await?;

    Ok(ApiResponse::<PaginatedAuditLogs>::success(logs))
}

/// チームの監査ログを取得（チームメンバーのみ）
pub async fn get_team_audit_logs(
    State(app_state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(team_id): Path<Uuid>,
    Query(query): Query<AuditLogQuery>,
) -> AppResult<impl IntoResponse> {
    // チームメンバーシップの確認
    let is_member = app_state
        .team_service
        .is_user_member_of_team(auth_user.claims.user_id, team_id)
        .await
        .map_err(|e| {
            internal_server_error(
                e,
                "audit_log_handler::get_team_audit_logs",
                "Failed to check team membership",
            )
        })?;

    // 権限チェックはミドルウェアで管理者権限を確認済み
    // ここでは追加でチームメンバーシップを確認
    if !is_member && !auth_user.claims.is_admin() {
        return Err(forbidden_error(
            "Not a team member",
            "audit_log_handler::get_team_audit_logs",
            "You must be a member of the team to view its audit logs",
        ));
    }

    info!(
        user_id = %auth_user.claims.user_id,
        team_id = %team_id,
        page = %query.page,
        per_page = %query.per_page,
        "Getting team audit logs"
    );

    let logs = app_state
        .audit_log_service
        .get_team_audit_logs(team_id, query.page, query.per_page)
        .await?;

    Ok(ApiResponse::<PaginatedAuditLogs>::success(logs))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupRequest {
    pub days_to_keep: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupResponse {
    pub deleted_count: u64,
}

/// 古い監査ログのクリーンアップ（管理者のみ）
pub async fn cleanup_old_audit_logs(
    State(app_state): State<AppState>,
    auth_user: AuthenticatedUser,
    axum::Json(request): axum::Json<CleanupRequest>,
) -> AppResult<impl IntoResponse> {
    // 権限チェックはミドルウェアで実施済み

    info!(
        admin_id = %auth_user.claims.user_id,
        days_to_keep = %request.days_to_keep,
        "Cleaning up old audit logs"
    );

    let deleted_count = app_state
        .audit_log_service
        .cleanup_old_logs(request.days_to_keep)
        .await?;

    Ok(ApiResponse::<CleanupResponse>::success(CleanupResponse {
        deleted_count,
    }))
}

/// 監査ログルーター
pub fn audit_log_router(app_state: AppState) -> Router {
    Router::new()
        // ユーザー自身の監査ログ
        .route("/audit-logs/me", get(get_my_audit_logs))
        // 特定ユーザーの監査ログ（管理者のみ）
        .route(
            "/admin/audit-logs/users/{user_id}",
            get(get_user_audit_logs)
                .route_layer(require_permission!(resources::AUDIT_LOG, Action::Admin)),
        )
        // チームの監査ログ（チームメンバーシップはハンドラー内でチェック）
        .route("/teams/{team_id}/audit-logs", get(get_team_audit_logs))
        // 古いログのクリーンアップ（管理者のみ）
        .route(
            "/admin/audit-logs/cleanup",
            axum::routing::post(cleanup_old_audit_logs)
                .route_layer(require_permission!(resources::AUDIT_LOG, Action::Admin)),
        )
        .with_state(app_state)
}
