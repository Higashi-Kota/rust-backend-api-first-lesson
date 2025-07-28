// src/api/handlers/audit_log_handler.rs
use crate::api::AppState;
use crate::error::AppResult;
use crate::extractors::ValidatedUuid;
use crate::middleware::auth::AuthenticatedUser;
use crate::middleware::authorization::{resources, Action};
use crate::require_permission;
use crate::shared::types::PaginatedResponse;
use crate::types::query::{PaginationQuery, SearchQuery};
use crate::types::{ApiResponse, SortQuery};
use crate::utils::error_helper::{forbidden_error, internal_server_error};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    #[serde(flatten)]
    pub sort: SortQuery,
    pub search: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    /// 作成日時の開始（以降）
    #[serde(default, with = "crate::types::optional_timestamp")]
    pub created_after: Option<DateTime<Utc>>,
    /// 作成日時の終了（以前）
    #[serde(default, with = "crate::types::optional_timestamp")]
    pub created_before: Option<DateTime<Utc>>,
}

impl AuditLogQuery {
    /// 許可されたソートフィールド
    pub fn allowed_sort_fields() -> &'static [&'static str] {
        &["created_at", "action", "resource_type", "user_id"]
    }
}

impl SearchQuery for AuditLogQuery {
    fn search_term(&self) -> Option<&str> {
        self.search.as_deref()
    }

    fn filters(&self) -> HashMap<String, String> {
        let mut filters = HashMap::new();

        if let Some(action) = &self.action {
            filters.insert("action".to_string(), action.clone());
        }
        if let Some(resource_type) = &self.resource_type {
            filters.insert("resource_type".to_string(), resource_type.clone());
        }
        if let Some(created_after) = &self.created_after {
            filters.insert("created_after".to_string(), created_after.to_rfc3339());
        }
        if let Some(created_before) = &self.created_before {
            filters.insert("created_before".to_string(), created_before.to_rfc3339());
        }

        filters
    }
}

/// ユーザー自身の監査ログを取得
pub async fn get_my_audit_logs(
    State(app_state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(query): Query<AuditLogQuery>,
) -> AppResult<impl IntoResponse> {
    let (page, per_page) = query.pagination.get_pagination();
    info!(
        user_id = %auth_user.claims.user_id,
        page = %page,
        per_page = %per_page,
        "Getting user's own audit logs"
    );

    let logs = app_state
        .audit_log_service
        .get_user_audit_logs(
            auth_user.claims.user_id,
            page as u64,
            per_page as u64,
            &query.sort,
            query.created_after,
            query.created_before,
        )
        .await?;

    let response = PaginatedResponse::new(
        logs.logs,
        logs.page as i32,
        logs.per_page as i32,
        logs.total as i64,
    );
    Ok(ApiResponse::success(response))
}

/// 特定ユーザーの監査ログを取得（管理者のみ）
pub async fn get_user_audit_logs(
    State(app_state): State<AppState>,
    auth_user: AuthenticatedUser,
    ValidatedUuid(user_id): ValidatedUuid,
    Query(query): Query<AuditLogQuery>,
) -> AppResult<impl IntoResponse> {
    // 権限チェックはミドルウェアで実施済み

    let (page, per_page) = query.pagination.get_pagination();
    info!(
        admin_id = %auth_user.claims.user_id,
        target_user_id = %user_id,
        page = %page,
        per_page = %per_page,
        "Admin getting user audit logs"
    );

    let logs = app_state
        .audit_log_service
        .get_user_audit_logs(
            user_id,
            page as u64,
            per_page as u64,
            &query.sort,
            query.created_after,
            query.created_before,
        )
        .await?;

    let response = PaginatedResponse::new(
        logs.logs,
        logs.page as i32,
        logs.per_page as i32,
        logs.total as i64,
    );
    Ok(ApiResponse::success(response))
}

/// チームの監査ログを取得（チームメンバーのみ）
pub async fn get_team_audit_logs(
    State(app_state): State<AppState>,
    auth_user: AuthenticatedUser,
    ValidatedUuid(team_id): ValidatedUuid,
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

    let (page, per_page) = query.pagination.get_pagination();
    info!(
        user_id = %auth_user.claims.user_id,
        team_id = %team_id,
        page = %page,
        per_page = %per_page,
        "Getting team audit logs"
    );

    let logs = app_state
        .audit_log_service
        .get_team_audit_logs(
            team_id,
            page as u64,
            per_page as u64,
            &query.sort,
            query.created_after,
            query.created_before,
        )
        .await?;

    let response = PaginatedResponse::new(
        logs.logs,
        logs.page as i32,
        logs.per_page as i32,
        logs.total as i64,
    );
    Ok(ApiResponse::success(response))
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
