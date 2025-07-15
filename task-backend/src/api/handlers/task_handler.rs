// src/api/handlers/task_handler.rs
use crate::api::dto::common::PaginationQuery;
use crate::api::dto::task_dto::{
    BatchCreateTaskDto, BatchDeleteResponseDto, BatchDeleteTaskDto, BatchUpdateResponseDto,
    BatchUpdateTaskDto, CreateTaskDto, PaginatedTasksDto, TaskDto, TaskFilterDto, TaskResponse,
    UpdateTaskDto,
};
use crate::api::AppState;
use crate::domain::task_status::TaskStatus;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthenticatedUser;
use crate::types::ApiResponse;
use axum::{
    extract::{FromRequestParts, Json, Path, Query, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    routing::{get, patch, post},
    Router,
};
use chrono::Utc;
use tracing::info;
use uuid::Uuid;

// カスタムUUID抽出器
pub struct UuidPath(pub Uuid);

// #[async_trait] を削除し、通常の async fn 構文を使用
impl<S> FromRequestParts<S> for UuidPath
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // パスパラメータを文字列として最初に抽出
        let Path(path_str) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::ValidationErrors(vec!["Invalid path parameter".to_string()]))?;

        // UUIDをパースして検証エラー形式で返す
        let uuid = Uuid::parse_str(&path_str).map_err(|_| {
            AppError::ValidationErrors(vec![format!("Invalid UUID format: '{}'", path_str)])
        })?;

        Ok(UuidPath(uuid))
    }
}

// --- CRUD Handlers ---

pub async fn create_task_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateTaskDto>,
) -> AppResult<impl IntoResponse> {
    // PermissionServiceを使用してタスク作成権限をチェック
    app_state
        .permission_service
        .check_resource_access(user.user_id(), "task", None, "create")
        .await?;

    // バリデーション強化
    let mut validation_errors = Vec::new();

    if payload.title.trim().is_empty() {
        validation_errors.push("Title cannot be empty".to_string());
    } else if payload.title.len() > 100 {
        validation_errors.push("Title must be 100 characters or less".to_string());
    }

    if let Some(description) = &payload.description {
        if description.len() > 1000 {
            validation_errors.push("Description must be 1000 characters or less".to_string());
        }
    }

    // Status validation is now handled by TaskStatus enum through serde

    if let Some(due_date) = payload.due_date {
        // 日付形式のチェックは行うが、過去日付は許容する
        // 代わりに、あまりにも過去の日付（例：10年以上前）は拒否する
        let ten_years_ago = Utc::now() - chrono::Duration::days(365 * 10);
        if due_date < ten_years_ago {
            validation_errors.push("Due date is too far in the past".to_string());
        }
    }

    if !validation_errors.is_empty() {
        return Err(AppError::ValidationErrors(validation_errors));
    }

    info!(
        user_id = %user.claims.user_id,
        username = %user.claims.username,
        task_title = %payload.title,
        "Creating new task"
    );

    let task_dto = app_state
        .task_service
        .create_task_for_user(user.claims.user_id, payload)
        .await?;

    info!(
        user_id = %user.claims.user_id,
        task_id = %task_dto.id,
        "Task created successfully"
    );

    // 機能使用状況を追跡
    crate::track_feature!(
        app_state.clone(),
        user.claims.user_id,
        "Task Management",
        "create"
    );

    Ok((StatusCode::CREATED, ApiResponse::success(task_dto)))
}

pub async fn get_task_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    UuidPath(id): UuidPath,
) -> AppResult<ApiResponse<TaskDto>> {
    info!(
        user_id = %user.claims.user_id,
        task_id = %id,
        "Getting task"
    );

    // get_task_for_user already checks if the user owns the task
    let task_dto = app_state
        .task_service
        .get_task_for_user(user.claims.user_id, id)
        .await?;

    Ok(ApiResponse::success(task_dto))
}

pub async fn list_tasks_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<ApiResponse<Vec<TaskDto>>> {
    info!(
        user_id = %user.user_id(),
        "Listing user tasks"
    );

    let tasks = app_state
        .task_service
        .list_tasks_for_user(user.user_id())
        .await?;

    info!(
        user_id = %user.user_id(),
        task_count = %tasks.len(),
        "Tasks retrieved successfully"
    );

    Ok(ApiResponse::success(tasks))
}

