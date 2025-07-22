// src/service/task_service.rs

use crate::api::dto::common::PaginationMeta;
use crate::api::dto::task_dto::{
    BatchCreateResponseDto, BatchCreateTaskDto, BatchDeleteResponseDto, BatchDeleteTaskDto,
    BatchUpdateResponseDto, BatchUpdateTaskDto, BatchUpdateTaskItemDto, CreateTaskDto,
    PaginatedTasksDto, TaskDto, TaskResponse, UpdateTaskDto,
};
use crate::api::dto::task_query_dto::TaskSearchQuery;
use crate::api::dto::team_task_dto::{
    AssignTaskRequest, CreateOrganizationTaskRequest, CreateTeamTaskRequest, TransferTaskRequest,
    TransferTaskResponse,
};
use crate::db::DbPool;
use crate::domain::audit_log_model::{AuditAction, AuditResult};
use crate::domain::permission::{Permission, PermissionResult, PermissionScope, Privilege};
use crate::domain::subscription_tier::SubscriptionTier;
use crate::domain::task_visibility::TaskVisibility;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthenticatedUser;
use crate::middleware::subscription_guard::check_feature_limit;
use crate::repository::task_repository::TaskRepository;
use crate::repository::team_repository::TeamRepository;
use crate::repository::user_repository::UserRepository;
use crate::service::audit_log_service::{
    AuditLogService, LogActionParams, TaskCreationParams, TaskTransferParams,
};
use crate::service::team_service::TeamService;
use crate::utils::error_helper::{forbidden_error, internal_server_error, not_found_error};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

pub struct TaskService {
    repo: Arc<TaskRepository>,
    user_repo: Arc<UserRepository>,
    team_repo: Arc<TeamRepository>,
    team_service: Arc<TeamService>,
    audit_log_service: Arc<AuditLogService>,
}

impl TaskService {
    pub fn new(
        db_pool: DbPool,
        team_service: Arc<TeamService>,
        audit_log_service: Arc<AuditLogService>,
    ) -> Self {
        Self {
            repo: Arc::new(TaskRepository::new(db_pool.clone())),
            user_repo: Arc::new(UserRepository::new(db_pool.clone())),
            team_repo: Arc::new(TeamRepository::new(db_pool.clone())),
            team_service,
            audit_log_service,
        }
    }

    // --- CRUD ---
    pub async fn create_task(&self, payload: CreateTaskDto) -> AppResult<TaskDto> {
        // 基本的な書き込み権限の例（実際の使用はハンドラーで行う）
        let _write_permission = Permission::write_own("tasks");
        let created_task = self.repo.create(payload).await?;
        Ok(created_task.into())
    }

    pub async fn create_task_for_user(
        &self,
        user_id: Uuid,
        payload: CreateTaskDto,
    ) -> AppResult<TaskDto> {
        // ユーザーのサブスクリプションティアを取得
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        let user_tier =
            SubscriptionTier::from_str(&user.subscription_tier).unwrap_or(SubscriptionTier::Free);

        // 現在のタスク数を取得
        let current_task_count = self.repo.count_user_tasks(user_id).await?;

        // タスク数制限チェック
        check_feature_limit(&user_tier, current_task_count, "tasks")?;

        let created_task = self.repo.create_for_user(user_id, payload).await?;

        // 監査ログを記録
        let log_params = LogActionParams {
            user_id,
            action: AuditAction::TaskCreated,
            resource_type: "task".to_string(),
            resource_id: Some(created_task.id),
            team_id: None,
            organization_id: None,
            details: Some(serde_json::json!({
                "title": created_task.title.clone(),
                "visibility": "personal"
            })),
            ip_address: None,
            user_agent: None,
            result: AuditResult::Success,
        };

        if let Err(e) = self.audit_log_service.log_action(log_params).await {
            error!("Failed to log task creation: {}", e);
        }

        Ok(created_task.into())
    }

