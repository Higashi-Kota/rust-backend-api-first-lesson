// task-backend/src/features/storage/repository/attachment_repository.rs

use crate::db;
use crate::features::storage::dto::{AttachmentSortBy, SortOrder};
use crate::features::task::models::task_attachment_model::{
    self, ActiveModel as AttachmentActiveModel, Entity as AttachmentEntity,
};
use chrono::Utc;
use sea_orm::{entity::*, DbConn, DbErr, DeleteResult, Set};
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

    /// ページング付きで添付ファイル一覧を取得
    pub async fn find_by_task_id_paginated(
        &self,
        task_id: Uuid,
        page: u64,
        per_page: u64,
        sort_by: Option<AttachmentSortBy>,
        sort_order: Option<SortOrder>,
    ) -> Result<(Vec<task_attachment_model::Model>, u64), DbErr> {
        self.prepare_connection().await?;

        let mut query =
            AttachmentEntity::find().filter(task_attachment_model::Column::TaskId.eq(task_id));

        // ソート設定
        let sort_order = sort_order.unwrap_or(SortOrder::Desc);
        let sort_by = sort_by.unwrap_or(AttachmentSortBy::CreatedAt);

        query = match (sort_by, sort_order) {
            (AttachmentSortBy::CreatedAt, SortOrder::Asc) => {
                query.order_by_asc(task_attachment_model::Column::CreatedAt)
            }
            (AttachmentSortBy::CreatedAt, SortOrder::Desc) => {
                query.order_by_desc(task_attachment_model::Column::CreatedAt)
            }
            (AttachmentSortBy::FileName, SortOrder::Asc) => {
                query.order_by_asc(task_attachment_model::Column::FileName)
            }
            (AttachmentSortBy::FileName, SortOrder::Desc) => {
                query.order_by_desc(task_attachment_model::Column::FileName)
            }
            (AttachmentSortBy::FileSize, SortOrder::Asc) => {
                query.order_by_asc(task_attachment_model::Column::FileSize)
            }
            (AttachmentSortBy::FileSize, SortOrder::Desc) => {
                query.order_by_desc(task_attachment_model::Column::FileSize)
            }
        };

        let paginator = query.paginate(&self.db, per_page);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page - 1).await?;

        Ok((items, total))
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
