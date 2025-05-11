// src/api/handlers/task_handler.rs
use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, patch},
    Router,
};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use serde::Deserialize;
use crate::service::task_service::TaskService;
use crate::api::dto::task_dto::*;
use crate::error::{AppResult, AppError};

// アプリケーションの状態を保持する構造体 (axum の State で渡される)
// Clone が必要
#[derive(Clone)]
pub struct AppState {
    pub task_service: Arc<TaskService>,
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
    Json(payload): Json<CreateTaskDto>,
) -> AppResult<impl IntoResponse> {
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
    
    if let Some(status) = &payload.status {
        let valid_statuses = ["todo", "in_progress", "completed", "cancelled"];
        if !valid_statuses.contains(&status.as_str()) {
            validation_errors.push(format!("Invalid status: '{}'. Must be one of: {}", 
                status, valid_statuses.join(", ")));
        }
    }
    
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
    
    let task_dto = app_state.task_service.create_task(payload).await?;
    Ok((StatusCode::CREATED, Json(task_dto)))
}

pub async fn get_task_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<TaskDto>> {
    let task_dto = app_state.task_service.get_task(id).await?;
    Ok(Json(task_dto))
}

pub async fn list_tasks_handler(
    State(app_state): State<AppState>,
) -> AppResult<Json<Vec<TaskDto>>> {
    let tasks = app_state.task_service.list_tasks().await?;
    Ok(Json(tasks))
}

pub async fn update_task_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTaskDto>,
) -> AppResult<Json<TaskDto>> {
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
    
    if let Some(status) = &payload.status {
        let valid_statuses = ["todo", "in_progress", "completed", "cancelled"];
        if !valid_statuses.contains(&status.as_str()) {
            validation_errors.push(format!("Invalid status: '{}'. Must be one of: {}", 
                status, valid_statuses.join(", ")));
        }
    }
    
    if let Some(due_date) = payload.due_date {
        // 日付形式のチェックは行うが、過去日付は許容する
        // 代わりに、あまりにも過去の日付（例：10年以上前）は拒否する
        let ten_years_ago = Utc::now() - chrono::Duration::days(365 * 10);
        if due_date < ten_years_ago {
            validation_errors.push("Due date is too far in the past".to_string());
        }
    }
    
    // ペイロードが空（何も更新しない）の場合の検証
    if payload.title.is_none() && payload.description.is_none() && 
       payload.status.is_none() && payload.due_date.is_none() {
        validation_errors.push("Update payload cannot be empty".to_string());
    }
    
    if !validation_errors.is_empty() {
        return Err(AppError::ValidationErrors(validation_errors));
    }
    
    let task_dto = app_state.task_service.update_task(id, payload).await?;
    Ok(Json(task_dto))
}

pub async fn delete_task_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    app_state.task_service.delete_task(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Batch Handlers ---

pub async fn create_tasks_batch_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<BatchCreateTaskDto>,
) -> AppResult<impl IntoResponse> {
    // バリデーション強化
    if payload.tasks.is_empty() {
        return Err(AppError::ValidationError("Batch create requires at least one task".to_string()));
    }
    
    if payload.tasks.len() > 100 {
        return Err(AppError::ValidationError("Maximum 100 tasks allowed per batch".to_string()));
    }
    
    // 各タスクのバリデーション
    let mut validation_errors = Vec::new();
    
    for (index, task) in payload.tasks.iter().enumerate() {
        if task.title.trim().is_empty() {
            validation_errors.push(format!("Task #{}: Title cannot be empty", index + 1));
        } else if task.title.len() > 100 {
            validation_errors.push(format!("Task #{}: Title must be 100 characters or less", index + 1));
        }
        
        if let Some(description) = &task.description {
            if description.len() > 1000 {
                validation_errors.push(format!("Task #{}: Description must be 1000 characters or less", index + 1));
            }
        }
        
        if let Some(status) = &task.status {
            let valid_statuses = ["todo", "in_progress", "completed", "cancelled"];
            if !valid_statuses.contains(&status.as_str()) {
                validation_errors.push(format!("Task #{}: Invalid status: '{}'. Must be one of: {}", 
                    index + 1, status, valid_statuses.join(", ")));
            }
        }
        
        if let Some(due_date) = task.due_date {
            // 日付形式のチェックは行うが、過去日付は許容する
            // 代わりに、あまりにも過去の日付（例：10年以上前）は拒否する
            let ten_years_ago = Utc::now() - chrono::Duration::days(365 * 10);
            if due_date < ten_years_ago {
                validation_errors.push(format!("Task #{}: Due date is too far in the past", index + 1));
            }
        }
    }
    
    if !validation_errors.is_empty() {
        return Err(AppError::ValidationErrors(validation_errors));
    }
    
    let response_dto = app_state.task_service.create_tasks_batch(payload).await?;
    Ok((StatusCode::CREATED, Json(response_dto)))
}

