// src/features/task/repository/task_repository.rs
use crate::core::task_status::TaskStatus;
use crate::db;
use crate::features::task::domain::task_model::{
    self, ActiveModel as TaskActiveModel, Entity as TaskEntity,
};
use crate::features::task::dto::{
    BatchUpdateTaskItemDto, CreateTaskDto, TaskFilterDto, UpdateTaskDto,
};
use chrono::Utc;
use sea_orm::{entity::*, query::*, DbConn, DbErr, DeleteResult, Set};
use sea_orm::{Condition, Order, PaginatorTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

pub struct TaskRepository {
    db: DbConn,
    schema: Option<String>,
}

#[allow(dead_code)] // TODO: Will be used when advanced task management features are integrated
impl TaskRepository {
    pub fn new(db: DbConn) -> Self {
        Self { db, schema: None }
    }

    // スキーマを設定する前に、各操作の前に呼び出すヘルパーメソッド
    async fn prepare_connection(&self) -> Result<(), DbErr> {
        if let Some(schema) = &self.schema {
            db::set_schema(&self.db, schema).await?;
        }
        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<task_model::Model>, DbErr> {
        self.prepare_connection().await?;
        TaskEntity::find_by_id(id).one(&self.db).await
    }

    pub async fn find_by_id_for_user(
        &self,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<Option<task_model::Model>, DbErr> {
        self.prepare_connection().await?;
        TaskEntity::find_by_id(id)
            .filter(task_model::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
    }

    pub async fn count_user_tasks(&self, user_id: Uuid) -> Result<usize, DbErr> {
        self.prepare_connection().await?;
        TaskEntity::find()
            .filter(task_model::Column::UserId.eq(user_id))
            .count(&self.db)
            .await
            .map(|count| count as usize)
    }

    pub async fn find_all(&self) -> Result<Vec<task_model::Model>, DbErr> {
        self.prepare_connection().await?;
        TaskEntity::find()
            .order_by_desc(task_model::Column::CreatedAt)
            .all(&self.db)
            .await
    }

    // find_by_user_id removed - use find_all_for_user instead

    pub async fn find_all_for_user(&self, user_id: Uuid) -> Result<Vec<task_model::Model>, DbErr> {
        self.prepare_connection().await?;
        TaskEntity::find()
            .filter(task_model::Column::UserId.eq(user_id))
            .order_by_desc(task_model::Column::CreatedAt)
            .all(&self.db)
            .await
    }

    pub async fn find_with_filter(
        &self,
        filter: &TaskFilterDto,
    ) -> Result<(Vec<task_model::Model>, u64), DbErr> {
        self.prepare_connection().await?;

        let mut query = TaskEntity::find();
        let mut conditions = Condition::all();

        // ステータスフィルタ
        if let Some(status) = &filter.status {
            conditions = conditions.add(task_model::Column::Status.eq(status.as_str()));
        }

        // タイトル検索
        if let Some(title_contains) = &filter.title_contains {
            conditions = conditions.add(task_model::Column::Title.contains(title_contains));
        }

        // 説明検索
        if let Some(desc_contains) = &filter.description_contains {
            conditions = conditions.add(task_model::Column::Description.contains(desc_contains));
        }

        // 期日フィルタ
        if let Some(due_before) = filter.due_date_before {
            conditions = conditions.add(task_model::Column::DueDate.lt(due_before));
        }

        if let Some(due_after) = filter.due_date_after {
            conditions = conditions.add(task_model::Column::DueDate.gt(due_after));
        }

        // 作成日フィルタ
        if let Some(created_after) = filter.created_after {
            conditions = conditions.add(task_model::Column::CreatedAt.gt(created_after));
        }

        if let Some(created_before) = filter.created_before {
            conditions = conditions.add(task_model::Column::CreatedAt.lt(created_before));
        }

        // 条件を適用
        query = query.filter(conditions);

        // ソート
        let sort_order = if filter.sort_order.as_deref() == Some("desc") {
            Order::Desc
        } else {
            Order::Asc
        };

        match filter.sort_by.as_deref() {
            Some("title") => query = query.order_by(task_model::Column::Title, sort_order),
            Some("due_date") => query = query.order_by(task_model::Column::DueDate, sort_order),
            Some("created_at") => query = query.order_by(task_model::Column::CreatedAt, sort_order),
            Some("status") => query = query.order_by(task_model::Column::Status, sort_order),
            _ => query = query.order_by(task_model::Column::CreatedAt, Order::Desc), // デフォルトは作成日の降順
        }

        // 総件数を取得
        let total_items = query.clone().count(&self.db).await?;

        // ページネーション
        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        // 最大ページサイズを制限
        let limit = std::cmp::min(limit, 100);

        // 結果を取得
        let tasks = query.limit(limit).offset(offset).all(&self.db).await?;

        Ok((tasks, total_items))
    }

    // 既存のfind_allメソッドを強化してページネーションを適用
    pub async fn find_all_paginated(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<task_model::Model>, u64), DbErr> {
        self.prepare_connection().await?;

        // ページサイズを制限（過大なページサイズを防止）
        let page_size = std::cmp::min(page_size, 100);
        let offset = (page - 1) * page_size;

        // ページネーションされたタスクを取得
        let tasks = TaskEntity::find()
            .order_by(task_model::Column::CreatedAt, Order::Desc)
            .limit(page_size)
            .offset(offset)
            .all(&self.db)
            .await?;

        // 総件数を取得
        let total_count = TaskEntity::find().count(&self.db).await?;

        Ok((tasks, total_count))
    }

    // ユーザー固有のフィルタリング付きタスク取得
    pub async fn find_with_filter_for_user(
        &self,
        user_id: Uuid,
        filter: &TaskFilterDto,
    ) -> Result<(Vec<task_model::Model>, u64), DbErr> {
        self.prepare_connection().await?;

        let mut query = TaskEntity::find();
        let mut conditions = Condition::all().add(task_model::Column::UserId.eq(user_id)); // ユーザーフィルタを追加

        // ステータスフィルタ
        if let Some(status) = &filter.status {
            conditions = conditions.add(task_model::Column::Status.eq(status.as_str()));
        }

        // タイトル検索
        if let Some(title_contains) = &filter.title_contains {
            conditions = conditions.add(task_model::Column::Title.contains(title_contains));
        }

        // 説明検索
        if let Some(desc_contains) = &filter.description_contains {
            conditions = conditions.add(task_model::Column::Description.contains(desc_contains));
        }

        // 期日フィルタ
        if let Some(due_before) = filter.due_date_before {
            conditions = conditions.add(task_model::Column::DueDate.lt(due_before));
        }

        if let Some(due_after) = filter.due_date_after {
            conditions = conditions.add(task_model::Column::DueDate.gt(due_after));
        }

        // 作成日フィルタ
        if let Some(created_after) = filter.created_after {
            conditions = conditions.add(task_model::Column::CreatedAt.gt(created_after));
        }

        if let Some(created_before) = filter.created_before {
            conditions = conditions.add(task_model::Column::CreatedAt.lt(created_before));
        }

        // 条件を適用
        query = query.filter(conditions);

        // ソート
        let sort_order = if filter.sort_order.as_deref() == Some("desc") {
            Order::Desc
        } else {
            Order::Asc
        };

        match filter.sort_by.as_deref() {
            Some("title") => query = query.order_by(task_model::Column::Title, sort_order),
            Some("due_date") => query = query.order_by(task_model::Column::DueDate, sort_order),
            Some("created_at") => query = query.order_by(task_model::Column::CreatedAt, sort_order),
            Some("status") => query = query.order_by(task_model::Column::Status, sort_order),
            _ => query = query.order_by(task_model::Column::CreatedAt, Order::Desc), // デフォルトは作成日の降順
        }

        // 総件数を取得
        let total_items = query.clone().count(&self.db).await?;

        // ページネーション
        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        // 最大ページサイズを制限
        let limit = std::cmp::min(limit, 100);

        // 結果を取得
        let tasks = query.limit(limit).offset(offset).all(&self.db).await?;

        Ok((tasks, total_items))
    }

    // ユーザー固有のページネーション付きタスク一覧取得
    pub async fn find_all_paginated_for_user(
        &self,
        user_id: Uuid,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<task_model::Model>, u64), DbErr> {
        self.prepare_connection().await?;

        // ページサイズを制限（過大なページサイズを防止）
        let page_size = std::cmp::min(page_size, 100);
        let offset = (page - 1) * page_size;

        // ページネーションされたタスクを取得（ユーザーIDでフィルタリング）
        let tasks = TaskEntity::find()
            .filter(task_model::Column::UserId.eq(user_id))
            .order_by(task_model::Column::CreatedAt, Order::Desc)
            .limit(page_size)
            .offset(offset)
            .all(&self.db)
            .await?;

        // 総件数を取得（ユーザーIDでフィルタリング）
        let total_count = TaskEntity::find()
            .filter(task_model::Column::UserId.eq(user_id))
            .count(&self.db)
            .await?;

        Ok((tasks, total_count))
    }

    pub async fn create(&self, payload: CreateTaskDto) -> Result<task_model::Model, DbErr> {
        self.prepare_connection().await?;

        let new_task = TaskActiveModel {
            title: Set(payload.title),
            description: Set(payload.description),
            status: Set(payload.status.unwrap_or(TaskStatus::Todo).to_string()),
            due_date: Set(payload.due_date),
            ..Default::default()
        };
        new_task.insert(&self.db).await
    }

    pub async fn create_for_user(
        &self,
        user_id: Uuid,
        payload: CreateTaskDto,
    ) -> Result<task_model::Model, DbErr> {
        self.prepare_connection().await?;

        let new_task = TaskActiveModel {
            user_id: Set(Some(user_id)),
            title: Set(payload.title),
            description: Set(payload.description),
            status: Set(payload.status.unwrap_or(TaskStatus::Todo).to_string()),
            priority: Set(payload.priority.unwrap_or("medium".to_string())),
            due_date: Set(payload.due_date),
            ..Default::default()
        };
        new_task.insert(&self.db).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        payload: UpdateTaskDto,
    ) -> Result<Option<task_model::Model>, DbErr> {
        self.prepare_connection().await?;

        let task = match TaskEntity::find_by_id(id).one(&self.db).await? {
            Some(t) => t,
            None => return Ok(None), // タスクが見つからなければ None を返す
        };

        let mut active_model: TaskActiveModel = task.clone().into(); // 元のデータを元に ActiveModel を作成
        let mut changed = false;

        if let Some(title_val) = payload.title {
            active_model.title = Set(title_val);
            changed = true;
        }

        if payload.description.is_some() {
            // is_some() は借用なのでOK
            active_model.description = Set(payload.description); // ここでムーブが発生
            changed = true;
        }

        if let Some(status_val) = payload.status {
            active_model.status = Set(status_val.to_string());
            changed = true;

            // タスクが完了状態に変更された場合、完了日時と完了時間を設定
            if status_val == TaskStatus::Completed && task.status != "completed" {
                let now = Utc::now();
                active_model.completed_at = Set(Some(now));

                // 完了までの時間を計算（時間単位）
                let duration_hours = (now - task.created_at).num_seconds() as f64 / 3600.0;
                active_model.completion_duration_hours = Set(Some(duration_hours));
            }
        }

        if let Some(priority_val) = payload.priority {
            active_model.priority = Set(priority_val);
            changed = true;
        }

        if payload.due_date.is_some() {
            // is_some() は借用なのでOK
            active_model.due_date = Set(payload.due_date); // ここでムーブが発生
            changed = true;
        }

        if changed {
            Ok(Some(active_model.update(&self.db).await?))
        } else {
            Ok(Some(task)) // 何も変更がなければ元のタスクを返す (updated_at は更新されない)
        }
    }

    pub async fn update_for_user(
        &self,
        user_id: Uuid,
        id: Uuid,
        payload: UpdateTaskDto,
    ) -> Result<Option<task_model::Model>, DbErr> {
        self.prepare_connection().await?;

        let task = match TaskEntity::find_by_id(id)
            .filter(task_model::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?
        {
            Some(t) => t,
            None => return Ok(None), // タスクが見つからないか、ユーザーのものでなければ None を返す
        };

        let mut active_model: TaskActiveModel = task.clone().into();
        let mut changed = false;

        if let Some(title_val) = payload.title {
            active_model.title = Set(title_val);
            changed = true;
        }

        if payload.description.is_some() {
            active_model.description = Set(payload.description);
            changed = true;
        }

        if let Some(status_val) = payload.status {
            active_model.status = Set(status_val.to_string());
            changed = true;

            // タスクが完了状態に変更された場合、完了日時と完了時間を設定
            if status_val == TaskStatus::Completed && task.status != "completed" {
                let now = Utc::now();
                active_model.completed_at = Set(Some(now));

                // 完了までの時間を計算（時間単位）
                let duration_hours = (now - task.created_at).num_seconds() as f64 / 3600.0;
                active_model.completion_duration_hours = Set(Some(duration_hours));
            }
        }

        if payload.due_date.is_some() {
            active_model.due_date = Set(payload.due_date);
            changed = true;
        }

        if changed {
            Ok(Some(active_model.update(&self.db).await?))
        } else {
            Ok(Some(task))
        }
    }

    pub async fn delete(&self, id: Uuid) -> Result<DeleteResult, DbErr> {
        self.prepare_connection().await?;
        TaskEntity::delete_by_id(id).exec(&self.db).await
    }

    pub async fn delete_for_user(&self, user_id: Uuid, id: Uuid) -> Result<DeleteResult, DbErr> {
        self.prepare_connection().await?;
        TaskEntity::delete_many()
            .filter(task_model::Column::Id.eq(id))
            .filter(task_model::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
    }

    pub async fn create_many(
        &self,
        payloads: Vec<CreateTaskDto>,
    ) -> Result<Vec<task_model::Model>, DbErr> {
        self.prepare_connection().await?;

        if payloads.is_empty() {
            return Ok(Vec::new());
        }

        // トランザクションを開始
        let txn = self.db.begin().await?;

        let mut created_models = Vec::with_capacity(payloads.len());

        for payload in payloads {
            let new_task = TaskActiveModel {
                title: Set(payload.title),
                description: Set(payload.description),
                status: Set(payload.status.unwrap_or(TaskStatus::Todo).to_string()),
                due_date: Set(payload.due_date),
                ..Default::default()
            };

            // モデルを挿入して結果を取得
            let model = new_task.insert(&txn).await?;
            created_models.push(model);
        }

        // トランザクションをコミット
        txn.commit().await?;

        Ok(created_models)
    }

    pub async fn create_many_for_user(
        &self,
        user_id: Uuid,
        payloads: Vec<CreateTaskDto>,
    ) -> Result<Vec<task_model::Model>, DbErr> {
        self.prepare_connection().await?;

        if payloads.is_empty() {
            return Ok(Vec::new());
        }

        // トランザクションを開始
        let txn = self.db.begin().await?;

        let mut created_models = Vec::with_capacity(payloads.len());

        for payload in payloads {
            let new_task = TaskActiveModel {
                user_id: Set(Some(user_id)),
                title: Set(payload.title),
                description: Set(payload.description),
                status: Set(payload.status.unwrap_or(TaskStatus::Todo).to_string()),
                due_date: Set(payload.due_date),
                ..Default::default()
            };

            // モデルを挿入して結果を取得
            let model = new_task.insert(&txn).await?;
            created_models.push(model);
        }

        // トランザクションをコミット
        txn.commit().await?;

        Ok(created_models)
    }

    pub async fn update_many(&self, items: Vec<BatchUpdateTaskItemDto>) -> Result<usize, DbErr> {
        self.prepare_connection().await?;

        let txn = self.db.begin().await?;
        let mut updated_count = 0;

        for item_payload in items {
            // `item` から `item_payload` に変更して明確化
            let task = match TaskEntity::find_by_id(item_payload.id).one(&txn).await? {
                Some(t) => t,
                None => continue, // 見つからないタスクはスキップ (またはエラー処理)
            };

            let mut active_model: TaskActiveModel = task.clone().into();
            let mut changed_in_item = false;

            if let Some(title_val) = item_payload.title {
                active_model.title = Set(title_val);
                changed_in_item = true;
            }
            if item_payload.description.is_some() {
                active_model.description = Set(item_payload.description);
                changed_in_item = true;
            }
            if let Some(status_val) = item_payload.status {
                active_model.status = Set(status_val.to_string());
                changed_in_item = true;
            }
            if item_payload.due_date.is_some() {
                active_model.due_date = Set(item_payload.due_date);
                changed_in_item = true;
            }

            if changed_in_item {
                active_model.update(&txn).await?;
                updated_count += 1;
            }
        }
        txn.commit().await?;
        Ok(updated_count)
    }

    pub async fn update_many_for_user(
        &self,
        user_id: Uuid,
        items: Vec<BatchUpdateTaskItemDto>,
    ) -> Result<usize, DbErr> {
        self.prepare_connection().await?;

        let txn = self.db.begin().await?;
        let mut updated_count = 0;

        for item_payload in items {
            // ユーザーIDでフィルタリングして、ユーザーのタスクのみ更新
            let task = match TaskEntity::find_by_id(item_payload.id)
                .filter(task_model::Column::UserId.eq(user_id))
                .one(&txn)
                .await?
            {
                Some(t) => t,
                None => continue, // 見つからないタスクまたは他のユーザーのタスクはスキップ
            };

            let mut active_model: TaskActiveModel = task.clone().into();
            let mut changed_in_item = false;

            if let Some(title_val) = item_payload.title {
                active_model.title = Set(title_val);
                changed_in_item = true;
            }
            if item_payload.description.is_some() {
                active_model.description = Set(item_payload.description);
                changed_in_item = true;
            }
            if let Some(status_val) = item_payload.status {
                active_model.status = Set(status_val.to_string());
                changed_in_item = true;
            }
            if item_payload.due_date.is_some() {
                active_model.due_date = Set(item_payload.due_date);
                changed_in_item = true;
            }

            if changed_in_item {
                active_model.update(&txn).await?;
                updated_count += 1;
            }
        }
        txn.commit().await?;
        Ok(updated_count)
    }

    pub async fn delete_many(&self, ids: Vec<Uuid>) -> Result<DeleteResult, DbErr> {
        self.prepare_connection().await?;

        if ids.is_empty() {
            return Ok(DeleteResult { rows_affected: 0 });
        }
        TaskEntity::delete_many()
            .filter(task_model::Column::Id.is_in(ids))
            .exec(&self.db)
            .await
    }

    pub async fn delete_many_for_user(
        &self,
        user_id: Uuid,
        ids: Vec<Uuid>,
    ) -> Result<DeleteResult, DbErr> {
        self.prepare_connection().await?;

        if ids.is_empty() {
            return Ok(DeleteResult { rows_affected: 0 });
        }
        TaskEntity::delete_many()
            .filter(task_model::Column::Id.is_in(ids))
            .filter(task_model::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
    }

    // Admin statistics methods
    pub async fn count_all_tasks(&self) -> Result<u64, DbErr> {
        self.prepare_connection().await?;
        TaskEntity::find().count(&self.db).await
    }

    pub async fn count_tasks_by_status(&self, status: &str) -> Result<u64, DbErr> {
        self.prepare_connection().await?;
        TaskEntity::find()
            .filter(task_model::Column::Status.eq(status))
            .count(&self.db)
            .await
    }

    pub async fn count_tasks_for_user(&self, user_id: Uuid) -> Result<u64, DbErr> {
        self.prepare_connection().await?;
        TaskEntity::find()
            .filter(task_model::Column::UserId.eq(user_id))
            .count(&self.db)
            .await
    }

    /// 優先度別のタスク数を取得
    pub async fn get_priority_distribution(&self) -> Result<Vec<(String, u64)>, DbErr> {
        self.prepare_connection().await?;

        use sea_orm::sea_query::Expr;
        use sea_orm::QuerySelect;

        let result = TaskEntity::find()
            .select_only()
            .column(task_model::Column::Priority)
            .column_as(Expr::col(task_model::Column::Id).count(), "count")
            .group_by(task_model::Column::Priority)
            .into_tuple::<(String, i64)>()
            .all(&self.db)
            .await?;

        Ok(result
            .into_iter()
            .map(|(priority, count)| (priority, count as u64))
            .collect())
    }

    /// 優先度別の平均完了日数を取得
    pub async fn get_average_completion_days_by_priority(
        &self,
    ) -> Result<Vec<(String, f64)>, DbErr> {
        self.prepare_connection().await?;

        let priorities = vec!["high", "medium", "low"];
        let mut result = Vec::new();

        for priority in priorities {
            let tasks = TaskEntity::find()
                .filter(task_model::Column::Status.eq("completed"))
                .filter(task_model::Column::Priority.eq(priority))
                .filter(task_model::Column::CompletedAt.is_not_null())
                .all(&self.db)
                .await?;

            if tasks.is_empty() {
                result.push((priority.to_string(), 0.0));
                continue;
            }

            let total_days: f64 = tasks
                .iter()
                .filter_map(|task| {
                    task.completed_at.map(|completed| {
                        (completed - task.created_at).num_seconds() as f64 / 86400.0
                    })
                })
                .sum();

            result.push((priority.to_string(), total_days / tasks.len() as f64));
        }

        Ok(result)
    }

    /// 週次トレンドデータを取得
    pub async fn get_weekly_trend_data(
        &self,
        weeks: u32,
    ) -> Result<Vec<(chrono::DateTime<Utc>, u64, u64)>, DbErr> {
        self.prepare_connection().await?;

        let mut result = Vec::new();

        for week in 0..weeks {
            let week_end = Utc::now() - chrono::Duration::weeks(week as i64);
            let week_start = week_end - chrono::Duration::weeks(1);

            // その週に作成されたタスク数
            let created_count = TaskEntity::find()
                .filter(task_model::Column::CreatedAt.gte(week_start))
                .filter(task_model::Column::CreatedAt.lt(week_end))
                .count(&self.db)
                .await?;

            // その週に完了したタスク数
            let completed_count = TaskEntity::find()
                .filter(task_model::Column::CompletedAt.gte(week_start))
                .filter(task_model::Column::CompletedAt.lt(week_end))
                .count(&self.db)
                .await?;

            result.push((week_start, created_count, completed_count));
        }

        result.reverse(); // 古い週から新しい週の順に並べ替え
        Ok(result)
    }

    /// ユーザー別の平均完了時間（時間）を取得
    pub async fn get_user_average_completion_hours(&self, user_id: Uuid) -> Result<f64, DbErr> {
        self.prepare_connection().await?;

        let completed_tasks = TaskEntity::find()
            .filter(task_model::Column::UserId.eq(user_id))
            .filter(task_model::Column::Status.eq("completed"))
            .filter(task_model::Column::CompletedAt.is_not_null())
            .all(&self.db)
            .await?;

        if completed_tasks.is_empty() {
            return Ok(0.0);
        }

        let total_hours: f64 = completed_tasks
            .iter()
            .filter_map(|task| {
                task.completed_at
                    .map(|completed| (completed - task.created_at).num_seconds() as f64 / 3600.0)
            })
            .sum();

        Ok(total_hours / completed_tasks.len() as f64)
    }

    /// List all tasks (admin only)
    pub async fn list_all_tasks(&self) -> Result<Vec<task_model::Model>, DbErr> {
        self.prepare_connection().await?;

        TaskEntity::find()
            .order_by_desc(task_model::Column::CreatedAt)
            .all(&self.db)
            .await
    }
}
