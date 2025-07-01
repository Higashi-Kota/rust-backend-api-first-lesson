// task-backend/src/domain/activity_log_model.rs

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "activity_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<Json>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::UserId",
        to = "crate::domain::user_model::Column::Id"
    )]
    User,
}

impl Related<crate::domain::user_model::Entity> for Entity {
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
            ..ActiveModelTrait::default()
        }
    }
}

impl Model {
    /// 新しいアクティビティログを作成
    pub fn new(
        user_id: Uuid,
        action: String,
        resource_type: String,
        resource_id: Option<Uuid>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        details: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            action,
            resource_type,
            resource_id,
            ip_address,
            user_agent,
            details,
            created_at: Utc::now(),
        }
    }

    /// ログインアクティビティを作成
    pub fn login(user_id: Uuid, ip_address: Option<String>, user_agent: Option<String>) -> Self {
        Self::new(
            user_id,
            "login".to_string(),
            "session".to_string(),
            None,
            ip_address,
            user_agent,
            None,
        )
    }
}