pub async fn update_tasks_batch_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<BatchUpdateTaskDto>,
) -> AppResult<Json<BatchUpdateResponseDto>> {
    // バリデーション強化
    if payload.tasks.is_empty() {
        return Err(AppError::ValidationError("Batch update requires at least one task".to_string()));
    }
    
    if payload.tasks.len() > 100 {
        return Err(AppError::ValidationError("Maximum 100 tasks allowed per batch".to_string()));
    }
    
    // 各タスクのバリデーション
    let mut validation_errors = Vec::new();
    
    for (index, task) in payload.tasks.iter().enumerate() {
        if let Some(title) = &task.title {
            if title.trim().is_empty() {
                validation_errors.push(format!("Task #{}: Title cannot be empty", index + 1));
            } else if title.len() > 100 {
                validation_errors.push(format!("Task #{}: Title must be 100 characters or less", index + 1));
            }
        }
        
        if let Some(description) = &task.description {
            if description.len() > 1000 {
                validation_errors.push(format!("Task #{}: Description must be 1000 characters or less", index + 1));
            }
        }
        
        if let Some(status) = &task.status {
            let valid_statuses = ["todo", "in_progress", "completed", "cancelled"];
            if !valid_statuses.contains(&status.as_str()) {
                validation_errors.push(format!("Task #{}: Invalid status: '{}'. Must be one of: {}", 
                    index + 1, status, valid_statuses.join(", ")));
            }
        }
        
        if let Some(due_date) = task.due_date {
            // 日付形式のチェックは行うが、過去日付は許容する
            // 代わりに、あまりにも過去の日付（例：10年以上前）は拒否する
            let ten_years_ago = Utc::now() - chrono::Duration::days(365 * 10);
            if due_date < ten_years_ago {
                validation_errors.push(format!("Task #{}: Due date is too far in the past", index + 1));
            }
        }
        
        // 各タスクの更新内容が空でないことを確認
        if task.title.is_none() && task.description.is_none() && 
           task.status.is_none() && task.due_date.is_none() {
            validation_errors.push(format!("Task #{}: Update data cannot be empty", index + 1));
        }
    }
    
    if !validation_errors.is_empty() {
        return Err(AppError::ValidationErrors(validation_errors));
    }
    
    let response_dto = app_state.task_service.update_tasks_batch(payload).await?;
    Ok(Json(response_dto))
}

pub async fn delete_tasks_batch_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<BatchDeleteTaskDto>, 
) -> AppResult<Json<BatchDeleteResponseDto>> {
    // バリデーション強化
    if payload.ids.is_empty() {
        return Err(AppError::ValidationError("Batch delete requires at least one task ID".to_string()));
    }
    
    if payload.ids.len() > 100 {
        return Err(AppError::ValidationError("Maximum 100 tasks allowed per batch delete".to_string()));
    }
    
    let response_dto = app_state.task_service.delete_tasks_batch(payload).await?;
    Ok(Json(response_dto))
}

// フィルタリング用ハンドラー
pub async fn filter_tasks_handler(
    State(app_state): State<AppState>,
    Query(filter): Query<TaskFilterDto>,
) -> AppResult<Json<PaginatedTasksDto>> {
    let paginated_tasks = app_state.task_service.filter_tasks(filter).await?;
    Ok(Json(paginated_tasks))
}

// ページネーション付きタスク一覧ハンドラー
pub async fn list_tasks_paginated_handler(
    State(app_state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedTasksDto>> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);
    
    let paginated_tasks = app_state.task_service.list_tasks_paginated(page, page_size).await?;
    Ok(Json(paginated_tasks))
}

// --- Router Setup ---
pub fn task_router(app_state: AppState) -> Router {
    Router::new()
        .route(
            "/tasks",
            get(list_tasks_handler)
            .post(create_task_handler)
        )
        .route(
            "/tasks/paginated",
            get(list_tasks_paginated_handler)
        )
        .route(
            "/tasks/filter",
            get(filter_tasks_handler)
        )
        .route(
            "/tasks/:id",
            get(get_task_handler)
            .patch(update_task_handler)
            .delete(delete_task_handler)
        )
        .route(
            "/tasks/batch/create",
            post(create_tasks_batch_handler)
        )
        .route(
            "/tasks/batch/update",
            patch(update_tasks_batch_handler)
        )
        .route(
            "/tasks/batch/delete",
            post(delete_tasks_batch_handler)
        )
        .with_state(app_state)
}