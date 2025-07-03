use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "feature_usage_metrics")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub feature_name: String,
    pub action_type: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user_model::Entity",
        from = "Column::UserId",
        to = "super::user_model::Column::Id"
    )]
    User,
}

impl Related<super::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureUsageInput {
    pub feature_name: String,
    pub action_type: String,
    pub metadata: Option<serde_json::Value>,
}

impl Model {
    pub fn new(user_id: Uuid, input: FeatureUsageInput) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            feature_name: input.feature_name,
            action_type: input.action_type,
            metadata: input.metadata,
            created_at: Utc::now(),
        }
    }
}