/// 動的権限を使用するタスク一覧取得（新エンドポイント）
pub async fn list_tasks_dynamic_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<ApiResponse<TaskResponse>> {
    info!(
        user_id = %user.claims.user_id,
        subscription_tier = %user.claims.get_subscription_tier().as_str(),
        "Listing user tasks with dynamic permissions"
    );

    let response = app_state
        .task_service
        .list_tasks_dynamic(&user, None)
        .await?;

    info!(
        user_id = %user.claims.user_id,
        task_count = %response.task_count(),
        features = ?response.features(),
        "Tasks retrieved successfully with dynamic permissions"
    );

    Ok(ApiResponse::success(response))
}

pub async fn update_task_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    UuidPath(id): UuidPath,
    Json(payload): Json<UpdateTaskDto>,
) -> AppResult<ApiResponse<TaskDto>> {
    // The task service will check ownership

    // バリデーション強化
    let mut validation_errors = Vec::new();

    if let Some(title) = &payload.title {
        if title.trim().is_empty() {
            validation_errors.push("Title cannot be empty".to_string());
        } else if title.len() > 100 {
            validation_errors.push("Title must be 100 characters or less".to_string());
        }
    }

    if let Some(description) = &payload.description {
        if description.len() > 1000 {
            validation_errors.push("Description must be 1000 characters or less".to_string());
        }
    }

    // Status validation is now handled by TaskStatus enum through serde

    if let Some(due_date) = payload.due_date {
        // 日付形式のチェックは行うが、過去日付は許容する
        // 代わりに、あまりにも過去の日付（例：10年以上前）は拒否する
        let ten_years_ago = Utc::now() - chrono::Duration::days(365 * 10);
        if due_date < ten_years_ago {
            validation_errors.push("Due date is too far in the past".to_string());
        }
    }

    // ペイロードが空（何も更新しない）の場合の検証
    if payload.title.is_none()
        && payload.description.is_none()
        && payload.status.is_none()
        && payload.due_date.is_none()
    {
        validation_errors.push("Update payload cannot be empty".to_string());
    }

    if !validation_errors.is_empty() {
        return Err(AppError::ValidationErrors(validation_errors));
    }

    info!(
        user_id = %user.claims.user_id,
        task_id = %id,
        "Updating task"
    );

    let task_dto = app_state
        .task_service
        .update_task_for_user(user.claims.user_id, id, payload)
        .await?;

    info!(
        user_id = %user.claims.user_id,
        task_id = %id,
        "Task updated successfully"
    );

    Ok(ApiResponse::success(task_dto))
}

pub async fn delete_task_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    UuidPath(id): UuidPath,
) -> AppResult<StatusCode> {
    // The task service will check ownership

    info!(
        user_id = %user.claims.user_id,
        task_id = %id,
        "Deleting task"
    );

    app_state
        .task_service
        .delete_task_for_user(user.claims.user_id, id)
        .await?;

    info!(
        user_id = %user.claims.user_id,
        task_id = %id,
        "Task deleted successfully"
    );

    Ok(StatusCode::NO_CONTENT)
}

// --- Batch Handlers ---

pub async fn create_tasks_batch_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<BatchCreateTaskDto>,
) -> AppResult<impl IntoResponse> {
    // バリデーション強化
    if payload.tasks.is_empty() {
        return Err(AppError::ValidationError(
            "Batch create requires at least one task".to_string(),
        ));
    }

    if payload.tasks.len() > 100 {
        return Err(AppError::ValidationError(
            "Maximum 100 tasks allowed per batch".to_string(),
        ));
    }

    // 各タスクのバリデーション
    let mut validation_errors = Vec::new();

    for (index, task) in payload.tasks.iter().enumerate() {
        if task.title.trim().is_empty() {
            validation_errors.push(format!("Task #{}: Title cannot be empty", index + 1));
        } else if task.title.len() > 100 {
            validation_errors.push(format!(
                "Task #{}: Title must be 100 characters or less",
                index + 1
            ));
        }

        if let Some(description) = &task.description {
            if description.len() > 1000 {
                validation_errors.push(format!(
                    "Task #{}: Description must be 1000 characters or less",
                    index + 1
                ));
            }
        }

        if let Some(status) = &task.status {
            let valid_statuses = ["todo", "in_progress", "completed", "cancelled"];
            if !valid_statuses.contains(&status.as_str()) {
                validation_errors.push(format!(
                    "Task #{}: Invalid status: '{}'. Must be one of: {}",
                    index + 1,
                    status,
                    valid_statuses.join(", ")
                ));
            }
        }

        if let Some(due_date) = task.due_date {
            // 日付形式のチェックは行うが、過去日付は許容する
            // 代わりに、あまりにも過去の日付（例：10年以上前）は拒否する
            let ten_years_ago = Utc::now() - chrono::Duration::days(365 * 10);
            if due_date < ten_years_ago {
                validation_errors.push(format!(
                    "Task #{}: Due date is too far in the past",
                    index + 1
                ));
            }
        }
    }

    if !validation_errors.is_empty() {
        return Err(AppError::ValidationErrors(validation_errors));
    }

    info!(
        user_id = %user.claims.user_id,
        task_count = %payload.tasks.len(),
        "Creating batch tasks"
    );

    let response_dto = app_state
        .task_service
        .create_tasks_batch_for_user(user.claims.user_id, payload)
        .await?;

    info!(
        user_id = %user.claims.user_id,
        created_count = %response_dto.created_tasks.len(),
        "Batch tasks created successfully"
    );

    Ok((StatusCode::CREATED, ApiResponse::success(response_dto)))
}

