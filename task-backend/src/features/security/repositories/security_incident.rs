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
}
