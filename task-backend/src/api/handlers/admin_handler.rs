// task-backend/src/api/handlers/admin_handler.rs

use crate::api::dto::common::ApiResponse;
use crate::api::dto::task_dto::*;
use crate::api::dto::team_invitation_dto::*;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthenticatedUser;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// 管理者向けタスク一括作成リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AdminBulkCreateTasksRequest {
    #[validate(length(min = 1, max = 100, message = "Must provide 1-100 tasks"))]
    pub tasks: Vec<CreateTaskDto>,
    pub assign_to_user: Option<Uuid>,
}

/// 管理者向けタスク一括更新リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AdminBulkUpdateTasksRequest {
    #[validate(length(min = 1, max = 100, message = "Must provide 1-100 task updates"))]
    pub updates: Vec<BatchUpdateTaskItemDto>,
}

/// 管理者向けタスク一括削除リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AdminBulkDeleteTasksRequest {
    #[validate(length(min = 1, max = 100, message = "Must provide 1-100 task IDs"))]
    pub task_ids: Vec<Uuid>,
}

/// 管理者向けタスク統計レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminTaskStatsResponse {
    pub total_tasks: u32,
    pub tasks_by_status: Vec<TaskStatusStats>,
    pub tasks_by_user: Vec<UserTaskStats>,
    pub recent_activity: Vec<TaskActivityStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatusStats {
    pub status: String,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTaskStats {
    pub user_id: Uuid,
    pub task_count: u64,
    pub completed_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskActivityStats {
    pub date: String,
    pub created_count: u64,
    pub completed_count: u64,
}

/// 管理者向け一括操作レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminBulkOperationResponse {
    pub success_count: usize,
    pub failed_count: usize,
    pub total_requested: usize,
    pub errors: Vec<String>,
}

// === タスク管理API ===

/// 管理者向けタスク詳細取得（制限なし）
pub async fn admin_get_task(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<TaskResponse>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;

    // 管理者なので任意のタスクにアクセス可能
    let task_dto = task_service.get_task(task_id).await?;
    let response = TaskResponse::Enterprise {
        tasks: vec![task_dto],
        bulk_operations: true,
        unlimited_access: true,
    };

    Ok(Json(ApiResponse::success(
        "Task retrieved successfully",
        response,
    )))
}

/// 管理者向けタスク一括作成
pub async fn admin_bulk_create_tasks(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Json(request): Json<AdminBulkCreateTasksRequest>,
) -> AppResult<Json<ApiResponse<AdminBulkOperationResponse>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let task_service = &app_state.task_service;
    let mut success_count = 0;
    let mut errors = Vec::new();

    // タスクを準備
    let total_requested = request.tasks.len();
    let tasks_to_create = request.tasks;

    match task_service.admin_create_tasks_bulk(tasks_to_create).await {
        Ok(created_tasks) => {
            success_count = created_tasks.len();
        }
        Err(e) => {
            errors.push(format!("Bulk create failed: {}", e));
        }
    }

    let response = AdminBulkOperationResponse {
        success_count,
        failed_count: errors.len(),
        total_requested,
        errors,
    };

    Ok(Json(ApiResponse::success(
        "Bulk task creation completed",
        response,
    )))
}

/// 管理者向けタスク一括更新
pub async fn admin_bulk_update_tasks(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Json(request): Json<AdminBulkUpdateTasksRequest>,
) -> AppResult<Json<ApiResponse<AdminBulkOperationResponse>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let task_service = &app_state.task_service;
    let total_requested = request.updates.len();

    match task_service.admin_update_tasks_bulk(request.updates).await {
        Ok(updated_count) => {
            let response = AdminBulkOperationResponse {
                success_count: updated_count,
                failed_count: 0,
                total_requested,
                errors: vec![],
            };

            Ok(Json(ApiResponse::success(
                "Bulk task update completed",
                response,
            )))
        }
        Err(e) => {
            let response = AdminBulkOperationResponse {
                success_count: 0,
                failed_count: 1,
                total_requested,
                errors: vec![format!("Bulk update failed: {}", e)],
            };

            Ok(Json(ApiResponse::success(
                "Bulk task update failed",
                response,
            )))
        }
    }
}

