// task-backend/src/domain/attachment_share_link_model.rs

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "attachment_share_links")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub attachment_id: Uuid,
    pub created_by: Uuid,
    pub share_token: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub expires_at: DateTime<Utc>,
    #[sea_orm(nullable)]
    pub max_access_count: Option<i32>,
    pub current_access_count: i32,
    pub is_revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::domain::task_attachment_model::Entity",
        from = "Column::AttachmentId",
        to = "crate::domain::task_attachment_model::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    TaskAttachment,

    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::CreatedBy",
        to = "crate::domain::user_model::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    User,
}

impl Related<crate::domain::task_attachment_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskAttachment.def()
    }
}

impl Related<crate::domain::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
