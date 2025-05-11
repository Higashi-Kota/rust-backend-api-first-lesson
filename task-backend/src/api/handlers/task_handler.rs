// src/api/handlers/task_handler.rs
use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, patch}, // patch を追加
    Router,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::service::task_service::TaskService;
use crate::api::dto::task_dto::*;
use crate::error::{AppResult, AppError}; // AppError をインポート

// アプリケーションの状態を保持する構造体 (axum の State で渡される)
// Clone が必要
#[derive(Clone)]
pub struct AppState {
    pub task_service: Arc<TaskService>,
}

// --- CRUD Handlers ---

pub async fn create_task_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateTaskDto>,
) -> AppResult<impl IntoResponse> {
    // バリデーションは DTO 側で行うか、ハンドラで明示的に行う
    // ここでは title が空でないか程度の簡単なチェック例
    if payload.title.trim().is_empty() {
        return Err(AppError::ValidationError("Title cannot be empty".to_string()));
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
    // ここでも payload.tasks の各要素に対するバリデーションが可能
    let response_dto = app_state.task_service.create_tasks_batch(payload).await?;
    Ok((StatusCode::CREATED, Json(response_dto)))
}

pub async fn update_tasks_batch_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<BatchUpdateTaskDto>,
) -> AppResult<Json<BatchUpdateResponseDto>> {
    let response_dto = app_state.task_service.update_tasks_batch(payload).await?;
    Ok(Json(response_dto))
}

pub async fn delete_tasks_batch_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<BatchDeleteTaskDto>, // クエリパラメータで "?ids=uuid1,uuid2" の形式も一般的
) -> AppResult<Json<BatchDeleteResponseDto>> {
    let response_dto = app_state.task_service.delete_tasks_batch(payload).await?;
    Ok(Json(response_dto))
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
            "/tasks/:id",
            get(get_task_handler)
            .patch(update_task_handler) // HTTP PATCH を使うことが多い
            .delete(delete_task_handler)
        )
        .route( // 一括操作用のエンドポイント
            "/tasks/batch/create", // または POST /tasks/batch (リクエストボディで操作種別を判別)
            post(create_tasks_batch_handler)
        )
        .route(
            "/tasks/batch/update", // または PATCH /tasks/batch
            patch(update_tasks_batch_handler) // 一括更新は PATCH が適切
        )
        .route(
            "/tasks/batch/delete", // または DELETE /tasks/batch (リクエストボディでIDリスト)
            post(delete_tasks_batch_handler) // DELETEメソッドはボディを持てないことがあるのでPOSTで代用も
        )
        .with_state(app_state) // Router全体にStateを適用
}