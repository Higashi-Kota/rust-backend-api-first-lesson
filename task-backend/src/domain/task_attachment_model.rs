// task-backend/src/domain/task_attachment_model.rs

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait, DbErr, Set};
use serde::{Deserialize, Serialize};

/// タスク添付ファイルエンティティ
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "task_attachments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub task_id: Uuid,
    pub uploaded_by: Uuid,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub storage_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::task_model::Entity",
        from = "Column::TaskId",
        to = "super::task_model::Column::Id"
    )]
    Task,
    #[sea_orm(
        belongs_to = "super::user_model::Entity",
        from = "Column::UploadedBy",
        to = "super::user_model::Column::Id"
    )]
    User,
}

impl Related<super::task_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl Related<super::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..ActiveModelTrait::default()
        }
    }

    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert {
            // 更新の場合のみ updated_at を更新
            self.updated_at = Set(Utc::now());
        }
        Ok(self)
    }
}

/// 画像ファイルの許可されるMIME type
pub const ALLOWED_IMAGE_MIME_TYPES: &[&str] =
    &["image/jpeg", "image/png", "image/gif", "image/webp"];

/// ドキュメントファイルの許可されるMIME type
pub const ALLOWED_DOCUMENT_MIME_TYPES: &[&str] = &[
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document", // .docx
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", // .xlsx
    "text/csv",
];

/// すべての許可されるMIME type
pub fn get_all_allowed_mime_types() -> Vec<&'static str> {
    let mut types = Vec::new();
    types.extend_from_slice(ALLOWED_IMAGE_MIME_TYPES);
    types.extend_from_slice(ALLOWED_DOCUMENT_MIME_TYPES);
    types
}

/// MIMEタイプが画像として許可されているかチェック
pub fn is_allowed_image_mime_type(mime_type: &str) -> bool {
    ALLOWED_IMAGE_MIME_TYPES.contains(&mime_type)
}

/// MIMEタイプがドキュメントとして許可されているかチェック
pub fn is_allowed_document_mime_type(mime_type: &str) -> bool {
    ALLOWED_DOCUMENT_MIME_TYPES.contains(&mime_type)
}

/// MIMEタイプが許可されているかチェック
pub fn is_allowed_mime_type(mime_type: &str) -> bool {
    is_allowed_image_mime_type(mime_type) || is_allowed_document_mime_type(mime_type)
}

/// ファイルサイズの制限（バイト単位）
pub const MAX_FILE_SIZE_FREE: i64 = 5 * 1024 * 1024; // 5MB
pub const MAX_FILE_SIZE_PRO: i64 = 50 * 1024 * 1024; // 50MB
pub const MAX_FILE_SIZE_ENTERPRISE: i64 = 500 * 1024 * 1024; // 500MB