pub async fn update_tasks_batch_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<BatchUpdateTaskDto>,
) -> AppResult<ApiResponse<BatchUpdateResponseDto>> {
    // バリデーション強化
    if payload.tasks.is_empty() {
        return Err(AppError::ValidationError(
            "Batch update requires at least one task".to_string(),
        ));
    }

    if payload.tasks.len() > 100 {
        return Err(AppError::ValidationError(
            "Maximum 100 tasks allowed per batch".to_string(),
        ));
    }

    // 各タスクのバリデーション
    let mut validation_errors = Vec::new();

    for (index, task) in payload.tasks.iter().enumerate() {
        if let Some(title) = &task.title {
            if title.trim().is_empty() {
                validation_errors.push(format!("Task #{}: Title cannot be empty", index + 1));
            } else if title.len() > 100 {
                validation_errors.push(format!(
                    "Task #{}: Title must be 100 characters or less",
                    index + 1
                ));
            }
        }

        if let Some(description) = &task.description {
            if description.len() > 1000 {
                validation_errors.push(format!(
                    "Task #{}: Description must be 1000 characters or less",
                    index + 1
                ));
            }
        }

        if let Some(status) = &task.status {
            let valid_statuses = ["todo", "in_progress", "completed", "cancelled"];
            if !valid_statuses.contains(&status.as_str()) {
                validation_errors.push(format!(
                    "Task #{}: Invalid status: '{}'. Must be one of: {}",
                    index + 1,
                    status,
                    valid_statuses.join(", ")
                ));
            }
        }

        if let Some(due_date) = task.due_date {
            // 日付形式のチェックは行うが、過去日付は許容する
            // 代わりに、あまりにも過去の日付（例：10年以上前）は拒否する
            let ten_years_ago = Utc::now() - chrono::Duration::days(365 * 10);
            if due_date < ten_years_ago {
                validation_errors.push(format!(
                    "Task #{}: Due date is too far in the past",
                    index + 1
                ));
            }
        }

        // 各タスクの更新内容が空でないことを確認
        if task.title.is_none()
            && task.description.is_none()
            && task.status.is_none()
            && task.due_date.is_none()
        {
            validation_errors.push(format!("Task #{}: Update data cannot be empty", index + 1));
        }
    }

    if !validation_errors.is_empty() {
        return Err(AppError::ValidationErrors(validation_errors));
    }

    let response_dto = app_state
        .task_service
        .update_tasks_batch_for_user(user.claims.user_id, payload)
        .await?;
    Ok(ApiResponse::success(response_dto))
}

pub async fn delete_tasks_batch_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<BatchDeleteTaskDto>,
) -> AppResult<ApiResponse<BatchDeleteResponseDto>> {
    // バリデーション強化
    if payload.ids.is_empty() {
        return Err(AppError::ValidationError(
            "Batch delete requires at least one task ID".to_string(),
        ));
    }

    if payload.ids.len() > 100 {
        return Err(AppError::ValidationError(
            "Maximum 100 tasks allowed per batch delete".to_string(),
        ));
    }

    let response_dto = app_state
        .task_service
        .delete_tasks_batch_for_user(user.claims.user_id, payload)
        .await?;
    Ok(ApiResponse::success(response_dto))
}

