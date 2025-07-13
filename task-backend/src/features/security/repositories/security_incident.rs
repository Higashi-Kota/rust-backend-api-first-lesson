// task-backend/src/features/security/repositories/security_incident.rs

use super::super::models::security_incident::{Column, Entity};
use crate::db::DbPool;
use crate::error::AppResult;
use chrono::{DateTime, Utc};
use sea_orm::*;

#[derive(Clone)]
pub struct SecurityIncidentRepository {
    db: DbPool,
}

impl SecurityIncidentRepository {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// 期間内のインシデント数を取得
    pub async fn count_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> AppResult<u64> {
        let count = Entity::find()
            .filter(Column::CreatedAt.gte(start))
            .filter(Column::CreatedAt.lt(end))
            .count(&self.db)
            .await?;

        Ok(count)
    }

    /// セキュリティインシデントを作成
    pub async fn create_incident(
        &self,
        incident_type: &str,
        description: &str,
        metadata: serde_json::Value,
        severity: &str,
        user_id: Option<uuid::Uuid>,
    ) -> AppResult<super::super::models::security_incident::Model> {
        use super::super::models::security_incident;

        let active_model = security_incident::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            incident_type: Set(incident_type.to_string()),
            description: Set(description.to_string()),
            severity: Set(severity.to_string()),
            details: Set(Some(serde_json::value::to_value(metadata).unwrap())),
            user_id: Set(user_id),
            status: Set("open".to_string()),
            ip_address: Set(None),
            resolved_at: Set(None),
            resolved_by: Set(None),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let result = active_model.insert(&self.db).await?;
        Ok(result)
    }
}
