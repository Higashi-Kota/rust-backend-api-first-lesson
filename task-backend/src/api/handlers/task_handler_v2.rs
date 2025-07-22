// src/api/handlers/task_handler_v2.rs
// 統一権限チェックミドルウェアを適用したタスクハンドラーのサンプル実装

use crate::api::dto::task_dto::{PaginatedTasksDto, TaskDto, UpdateTaskDto}; // TaskDto is used in ApiResponse
use crate::api::dto::task_query_dto::TaskSearchQuery;
use crate::api::dto::team_task_dto::{
    AssignTaskRequest, CreateOrganizationTaskRequest, CreateTeamTaskRequest, TransferTaskRequest,
    TransferTaskResponse,
};
use crate::api::AppState;
use crate::error::AppResult;
use crate::middleware::auth::AuthenticatedUser;
use crate::types::ApiResponse;
use crate::utils::error_helper::convert_validation_errors;
use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

// 統一権限チェックミドルウェアのサンプル実装は削除されました
// 実装は multi_tenant_task_router で統一されています

/// スコープベースのタスク一覧取得ハンドラー
pub async fn list_tasks_with_scope(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(query): Query<TaskSearchQuery>,
) -> AppResult<impl IntoResponse> {
    info!(
        user_id = %auth_user.claims.user_id,
        visibility = ?query.visibility,
        team_id = ?query.team_id,
        "Listing tasks with scope filter"
    );

    let tasks = app_state
        .task_service
        .get_tasks_with_scope(&auth_user, query)
        .await?;

    Ok(ApiResponse::<PaginatedTasksDto>::success(tasks))
}

/// チームタスク作成ハンドラー
pub async fn create_team_task(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(team_id): Path<Uuid>,
    Json(mut payload): Json<CreateTeamTaskRequest>,
) -> AppResult<impl IntoResponse> {
    // パスパラメータのteam_idをペイロードに設定
    payload.team_id = team_id;

    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "task_handler_v2::create_team_task"))?;

    info!(
        user_id = %auth_user.claims.user_id,
        team_id = %payload.team_id,
        title = %payload.title,
        "Creating team task"
    );

    let task_dto = app_state
        .task_service
        .create_team_task(&auth_user, payload)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::<TaskDto>::success(task_dto)),
    ))
}

/// 組織タスク作成ハンドラー
pub async fn create_organization_task(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(organization_id): Path<Uuid>,
    Json(mut payload): Json<CreateOrganizationTaskRequest>,
) -> AppResult<impl IntoResponse> {
    // パスパラメータのorganization_idをペイロードに設定
    payload.organization_id = organization_id;

    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "task_handler_v2::create_organization_task"))?;

    info!(
        user_id = %auth_user.claims.user_id,
        organization_id = %payload.organization_id,
        title = %payload.title,
        "Creating organization task"
    );

    let task_dto = app_state
        .task_service
        .create_organization_task(&auth_user, payload)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::<TaskDto>::success(task_dto)),
    ))
}

/// タスク割り当てハンドラー
pub async fn assign_task(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<AssignTaskRequest>,
) -> AppResult<impl IntoResponse> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "task_handler_v2::assign_task"))?;

    info!(
        user_id = %auth_user.claims.user_id,
        task_id = %task_id,
        assigned_to = ?payload.assigned_to,
        "Assigning task"
    );

    let task_dto = app_state
        .task_service
        .assign_task(&auth_user, task_id, payload)
        .await?;

    Ok(ApiResponse::<TaskDto>::success(task_dto))
}

/// マルチテナントタスク更新ハンドラー
pub async fn update_multi_tenant_task(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<UpdateTaskDto>,
) -> AppResult<impl IntoResponse> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "task_handler_v2::update_multi_tenant_task"))?;

    info!(
        user_id = %auth_user.claims.user_id,
        task_id = %task_id,
        "Updating multi-tenant task"
    );

    let task_dto = app_state
        .task_service
        .update_multi_tenant_task(&auth_user, task_id, payload)
        .await?;

    Ok(ApiResponse::<TaskDto>::success(task_dto))
}

/// マルチテナントタスク削除ハンドラー
pub async fn delete_multi_tenant_task(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    info!(
        user_id = %auth_user.claims.user_id,
        task_id = %task_id,
        "Deleting multi-tenant task"
    );

    app_state
        .task_service
        .delete_multi_tenant_task(&auth_user, task_id)
        .await?;

    Ok(ApiResponse::<()>::success(()))
}

/// タスク引き継ぎハンドラー
pub async fn transfer_task(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<TransferTaskRequest>,
) -> AppResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "task_handler_v2::transfer_task"))?;

    info!(
        user_id = %auth_user.claims.user_id,
        task_id = %task_id,
        new_assignee = %payload.new_assignee,
        "Transferring task to new assignee"
    );

    let response = app_state
        .task_service
        .transfer_task(&auth_user, task_id, payload)
        .await?;

    Ok(ApiResponse::<TransferTaskResponse>::success(response))
}

// task_router_v2 は削除されました - multi_tenant_task_router を使用してください

/// マルチテナント対応タスクルーター
pub fn multi_tenant_task_router(app_state: AppState) -> Router {
    use axum::routing::patch;

    Router::new()
        // スコープベースのタスク一覧取得
        .route("/tasks/scoped", get(list_tasks_with_scope))
        // チームタスク作成
        .route("/teams/{team_id}/tasks", post(create_team_task))
        // 組織タスク作成
        .route(
            "/organizations/{organization_id}/tasks",
            post(create_organization_task),
        )
        // タスク割り当て
        .route("/tasks/{id}/assign", post(assign_task))
        // タスク引き継ぎ
        .route("/tasks/{id}/transfer", post(transfer_task))
        // マルチテナントタスク更新
        .route(
            "/tasks/{id}/multi-tenant",
            patch(update_multi_tenant_task).delete(delete_multi_tenant_task),
        )
        .with_state(app_state)
}

#[cfg(test)]
mod tests {
    // テストケースは必要に応じて実装
    // 統合テストはtestsディレクトリで実装
}
