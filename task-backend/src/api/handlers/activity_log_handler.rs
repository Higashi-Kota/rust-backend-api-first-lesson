use axum::{
    extract::{Query, State},
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    api::dto::common::PaginationQuery,
    api::AppState,
    domain::activity_log_model,
    error::AppResult,
    middleware::auth::AuthenticatedUser,
    repository::activity_log_repository::{ActivityLogFilter, ActivityLogRepository},
    shared::types::PaginatedResponse,
    types::{query::SearchQuery, ApiResponse, SortQuery, Timestamp},
    utils::error_helper::internal_server_error,
};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ActivityLogQuery {
    /// ユーザーIDでフィルタ（管理者のみ）
    pub user_id: Option<Uuid>,
    /// リソースタイプでフィルタ
    pub resource_type: Option<String>,
    /// アクションでフィルタ
    pub action: Option<String>,
    /// 作成日時の開始（以降）
    pub created_after: Option<DateTime<Utc>>,
    /// 作成日時の終了（以前）
    pub created_before: Option<DateTime<Utc>>,
    /// ページネーションパラメータ
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    /// ソートパラメータ
    #[serde(flatten)]
    pub sort: SortQuery,
    /// 検索用
    pub search: Option<String>,
}

impl ActivityLogQuery {
    /// 許可されたソートフィールド
    pub fn allowed_sort_fields() -> &'static [&'static str] {
        &["created_at", "action", "resource_type", "user_id"]
    }
}

impl SearchQuery for ActivityLogQuery {
    fn search_term(&self) -> Option<&str> {
        self.search.as_deref()
    }

    fn filters(&self) -> HashMap<String, String> {
        let mut filters = HashMap::new();

        if let Some(id) = &self.user_id {
            filters.insert("user_id".to_string(), id.to_string());
        }
        if let Some(resource_type) = &self.resource_type {
            filters.insert("resource_type".to_string(), resource_type.clone());
        }
        if let Some(action) = &self.action {
            filters.insert("action".to_string(), action.clone());
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

#[derive(Debug, Serialize)]
pub struct ActivityLogResponse {
    pub logs: Vec<ActivityLogDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityLogDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: Timestamp,
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
            created_at: Timestamp::from_datetime(model.created_at),
        }
    }
}

/// ユーザー自身のアクティビティログを取得
pub async fn get_my_activity_logs(
    user: AuthenticatedUser,
    Query(query): Query<ActivityLogQuery>,
    State(app_state): State<AppState>,
) -> AppResult<ApiResponse<PaginatedResponse<ActivityLogDto>>> {
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
) -> AppResult<ApiResponse<PaginatedResponse<ActivityLogDto>>> {
    // 権限チェックはミドルウェアで実施済み

    get_activity_logs_internal(query, app_state.activity_log_repo.clone()).await
}

/// 内部的なログ取得処理
async fn get_activity_logs_internal(
    query: ActivityLogQuery,
    activity_log_repo: Arc<ActivityLogRepository>,
) -> AppResult<ApiResponse<PaginatedResponse<ActivityLogDto>>> {
    let (page, per_page) = query.pagination.get_pagination();
    let page = page as u64;
    let per_page = per_page as u64;

    // リポジトリのクエリ機能を使用してログを取得
    let filter = ActivityLogFilter {
        user_id: query.user_id,
        resource_type: query.resource_type,
        action: query.action,
        created_after: query.created_after,
        created_before: query.created_before,
        page,
        per_page,
        sort: query.sort,
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
        })?;

    // ModelをDTOに変換
    let log_dtos: Vec<ActivityLogDto> = logs.into_iter().map(ActivityLogDto::from).collect();

    let response = PaginatedResponse::new(log_dtos, page as i32, per_page as i32, total as i64);

    Ok(ApiResponse::success(response))
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