    pub async fn get_task(&self, id: Uuid) -> AppResult<TaskDto> {
        // 基本的な読み取り権限の例
        let _read_permission = Permission::read_own("tasks");
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
        // 管理者用のグローバル権限の例
        let _admin_permission = Permission::admin_global("tasks");
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
    pub async fn filter_tasks(&self, query: TaskSearchQuery) -> AppResult<PaginatedTasksDto> {
        let (tasks, total_count) = self.repo.find_with_filter(&query).await?;

        // タスクモデルをDTOに変換
        let task_dtos: Vec<TaskDto> = tasks.into_iter().map(Into::into).collect();

        // ページネーション情報を取得
        let (page, per_page) = query.pagination.get_pagination();
        let pagination = PaginationMeta::new(page, per_page, total_count as i64);

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

    // ユーザー固有のフィルタリング付きタスク取得（廃止予定 - search_tasks_for_userを使用してください）
    pub async fn filter_tasks_for_user(
        &self,
        user_id: uuid::Uuid,
        query: TaskSearchQuery,
    ) -> AppResult<PaginatedTasksDto> {
        let (tasks, total_count) = self.repo.find_with_filter_for_user(user_id, &query).await?;

        // タスクモデルをDTOに変換
        let task_dtos: Vec<TaskDto> = tasks.into_iter().map(Into::into).collect();

        // ページネーション情報を取得
        let (page, per_page) = query.pagination.get_pagination();
        let pagination = PaginationMeta::new(page, per_page, total_count as i64);

        Ok(PaginatedTasksDto {
            items: task_dtos,
            pagination,
        })
    }

    // 統一クエリパターンを使用した検索
    pub async fn search_tasks_for_user(
        &self,
        user_id: uuid::Uuid,
        query: TaskSearchQuery,
    ) -> AppResult<PaginatedTasksDto> {
        // 直接TaskSearchQueryを使用
        self.filter_tasks_for_user(user_id, query).await
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
        query: Option<TaskSearchQuery>,
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
                self.execute_task_query(user, query, privilege, scope).await
            }
            PermissionResult::Denied { reason } => Err(AppError::Forbidden(reason)),
        }
    }

    /// 動的権限システムによるタスク検索（統一クエリパターン使用）
    pub async fn search_tasks_dynamic(
        &self,
        user: &AuthenticatedUser,
        query: Option<TaskSearchQuery>,
    ) -> AppResult<TaskResponse> {
        // 直接TaskSearchQueryを使用
        self.list_tasks_dynamic(user, query).await
    }

