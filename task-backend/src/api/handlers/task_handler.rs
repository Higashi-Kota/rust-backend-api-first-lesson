// src/api/handlers/task_handler.rs
use crate::api::dto::common::PaginationQuery;
use crate::api::dto::dynamic_permission_dto::{DynamicTaskResponse, SubscriptionTier, TierLimits};
use crate::api::dto::task_dto::{
    BatchCreateTaskDto, BatchDeleteResponseDto, BatchDeleteTaskDto, BatchUpdateResponseDto,
    BatchUpdateTaskDto, CreateTaskDto, PaginatedTasksDto, TaskDto, UpdateTaskDto,
};
use crate::api::dto::task_query_dto::TaskSearchQuery;
use crate::api::dto::team_task_dto::{
    AssignTaskRequest, CreateOrganizationTaskRequest, CreateTeamTaskRequest, TransferTaskRequest,
    TransferTaskResponse,
};
use crate::api::AppState;
use crate::domain::task_status::TaskStatus;
use crate::error::{AppError, AppResult};
use crate::extractors::ValidatedUuid;
use crate::middleware::auth::AuthenticatedUser;
use crate::middleware::authorization::{resources, Action, PermissionContext};
use crate::require_permission;
use crate::types::ApiResponse;
use crate::utils::error_helper::{convert_validation_errors, internal_server_error};
use axum::{
    extract::{Extension, Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Router,
};
use chrono::Utc;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

// --- CRUD Handlers ---

pub async fn create_task_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateTaskDto>,
) -> AppResult<impl IntoResponse> {
    // TODO: 統一権限チェックミドルウェアへ移行予定
    // require_permission!(resources::TASK, Action::Create)を使用
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
        return Err(AppError::BadRequest(validation_errors.join(", ")));
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
    ValidatedUuid(id): ValidatedUuid,
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

/// PermissionContextを活用したタスク取得ハンドラー
/// ミドルウェアで検証済みの権限情報を使用
pub async fn get_task_with_context_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Extension(permission_ctx): Extension<PermissionContext>,
    ValidatedUuid(id): ValidatedUuid,
) -> AppResult<ApiResponse<TaskDto>> {
    info!(
        user_id = %user.claims.user_id,
        task_id = %id,
        resource = %permission_ctx.resource,
        action = ?permission_ctx.action,
        "Getting task with permission context"
    );

    // PermissionContextを使用した追加の権限チェック
    app_state
        .task_service
        .check_task_access_with_context(&permission_ctx, id)
        .await?;

    // タスクを取得
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

// --- Batch Handlers ---

pub async fn create_tasks_batch_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<BatchCreateTaskDto>,
) -> AppResult<impl IntoResponse> {
    // Freeティアユーザーはバッチ作成不可
    use crate::domain::subscription_tier::SubscriptionTier;
    let user_data = app_state
        .user_service
        .get_user_by_id(user.user_id())
        .await
        .map_err(|e| {
            internal_server_error(
                e,
                "task_handler::create_tasks_batch",
                "Failed to fetch user",
            )
        })?;

    // 管理者はEnterpriseティアとして扱う
    let user_tier = if user.is_admin() {
        SubscriptionTier::Enterprise
    } else {
        SubscriptionTier::from_str(&user_data.subscription_tier).unwrap_or(SubscriptionTier::Free)
    };

    if user_tier == SubscriptionTier::Free {
        return Err(AppError::Forbidden(
            "Batch task creation is a premium feature. Please upgrade your subscription to use this feature.".to_string(),
        ));
    }

    // バリデーション強化
    if payload.tasks.is_empty() {
        return Err(AppError::BadRequest(
            "Batch create requires at least one task".to_string(),
        ));
    }

    if payload.tasks.len() > 100 {
        return Err(AppError::BadRequest(
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
        return Err(AppError::BadRequest(validation_errors.join(", ")));
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
        return Err(AppError::BadRequest(
            "Batch update requires at least one task".to_string(),
        ));
    }

    if payload.tasks.len() > 100 {
        return Err(AppError::BadRequest(
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
        return Err(AppError::BadRequest(validation_errors.join(", ")));
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
        return Err(AppError::BadRequest(
            "Batch delete requires at least one task ID".to_string(),
        ));
    }

    if payload.ids.len() > 100 {
        return Err(AppError::BadRequest(
            "Maximum 100 tasks allowed per batch delete".to_string(),
        ));
    }

    let response_dto = app_state
        .task_service
        .delete_tasks_batch_for_user(user.claims.user_id, payload)
        .await?;
    Ok(ApiResponse::success(response_dto))
}

// filter_tasks_handler removed - use search_tasks_handler instead

// 統一検索クエリハンドラー
pub async fn search_tasks_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<TaskSearchQuery>,
) -> AppResult<ApiResponse<PaginatedTasksDto>> {
    info!(
        user_id = %user.claims.user_id,
        search = ?query.search,
        page = ?query.pagination.page,
        per_page = ?query.pagination.per_page,
        allowed_sort_fields = ?TaskSearchQuery::allowed_sort_fields(),
        "Searching tasks with unified query"
    );

    let paginated_tasks = app_state
        .task_service
        .search_tasks_for_user(user.claims.user_id, query)
        .await?;

    Ok(ApiResponse::success(paginated_tasks))
}

// ページネーション付きタスク一覧ハンドラー
pub async fn list_tasks_paginated_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(params): Query<PaginationQuery>,
) -> AppResult<ApiResponse<PaginatedTasksDto>> {
    let (page, per_page) = params.get_pagination();
    let page = page as u64;
    let per_page = per_page as u64;

    let paginated_tasks = app_state
        .task_service
        .list_tasks_paginated_for_user(user.claims.user_id, page, per_page)
        .await?;
    Ok(ApiResponse::success(paginated_tasks))
}

// --- Multi-tenant Handlers (from task_handler_v2.rs) ---

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
    ValidatedUuid(team_id): ValidatedUuid,
    Json(mut payload): Json<CreateTeamTaskRequest>,
) -> AppResult<impl IntoResponse> {
    // パスパラメータのteam_idをペイロードに設定
    payload.team_id = team_id;

    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "task_handler::create_team_task"))?;

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
    ValidatedUuid(organization_id): ValidatedUuid,
    Json(mut payload): Json<CreateOrganizationTaskRequest>,
) -> AppResult<impl IntoResponse> {
    // パスパラメータのorganization_idをペイロードに設定
    payload.organization_id = organization_id;

    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "task_handler::create_organization_task"))?;

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
    ValidatedUuid(task_id): ValidatedUuid,
    Json(payload): Json<AssignTaskRequest>,
) -> AppResult<impl IntoResponse> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "task_handler::assign_task"))?;

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
    ValidatedUuid(task_id): ValidatedUuid,
    Json(payload): Json<UpdateTaskDto>,
) -> AppResult<impl IntoResponse> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "task_handler::update_multi_tenant_task"))?;

    info!(
        user_id = %auth_user.claims.user_id,
        task_id = %task_id,
        "Updating multi-tenant task"
    );

    let task_dto = app_state
        .task_service
        .update_task_for_user(auth_user.claims.user_id, task_id, payload)
        .await?;

    Ok(ApiResponse::<TaskDto>::success(task_dto))
}