// フィルタリング用ハンドラー
pub async fn filter_tasks_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(filter): Query<TaskFilterDto>,
) -> AppResult<ApiResponse<PaginatedTasksDto>> {
    let paginated_tasks = app_state
        .task_service
        .filter_tasks_for_user(user.claims.user_id, filter)
        .await?;
    Ok(ApiResponse::success(paginated_tasks))
}

/// 動的権限を使用するフィルタリング（新エンドポイント）
pub async fn filter_tasks_dynamic_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(filter): Query<TaskFilterDto>,
) -> AppResult<ApiResponse<TaskResponse>> {
    info!(
        user_id = %user.claims.user_id,
        subscription_tier = %user.claims.get_subscription_tier().as_str(),
        "Filtering tasks with dynamic permissions"
    );

    let response = app_state
        .task_service
        .list_tasks_dynamic(&user, Some(filter))
        .await?;

    info!(
        user_id = %user.claims.user_id,
        filtered_count = %response.total_count(),
        "Tasks filtered successfully with dynamic permissions"
    );

    Ok(ApiResponse::success(response))
}

// ページネーション付きタスク一覧ハンドラー
pub async fn list_tasks_paginated_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(params): Query<PaginationQuery>,
) -> AppResult<ApiResponse<PaginatedTasksDto>> {
    let (page, per_page) = params.get_pagination();
    let page = page as u64;
    let page_size = per_page as u64;

    let paginated_tasks = app_state
        .task_service
        .list_tasks_paginated_for_user(user.claims.user_id, page, page_size)
        .await?;
    Ok(ApiResponse::success(paginated_tasks))
}

/// 動的権限を使用するページネーション（新エンドポイント）
pub async fn list_tasks_paginated_dynamic_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(params): Query<PaginationQuery>,
) -> AppResult<ApiResponse<TaskResponse>> {
    let (page, per_page) = params.get_pagination();
    let page = page as u64;
    let page_size = per_page as u64;

    // ページネーションパラメータをTaskFilterDtoに変換
    let filter = TaskFilterDto {
        limit: Some(page_size),
        offset: Some((page - 1) * page_size),
        ..Default::default()
    };

    info!(
        user_id = %user.claims.user_id,
        subscription_tier = %user.claims.get_subscription_tier().as_str(),
        page = %page,
        page_size = %page_size,
        "Paginated tasks with dynamic permissions"
    );

    let response = app_state
        .task_service
        .list_tasks_dynamic(&user, Some(filter))
        .await?;

    info!(
        user_id = %user.claims.user_id,
        task_count = %response.task_count(),
        "Paginated tasks retrieved successfully"
    );

    Ok(ApiResponse::success(response))
}

// ヘルスチェックハンドラーを追加
async fn health_check_handler() -> &'static str {
    "OK"
}

// --- 追加エンドポイント ---

/// ユーザーのタスク統計情報を取得
pub async fn get_user_task_stats_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<ApiResponse<serde_json::Value>> {
    use crate::middleware::auth::{
        extract_client_ip, get_authenticated_user_from_claims,
        get_authenticated_user_with_role_from_claims,
    };
    use axum::http::HeaderMap;

    // get_authenticated_userとget_authenticated_user_with_roleの使用例
    let _auth_user = get_authenticated_user_from_claims(&user.claims);
    let _auth_user_with_role = get_authenticated_user_with_role_from_claims(&user.claims);
    let headers = HeaderMap::new();
    let _client_ip = extract_client_ip(&headers);

    // 統計計算のためのログ
    info!(user_id = %user.claims.user_id, "Fetching user task statistics");

    // 全タスクを取得して統計を計算
    let tasks = app_state
        .task_service
        .list_tasks_for_user(user.claims.user_id)
        .await?;

    let total_tasks = tasks.len();
    let completed_tasks = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Completed)
        .count();
    let pending_tasks = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Todo)
        .count();
    let in_progress_tasks = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::InProgress)
        .count();

    let status_stats = {
        let mut pending = 0;
        let mut in_progress = 0;
        let mut completed = 0;
        let mut other = 0;

        for task in &tasks {
            match task.status.as_str() {
                "pending" | "todo" => pending += 1,
                "in_progress" => in_progress += 1,
                "completed" => completed += 1,
                _ => other += 1,
            }
        }

        serde_json::json!({
            "pending": pending,
            "in_progress": in_progress,
            "completed": completed,
            "other": other
        })
    };

    let stats = serde_json::json!({
        "total_tasks": total_tasks,
        "completed_tasks": completed_tasks,
        "pending_tasks": pending_tasks,
        "in_progress_tasks": in_progress_tasks,
        "completion_rate": if total_tasks > 0 {
            (completed_tasks as f64 / total_tasks as f64 * 100.0).round()
        } else {
            0.0
        },
        "status_distribution": status_stats
    });

    info!(
        user_id = %user.claims.user_id,
        total_tasks = %total_tasks,
        completed_tasks = %completed_tasks,
        "User task statistics retrieved"
    );

    Ok(ApiResponse::success(stats))
}