    /// 動的権限に基づいてクエリを実行
    async fn execute_task_query(
        &self,
        user: &AuthenticatedUser,
        query: Option<TaskSearchQuery>,
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
                    query,
                )
                .await
            }

            // Enterprise tier: Global scope, unlimited features
            (PermissionScope::Global, Some(privilege_info))
                if privilege_info.subscription_tier
                    == crate::domain::subscription_tier::SubscriptionTier::Enterprise =>
            {
                self.list_all_tasks_unlimited(query).await
            }

            // Admin access: Always unlimited
            _ if user.claims.is_admin() => self.list_all_tasks_unlimited(query).await,

            // Default: Limited access to own tasks only
            _ => {
                let basic_query = query.unwrap_or_default();
                let result = self
                    .filter_tasks_for_user(user.claims.user_id, basic_query)
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

        let mut query = TaskSearchQuery::default();
        query.pagination.per_page = max_items.min(100);

        let result = self.filter_tasks_for_user(user_id, query).await?;
        Ok(TaskResponse::Limited(result))
    }

    /// Pro tier: Team tasks with features
    async fn list_tasks_for_team_with_features(
        &self,
        user_id: Uuid,
        features: &[String],
        query: Option<TaskSearchQuery>,
    ) -> AppResult<TaskResponse> {
        let mut enhanced_query = query.unwrap_or_default();
        enhanced_query.pagination.per_page = enhanced_query.pagination.per_page.min(100); // Pro tier limit

        if features.contains(&"advanced_filter".to_string()) {
            // Enhanced filtering capabilities for Pro users
            let result = self.filter_tasks_for_user(user_id, enhanced_query).await?;
            Ok(TaskResponse::Enhanced(result))
        } else {
            let result = self.filter_tasks_for_user(user_id, enhanced_query).await?;
            Ok(TaskResponse::Limited(result))
        }
    }

    /// Enterprise tier: All tasks unlimited
    async fn list_all_tasks_unlimited(
        &self,
        query: Option<TaskSearchQuery>,
    ) -> AppResult<TaskResponse> {
        let enhanced_query = query.unwrap_or_default();
        let result = self.filter_tasks(enhanced_query).await?;
        Ok(TaskResponse::Unlimited(result))
    }

    // Admin専用メソッド群
    pub async fn get_admin_task_statistics(
        &self,
    ) -> AppResult<crate::api::handlers::admin_handler::AdminTaskStatsResponse> {
        use crate::api::handlers::admin_handler::{AdminTaskStatsResponse, TaskStatusStats};

        let total_tasks = self.repo.count_all_tasks().await.map_err(|e| {
            internal_server_error(
                e,
                "task_service::admin_get_task_stats",
                "Failed to count tasks",
            )
        })? as u32;

        let pending_count = self
            .repo
            .count_tasks_by_status("pending")
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::admin_get_task_stats",
                    "Failed to count pending tasks",
                )
            })? as u32;

        let in_progress_count = self
            .repo
            .count_tasks_by_status("in_progress")
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::admin_get_task_stats",
                    "Failed to count in_progress tasks",
                )
            })? as u32;

        let completed_count = self
            .repo
            .count_tasks_by_status("completed")
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::admin_get_task_stats",
                    "Failed to count completed tasks",
                )
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

    pub async fn count_tasks_for_user(&self, user_id: Uuid) -> AppResult<u64> {
        let count = self.repo.count_tasks_for_user(user_id).await.map_err(|e| {
            internal_server_error(
                e,
                "task_service::count_tasks_for_user",
                "Failed to count tasks for user",
            )
        })?;
        Ok(count)
    }

    /// 全タスク数を取得
    pub async fn count_all_tasks(&self) -> AppResult<u64> {
        self.repo.count_all_tasks().await.map_err(|e| {
            internal_server_error(
                e,
                "task_service::count_all_tasks",
                "Failed to count all tasks",
            )
        })
    }

    /// 完了済みタスク数を取得
    pub async fn count_completed_tasks(&self) -> AppResult<u64> {
        self.repo
            .count_tasks_by_status("completed")
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::count_completed_tasks",
                    "Failed to count completed tasks",
                )
            })
    }

    // Analytics methods for admin handlers
    pub async fn get_priority_distribution(&self) -> AppResult<Vec<(String, u64)>> {
        self.repo.get_priority_distribution().await.map_err(|e| {
            internal_server_error(
                e,
                "task_service::get_priority_distribution",
                "Failed to get priority distribution",
            )
        })
    }

    pub async fn get_average_completion_days_by_priority(&self) -> AppResult<Vec<(String, f64)>> {
        self.repo
            .get_average_completion_days_by_priority()
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::get_average_completion_days_by_priority",
                    "Failed to get average completion days by priority",
                )
            })
    }

    pub async fn get_weekly_trend_data(
        &self,
        weeks: u32,
    ) -> AppResult<Vec<(DateTime<Utc>, u64, u64)>> {
        self.repo.get_weekly_trend_data(weeks).await.map_err(|e| {
            internal_server_error(
                e,
                "task_service::get_weekly_trend_data",
                "Failed to get weekly trend data",
            )
        })
    }

    pub async fn get_user_average_completion_hours(&self, user_id: Uuid) -> AppResult<f64> {
        self.repo
            .get_user_average_completion_hours(user_id)
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::get_user_average_completion_hours",
                    "Failed to get user average completion hours",
                )
            })
    }

    // Unused admin methods removed - use admin_* methods instead

    // --- マルチテナント対応メソッド ---

    /// スコープベースのタスク取得
    pub async fn get_tasks_with_scope(
        &self,
        user: &AuthenticatedUser,
        query: TaskSearchQuery,
    ) -> AppResult<PaginatedTasksDto> {
        // ユーザーの所属チームと組織を取得
        let user_teams = self
            .team_service
            .get_user_team_ids(user.claims.user_id)
            .await?;

        // ユーザーの組織IDを取得
        let user_model = self
            .user_repo
            .find_by_id(user.claims.user_id)
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::get_tasks_with_scope",
                    "Failed to fetch user information",
                )
            })?
            .ok_or_else(|| {
                not_found_error(
                    "User not found",
                    "task_service::get_tasks_with_scope",
                    "User information could not be retrieved",
                )
            })?;

        let user_organization_id = user_model.organization_id;

        // クエリのスコープに基づいてフィルタリング
        let (tasks, total_count) = match query.visibility {
            Some(TaskVisibility::Personal) => {
                // 個人タスクのみ
                self.repo
                    .find_personal_tasks(user.claims.user_id, &query)
                    .await?
            }
            Some(TaskVisibility::Team) => {
                // チームタスク
                if let Some(team_id) = query.team_id {
                    // 特定のチームのタスクのみ
                    if !user_teams.contains(&team_id) {
                        return Err(forbidden_error(
                            "User is not a member of the specified team",
                            "task_service::get_tasks_with_scope",
                            "You don't have access to this team's tasks",
                        ));
                    }
                    self.repo.find_team_tasks(team_id, &query).await?
                } else {
                    // ユーザーが所属する全チームのタスク
                    self.repo.find_tasks_in_teams(&user_teams, &query).await?
                }
            }
            Some(TaskVisibility::Organization) => {
                // 組織タスク
                if let Some(org_id) = user_organization_id {
                    self.repo.find_organization_tasks(org_id, &query).await?
                } else {
                    return Err(forbidden_error(
                        "User is not a member of any organization",
                        "task_service::get_tasks_with_scope",
                        "You need to be part of an organization to view organization tasks",
                    ));
                }
            }
            None => {
                // スコープ指定なしの場合は、アクセス可能な全タスクを返す
                self.repo
                    .find_accessible_tasks(
                        user.claims.user_id,
                        &user_teams,
                        user_organization_id,
                        &query,
                    )
                    .await?
            }
        };

        let task_dtos: Vec<TaskDto> = tasks.into_iter().map(Into::into).collect();
        let (page, per_page) = query.pagination.get_pagination();
        let pagination = PaginationMeta::new(page, per_page, total_count as i64);

        Ok(PaginatedTasksDto {
            items: task_dtos,
            pagination,
        })
    }

    /// チームタスクの作成
    pub async fn create_team_task(
        &self,
        user: &AuthenticatedUser,
        payload: CreateTeamTaskRequest,
    ) -> AppResult<TaskDto> {
        // チームメンバーシップの確認
        let is_member = self
            .team_service
            .is_user_member_of_team(user.claims.user_id, payload.team_id)
            .await?;

        if !is_member {
            return Err(forbidden_error(
                "User is not a member of the team",
                "task_service::create_team_task",
                "You must be a member of the team to create team tasks",
            ));
        }

        // チーム情報の取得
        let team = self
            .team_repo
            .find_by_id(payload.team_id)
            .await?
            .ok_or_else(|| {
                not_found_error(
                    "Team not found",
                    "task_service::create_team_task",
                    "The specified team does not exist",
                )
            })?;

        // CreateTaskDtoに変換
        let create_dto = CreateTaskDto {
            title: payload.title,
            description: payload.description,
            status: payload.status,
            priority: payload.priority,
            due_date: payload
                .due_date
                .map(|ts| DateTime::from_timestamp(ts, 0).unwrap()),
        };

        // タスクの作成
        let task = self
            .repo
            .create_team_task(
                payload.team_id,
                team.organization_id, // Pass Option<Uuid> directly
                create_dto,
                payload.visibility.unwrap_or(TaskVisibility::Team),
                payload.assigned_to,
            )
            .await?;

        // 監査ログの記録
        if let Err(e) = self
            .audit_log_service
            .log_task_creation(TaskCreationParams {
                user_id: user.claims.user_id,
                task_id: task.id,
                task_title: task.title.clone(),
                team_id: Some(payload.team_id),
                organization_id: team.organization_id,
                visibility: "team".to_string(),
                ip_address: None, // IP address - TODO: Extract from request context
                user_agent: None, // User agent - TODO: Extract from request context
            })
            .await
        {
            tracing::error!(
                error = %e,
                user_id = %user.claims.user_id,
                task_id = %task.id,
                "Failed to record audit log for team task creation"
            );
        }

        Ok(task.into())
    }

    /// 組織タスクの作成
    pub async fn create_organization_task(
        &self,
        user: &AuthenticatedUser,
        payload: CreateOrganizationTaskRequest,
    ) -> AppResult<TaskDto> {
        // ユーザーの組織確認
        let user_model = self
            .user_repo
            .find_by_id(user.claims.user_id)
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::create_organization_task",
                    "Failed to fetch user information",
                )
            })?
            .ok_or_else(|| {
                not_found_error(
                    "User not found",
                    "task_service::create_organization_task",
                    "User information could not be retrieved",
                )
            })?;

        if user_model.organization_id != Some(payload.organization_id) {
            return Err(forbidden_error(
                "User is not a member of the organization",
                "task_service::create_organization_task",
                "You must be a member of the organization to create organization tasks",
            ));
        }

        // CreateTaskDtoに変換
        let create_dto = CreateTaskDto {
            title: payload.title,
            description: payload.description,
            status: payload.status,
            priority: payload.priority,
            due_date: payload
                .due_date
                .map(|ts| DateTime::from_timestamp(ts, 0).unwrap()),
        };

        // タスクの作成
        let task = self
            .repo
            .create_organization_task(payload.organization_id, create_dto, payload.assigned_to)
            .await?;

        // 監査ログの記録
        if let Err(e) = self
            .audit_log_service
            .log_task_creation(TaskCreationParams {
                user_id: user.claims.user_id,
                task_id: task.id,
                task_title: task.title.clone(),
                team_id: None,
                organization_id: Some(payload.organization_id),
                visibility: "organization".to_string(),
                ip_address: None, // IP address - TODO: Extract from request context
                user_agent: None, // User agent - TODO: Extract from request context
            })
            .await
        {
            tracing::error!(
                error = %e,
                user_id = %user.claims.user_id,
                task_id = %task.id,
                "Failed to record audit log for organization task creation"
            );
        }

        Ok(task.into())
    }

    /// タスクの割り当て
    pub async fn assign_task(
        &self,
        user: &AuthenticatedUser,
        task_id: Uuid,
        payload: AssignTaskRequest,
    ) -> AppResult<TaskDto> {
        // タスクの取得
        let task = self.repo.find_by_id(task_id).await?.ok_or_else(|| {
            not_found_error(
                "Task not found",
                "task_service::assign_task",
                "The specified task does not exist",
            )
        })?;

        // アクセス権限の確認
        let user_teams = self
            .team_service
            .get_user_team_ids(user.claims.user_id)
            .await?;

        // ユーザーの組織IDを取得
        let user_model = self
            .user_repo
            .find_by_id(user.claims.user_id)
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::assign_task",
                    "Failed to fetch user information",
                )
            })?
            .ok_or_else(|| {
                not_found_error(
                    "User not found",
                    "task_service::assign_task",
                    "User information could not be retrieved",
                )
            })?;

        let user_organization_id = user_model.organization_id;

        let can_assign = match task.visibility {
            TaskVisibility::Personal => {
                // 個人タスクは所有者のみ割り当て可能
                task.is_owned_by(&user.claims.user_id)
            }
            TaskVisibility::Team => {
                // チームタスクはチームメンバーが割り当て可能
                task.team_id.is_some_and(|tid| user_teams.contains(&tid))
            }
            TaskVisibility::Organization => {
                // 組織タスクは組織メンバーが割り当て可能
                task.organization_id == user_organization_id
            }
        };

        if !can_assign {
            return Err(forbidden_error(
                "User does not have permission to assign this task",
                "task_service::assign_task",
                "You don't have permission to assign this task",
            ));
        }

        // 割り当て先ユーザーの権限確認
        if let Some(assigned_to_id) = payload.assigned_to {
            // 割り当て先がタスクのスコープ内にいるか確認
            let assigned_user_teams = self.team_service.get_user_team_ids(assigned_to_id).await?;

            // 割り当て先ユーザーの組織IDを取得
            let assigned_user_model = self
                .user_repo
                .find_by_id(assigned_to_id)
                .await
                .map_err(|e| {
                    internal_server_error(
                        e,
                        "task_service::assign_task",
                        "Failed to fetch assigned user information",
                    )
                })?
                .ok_or_else(|| {
                    not_found_error(
                        "Assigned user not found",
                        "task_service::assign_task",
                        "The user you're trying to assign to does not exist",
                    )
                })?;

            let assigned_user_org_id = assigned_user_model.organization_id;

            let can_be_assigned = match task.visibility {
                TaskVisibility::Personal => false, // 個人タスクは他人に割り当てできない
                TaskVisibility::Team => task
                    .team_id
                    .is_some_and(|tid| assigned_user_teams.contains(&tid)),
                TaskVisibility::Organization => task.organization_id == assigned_user_org_id,
            };

            if !can_be_assigned {
                return Err(forbidden_error(
                    "Cannot assign task to user outside of scope",
                    "task_service::assign_task",
                    "The user you're trying to assign to doesn't have access to this task",
                ));
            }
        }

        // タスクの更新
        let updated_task = self
            .repo
            .update_task_assignment(task_id, payload.assigned_to)
            .await?
            .ok_or_else(|| {
                internal_server_error(
                    "Failed to update task assignment",
                    "task_service::assign_task",
                    "An error occurred while updating the task assignment",
                )
            })?;

        Ok(updated_task.into())
    }

    /// マルチテナントタスクの更新
    pub async fn update_multi_tenant_task(
        &self,
        user: &AuthenticatedUser,
        task_id: Uuid,
        payload: UpdateTaskDto,
    ) -> AppResult<TaskDto> {
        // タスクの取得
        let task = self.repo.find_by_id(task_id).await?.ok_or_else(|| {
            not_found_error(
                "Task not found",
                "task_service::update_multi_tenant_task",
                "The specified task does not exist",
            )
        })?;

        // アクセス権限の確認
        let user_teams = self
            .team_service
            .get_user_team_ids(user.claims.user_id)
            .await?;

        // ユーザーの組織IDを取得
        let user_model = self
            .user_repo
            .find_by_id(user.claims.user_id)
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::update_multi_tenant_task",
                    "Failed to fetch user information",
                )
            })?
            .ok_or_else(|| {
                not_found_error(
                    "User not found",
                    "task_service::update_multi_tenant_task",
                    "User information could not be retrieved",
                )
            })?;

        let user_organization_id = user_model.organization_id;

        let can_update = match task.visibility {
            TaskVisibility::Personal => {
                // 個人タスクは所有者のみ更新可能
                task.is_owned_by(&user.claims.user_id)
            }
            TaskVisibility::Team => {
                // チームタスクはチームメンバーが更新可能
                task.team_id.is_some_and(|tid| user_teams.contains(&tid))
            }
            TaskVisibility::Organization => {
                // 組織タスクは組織メンバーが更新可能
                task.organization_id == user_organization_id
            }
        };

        if !can_update {
            return Err(forbidden_error(
                "User does not have permission to update this task",
                "task_service::update_multi_tenant_task",
                "You don't have permission to update this task",
            ));
        }

        // タスクの更新
        let updated_task = self.repo.update(task_id, payload).await?.ok_or_else(|| {
            internal_server_error(
                "Failed to update task",
                "task_service::update_multi_tenant_task",
                "An error occurred while updating the task",
            )
        })?;

        Ok(updated_task.into())
    }

    /// マルチテナントタスクの削除
    pub async fn delete_multi_tenant_task(
        &self,
        user: &AuthenticatedUser,
        task_id: Uuid,
    ) -> AppResult<()> {
        // タスクの取得
        let task = self.repo.find_by_id(task_id).await?.ok_or_else(|| {
            not_found_error(
                "Task not found",
                "task_service::delete_multi_tenant_task",
                "The specified task does not exist",
            )
        })?;

        // アクセス権限の確認
        let user_teams = self
            .team_service
            .get_user_team_ids(user.claims.user_id)
            .await?;

        // ユーザーの組織IDを取得
        let user_model = self
            .user_repo
            .find_by_id(user.claims.user_id)
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::delete_multi_tenant_task",
                    "Failed to fetch user information",
                )
            })?
            .ok_or_else(|| {
                not_found_error(
                    "User not found",
                    "task_service::delete_multi_tenant_task",
                    "User information could not be retrieved",
                )
            })?;

        let user_organization_id = user_model.organization_id;

        let can_delete = match task.visibility {
            TaskVisibility::Personal => {
                // 個人タスクは所有者のみ削除可能
                task.is_owned_by(&user.claims.user_id)
            }
            TaskVisibility::Team => {
                // チームタスクはチームメンバーが削除可能
                task.team_id.is_some_and(|tid| user_teams.contains(&tid))
            }
            TaskVisibility::Organization => {
                // 組織タスクは組織メンバーが削除可能
                task.organization_id == user_organization_id
            }
        };

        if !can_delete {
            return Err(forbidden_error(
                "User does not have permission to delete this task",
                "task_service::delete_multi_tenant_task",
                "You don't have permission to delete this task",
            ));
        }

        // タスクの削除
        let delete_result = self.repo.delete(task_id).await?;
        if delete_result.rows_affected == 0 {
            Err(internal_server_error(
                "Failed to delete task",
                "task_service::delete_multi_tenant_task",
                "An error occurred while deleting the task",
            ))
        } else {
            Ok(())
        }
    }

    /// タスクの引き継ぎ
    pub async fn transfer_task(
        &self,
        user: &AuthenticatedUser,
        task_id: Uuid,
        request: TransferTaskRequest,
    ) -> AppResult<TransferTaskResponse> {
        // タスクを取得
        let task = self
            .repo
            .find_by_id(task_id)
            .await
            .map_err(|e| {
                internal_server_error(e, "task_service::transfer_task", "Failed to retrieve task")
            })?
            .ok_or_else(|| {
                not_found_error(
                    "Task not found",
                    "task_service::transfer_task",
                    "The specified task was not found",
                )
            })?;

        // ユーザーの所属チームを取得
        let user_teams = self
            .team_service
            .get_user_team_ids(user.claims.user_id)
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::transfer_task",
                    "Failed to retrieve user teams",
                )
            })?;

        // ユーザーの組織IDを取得
        let user_model = self
            .user_repo
            .find_by_id(user.claims.user_id)
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::transfer_task",
                    "Failed to retrieve user information",
                )
            })?
            .ok_or_else(|| {
                not_found_error(
                    "User not found",
                    "task_service::transfer_task",
                    "User information could not be retrieved",
                )
            })?;

        let user_organization_id = user_model.organization_id;

        // 新しい担当者の確認
        let new_assignee = self
            .user_repo
            .find_by_id(request.new_assignee)
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::transfer_task",
                    "Failed to retrieve new assignee information",
                )
            })?
            .ok_or_else(|| {
                not_found_error(
                    "New assignee not found",
                    "task_service::transfer_task",
                    "The specified new assignee was not found",
                )
            })?;

        // 権限チェック：タスクの可視性に基づいて引き継ぎ権限を確認
        let can_transfer = match task.visibility {
            TaskVisibility::Personal => {
                // 個人タスクは所有者のみ引き継ぎ可能
                task.is_owned_by(&user.claims.user_id)
            }
            TaskVisibility::Team => {
                // チームタスクはチームメンバーが引き継ぎ可能
                if let Some(team_id) = task.team_id {
                    // 新しい担当者もチームメンバーである必要がある
                    let new_assignee_teams = self
                        .team_service
                        .get_user_team_ids(request.new_assignee)
                        .await
                        .map_err(|e| {
                            internal_server_error(
                                e,
                                "task_service::transfer_task",
                                "Failed to retrieve new assignee teams",
                            )
                        })?;

                    user_teams.contains(&team_id) && new_assignee_teams.contains(&team_id)
                } else {
                    false
                }
            }
            TaskVisibility::Organization => {
                // 組織タスクは組織メンバーが引き継ぎ可能
                task.organization_id == user_organization_id
                    && new_assignee.organization_id == user_organization_id
            }
        };

        if !can_transfer {
            return Err(forbidden_error(
                "User does not have permission to transfer this task",
                "task_service::transfer_task",
                "You don't have permission to transfer this task",
            ));
        }

        // タスクの更新
        let previous_assignee = task.assigned_to;

        let _updated_task = self
            .repo
            .update_task_assignment(task_id, Some(request.new_assignee))
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::transfer_task",
                    "Failed to update task assignment",
                )
            })?
            .ok_or_else(|| {
                not_found_error(
                    "Task not found after update",
                    "task_service::transfer_task",
                    "Task could not be found after update",
                )
            })?;

        // 監査ログの記録
        if let Err(e) = self
            .audit_log_service
            .log_task_transfer(TaskTransferParams {
                user_id: user.claims.user_id,
                task_id,
                previous_assignee,
                new_assignee: request.new_assignee,
                team_id: task.team_id,
                organization_id: task.organization_id,
                reason: request.reason.clone(),
                ip_address: None, // IP address - TODO: Extract from request context
                user_agent: None, // User agent - TODO: Extract from request context
            })
            .await
        {
            // 監査ログの記録に失敗してもタスクの引き継ぎ自体は成功とする
            tracing::error!(
                error = %e,
                user_id = %user.claims.user_id,
                task_id = %task_id,
                "Failed to record audit log for task transfer"
            );
        }

        Ok(TransferTaskResponse {
            task_id,
            previous_assignee,
            new_assignee: request.new_assignee,
            transferred_at: Utc::now().timestamp(),
            transferred_by: user.claims.user_id,
            reason: request.reason,
        })
    }
}