/// マルチテナントタスク削除ハンドラー
pub async fn delete_multi_tenant_task(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    ValidatedUuid(task_id): ValidatedUuid,
) -> AppResult<StatusCode> {
    info!(
        user_id = %auth_user.claims.user_id,
        task_id = %task_id,
        "Deleting multi-tenant task"
    );

    app_state
        .task_service
        .delete_task_for_user(auth_user.claims.user_id, task_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// タスク引き継ぎハンドラー
pub async fn transfer_task(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    ValidatedUuid(task_id): ValidatedUuid,
    Json(payload): Json<TransferTaskRequest>,
) -> AppResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "task_handler::transfer_task"))?;

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
        .ok_or_else(|| AppError::BadRequest("task_ids array is required".to_string()))?;

    let new_status = payload
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("status string is required".to_string()))?;

    if !["pending", "in_progress", "completed"].contains(&new_status) {
        return Err(AppError::BadRequest(
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

/// 動的権限ベースのタスク取得
pub async fn get_tasks_dynamic_permissions(
    State(app_state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<TaskSearchQuery>,
) -> AppResult<impl IntoResponse> {
    info!(
        user_id = %user.user_id(),
        "Fetching tasks with dynamic permissions"
    );

    // ユーザーのサブスクリプションティアを取得（現在はモック）
    let tier = if user.is_admin() {
        SubscriptionTier::Enterprise
    } else {
        // TODO: 実際のサブスクリプション情報をデータベースから取得
        SubscriptionTier::Free
    };

    // ティアに基づく制限を取得
    let limits = TierLimits::for_tier(tier);

    // タスクを取得
    let tasks = app_state
        .task_service
        .search_tasks_for_user(user.user_id(), params)
        .await?;

    // ティアに基づいてレスポンスをフォーマット
    let response = match tier {
        SubscriptionTier::Free => {
            let task_count = tasks.items.len();
            let limit_reached = limits.max_tasks.is_some_and(|max| task_count >= max);

            DynamicTaskResponse::Free {
                tasks: tasks.items,
                quota_info: format!(
                    "You are using {} of {} tasks",
                    task_count,
                    limits.max_tasks.unwrap_or(0)
                ),
                limit_reached,
            }
        }
        SubscriptionTier::Pro => DynamicTaskResponse::Pro {
            tasks: tasks.items,
            features: vec![
                "Bulk operations".to_string(),
                "Advanced analytics".to_string(),
                "Priority support".to_string(),
            ],
            export_available: true,
        },
        SubscriptionTier::Enterprise => DynamicTaskResponse::Unlimited {
            items: tasks.items,
            pagination: tasks.pagination,
        },
    };

    Ok(ApiResponse::success(response))
}

// ヘルスチェックハンドラーを追加
pub async fn health_check_handler() -> &'static str {
    "OK"
}

// --- Admin functionality moved to admin_handler.rs ---

// --- Router Setup ---
// 統一されたタスクルーター（統一権限チェックミドルウェア適用済み）
pub fn task_router(app_state: AppState) -> Router {
    Router::new()
        // === 基本的なタスク操作 ===
        // タスク一覧取得・作成
        .route(
            "/tasks",
            get(list_tasks_handler)
                .route_layer(require_permission!(resources::TASK, Action::View))
                .post(create_task_handler)
                .route_layer(require_permission!(resources::TASK, Action::Create)),
        )
        // ページネーション付きタスク一覧
        .route(
            "/tasks/paginated",
            get(list_tasks_paginated_handler)
                .route_layer(require_permission!(resources::TASK, Action::View)),
        )
        // タスク検索
        .route(
            "/tasks/search",
            get(search_tasks_handler)
                .route_layer(require_permission!(resources::TASK, Action::View)),
        )
        // 個別タスク操作（取得・更新・削除）
        // マルチテナント対応: チームタスクも個人タスクも同じエンドポイントで処理
        .route("/tasks/{id}", get(get_task_handler))
        .route("/tasks/{id}", patch(update_multi_tenant_task))
        .route("/tasks/{id}", delete(delete_multi_tenant_task))
        // PermissionContextを使用した詳細な権限チェック付きタスク取得
        .route(
            "/tasks/{id}/with-context",
            get(get_task_with_context_handler)
                .route_layer(require_permission!(resources::TASK, Action::View)),
        )
        // === バッチ操作 ===
        .route(
            "/tasks/batch/create",
            post(create_tasks_batch_handler)
                .route_layer(require_permission!(resources::TASK, Action::Create)),
        )
        .route("/tasks/batch/update", patch(update_tasks_batch_handler))
        .route(
            "/tasks/batch/delete",
            post(delete_tasks_batch_handler)
                .route_layer(require_permission!(resources::TASK, Action::View)),
        )
        .route("/tasks/batch/status", patch(bulk_update_status_handler))
        // === マルチテナント機能 ===
        // スコープベースのタスク取得
        .route(
            "/tasks/scoped",
            get(list_tasks_with_scope)
                .route_layer(require_permission!(resources::TASK, Action::View)),
        )
        // チームタスク作成
        .route(
            "/teams/{team_id}/tasks",
            post(create_team_task)
                .route_layer(require_permission!(resources::TEAM, Action::Update)),
        )
        // 組織タスク作成
        .route(
            "/organizations/{organization_id}/tasks",
            post(create_organization_task)
                .route_layer(require_permission!(resources::ORGANIZATION, Action::Update)),
        )
        // タスク割り当て
        .route("/tasks/{id}/assign", post(assign_task))
        // タスク引き継ぎ
        .route("/tasks/{id}/transfer", post(transfer_task))
        // === 統計・ユーティリティ ===
        .route(
            "/tasks/stats",
            get(get_user_task_stats_handler)
                .route_layer(require_permission!(resources::TASK, Action::View)),
        )
        // === 動的権限 ===
        .route(
            "/tasks/dynamic",
            get(get_tasks_dynamic_permissions)
                .route_layer(require_permission!(resources::TASK, Action::View)),
        )
        // ヘルスチェック（認証不要）
        .route("/health", get(health_check_handler))
        .with_state(app_state)
}

// AppStateを使用したルーター構築用ヘルパー関数
pub fn task_router_with_state(app_state: AppState) -> Router {
    task_router(app_state)
}

// Admin functionality moved to admin_handler.rs
