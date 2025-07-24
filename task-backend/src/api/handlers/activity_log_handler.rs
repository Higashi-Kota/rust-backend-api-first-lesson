use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    api::AppState,
    domain::activity_log_model,
    middleware::auth::AuthenticatedUser,
    repository::activity_log_repository::{ActivityLogFilter, ActivityLogRepository},
    utils::error_helper::internal_server_error,
};

#[derive(Debug, Deserialize)]
pub struct ActivityLogQuery {
    /// ユーザーIDでフィルタ（管理者のみ）
    pub user_id: Option<Uuid>,
    /// リソースタイプでフィルタ
    pub resource_type: Option<String>,
    /// アクションでフィルタ
    pub action: Option<String>,
    /// 開始日時
    pub from: Option<DateTime<Utc>>,
    /// 終了日時
    pub to: Option<DateTime<Utc>>,
    /// ページ番号（1始まり）
    pub page: Option<u64>,
    /// ページサイズ
    pub per_page: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ActivityLogResponse {
    pub logs: Vec<ActivityLogDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

#[derive(Debug, Serialize)]
pub struct ActivityLogDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl From<activity_log_model::Model> for ActivityLogDto {
    fn from(model: activity_log_model::Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            action: model.action,
            resource_type: model.resource_type,
            resource_id: model.resource_id,
            ip_address: model.ip_address,
            user_agent: model.user_agent,
            details: model.details,
            created_at: model.created_at,
        }
    }
}

/// ユーザー自身のアクティビティログを取得
pub async fn get_my_activity_logs(
    user: AuthenticatedUser,
    Query(query): Query<ActivityLogQuery>,
    State(app_state): State<AppState>,
) -> Result<Response, Response> {
    // ユーザー自身のログのみ取得可能
    let mut query = query;
    query.user_id = Some(user.claims.user_id);

    get_activity_logs_internal(query, app_state.activity_log_repo.clone()).await
}

/// 管理者用：すべてのアクティビティログを取得
pub async fn get_all_activity_logs(
    _user: AuthenticatedUser,
    Query(query): Query<ActivityLogQuery>,
    State(app_state): State<AppState>,
) -> Result<Response, Response> {
    // 権限チェックはミドルウェアで実施済み

    get_activity_logs_internal(query, app_state.activity_log_repo.clone()).await
}

/// 内部的なログ取得処理
async fn get_activity_logs_internal(
    query: ActivityLogQuery,
    activity_log_repo: Arc<ActivityLogRepository>,
) -> Result<Response, Response> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    // リポジトリのクエリ機能を使用してログを取得
    let filter = ActivityLogFilter {
        user_id: query.user_id,
        resource_type: query.resource_type,
        action: query.action,
        from: query.from,
        to: query.to,
        page,
        per_page,
    };

    let (logs, total) = activity_log_repo
        .find_with_query(filter)
        .await
        .map_err(|e| {
            internal_server_error(
                e,
                "activity_log_handler::get_activity_logs_internal",
                "Failed to fetch activity logs",
            )
            .into_response()
        })?;

    // ModelをDTOに変換
    let log_dtos: Vec<ActivityLogDto> = logs.into_iter().map(ActivityLogDto::from).collect();

    let response = ActivityLogResponse {
        logs: log_dtos,
        total,
        page,
        per_page,
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

/// アクティビティログのルーター
pub fn activity_log_router(app_state: AppState) -> Router {
    use crate::middleware::authorization::{resources, Action};
    use crate::require_permission;

    Router::new()
        .route("/activity-logs/me", get(get_my_activity_logs))
        .route(
            "/admin/activity-logs",
            get(get_all_activity_logs)
                .route_layer(require_permission!(resources::AUDIT_LOG, Action::View)),
        )
        .with_state(app_state)
}
