// task-backend/src/repository/attachment_repository.rs

use crate::api::dto::attachment_query_dto::AttachmentSearchQuery;
use crate::db;
use crate::domain::task_attachment_model::{
    self, ActiveModel as AttachmentActiveModel, Entity as AttachmentEntity,
};
use crate::types::{SortOrder as TypesSortOrder, SortQuery};
use chrono::Utc;
use sea_orm::{entity::*, Condition, DbConn, DbErr, DeleteResult, Set};
use sea_orm::{PaginatorTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

pub struct AttachmentRepository {
    db: DbConn,
    schema: Option<String>,
}

impl AttachmentRepository {
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

    /// IDで添付ファイルを取得
    pub async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<task_attachment_model::Model>, DbErr> {
        self.prepare_connection().await?;
        AttachmentEntity::find_by_id(id).one(&self.db).await
    }

    /// 新しい添付ファイルを作成
    pub async fn create(
        &self,
        data: CreateAttachmentDto,
    ) -> Result<task_attachment_model::Model, DbErr> {
        self.prepare_connection().await?;

        let attachment = AttachmentActiveModel {
            id: Set(Uuid::new_v4()),
            task_id: Set(data.task_id),
            uploaded_by: Set(data.uploaded_by),
            file_name: Set(data.file_name),
            file_size: Set(data.file_size),
            mime_type: Set(data.mime_type),
            storage_key: Set(data.storage_key),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        attachment.insert(&self.db).await
    }

    /// 添付ファイルを削除
    pub async fn delete(&self, id: Uuid) -> Result<DeleteResult, DbErr> {
        self.prepare_connection().await?;
        AttachmentEntity::delete_by_id(id).exec(&self.db).await
    }

    /// ユーザーのストレージ使用量を計算
    pub async fn calculate_user_storage_usage(&self, user_id: Uuid) -> Result<i64, DbErr> {
        self.prepare_connection().await?;

        let attachments = AttachmentEntity::find()
            .filter(task_attachment_model::Column::UploadedBy.eq(user_id))
            .all(&self.db)
            .await?;

        let total_size: i64 = attachments.iter().map(|a| a.file_size).sum();
        Ok(total_size)
    }

    /// 添付ファイルを検索（統一クエリパターン版）
    pub async fn search_attachments(
        &self,
        query: &AttachmentSearchQuery,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<task_attachment_model::Model>, u64), DbErr> {
        self.prepare_connection().await?;

        let mut condition = Condition::all();

        // 検索条件の適用
        if let Some(search_term) = &query.search {
            let search_pattern = format!("%{}%", search_term);
            condition =
                condition.add(task_attachment_model::Column::FileName.like(&search_pattern));
        }

        if let Some(task_id) = &query.task_id {
            condition = condition.add(task_attachment_model::Column::TaskId.eq(*task_id));
        }

        if let Some(uploaded_by) = &query.uploaded_by {
            condition = condition.add(task_attachment_model::Column::UploadedBy.eq(*uploaded_by));
        }

        if let Some(file_type) = &query.file_type {
            condition = condition
                .add(task_attachment_model::Column::MimeType.like(format!("{}%", file_type)));
        }

        if let Some(min_size) = query.min_size {
            condition = condition.add(task_attachment_model::Column::FileSize.gte(min_size));
        }

        if let Some(max_size) = query.max_size {
            condition = condition.add(task_attachment_model::Column::FileSize.lte(max_size));
        }

        let mut db_query = AttachmentEntity::find().filter(condition);

        // ソートの適用
        db_query = self.apply_sorting(db_query, &query.sort);

        // ページネーション
        let page_size = per_page as u64;
        let offset = ((page - 1) * per_page) as u64;

        let paginator = db_query.paginate(&self.db, page_size);
        let total_count = paginator.num_items().await?;
        let items = paginator.fetch_page(offset / page_size).await?;

        Ok((items, total_count))
    }

    /// ソート適用ヘルパー
    fn apply_sorting(
        &self,
        mut query: sea_orm::Select<task_attachment_model::Entity>,
        sort: &SortQuery,
    ) -> sea_orm::Select<task_attachment_model::Entity> {
        if let Some(sort_by) = &sort.sort_by {
            let allowed_fields = AttachmentSearchQuery::allowed_sort_fields();

            if allowed_fields.contains(&sort_by.as_str()) {
                match sort_by.as_str() {
                    "file_name" => {
                        query = match sort.sort_order {
                            TypesSortOrder::Asc => {
                                query.order_by_asc(task_attachment_model::Column::FileName)
                            }
                            TypesSortOrder::Desc => {
                                query.order_by_desc(task_attachment_model::Column::FileName)
                            }
                        };
                    }
                    "file_size" => {
                        query = match sort.sort_order {
                            TypesSortOrder::Asc => {
                                query.order_by_asc(task_attachment_model::Column::FileSize)
                            }
                            TypesSortOrder::Desc => {
                                query.order_by_desc(task_attachment_model::Column::FileSize)
                            }
                        };
                    }
                    "uploaded_at" => {
                        query = match sort.sort_order {
                            TypesSortOrder::Asc => {
                                query.order_by_asc(task_attachment_model::Column::CreatedAt)
                            }
                            TypesSortOrder::Desc => {
                                query.order_by_desc(task_attachment_model::Column::CreatedAt)
                            }
                        };
                    }
                    "file_type" => {
                        query = match sort.sort_order {
                            TypesSortOrder::Asc => {
                                query.order_by_asc(task_attachment_model::Column::MimeType)
                            }
                            TypesSortOrder::Desc => {
                                query.order_by_desc(task_attachment_model::Column::MimeType)
                            }
                        };
                    }
                    _ => {
                        // デフォルトは作成日時の降順
                        query = query.order_by_desc(task_attachment_model::Column::CreatedAt);
                    }
                }
            } else {
                // 許可されていないフィールドの場合はデフォルト
                query = query.order_by_desc(task_attachment_model::Column::CreatedAt);
            }
        } else {
            // sort_byが指定されていない場合はデフォルト
            query = query.order_by_desc(task_attachment_model::Column::CreatedAt);
        }

        query
    }
}

/// 添付ファイル作成用DTO
#[derive(Debug)]
pub struct CreateAttachmentDto {
    pub task_id: Uuid,
    pub uploaded_by: Uuid,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub storage_key: String,
}
