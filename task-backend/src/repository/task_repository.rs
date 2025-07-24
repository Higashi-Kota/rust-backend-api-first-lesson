// src/repository/task_repository.rs
use crate::api::dto::task_dto::{BatchUpdateTaskItemDto, CreateTaskDto, UpdateTaskDto};
use crate::api::dto::task_query_dto::TaskSearchQuery;
use crate::db;
use crate::domain::task_model::{self, ActiveModel as TaskActiveModel, Entity as TaskEntity};
use crate::domain::task_status::TaskStatus;
use crate::domain::task_visibility::TaskVisibility;
use chrono::Utc;
use sea_orm::{entity::*, query::*, DbConn, DbErr, DeleteResult, Set};
use sea_orm::{Condition, Order, PaginatorTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

pub struct TaskRepository {
    db: DbConn,
    schema: Option<String>,
}

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
        query: &TaskSearchQuery,
    ) -> Result<(Vec<task_model::Model>, u64), DbErr> {
        self.prepare_connection().await?;

        let mut db_query = TaskEntity::find();
        let mut conditions = Condition::all().add(task_model::Column::UserId.eq(user_id)); // ユーザーフィルタを追加

        // ステータスフィルタ
        if let Some(status) = &query.status {
            conditions = conditions.add(task_model::Column::Status.eq(status.as_str()));
        }

        // 検索（タイトルと説明文のOR検索）
        if let Some(search_term) = &query.search {
            conditions = conditions.add(
                Condition::any()
                    .add(task_model::Column::Title.contains(search_term))
                    .add(task_model::Column::Description.contains(search_term)),
            );
        }

        // 優先度フィルタ
        if let Some(priority) = &query.priority {
            conditions = conditions.add(task_model::Column::Priority.eq(priority));
        }

        // 期日フィルタ
        if let Some(due_before) = query.due_date_before {
            conditions = conditions.add(task_model::Column::DueDate.lt(due_before));
        }

        if let Some(due_after) = query.due_date_after {
            conditions = conditions.add(task_model::Column::DueDate.gt(due_after));
        }

        // 作成日フィルタ
        if let Some(created_after) = query.created_after {
            conditions = conditions.add(task_model::Column::CreatedAt.gt(created_after));
        }

        if let Some(created_before) = query.created_before {
            conditions = conditions.add(task_model::Column::CreatedAt.lt(created_before));
        }

        // 条件を適用
        db_query = db_query.filter(conditions);

        // ソート
        let sort_order = match query.sort.sort_order {
            crate::types::SortOrder::Asc => Order::Asc,
            crate::types::SortOrder::Desc => Order::Desc,
        };

        match query.sort.sort_by.as_deref() {
            Some("title") => db_query = db_query.order_by(task_model::Column::Title, sort_order),
            Some("due_date") => {
                db_query = db_query.order_by(task_model::Column::DueDate, sort_order)
            }
            Some("created_at") => {
                db_query = db_query.order_by(task_model::Column::CreatedAt, sort_order)
            }
            Some("updated_at") => {
                db_query = db_query.order_by(task_model::Column::UpdatedAt, sort_order)
            }
            Some("priority") => {
                db_query = db_query.order_by(task_model::Column::Priority, sort_order)
            }
            Some("status") => db_query = db_query.order_by(task_model::Column::Status, sort_order),
            _ => db_query = db_query.order_by(task_model::Column::CreatedAt, Order::Desc), // デフォルトは作成日の降順
        }

        // 総件数を取得
        let total_items = db_query.clone().count(&self.db).await?;

        // ページネーション
        let (page, per_page) = query.pagination.get_pagination();
        let offset = ((page - 1) * per_page) as u64;
        let limit = per_page as u64;

        // 最大ページサイズを制限
        let limit = std::cmp::min(limit, 100);

        // 結果を取得
        let tasks = db_query.limit(limit).offset(offset).all(&self.db).await?;

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
                let duration_hours = (now - task.created_at).num_hours() as f64;
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

    pub async fn delete(&self, id: Uuid) -> Result<DeleteResult, DbErr> {
        self.prepare_connection().await?;
        TaskEntity::delete_by_id(id).exec(&self.db).await
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
                    task.completed_at
                        .map(|completed| (completed - task.created_at).num_days() as f64)
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
                    .map(|completed| (completed - task.created_at).num_hours() as f64)
            })
            .sum();

        Ok(total_hours / completed_tasks.len() as f64)
    }

    // --- マルチテナント対応メソッド ---

    /// 個人タスクの取得
    pub async fn find_personal_tasks(
        &self,
        user_id: Uuid,
        query: &TaskSearchQuery,
    ) -> Result<(Vec<task_model::Model>, u64), DbErr> {
        self.prepare_connection().await?;

        let mut db_query = TaskEntity::find();
        let mut conditions = Condition::all()
            .add(task_model::Column::Visibility.eq(TaskVisibility::Personal.as_str()))
            .add(
                Condition::any()
                    .add(task_model::Column::UserId.eq(user_id))
                    .add(task_model::Column::AssignedTo.eq(user_id)),
            );

        // 共通フィルタを適用
        conditions = self.apply_common_filters(conditions, query);

        db_query = db_query.filter(conditions);

        // ソートを適用
        db_query = self.apply_sorting(db_query, query);

        // ページネーション
        let (page, per_page) = query.pagination.get_pagination();
        let paginator = db_query.paginate(&self.db, per_page as u64);

        let total_count = paginator.num_items().await?;
        let items = paginator.fetch_page((page - 1) as u64).await?;

        Ok((items, total_count))
    }

    /// チームタスクの取得（特定チーム）
    pub async fn find_team_tasks(
        &self,
        team_id: Uuid,
        query: &TaskSearchQuery,
    ) -> Result<(Vec<task_model::Model>, u64), DbErr> {
        self.prepare_connection().await?;

        let mut db_query = TaskEntity::find();
        let mut conditions = Condition::all()
            .add(task_model::Column::TeamId.eq(team_id))
            .add(task_model::Column::Visibility.eq(TaskVisibility::Team.as_str()));

        conditions = self.apply_common_filters(conditions, query);

        db_query = db_query.filter(conditions);
        db_query = self.apply_sorting(db_query, query);

        let (page, per_page) = query.pagination.get_pagination();
        let paginator = db_query.paginate(&self.db, per_page as u64);

        let total_count = paginator.num_items().await?;
        let items = paginator.fetch_page((page - 1) as u64).await?;

        Ok((items, total_count))
    }

    /// 複数チームのタスクを取得
    pub async fn find_tasks_in_teams(
        &self,
        team_ids: &[Uuid],
        query: &TaskSearchQuery,
    ) -> Result<(Vec<task_model::Model>, u64), DbErr> {
        self.prepare_connection().await?;

        if team_ids.is_empty() {
            return Ok((vec![], 0));
        }

        let mut db_query = TaskEntity::find();
        let mut conditions = Condition::all()
            .add(task_model::Column::TeamId.is_in(team_ids.to_vec()))
            .add(task_model::Column::Visibility.eq(TaskVisibility::Team.as_str()));

        conditions = self.apply_common_filters(conditions, query);

        db_query = db_query.filter(conditions);
        db_query = self.apply_sorting(db_query, query);

        let (page, per_page) = query.pagination.get_pagination();
        let paginator = db_query.paginate(&self.db, per_page as u64);

        let total_count = paginator.num_items().await?;
        let items = paginator.fetch_page((page - 1) as u64).await?;

        Ok((items, total_count))
    }

    /// 組織タスクの取得
    pub async fn find_organization_tasks(
        &self,
        organization_id: Uuid,
        query: &TaskSearchQuery,
    ) -> Result<(Vec<task_model::Model>, u64), DbErr> {
        self.prepare_connection().await?;

        let mut db_query = TaskEntity::find();
        let mut conditions = Condition::all()
            .add(task_model::Column::OrganizationId.eq(organization_id))
            .add(task_model::Column::Visibility.eq(TaskVisibility::Organization.as_str()));

        conditions = self.apply_common_filters(conditions, query);

        db_query = db_query.filter(conditions);
        db_query = self.apply_sorting(db_query, query);

        let (page, per_page) = query.pagination.get_pagination();
        let paginator = db_query.paginate(&self.db, per_page as u64);

        let total_count = paginator.num_items().await?;
        let items = paginator.fetch_page((page - 1) as u64).await?;

        Ok((items, total_count))
    }

    /// アクセス可能な全タスクの取得
    pub async fn find_accessible_tasks(
        &self,
        user_id: Uuid,
        team_ids: &[Uuid],
        organization_id: Option<Uuid>,
        query: &TaskSearchQuery,
    ) -> Result<(Vec<task_model::Model>, u64), DbErr> {
        self.prepare_connection().await?;

        let mut db_query = TaskEntity::find();
        let mut access_conditions = Condition::any();

        // 個人タスク（所有または割り当て）
        access_conditions = access_conditions.add(
            Condition::all()
                .add(task_model::Column::Visibility.eq(TaskVisibility::Personal.as_str()))
                .add(
                    Condition::any()
                        .add(task_model::Column::UserId.eq(user_id))
                        .add(task_model::Column::AssignedTo.eq(user_id)),
                ),
        );

        // チームタスク
        if !team_ids.is_empty() {
            access_conditions = access_conditions.add(
                Condition::all()
                    .add(task_model::Column::Visibility.eq(TaskVisibility::Team.as_str()))
                    .add(task_model::Column::TeamId.is_in(team_ids.to_vec())),
            );
        }

        // 組織タスク
        if let Some(org_id) = organization_id {
            access_conditions = access_conditions.add(
                Condition::all()
                    .add(task_model::Column::Visibility.eq(TaskVisibility::Organization.as_str()))
                    .add(task_model::Column::OrganizationId.eq(org_id)),
            );
        }

        let mut conditions = Condition::all().add(access_conditions);
        conditions = self.apply_common_filters(conditions, query);

        db_query = db_query.filter(conditions);
        db_query = self.apply_sorting(db_query, query);

        let (page, per_page) = query.pagination.get_pagination();
        let paginator = db_query.paginate(&self.db, per_page as u64);

        let total_count = paginator.num_items().await?;
        let items = paginator.fetch_page((page - 1) as u64).await?;

        Ok((items, total_count))
    }

    /// チームタスクの作成
    pub async fn create_team_task(
        &self,
        team_id: Uuid,
        organization_id: Option<Uuid>,
        payload: CreateTaskDto,
        visibility: TaskVisibility,
        assigned_to: Option<Uuid>,
    ) -> Result<task_model::Model, DbErr> {
        self.prepare_connection().await?;

        let mut new_task = TaskActiveModel::new();
        new_task.title = Set(payload.title);
        new_task.description = Set(payload.description);
        new_task.status = Set(payload.status.unwrap_or(TaskStatus::Todo).to_string());
        new_task.priority = Set(payload.priority.unwrap_or_else(|| "medium".to_string()));
        new_task.due_date = Set(payload.due_date);

        // ヘルパーメソッドを使用してチームタスクとして設定
        if let Some(org_id) = organization_id {
            new_task.set_as_team_task(team_id, org_id);
        } else {
            // organization_idがない場合は手動で設定
            new_task.team_id = Set(Some(team_id));
            new_task.visibility = Set(visibility);
            new_task.user_id = Set(None);
        }

        new_task.assign_to(assigned_to);

        new_task.insert(&self.db).await
    }

    /// 組織タスクの作成
    pub async fn create_organization_task(
        &self,
        organization_id: Uuid,
        payload: CreateTaskDto,
        assigned_to: Option<Uuid>,
    ) -> Result<task_model::Model, DbErr> {
        self.prepare_connection().await?;

        let mut new_task = TaskActiveModel::new();
        new_task.title = Set(payload.title);
        new_task.description = Set(payload.description);
        new_task.status = Set(payload.status.unwrap_or(TaskStatus::Todo).to_string());
        new_task.priority = Set(payload.priority.unwrap_or_else(|| "medium".to_string()));
        new_task.due_date = Set(payload.due_date);

        // ヘルパーメソッドを使用して組織タスクとして設定
        new_task.set_as_organization_task(organization_id);
        new_task.assign_to(assigned_to);

        new_task.insert(&self.db).await
    }

    /// タスクの割り当て更新
    pub async fn update_task_assignment(
        &self,
        task_id: Uuid,
        assigned_to: Option<Uuid>,
    ) -> Result<Option<task_model::Model>, DbErr> {
        self.prepare_connection().await?;

        let task = match TaskEntity::find_by_id(task_id).one(&self.db).await? {
            Some(t) => t,
            None => return Ok(None),
        };

        let mut active_model: TaskActiveModel = task.into();
        // ヘルパーメソッドを使用して割り当て
        active_model.assign_to(assigned_to);

        Ok(Some(active_model.update(&self.db).await?))
    }

    // 共通フィルタ適用ヘルパー
    fn apply_common_filters(
        &self,
        mut conditions: Condition,
        query: &TaskSearchQuery,
    ) -> Condition {
        // ステータスフィルタ
        if let Some(status) = &query.status {
            conditions = conditions.add(task_model::Column::Status.eq(status.as_str()));
        }

        // 検索（タイトルと説明文のOR検索）
        if let Some(search_term) = &query.search {
            conditions = conditions.add(
                Condition::any()
                    .add(task_model::Column::Title.contains(search_term))
                    .add(task_model::Column::Description.contains(search_term)),
            );
        }

        // 担当者フィルタ
        if let Some(assigned_to) = &query.assigned_to {
            conditions = conditions.add(task_model::Column::AssignedTo.eq(*assigned_to));
        }

        // 優先度フィルタ
        if let Some(priority) = &query.priority {
            conditions = conditions.add(task_model::Column::Priority.eq(priority));
        }

        // 期限前フィルタ
        if let Some(due_before) = &query.due_date_before {
            conditions = conditions.add(task_model::Column::DueDate.lte(*due_before));
        }

        // 期限後フィルタ
        if let Some(due_after) = &query.due_date_after {
            conditions = conditions.add(task_model::Column::DueDate.gte(*due_after));
        }

        // 作成日時フィルタ
        if let Some(created_after) = &query.created_after {
            conditions = conditions.add(task_model::Column::CreatedAt.gte(*created_after));
        }

        if let Some(created_before) = &query.created_before {
            conditions = conditions.add(task_model::Column::CreatedAt.lte(*created_before));
        }

        conditions
    }

    // ソート適用ヘルパー
    fn apply_sorting(
        &self,
        mut db_query: Select<TaskEntity>,
        query: &TaskSearchQuery,
    ) -> Select<TaskEntity> {
        use crate::types::SortOrder;

        if let Some(sort_by) = &query.sort.sort_by {
            match sort_by.as_str() {
                "title" => {
                    db_query = match query.sort.sort_order {
                        SortOrder::Asc => db_query.order_by_asc(task_model::Column::Title),
                        SortOrder::Desc => db_query.order_by_desc(task_model::Column::Title),
                    };
                }
                "due_date" => {
                    db_query = match query.sort.sort_order {
                        SortOrder::Asc => db_query.order_by_asc(task_model::Column::DueDate),
                        SortOrder::Desc => db_query.order_by_desc(task_model::Column::DueDate),
                    };
                }
                "priority" => {
                    db_query = match query.sort.sort_order {
                        SortOrder::Asc => db_query.order_by_asc(task_model::Column::Priority),
                        SortOrder::Desc => db_query.order_by_desc(task_model::Column::Priority),
                    };
                }
                "status" => {
                    db_query = match query.sort.sort_order {
                        SortOrder::Asc => db_query.order_by_asc(task_model::Column::Status),
                        SortOrder::Desc => db_query.order_by_desc(task_model::Column::Status),
                    };
                }
                "updated_at" => {
                    db_query = match query.sort.sort_order {
                        SortOrder::Asc => db_query.order_by_asc(task_model::Column::UpdatedAt),
                        SortOrder::Desc => db_query.order_by_desc(task_model::Column::UpdatedAt),
                    };
                }
                "visibility" => {
                    db_query = match query.sort.sort_order {
                        SortOrder::Asc => db_query.order_by_asc(task_model::Column::Visibility),
                        SortOrder::Desc => db_query.order_by_desc(task_model::Column::Visibility),
                    };
                }
                "assigned_to" => {
                    db_query = match query.sort.sort_order {
                        SortOrder::Asc => db_query.order_by_asc(task_model::Column::AssignedTo),
                        SortOrder::Desc => db_query.order_by_desc(task_model::Column::AssignedTo),
                    };
                }
                _ => {
                    // デフォルトは作成日時の降順
                    db_query = db_query.order_by_desc(task_model::Column::CreatedAt);
                }
            }
        } else {
            // ソート指定がない場合はデフォルトで作成日時の降順
            db_query = db_query.order_by_desc(task_model::Column::CreatedAt);
        }

        db_query
    }
}
