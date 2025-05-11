// src/repository/task_repository.rs
use sea_orm::{entity::*, query::*, DbConn, DbErr, DeleteResult, InsertResult, Set}; // ConnectionTrait を追加
use uuid::Uuid;
use crate::domain::task_model::{self, Entity as TaskEntity, ActiveModel as TaskActiveModel};
use crate::api::dto::task_dto::{CreateTaskDto, UpdateTaskDto, BatchUpdateTaskItemDto};

pub struct TaskRepository {
    db: DbConn,
}

impl TaskRepository {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<task_model::Model>, DbErr> {
        TaskEntity::find_by_id(id).one(&self.db).await
    }

    pub async fn find_all(&self) -> Result<Vec<task_model::Model>, DbErr> {
        TaskEntity::find().all(&self.db).await
    }

    pub async fn create(&self, payload: CreateTaskDto) -> Result<task_model::Model, DbErr> {
        let new_task = TaskActiveModel {
            title: Set(payload.title),
            description: Set(payload.description),
            status: Set(payload.status.unwrap_or_else(|| "todo".to_string())),
            due_date: Set(payload.due_date),
            ..Default::default()
        };
        new_task.insert(&self.db).await
    }

    pub async fn update(&self, id: Uuid, payload: UpdateTaskDto) -> Result<Option<task_model::Model>, DbErr> {
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
        // payload.description は Option<String>
        // active_model.description は ActiveValue<Option<String>> を期待
        // Set に Option<String> を渡す
        // もし dto.description が Some(v) なら Set(Some(v)) に、None なら Set(None) (DB NULL) になる。
        // 「フィールドがリクエストに含まれていたら更新、含まれていなければ何もしない」という
        // 厳密な部分更新をしたい場合は、DTOの型を Option<Option<String>> にするなどの工夫が必要。
        // ここでは、DTOのフィールドの値でDBを更新する（NoneならNULLになる）という前提。
        // エラーを避けるため、payload.description は一度だけ評価する。
        if payload.description.is_some() { // is_some() は借用なのでOK
            active_model.description = Set(payload.description); // ここでムーブが発生
            changed = true;
        }
        // payload.status は Option<String>
        if let Some(status_val) = payload.status { // ここでムーブが発生
            active_model.status = Set(status_val);
            changed = true;
        }
        // payload.due_date は Option<DateTime<Utc>>
        if payload.due_date.is_some() { // is_some() は借用なのでOK
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
        TaskEntity::delete_by_id(id).exec(&self.db).await
    }

    #[allow(dead_code)] // この行を追加して警告を抑制
    pub async fn create_many(&self, payloads: Vec<CreateTaskDto>) -> Result<InsertResult<TaskActiveModel>, DbErr> {
        if payloads.is_empty() {
            return Ok(InsertResult { last_insert_id: Uuid::nil() });
        }
        let new_tasks: Vec<TaskActiveModel> = payloads
            .into_iter()
            .map(|payload| TaskActiveModel {
                title: Set(payload.title),
                description: Set(payload.description),
                status: Set(payload.status.unwrap_or_else(|| "todo".to_string())),
                due_date: Set(payload.due_date),
                ..Default::default()
            })
            .collect();

        TaskEntity::insert_many(new_tasks).exec(&self.db).await
    }

    pub async fn update_many(&self, items: Vec<BatchUpdateTaskItemDto>) -> Result<usize, DbErr> {
        let txn = self.db.begin().await?;
        let mut updated_count = 0;

        for item_payload in items { // `item` から `item_payload` に変更して明確化
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
                active_model.status = Set(status_val);
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
        if ids.is_empty() {
            return Ok(DeleteResult { rows_affected: 0 });
        }
        TaskEntity::delete_many()
            .filter(task_model::Column::Id.is_in(ids))
            .exec(&self.db)
            .await
    }
}