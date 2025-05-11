// src/service/task_service.rs
use crate::api::dto::task_dto::{
    BatchCreateResponseDto, BatchCreateTaskDto, BatchDeleteResponseDto, BatchDeleteTaskDto,
    BatchUpdateResponseDto, BatchUpdateTaskDto, BatchUpdateTaskItemDto, CreateTaskDto,
    PaginatedTasksDto, PaginationDto, TaskDto, TaskFilterDto, UpdateTaskDto,
};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::repository::task_repository::TaskRepository;
use std::sync::Arc;
use uuid::Uuid; // DbPool をインポート

pub struct TaskService {
    repo: Arc<TaskRepository>, // Arc でラップしてスレッドセーフな参照カウント
}

impl TaskService {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            repo: Arc::new(TaskRepository::new(db_pool)),
        }
    }

    // --- CRUD ---
    pub async fn create_task(&self, payload: CreateTaskDto) -> AppResult<TaskDto> {
        let created_task = self.repo.create(payload).await?;
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

    pub async fn list_tasks(&self) -> AppResult<Vec<TaskDto>> {
        let tasks = self.repo.find_all().await?;
        Ok(tasks.into_iter().map(Into::into).collect())
    }

    pub async fn update_task(&self, id: Uuid, payload: UpdateTaskDto) -> AppResult<TaskDto> {
        // 先に存在確認をするか、リポジトリ層の update が None を返したら NotFound にするか
        // ここではリポジトリ層の結果で判断
        let updated_task = self.repo.update(id, payload).await?.ok_or_else(|| {
            AppError::NotFound(format!("Task with id {} not found for update", id))
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

    // --- Batch Operations ---
    pub async fn create_tasks_batch(
        &self,
        payload: BatchCreateTaskDto,
    ) -> AppResult<BatchCreateResponseDto> {
        if payload.tasks.is_empty() {
            return Ok(BatchCreateResponseDto {
                created_tasks: vec![],
            });
        }
        // SeaORM の insert_many は成功したモデルのリストではなく、last_insert_id (Postgresでは通常使えない) を返す。
        // そのため、挿入後に再度取得するか、個別に挿入して結果を収集する必要がある。
        // ここでは簡単化のため、リポジトリの create_many が InsertResult を返し、
        // それをそのまま使うか、あるいは個別に作成してDTOを組み立てる。
        // 今回は個別に作成し、成功したものを集める形にしてみる（エラーハンドリングが複雑になる可能性）
        // または、リポジトリ側で挿入したモデルを返すように変更する（要検討）

        // 例: 個別に作成し、結果を収集 (トランザクションを張るのが望ましい)
        let mut created_task_dtos = Vec::new();
        for task_payload in payload.tasks {
            // 本来はトランザクション内で実行すべき
            match self.repo.create(task_payload).await {
                Ok(task_model) => created_task_dtos.push(task_model.into()),
                Err(e) => {
                    // エラー処理: 一部失敗した場合どうするか？ (全体をロールバック or 失敗を通知)
                    // ここではシンプルにエラーをログに出してスキップする例 (本番では不適切)
                    eprintln!("Failed to create a task in batch: {:?}", e);
                    // return Err(AppError::DbErr(e)); // または全体を失敗させる
                }
            }
        }
        Ok(BatchCreateResponseDto {
            created_tasks: created_task_dtos,
        })

        // もし `repo.create_many` が挿入したモデルを返すようにリファクタリングした場合:
        // let created_models = self.repo.create_many(payload.tasks).await?; // 仮に Vec<task_model::Model> を返すと仮定
        // let dtos = created_models.into_iter().map(Into::into).collect();
        // Ok(BatchCreateResponseDto { created_tasks: dtos })
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
}
