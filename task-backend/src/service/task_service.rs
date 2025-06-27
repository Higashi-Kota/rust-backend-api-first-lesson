// src/service/task_service.rs

use crate::api::dto::task_dto::{
    BatchCreateResponseDto, BatchCreateTaskDto, BatchDeleteResponseDto, BatchDeleteTaskDto,
    BatchUpdateResponseDto, BatchUpdateTaskDto, BatchUpdateTaskItemDto, CreateTaskDto,
    PaginatedTasksDto, TaskDto, TaskFilterDto, TaskResponse, UpdateTaskDto,
};
use crate::api::dto::PaginationMeta;
use crate::db::DbPool;
use crate::domain::permission::{PermissionResult, PermissionScope, Privilege};
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthenticatedUser;
use crate::repository::task_repository::TaskRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct TaskService {
    repo: Arc<TaskRepository>,
}

impl TaskService {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            repo: Arc::new(TaskRepository::new(db_pool)),
        }
    }

    // スキーマを指定するコンストラクタを追加
    pub fn with_schema(db_pool: DbPool, schema: String) -> Self {
        Self {
            repo: Arc::new(TaskRepository::with_schema(db_pool, schema)),
        }
    }

    // --- CRUD ---
    pub async fn create_task(&self, payload: CreateTaskDto) -> AppResult<TaskDto> {
        let created_task = self.repo.create(payload).await?;
        Ok(created_task.into())
    }

    pub async fn create_task_for_user(
        &self,
        user_id: Uuid,
        payload: CreateTaskDto,
    ) -> AppResult<TaskDto> {
        let created_task = self.repo.create_for_user(user_id, payload).await?;
        Ok(created_task.into())
    }

    pub async fn get_task(&self, id: Uuid) -> AppResult<TaskDto> {
        let task = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Task with id {} not found", id)))?;
        Ok(task.into())
    }

    pub async fn get_task_for_user(&self, user_id: Uuid, id: Uuid) -> AppResult<TaskDto> {
        let task = self
            .repo
            .find_by_id_for_user(user_id, id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Task with id {} not found or not accessible", id))
            })?;
        Ok(task.into())
    }

    pub async fn list_tasks(&self) -> AppResult<Vec<TaskDto>> {
        let tasks = self.repo.find_all().await?;
        Ok(tasks.into_iter().map(Into::into).collect())
    }

    pub async fn list_tasks_for_user(&self, user_id: Uuid) -> AppResult<Vec<TaskDto>> {
        let tasks = self.repo.find_all_for_user(user_id).await?;
        Ok(tasks.into_iter().map(Into::into).collect())
    }

    pub async fn update_task(&self, id: Uuid, payload: UpdateTaskDto) -> AppResult<TaskDto> {
        let updated_task = self.repo.update(id, payload).await?.ok_or_else(|| {
            AppError::NotFound(format!("Task with id {} not found for update", id))
        })?;
        Ok(updated_task.into())
    }

    pub async fn update_task_for_user(
        &self,
        user_id: Uuid,
        id: Uuid,
        payload: UpdateTaskDto,
    ) -> AppResult<TaskDto> {
        let updated_task = self
            .repo
            .update_for_user(user_id, id, payload)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Task with id {} not found for update or not accessible",
                    id
                ))
            })?;
        Ok(updated_task.into())
    }

    pub async fn delete_task(&self, id: Uuid) -> AppResult<()> {
        let delete_result = self.repo.delete(id).await?;
        if delete_result.rows_affected == 0 {
            Err(AppError::NotFound(format!(
                "Task with id {} not found for deletion",
                id
            )))
        } else {
            Ok(())
        }
    }

    pub async fn delete_task_for_user(&self, user_id: Uuid, id: Uuid) -> AppResult<()> {
        let delete_result = self.repo.delete_for_user(user_id, id).await?;
        if delete_result.rows_affected == 0 {
            Err(AppError::NotFound(format!(
                "Task with id {} not found for deletion or not accessible",
                id
            )))
        } else {
            Ok(())
        }
    }

    // --- Batch Operations ---
    // create_tasks_batch削除 - admin_bulk_create_tasks_bulkに統一

    pub async fn create_tasks_batch_for_user(
        &self,
        user_id: Uuid,
        payload: BatchCreateTaskDto,
    ) -> AppResult<BatchCreateResponseDto> {
        if payload.tasks.is_empty() {
            return Ok(BatchCreateResponseDto {
                created_tasks: vec![],
                created_count: 0,
            });
        }

        // リポジトリの create_many_for_user メソッドを使用
        let created_models = self
            .repo
            .create_many_for_user(user_id, payload.tasks)
            .await?;

        // モデルをDTOに変換
        let created_task_dtos: Vec<TaskDto> = created_models.into_iter().map(Into::into).collect();
        let count = created_task_dtos.len();

        Ok(BatchCreateResponseDto {
            created_tasks: created_task_dtos,
            created_count: count,
        })
    }

    // update_tasks_batch削除 - admin_bulk_update_tasks_bulkに統一

    pub async fn update_tasks_batch_for_user(
        &self,
        user_id: Uuid,
        payload: BatchUpdateTaskDto,
    ) -> AppResult<BatchUpdateResponseDto> {
        if payload.tasks.is_empty() {
            return Ok(BatchUpdateResponseDto { updated_count: 0 });
        }
        let items_to_update: Vec<BatchUpdateTaskItemDto> = payload.tasks.into_iter().collect();
        let updated_count = self
            .repo
            .update_many_for_user(user_id, items_to_update)
            .await?;
        Ok(BatchUpdateResponseDto { updated_count })
    }

    // delete_tasks_batch削除 - admin_bulk_delete_tasks_bulkに統一

    pub async fn delete_tasks_batch_for_user(
        &self,
        user_id: Uuid,
        payload: BatchDeleteTaskDto,
    ) -> AppResult<BatchDeleteResponseDto> {
        if payload.ids.is_empty() {
            return Ok(BatchDeleteResponseDto { deleted_count: 0 });
        }
        let delete_result = self.repo.delete_many_for_user(user_id, payload.ids).await?;
        Ok(BatchDeleteResponseDto {
            deleted_count: delete_result.rows_affected as usize,
        })
    }

    // --- Admin Operations ---
    pub async fn admin_create_tasks_bulk(
        &self,
        tasks: Vec<CreateTaskDto>,
    ) -> AppResult<Vec<TaskDto>> {
        let created_models = self.repo.create_many(tasks).await?;
        Ok(created_models.into_iter().map(Into::into).collect())
    }

    pub async fn admin_update_tasks_bulk(
        &self,
        updates: Vec<BatchUpdateTaskItemDto>,
    ) -> AppResult<usize> {
        self.repo.update_many(updates).await.map_err(Into::into)
    }

    pub async fn admin_delete_tasks_bulk(&self, task_ids: Vec<Uuid>) -> AppResult<u64> {
        let result = self.repo.delete_many(task_ids).await?;
        Ok(result.rows_affected)
    }

    // フィルタリング機能を追加
    pub async fn filter_tasks(&self, filter: TaskFilterDto) -> AppResult<PaginatedTasksDto> {
        let (tasks, total_count) = self.repo.find_with_filter(&filter).await?;

        // タスクモデルをDTOに変換
        let task_dtos: Vec<TaskDto> = tasks.into_iter().map(Into::into).collect();

        // ページネーション情報を計算
        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);
        let current_page = if limit > 0 { offset / limit + 1 } else { 1 };

        let pagination = PaginationMeta::new(current_page as i32, limit as i32, total_count as i64);

        Ok(PaginatedTasksDto {
            items: task_dtos,
            pagination,
        })
    }

    // ページネーション付きのタスク一覧取得
    pub async fn list_tasks_paginated(
        &self,
        page: u64,
        page_size: u64,
    ) -> AppResult<PaginatedTasksDto> {
        let page = if page == 0 { 1 } else { page };
        let page_size = if page_size == 0 { 10 } else { page_size };

        let (tasks, total_count) = self.repo.find_all_paginated(page, page_size).await?;

        // タスクモデルをDTOに変換
        let task_dtos: Vec<TaskDto> = tasks.into_iter().map(Into::into).collect();

        // ページネーション情報を計算

        let pagination = PaginationMeta::new(page as i32, page_size as i32, total_count as i64);

        Ok(PaginatedTasksDto {
            items: task_dtos,
            pagination,
        })
    }

    // ユーザー固有のフィルタリング付きタスク取得
    pub async fn filter_tasks_for_user(
        &self,
        user_id: uuid::Uuid,
        filter: TaskFilterDto,
    ) -> AppResult<PaginatedTasksDto> {
        let (tasks, total_count) = self
            .repo
            .find_with_filter_for_user(user_id, &filter)
            .await?;

        // タスクモデルをDTOに変換
        let task_dtos: Vec<TaskDto> = tasks.into_iter().map(Into::into).collect();

        // limit/offsetをpage/page_sizeに変換
        let page_size = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);
        let page = (offset / page_size) + 1;

        let pagination = PaginationMeta::new(page as i32, page_size as i32, total_count as i64);

        Ok(PaginatedTasksDto {
            items: task_dtos,
            pagination,
        })
    }

    // ユーザー固有のページネーション付きタスク一覧取得
    pub async fn list_tasks_paginated_for_user(
        &self,
        user_id: uuid::Uuid,
        page: u64,
        page_size: u64,
    ) -> AppResult<PaginatedTasksDto> {
        let page = if page == 0 { 1 } else { page };
        let page_size = if page_size == 0 { 10 } else { page_size };

        let (tasks, total_count) = self
            .repo
            .find_all_paginated_for_user(user_id, page, page_size)
            .await?;

        // タスクモデルをDTOに変換
        let task_dtos: Vec<TaskDto> = tasks.into_iter().map(Into::into).collect();

        // ページネーション情報を計算

        let pagination = PaginationMeta::new(page as i32, page_size as i32, total_count as i64);

        Ok(PaginatedTasksDto {
            items: task_dtos,
            pagination,
        })
    }

    // --- 動的権限システムメソッド (CLAUDE.md design implementation) ---

    /// 動的権限システムによるタスク一覧取得
    pub async fn list_tasks_dynamic(
        &self,
        user: &AuthenticatedUser,
        filter: Option<TaskFilterDto>,
    ) -> AppResult<TaskResponse> {
        let permission_result = if let Some(ref role) = user.claims.role {
            role.can_perform_action("tasks", "read", None)
        } else {
            // Fallback for basic permission check
            PermissionResult::Allowed {
                privilege: None,
                scope: PermissionScope::Own,
            }
        };

        match permission_result {
            PermissionResult::Allowed { privilege, scope } => {
                self.execute_task_query(user, filter, privilege, scope)
                    .await
            }
            PermissionResult::Denied { reason } => Err(AppError::Forbidden(reason)),
        }
    }

    /// 動的権限に基づいてクエリを実行
    async fn execute_task_query(
        &self,
        user: &AuthenticatedUser,
        filter: Option<TaskFilterDto>,
        privilege: Option<Privilege>,
        scope: PermissionScope,
    ) -> AppResult<TaskResponse> {
        match (scope, privilege.as_ref()) {
            // Free tier: Own scope only, basic features
            (PermissionScope::Own, Some(privilege_info))
                if privilege_info.subscription_tier
                    == crate::domain::subscription_tier::SubscriptionTier::Free =>
            {
                self.list_tasks_for_user_limited(user.claims.user_id, privilege_info.quota.as_ref())
                    .await
            }

            // Pro tier: Team scope, advanced features
            (PermissionScope::Team, Some(privilege_info))
                if privilege_info.subscription_tier
                    == crate::domain::subscription_tier::SubscriptionTier::Pro =>
            {
                self.list_tasks_for_team_with_features(
                    user.claims.user_id,
                    &privilege_info
                        .quota
                        .as_ref()
                        .map(|q| q.features.clone())
                        .unwrap_or_default(),
                    filter,
                )
                .await
            }

            // Enterprise tier: Global scope, unlimited features
            (PermissionScope::Global, Some(privilege_info))
                if privilege_info.subscription_tier
                    == crate::domain::subscription_tier::SubscriptionTier::Enterprise =>
            {
                self.list_all_tasks_unlimited(filter).await
            }

            // Admin access: Always unlimited
            _ if user.claims.is_admin() => self.list_all_tasks_unlimited(filter).await,

            // Default: Limited access to own tasks only
            _ => {
                let basic_filter = filter.unwrap_or_default();
                let result = self
                    .filter_tasks_for_user(user.claims.user_id, basic_filter)
                    .await?;
                Ok(TaskResponse::Limited(result))
            }
        }
    }

    /// Free tier: Own tasks with limits
    async fn list_tasks_for_user_limited(
        &self,
        user_id: Uuid,
        quota: Option<&crate::domain::permission::PermissionQuota>,
    ) -> AppResult<TaskResponse> {
        let max_items = quota.and_then(|q| q.max_items).unwrap_or(100);

        let filter = TaskFilterDto {
            limit: Some(max_items as u64),
            ..Default::default()
        };

        let result = self.filter_tasks_for_user(user_id, filter).await?;
        Ok(TaskResponse::Limited(result))
    }

    /// Pro tier: Team tasks with features
    async fn list_tasks_for_team_with_features(
        &self,
        user_id: Uuid,
        features: &[String],
        filter: Option<TaskFilterDto>,
    ) -> AppResult<TaskResponse> {
        let mut enhanced_filter = filter.unwrap_or_default();
        enhanced_filter.limit = Some(10_000); // Pro tier limit

        if features.contains(&"advanced_filter".to_string()) {
            // Enhanced filtering capabilities for Pro users
            let result = self.filter_tasks_for_user(user_id, enhanced_filter).await?;
            Ok(TaskResponse::Enhanced(result))
        } else {
            let result = self.filter_tasks_for_user(user_id, enhanced_filter).await?;
            Ok(TaskResponse::Limited(result))
        }
    }

    /// Enterprise tier: All tasks unlimited
    async fn list_all_tasks_unlimited(
        &self,
        filter: Option<TaskFilterDto>,
    ) -> AppResult<TaskResponse> {
        let enhanced_filter = filter.unwrap_or_default();
        let result = self.filter_tasks(enhanced_filter).await?;
        Ok(TaskResponse::Unlimited(result))
    }

    // Admin専用メソッド群
    pub async fn get_admin_task_statistics(
        &self,
    ) -> AppResult<crate::api::handlers::admin_handler::AdminTaskStatsResponse> {
        use crate::api::handlers::admin_handler::{AdminTaskStatsResponse, TaskStatusStats};

        let total_tasks =
            self.repo.count_all_tasks().await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to count tasks: {}", e))
            })? as u32;

        let pending_count = self
            .repo
            .count_tasks_by_status("pending")
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to count pending tasks: {}", e))
            })? as u32;

        let in_progress_count = self
            .repo
            .count_tasks_by_status("in_progress")
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to count in_progress tasks: {}", e))
            })? as u32;

        let completed_count = self
            .repo
            .count_tasks_by_status("completed")
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to count completed tasks: {}", e))
            })? as u32;

        Ok(AdminTaskStatsResponse {
            total_tasks,
            tasks_by_status: vec![
                TaskStatusStats {
                    status: "pending".to_string(),
                    count: pending_count,
                },
                TaskStatusStats {
                    status: "in_progress".to_string(),
                    count: in_progress_count,
                },
                TaskStatusStats {
                    status: "completed".to_string(),
                    count: completed_count,
                },
            ],
            tasks_by_user: vec![],   // Can be implemented later if needed
            recent_activity: vec![], // Can be implemented later if needed
        })
    }

    #[allow(dead_code)]
    pub async fn count_tasks_for_user(&self, user_id: Uuid) -> AppResult<u64> {
        let count = self.repo.count_tasks_for_user(user_id).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to count tasks for user: {}", e))
        })?;
        Ok(count)
    }

    // Unused admin methods removed - use admin_* methods instead
}
