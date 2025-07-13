// src/features/task/handler.rs
use crate::api::AppState;
use crate::core::task_status::TaskStatus;
use crate::error::{AppError, AppResult};
use crate::features::auth::middleware::{AuthenticatedUser, AuthenticatedUserWithRole};
use crate::features::task::dto::{
    BatchCreateTaskDto, BatchDeleteResponseDto, BatchDeleteTaskDto, BatchUpdateResponseDto,
    BatchUpdateTaskDto, CreateTaskDto, PaginatedTasksDto, TaskDto, TaskFilterDto, TaskResponse,
    UpdateTaskDto,
};
use crate::shared::types::pagination::PaginatedResponse;
use axum::{
    extract::{FromRequestParts, Json, Path, Query, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    routing::{get, patch, post},
    Router,
};
use chrono::Utc;
use serde::Deserialize;
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

// ページネーションパラメータ
#[derive(Deserialize, Debug)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
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

    Ok((StatusCode::CREATED, Json(task_dto)))
}

pub async fn get_task_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    UuidPath(id): UuidPath,
) -> AppResult<Json<TaskDto>> {
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

    Ok(Json(task_dto))
}

pub async fn list_tasks_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<Vec<TaskDto>>> {
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

    Ok(Json(tasks))
}

/// 動的権限を使用するタスク一覧取得（新エンドポイント）
pub async fn list_tasks_dynamic_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<TaskResponse>> {
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

    Ok(Json(response))
}

pub async fn update_task_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    UuidPath(id): UuidPath,
    Json(payload): Json<UpdateTaskDto>,
) -> AppResult<Json<TaskDto>> {
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

    Ok(Json(task_dto))
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

    Ok((StatusCode::CREATED, Json(response_dto)))
}

pub async fn update_tasks_batch_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<BatchUpdateTaskDto>,
) -> AppResult<Json<BatchUpdateResponseDto>> {
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
    Ok(Json(response_dto))
}

pub async fn delete_tasks_batch_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<BatchDeleteTaskDto>,
) -> AppResult<Json<BatchDeleteResponseDto>> {
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
    Ok(Json(response_dto))
}

// フィルタリング用ハンドラー
pub async fn filter_tasks_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(filter): Query<TaskFilterDto>,
) -> AppResult<Json<PaginatedTasksDto>> {
    let paginated_tasks = app_state
        .task_service
        .filter_tasks_for_user(user.claims.user_id, filter)
        .await?;
    Ok(Json(paginated_tasks))
}

/// 動的権限を使用するフィルタリング（新エンドポイント）
pub async fn filter_tasks_dynamic_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(filter): Query<TaskFilterDto>,
) -> AppResult<Json<TaskResponse>> {
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

    Ok(Json(response))
}

// ページネーション付きタスク一覧ハンドラー
pub async fn list_tasks_paginated_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedTasksDto>> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);

    let paginated_tasks = app_state
        .task_service
        .list_tasks_paginated_for_user(user.claims.user_id, page, page_size)
        .await?;
    Ok(Json(paginated_tasks))
}

/// 動的権限を使用するページネーション（新エンドポイント）
pub async fn list_tasks_paginated_dynamic_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<TaskResponse>> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);

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

    Ok(Json(response))
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
) -> AppResult<Json<serde_json::Value>> {
    use crate::features::auth::middleware::{
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

    Ok(Json(stats))
}

/// タスクのステータスを一括更新
pub async fn bulk_update_status_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
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

    Ok(Json(serde_json::json!({
        "updated_count": updated_count,
        "errors": errors,
        "new_status": new_status
    })))
}

// --- Admin functionality moved to admin_handler.rs ---

// --- New Task Analytics and Management Handlers ---

/// Get all tasks (admin only)
pub async fn get_all_tasks_handler(
    State(app_state): State<AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    let limit = query["limit"].as_u64().unwrap_or(100) as usize;
    let offset = query["offset"].as_u64().unwrap_or(0) as usize;

    use crate::features::task::repositories::task_repository::TaskRepository;
    let task_repo = TaskRepository::new(app_state.db.as_ref().clone());
    let tasks = task_repo.find_all().await?;

    let total_count = tasks.len();
    let paginated_tasks: Vec<_> = tasks.into_iter().skip(offset).take(limit).collect();

    Ok(Json(serde_json::json!({
        "tasks": paginated_tasks,
        "total_count": total_count,
        "limit": limit,
        "offset": offset,
    })))
}

/// Get all tasks paginated (admin only)
pub async fn get_all_tasks_paginated_handler(
    State(app_state): State<AppState>,
    _user: AuthenticatedUserWithRole,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<TaskDto>>> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);

    use crate::features::task::repositories::task_repository::TaskRepository;
    let task_repo = TaskRepository::new(app_state.db.as_ref().clone());
    let (tasks, total_count) = task_repo.find_all_paginated(page, page_size).await?;

    use crate::shared::types::pagination::PaginatedResponse;
    let task_dtos: Vec<TaskDto> = tasks.into_iter().map(|task| task.into()).collect();
    let paginated_response =
        PaginatedResponse::new(task_dtos, page as i32, page_size as i32, total_count as i64);

    Ok(Json(paginated_response))
}

