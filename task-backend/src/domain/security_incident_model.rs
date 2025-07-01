// task-backend/src/domain/security_incident_model.rs

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "security_incidents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub incident_type: String,
    pub severity: String,
    pub user_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub description: String,
    pub details: Option<Json>,
    pub status: String,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::UserId",
        to = "crate::domain::user_model::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::ResolvedBy",
        to = "crate::domain::user_model::Column::Id"
    )]
    Resolver,
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
            status: Set("open".to_string()),
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
            self.updated_at = Set(Utc::now());
        }
        Ok(self)
    }
}

/// インシデントの重要度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// インシデントのステータス
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}
