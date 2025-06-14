// src/service/task_service.rs
#![allow(dead_code)]

use crate::api::dto::task_dto::{
    BatchCreateResponseDto, BatchCreateTaskDto, BatchDeleteResponseDto, BatchDeleteTaskDto,
    BatchUpdateResponseDto, BatchUpdateTaskDto, BatchUpdateTaskItemDto, CreateTaskDto,
    PaginatedTasksDto, PaginationDto, TaskDto, TaskFilterDto, UpdateTaskDto,
};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
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
    pub async fn create_tasks_batch(
        &self,
        payload: BatchCreateTaskDto,
    ) -> AppResult<BatchCreateResponseDto> {
        if payload.tasks.is_empty() {
            return Ok(BatchCreateResponseDto {
                created_tasks: vec![],
                created_count: 0,
            });
        }

        // リポジトリの create_many メソッドを使用
        let created_models = self.repo.create_many(payload.tasks).await?;

        // モデルをDTOに変換
        let created_task_dtos: Vec<TaskDto> = created_models.into_iter().map(Into::into).collect();
        let count = created_task_dtos.len();

        Ok(BatchCreateResponseDto {
            created_tasks: created_task_dtos,
            created_count: count,
        })
    }

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

    pub async fn update_tasks_batch(
        &self,
        payload: BatchUpdateTaskDto,
    ) -> AppResult<BatchUpdateResponseDto> {
        if payload.tasks.is_empty() {
            return Ok(BatchUpdateResponseDto { updated_count: 0 });
        }
        let items_to_update: Vec<BatchUpdateTaskItemDto> = payload.tasks.into_iter().collect();
        let updated_count = self.repo.update_many(items_to_update).await?;
        Ok(BatchUpdateResponseDto { updated_count })
    }

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

    pub async fn delete_tasks_batch(
        &self,
        payload: BatchDeleteTaskDto,
    ) -> AppResult<BatchDeleteResponseDto> {
        if payload.ids.is_empty() {
            return Ok(BatchDeleteResponseDto { deleted_count: 0 });
        }
        let delete_result = self.repo.delete_many(payload.ids).await?;
        Ok(BatchDeleteResponseDto {
            deleted_count: delete_result.rows_affected as usize,
        })
    }

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

    // フィルタリング機能を追加
    pub async fn filter_tasks(&self, filter: TaskFilterDto) -> AppResult<PaginatedTasksDto> {
        let (tasks, total_count) = self.repo.find_with_filter(&filter).await?;

        // タスクモデルをDTOに変換
        let task_dtos: Vec<TaskDto> = tasks.into_iter().map(Into::into).collect();

        // ページネーション情報を計算
        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);
        let current_page = if limit > 0 { offset / limit + 1 } else { 1 };
        let total_pages = if limit > 0 {
            total_count.div_ceil(limit)
        } else {
            1
        };

        let pagination = PaginationDto {
            current_page,
            page_size: limit,
            total_items: total_count,
            total_pages,
            has_next_page: current_page < total_pages,
            has_previous_page: current_page > 1,
        };

        Ok(PaginatedTasksDto {
            tasks: task_dtos,
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
        let total_pages = total_count.div_ceil(page_size);

        let pagination = PaginationDto {
            current_page: page,
            page_size,
            total_items: total_count,
            total_pages,
            has_next_page: page < total_pages,
            has_previous_page: page > 1,
        };

        Ok(PaginatedTasksDto {
            tasks: task_dtos,
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
        let total_pages = total_count.div_ceil(page_size);

        let pagination = PaginationDto {
            current_page: page,
            page_size,
            total_items: total_count,
            total_pages,
            has_next_page: page < total_pages,
            has_previous_page: page > 1,
        };

        Ok(PaginatedTasksDto {
            tasks: task_dtos,
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
        let total_pages = total_count.div_ceil(page_size);

        let pagination = PaginationDto {
            current_page: page,
            page_size,
            total_items: total_count,
            total_pages,
            has_next_page: page < total_pages,
            has_previous_page: page > 1,
        };

        Ok(PaginatedTasksDto {
            tasks: task_dtos,
            pagination,
        })
    }

    // Admin専用メソッド群
    pub async fn list_all_tasks(&self) -> AppResult<Vec<TaskDto>> {
        let tasks = self.repo.find_all().await?;
        Ok(tasks.into_iter().map(Into::into).collect())
    }

    pub async fn list_tasks_by_user_id(&self, user_id: Uuid) -> AppResult<Vec<TaskDto>> {
        let tasks = self.repo.find_by_user_id(user_id).await?;
        Ok(tasks.into_iter().map(Into::into).collect())
    }

    pub async fn delete_task_by_id(&self, id: Uuid) -> AppResult<()> {
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
}