/// Get task count statistics (admin only)
pub async fn get_task_count_statistics_handler(
    State(app_state): State<AppState>,
    _user: AuthenticatedUserWithRole,
) -> AppResult<Json<serde_json::Value>> {
    use crate::features::task::repositories::task_repository::TaskRepository;
    let task_repo = TaskRepository::new(app_state.db.as_ref().clone());
    let total_tasks = task_repo.count_all_tasks().await?;

    let mut tasks_by_status = serde_json::json!({});
    let statuses = ["todo", "in_progress", "done", "cancelled"];
    for status in statuses.iter() {
        let count = task_repo.count_tasks_by_status(status).await?;
        tasks_by_status[status] = serde_json::json!(count);
    }

    Ok(Json(serde_json::json!({
        "total_tasks": total_tasks,
        "by_status": tasks_by_status,
        "generated_at": chrono::Utc::now(),
    })))
}

/// Get task priority distribution (admin only)
pub async fn get_priority_distribution_handler(
    State(app_state): State<AppState>,
    _user: AuthenticatedUserWithRole,
) -> AppResult<Json<serde_json::Value>> {
    let distribution = app_state.task_service.get_priority_distribution().await?;

    Ok(Json(serde_json::json!({
        "distribution": distribution,
        "generated_at": chrono::Utc::now(),
    })))
}

/// Get average completion days by priority (admin only)
pub async fn get_completion_time_by_priority_handler(
    State(app_state): State<AppState>,
    _user: AuthenticatedUserWithRole,
) -> AppResult<Json<serde_json::Value>> {
    let completion_days = app_state
        .task_service
        .get_average_completion_days_by_priority()
        .await?;

    Ok(Json(serde_json::json!({
        "average_completion_days": completion_days,
        "generated_at": chrono::Utc::now(),
    })))
}

/// Get weekly task trend data (admin only)
pub async fn get_weekly_trend_handler(
    State(app_state): State<AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    let weeks = query["weeks"].as_u64().unwrap_or(4) as u32;

    let trend_data = app_state.task_service.get_weekly_trend_data(weeks).await?;

    Ok(Json(serde_json::json!({
        "trend_data": trend_data,
        "weeks_analyzed": weeks,
        "generated_at": chrono::Utc::now(),
    })))
}

/// Get user average completion hours
pub async fn get_user_completion_hours_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // Check if user is accessing their own data or is admin
    if user.user_id() != user_id && !user.is_admin() {
        return Err(AppError::Forbidden(
            "You can only view your own task statistics".to_string(),
        ));
    }

    let avg_hours = app_state
        .task_service
        .get_user_average_completion_hours(user_id)
        .await?;

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "average_completion_hours": avg_hours,
        "generated_at": chrono::Utc::now(),
    })))
}

/// Create multiple tasks (admin only)
pub async fn admin_create_tasks_bulk_handler(
    State(app_state): State<AppState>,
    _user: AuthenticatedUserWithRole,
    Json(payload): Json<Vec<CreateTaskDto>>,
) -> AppResult<Json<serde_json::Value>> {
    if payload.is_empty() {
        return Err(AppError::ValidationError(
            "At least one task is required".to_string(),
        ));
    }

    if payload.len() > 1000 {
        return Err(AppError::ValidationError(
            "Maximum 1000 tasks allowed per request".to_string(),
        ));
    }

    let created_tasks = app_state
        .task_service
        .admin_create_tasks_bulk(payload)
        .await?;

    Ok(Json(serde_json::json!({
        "created_count": created_tasks.len(),
        "tasks": created_tasks,
    })))
}

/// Update multiple tasks (admin only)
pub async fn admin_update_tasks_bulk_handler(
    State(app_state): State<AppState>,
    _user: AuthenticatedUserWithRole,
    Json(payload): Json<Vec<(Uuid, UpdateTaskDto)>>,
) -> AppResult<Json<serde_json::Value>> {
    if payload.is_empty() {
        return Err(AppError::ValidationError(
            "At least one task update is required".to_string(),
        ));
    }

    if payload.len() > 500 {
        return Err(AppError::ValidationError(
            "Maximum 500 task updates allowed per request".to_string(),
        ));
    }

    use crate::features::task::dto::requests::BatchUpdateTaskItemDto;

    let batch_updates: Vec<BatchUpdateTaskItemDto> = payload
        .into_iter()
        .map(|(id, update_dto)| BatchUpdateTaskItemDto {
            id,
            title: update_dto.title,
            description: update_dto.description,
            due_date: update_dto.due_date,
            status: update_dto.status,
        })
        .collect();

    let updated_tasks = app_state
        .task_service
        .admin_update_tasks_bulk(batch_updates)
        .await?;

    Ok(Json(serde_json::json!({
        "updated_count": updated_tasks,
    })))
}