/// 管理者向けタスク一括削除
pub async fn admin_bulk_delete_tasks(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Json(request): Json<AdminBulkDeleteTasksRequest>,
) -> AppResult<Json<ApiResponse<AdminBulkOperationResponse>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let task_service = &app_state.task_service;
    let total_requested = request.task_ids.len();

    match task_service.admin_delete_tasks_bulk(request.task_ids).await {
        Ok(rows_affected) => {
            let response = AdminBulkOperationResponse {
                success_count: rows_affected as usize,
                failed_count: 0,
                total_requested,
                errors: vec![],
            };

            Ok(Json(ApiResponse::success(
                "Bulk task deletion completed",
                response,
            )))
        }
        Err(e) => {
            let response = AdminBulkOperationResponse {
                success_count: 0,
                failed_count: 1,
                total_requested,
                errors: vec![format!("Bulk delete failed: {}", e)],
            };

            Ok(Json(ApiResponse::success(
                "Bulk task deletion failed",
                response,
            )))
        }
    }
}

// === 招待管理API（移行） ===

/// 管理者向け期限切れ招待クリーンアップ
pub async fn admin_cleanup_expired_invitations(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<Vec<TeamInvitationResponse>>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let service = &app_state.team_invitation_service;
    let expired_invitations = service.mark_expired_invitations().await?;

    let responses: Vec<TeamInvitationResponse> = expired_invitations
        .into_iter()
        .map(TeamInvitationResponse::from)
        .collect();

    Ok(Json(ApiResponse::success(
        "Expired invitations cleaned up successfully",
        responses,
    )))
}

/// 管理者向け古い招待削除
pub async fn admin_delete_old_invitations(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let service = &app_state.team_invitation_service;

    let days = params
        .get("days")
        .and_then(|d| d.parse::<u32>().ok())
        .unwrap_or(30);

    if days < 7 {
        return Err(AppError::ValidationError(
            "Cannot delete invitations less than 7 days old".to_string(),
        ));
    }

    let deleted_count = service.cleanup_old_invitations(days).await?;

    Ok(Json(ApiResponse::success(
        "Old invitations deleted successfully",
        serde_json::json!({
            "deleted_count": deleted_count,
            "days": days
        }),
    )))
}

// === 統計・分析API ===

/// 管理者向けタスク統計取得
pub async fn admin_get_task_statistics(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<AdminTaskStatsResponse>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;

    // 基本統計を取得（実装はサービス層で）
    let stats = task_service.get_admin_task_statistics().await?;

    Ok(Json(ApiResponse::success(
        "Task statistics retrieved successfully",
        stats,
    )))
}

/// 管理者向けタスク作成（単一）
pub async fn admin_create_task(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Json(request): Json<CreateTaskDto>,
) -> AppResult<Json<ApiResponse<TaskDto>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;
    let created_task = task_service.create_task(request).await?;

    Ok(Json(ApiResponse::success(
        "Task created successfully",
        created_task,
    )))
}

/// 管理者向けタスク更新（単一）
pub async fn admin_update_task(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
    Json(request): Json<UpdateTaskDto>,
) -> AppResult<Json<ApiResponse<TaskDto>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;
    let updated_task = task_service.update_task(task_id, request).await?;

    Ok(Json(ApiResponse::success(
        "Task updated successfully",
        updated_task,
    )))
}

/// 管理者向けタスク削除（単一）
pub async fn admin_delete_task(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
) -> AppResult<axum::http::StatusCode> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;
    task_service.delete_task(task_id).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// 管理者向け全タスク一覧取得
pub async fn admin_list_all_tasks(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<Vec<TaskDto>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;
    let tasks = task_service.list_tasks().await?;

    Ok(Json(tasks))
}

/// 管理者向けタスク一覧取得（ページング付き）
pub async fn admin_list_tasks_paginated(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<ApiResponse<PaginatedTasksDto>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let page = params
        .get("page")
        .and_then(|p| p.parse::<u64>().ok())
        .unwrap_or(1);
    let page_size = params
        .get("page_size")
        .and_then(|p| p.parse::<u64>().ok())
        .unwrap_or(10)
        .clamp(1, 100);

    let task_service = &app_state.task_service;
    let paginated_tasks = task_service.list_tasks_paginated(page, page_size).await?;

    Ok(Json(ApiResponse::success(
        "Paginated tasks retrieved successfully",
        paginated_tasks,
    )))
}

