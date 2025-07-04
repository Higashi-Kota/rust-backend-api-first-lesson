// task-backend/src/domain/share_link_access_log_model.rs

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "share_link_access_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub share_link_id: Uuid,
    #[sea_orm(column_type = "Text", nullable)]
    pub ip_address: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub user_agent: Option<String>,
    pub accessed_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::domain::attachment_share_link_model::Entity",
        from = "Column::ShareLinkId",
        to = "crate::domain::attachment_share_link_model::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    AttachmentShareLink,
}

impl Related<crate::domain::attachment_share_link_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AttachmentShareLink.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