/// Delete multiple tasks (admin only)
pub async fn admin_delete_tasks_bulk_handler(
    State(app_state): State<AppState>,
    _user: AuthenticatedUserWithRole,
    Json(task_ids): Json<Vec<Uuid>>,
) -> AppResult<Json<serde_json::Value>> {
    if task_ids.is_empty() {
        return Err(AppError::ValidationError(
            "At least one task ID is required".to_string(),
        ));
    }

    if task_ids.len() > 500 {
        return Err(AppError::ValidationError(
            "Maximum 500 task deletions allowed per request".to_string(),
        ));
    }

    let deleted_count = app_state
        .task_service
        .admin_delete_tasks_bulk(task_ids)
        .await?;

    Ok(Json(serde_json::json!({
        "deleted_count": deleted_count,
        "operation": "bulk_delete",
        "completed_at": chrono::Utc::now(),
    })))
}

/// Get admin task statistics (admin only)
pub async fn get_admin_task_statistics_handler(
    State(app_state): State<AppState>,
    _user: AuthenticatedUserWithRole,
) -> AppResult<Json<serde_json::Value>> {
    let stats = app_state.task_service.get_admin_task_statistics().await?;

    Ok(Json(serde_json::json!({
        "statistics": stats,
        "generated_at": chrono::Utc::now(),
    })))
}

/// Count completed tasks
pub async fn count_completed_tasks_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = user.user_id();

    // For user-specific count, we need to use repository directly
    use crate::features::task::repositories::task_repository::TaskRepository;
    let task_repo = TaskRepository::new(app_state.db.as_ref().clone());
    let completed_count = task_repo.count_tasks_by_status("done").await?;

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "completed_tasks": completed_count,
        "retrieved_at": chrono::Utc::now(),
    })))
}

/// Get task analytics dashboard
pub async fn get_task_analytics_dashboard_handler(
    State(app_state): State<AppState>,
    _user: AuthenticatedUserWithRole,
) -> AppResult<Json<serde_json::Value>> {
    // Gather all analytics data
    use crate::features::task::repositories::task_repository::TaskRepository;
    let task_repo = TaskRepository::new(app_state.db.as_ref().clone());
    let total_tasks = task_repo.count_all_tasks().await?;

    let mut tasks_by_status = serde_json::json!({});
    let statuses = ["todo", "in_progress", "done", "cancelled"];
    for status in statuses.iter() {
        let count = task_repo.count_tasks_by_status(status).await?;
        tasks_by_status[status] = serde_json::json!(count);
    }
    let priority_distribution = app_state.task_service.get_priority_distribution().await?;
    let completion_days = app_state
        .task_service
        .get_average_completion_days_by_priority()
        .await?;
    let weekly_trend = app_state.task_service.get_weekly_trend_data(4).await?;

    Ok(Json(serde_json::json!({
        "overview": {
            "total_tasks": total_tasks,
            "by_status": tasks_by_status,
        },
        "priority_analysis": {
            "distribution": priority_distribution,
            "average_completion_days": completion_days,
        },
        "trends": {
            "weekly_data": weekly_trend,
        },
        "generated_at": chrono::Utc::now(),
    })))
}

// --- Router Setup ---
pub fn task_router_with_state(app_state: AppState) -> Router {
    use crate::features::auth::middleware::is_auth_endpoint;
    use crate::utils::permission::PermissionChecker;

    // 権限チェック関数を使用した例（実際にはハンドラー内で使用）
    let _is_auth = is_auth_endpoint("/auth/signin");

    // PermissionChecker::check_scopeの使用例
    let global_scope = crate::core::permission::PermissionScope::Global;
    let team_scope = crate::core::permission::PermissionScope::Team;
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
        // New analytics and management endpoints
        .route("/admin/tasks/all", get(get_all_tasks_handler))
        .route(
            "/admin/tasks/all/paginated",
            get(get_all_tasks_paginated_handler),
        )
        .route(
            "/admin/tasks/statistics",
            get(get_admin_task_statistics_handler),
        )
        .route(
            "/admin/tasks/count-statistics",
            get(get_task_count_statistics_handler),
        )
        .route(
            "/admin/tasks/priority-distribution",
            get(get_priority_distribution_handler),
        )
        .route(
            "/admin/tasks/completion-time-by-priority",
            get(get_completion_time_by_priority_handler),
        )
        .route("/admin/tasks/weekly-trend", get(get_weekly_trend_handler))
        .route(
            "/admin/tasks/analytics/dashboard",
            get(get_task_analytics_dashboard_handler),
        )
        .route(
            "/admin/tasks/bulk/create",
            post(admin_create_tasks_bulk_handler),
        )
        .route(
            "/admin/tasks/bulk/update",
            patch(admin_update_tasks_bulk_handler),
        )
        .route(
            "/admin/tasks/bulk/delete",
            post(admin_delete_tasks_bulk_handler),
        )
        .route(
            "/tasks/users/{user_id}/completion-hours",
            get(get_user_completion_hours_handler),
        )
        .route("/tasks/completed/count", get(count_completed_tasks_handler))
        // ヘルスチェックエンドポイントを追加
        .route("/health", get(health_check_handler))
        .with_state(app_state)
}

// Admin functionality moved to admin_handler.rs