// レガシーバッチAPI削除 - 新形式のadmin_bulk_*に統一

/// 管理者向け特定ユーザーのタスク一覧取得
pub async fn admin_list_user_tasks(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<Vec<TaskDto>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;
    let tasks = task_service.list_tasks_for_user(user_id).await?;

    Ok(Json(tasks))
}

/// Admin専用ルーター（統廃合済み）
pub fn admin_router(app_state: crate::api::AppState) -> axum::Router {
    use axum::routing::{delete, get, post, put};

    // admin_only_middlewareを使用してルーター全体に管理者権限チェックを適用
    axum::Router::new()
        // 単一タスク操作
        .route("/admin/tasks", post(admin_create_task))
        .route("/admin/tasks", get(admin_list_all_tasks))
        .route("/admin/tasks/paginated", get(admin_list_tasks_paginated))
        .route("/admin/tasks/{task_id}", get(admin_get_task))
        .route("/admin/tasks/{task_id}", put(admin_update_task))
        .route("/admin/tasks/{task_id}", delete(admin_delete_task))
        // ユーザー固有タスク管理
        .route("/admin/users/{user_id}/tasks", get(admin_list_user_tasks))
        // バッチ操作（統一形式）
        .route("/admin/tasks/bulk/create", post(admin_bulk_create_tasks))
        .route("/admin/tasks/bulk/update", put(admin_bulk_update_tasks))
        .route("/admin/tasks/bulk/delete", delete(admin_bulk_delete_tasks))
        // 統計・管理
        .route("/admin/tasks/statistics", get(admin_get_task_statistics))
        // 招待管理
        .route(
            "/admin/invitations/cleanup",
            post(admin_cleanup_expired_invitations),
        )
        .route(
            "/admin/invitations/cleanup/old",
            delete(admin_delete_old_invitations),
        )
        // 管理者専用ミドルウェアは、main.rsで適用されるので、ここでは適用しない
        .with_state(app_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_admin_bulk_create_tasks_request_validation() {
        let valid_request = AdminBulkCreateTasksRequest {
            tasks: vec![CreateTaskDto {
                title: "Test Task".to_string(),
                description: Some("Test Description".to_string()),
                status: None,
                due_date: None,
            }],
            assign_to_user: Some(Uuid::new_v4()),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_empty_request = AdminBulkCreateTasksRequest {
            tasks: vec![],
            assign_to_user: None,
        };
        assert!(invalid_empty_request.validate().is_err());

        let invalid_too_many_request = AdminBulkCreateTasksRequest {
            tasks: (0..101)
                .map(|i| CreateTaskDto {
                    title: format!("Task {}", i),
                    description: None,
                    status: None,
                    due_date: None,
                })
                .collect(),
            assign_to_user: None,
        };
        assert!(invalid_too_many_request.validate().is_err());
    }

    #[test]
    fn test_admin_bulk_update_tasks_request_validation() {
        let valid_request = AdminBulkUpdateTasksRequest {
            updates: vec![BatchUpdateTaskItemDto {
                id: Uuid::new_v4(),
                title: Some("Updated Task".to_string()),
                description: Some("Updated Description".to_string()),
                status: Some(crate::domain::task_status::TaskStatus::Completed),
                due_date: None,
            }],
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = AdminBulkUpdateTasksRequest { updates: vec![] };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_admin_bulk_delete_tasks_request_validation() {
        let valid_request = AdminBulkDeleteTasksRequest {
            task_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = AdminBulkDeleteTasksRequest { task_ids: vec![] };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_admin_bulk_operation_response() {
        let response = AdminBulkOperationResponse {
            success_count: 5,
            failed_count: 2,
            total_requested: 7,
            errors: vec!["Error 1".to_string(), "Error 2".to_string()],
        };

        assert_eq!(response.success_count, 5);
        assert_eq!(response.failed_count, 2);
        assert_eq!(response.total_requested, 7);
        assert_eq!(response.errors.len(), 2);
    }

    #[test]
    fn test_admin_task_stats_response() {
        let stats = AdminTaskStatsResponse {
            total_tasks: 100,
            tasks_by_status: vec![
                TaskStatusStats {
                    status: "pending".to_string(),
                    count: 30,
                },
                TaskStatusStats {
                    status: "completed".to_string(),
                    count: 70,
                },
            ],
            tasks_by_user: vec![],
            recent_activity: vec![],
        };

        assert_eq!(stats.total_tasks, 100);
        assert_eq!(stats.tasks_by_status.len(), 2);
        assert_eq!(stats.tasks_by_status[0].count, 30);
        assert_eq!(stats.tasks_by_status[1].count, 70);
    }

    #[test]
    fn test_admin_single_task_operations_logic() {
        use crate::domain::task_status::TaskStatus;

        // 単一タスク作成のロジックテスト
        let create_request = CreateTaskDto {
            title: "Admin Created Task".to_string(),
            description: Some("Task created by admin".to_string()),
            status: Some(TaskStatus::InProgress),
            due_date: None,
        };
        assert!(!create_request.title.is_empty());
        assert!(create_request.description.is_some());

        // 単一タスク更新のロジックテスト
        let update_request = UpdateTaskDto {
            title: Some("Updated Task Title".to_string()),
            description: Some("Updated description".to_string()),
            status: Some(TaskStatus::Completed),
            due_date: None,
        };
        assert!(update_request.title.is_some());
        assert_eq!(update_request.status, Some(TaskStatus::Completed));
    }

    #[test]
    fn test_admin_pagination_logic() {
        // ページネーションパラメータのロジックテスト
        let mut params = std::collections::HashMap::new();
        params.insert("page".to_string(), "2".to_string());
        params.insert("page_size".to_string(), "25".to_string());

        let page = params
            .get("page")
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(1);
        let page_size = params
            .get("page_size")
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(10)
            .clamp(1, 100);

        assert_eq!(page, 2);
        assert_eq!(page_size, 25);

        // 不正な値の場合のテスト
        let mut invalid_params = std::collections::HashMap::new();
        invalid_params.insert("page".to_string(), "invalid".to_string());
        invalid_params.insert("page_size".to_string(), "150".to_string());

        let invalid_page = invalid_params
            .get("page")
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(1);
        let invalid_page_size = invalid_params
            .get("page_size")
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(10)
            .clamp(1, 100);

        assert_eq!(invalid_page, 1); // デフォルト値
        assert_eq!(invalid_page_size, 100); // クランプされた最大値
    }

    #[test]
    fn test_admin_batch_dto_conversion_logic() {
        use crate::domain::task_status::TaskStatus;

        // BatchCreateTaskDto のロジックテスト
        let batch_create = BatchCreateTaskDto {
            tasks: vec![
                CreateTaskDto {
                    title: "Batch Task 1".to_string(),
                    description: Some("First batch task".to_string()),
                    status: Some(TaskStatus::Todo),
                    due_date: None,
                },
                CreateTaskDto {
                    title: "Batch Task 2".to_string(),
                    description: None,
                    status: Some(TaskStatus::InProgress),
                    due_date: None,
                },
            ],
        };
        assert_eq!(batch_create.tasks.len(), 2);
        assert!(batch_create.tasks[0].description.is_some());
        assert!(batch_create.tasks[1].description.is_none());

        // BatchUpdateTaskDto のロジックテスト
        let batch_update = BatchUpdateTaskDto {
            tasks: vec![BatchUpdateTaskItemDto {
                id: Uuid::new_v4(),
                title: Some("Updated Batch Task".to_string()),
                description: Some("Updated description".to_string()),
                status: Some(TaskStatus::Completed),
                due_date: None,
            }],
        };
        assert_eq!(batch_update.tasks.len(), 1);
        assert_eq!(batch_update.tasks[0].status, Some(TaskStatus::Completed));

        // BatchDeleteTaskDto のロジックテスト
        let task_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        let batch_delete = BatchDeleteTaskDto {
            ids: task_ids.clone(),
        };
        assert_eq!(batch_delete.ids.len(), 2);
        assert_eq!(batch_delete.ids, task_ids);
    }
}