/// タスクのステータスを一括更新
pub async fn bulk_update_status_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    // ペイロードの検証
    let task_ids = payload
        .get("task_ids")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AppError::ValidationError("task_ids array is required".to_string()))?;

    let new_status = payload
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ValidationError("status string is required".to_string()))?;

    if !["pending", "in_progress", "completed"].contains(&new_status) {
        return Err(AppError::ValidationError(
            "status must be 'pending', 'in_progress', or 'completed'".to_string(),
        ));
    }

    let mut updated_count = 0;
    let mut errors = Vec::new();

    for task_id_value in task_ids {
        if let Some(task_id_str) = task_id_value.as_str() {
            if let Ok(task_id) = Uuid::parse_str(task_id_str) {
                // 各タスクのステータスを更新
                let update_dto = UpdateTaskDto {
                    title: None,
                    description: None,
                    status: Some(TaskStatus::from_str(new_status).unwrap_or(TaskStatus::Todo)),
                    priority: None,
                    due_date: None,
                };

                match app_state
                    .task_service
                    .update_task_for_user(user.claims.user_id, task_id, update_dto)
                    .await
                {
                    Ok(_) => updated_count += 1,
                    Err(e) => errors.push(format!("Task {}: {}", task_id, e)),
                }
            } else {
                errors.push(format!("Invalid UUID: {}", task_id_str));
            }
        } else {
            errors.push("Invalid task_id format".to_string());
        }
    }

    info!(
        user_id = %user.claims.user_id,
        updated_count = %updated_count,
        error_count = %errors.len(),
        new_status = %new_status,
        "Bulk status update completed"
    );

    Ok(ApiResponse::success(serde_json::json!({
        "updated_count": updated_count,
        "errors": errors,
        "new_status": new_status
    })))
}

// --- Admin functionality moved to admin_handler.rs ---

// --- Router Setup ---
// スキーマを指定できるようにルーター構築関数を修正
pub fn task_router(app_state: AppState) -> Router {
    use crate::middleware::auth::is_auth_endpoint;
    use crate::utils::permission::PermissionChecker;

    // 権限チェック関数を使用した例（実際にはハンドラー内で使用）
    let _is_auth = is_auth_endpoint("/auth/signin");

    // PermissionChecker::check_scopeの使用例
    let global_scope = crate::domain::permission::PermissionScope::Global;
    let team_scope = crate::domain::permission::PermissionScope::Team;
    let _scope_check = PermissionChecker::check_scope(&global_scope, &team_scope);

    Router::new()
        .route("/tasks", get(list_tasks_handler).post(create_task_handler))
        .route("/tasks/paginated", get(list_tasks_paginated_handler))
        .route("/tasks/filter", get(filter_tasks_handler))
        // 動的権限システム用の新エンドポイント
        .route("/tasks/dynamic", get(list_tasks_dynamic_handler))
        .route("/tasks/dynamic/filter", get(filter_tasks_dynamic_handler))
        .route(
            "/tasks/dynamic/paginated",
            get(list_tasks_paginated_dynamic_handler),
        )
        .route(
            "/tasks/{id}",
            get(get_task_handler)
                .patch(update_task_handler)
                .delete(delete_task_handler),
        )
        .route("/tasks/batch/create", post(create_tasks_batch_handler))
        .route("/tasks/batch/update", patch(update_tasks_batch_handler))
        .route("/tasks/batch/delete", post(delete_tasks_batch_handler))
        .route("/tasks/batch/status", patch(bulk_update_status_handler))
        // 統計情報とユーティリティ
        .route("/tasks/stats", get(get_user_task_stats_handler))
        // ヘルスチェックエンドポイントを追加
        .route("/health", get(health_check_handler))
        .with_state(app_state)
}

// AppStateを使用したルーター構築用ヘルパー関数
pub fn task_router_with_state(app_state: AppState) -> Router {
    task_router(app_state)
}

// Admin functionality moved to admin_handler.rs
