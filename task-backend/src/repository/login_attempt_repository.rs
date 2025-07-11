// task-backend/src/repository/login_attempt_repository.rs

use crate::db::DbPool;
use crate::domain::login_attempt_model::{ActiveModel, Column, Entity, Model};
use crate::error::AppResult;
use chrono::{DateTime, Utc};
use sea_orm::sea_query::ExprTrait;
use sea_orm::*;

#[derive(Clone)]
pub struct LoginAttemptRepository {
    db: DbPool,
}

impl LoginAttemptRepository {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// ログイン試行を記録
    pub async fn create(&self, attempt: &Model) -> AppResult<Model> {
        let active_model = ActiveModel {
            id: Set(attempt.id),
            email: Set(attempt.email.clone()),
            user_id: Set(attempt.user_id),
            success: Set(attempt.success),
            failure_reason: Set(attempt.failure_reason.clone()),
            ip_address: Set(attempt.ip_address.clone()),
            user_agent: Set(attempt.user_agent.clone()),
            device_type: Set(attempt.device_type.clone()),
            browser_name: Set(attempt.browser_name.clone()),
            country: Set(attempt.country.clone()),
            suspicious_score: Set(attempt.suspicious_score),
            attempted_at: Set(attempt.attempted_at),
        };

        let result = active_model.insert(&self.db).await?;
        Ok(result)
    }

    /// 期間内の失敗ログイン総数を取得
    pub async fn count_total_failed_attempts(&self, since: DateTime<Utc>) -> AppResult<u64> {
        let count = Entity::find()
            .filter(Column::Success.eq(false))
            .filter(Column::AttemptedAt.gte(since))
            .count(&self.db)
            .await?;

        Ok(count)
    }

    /// 不審なアクティビティを検出（同一IPから多数の失敗試行）
    pub async fn find_suspicious_ips(
        &self,
        threshold: u64,
        hours: i64,
    ) -> AppResult<Vec<(String, u64)>> {
        let since = Utc::now() - chrono::Duration::hours(hours);

        let results = Entity::find()
            .filter(Column::Success.eq(false))
            .filter(Column::AttemptedAt.gte(since))
            .select_only()
            .column(Column::IpAddress)
            .column_as(Column::Id.count(), "count")
            .group_by(Column::IpAddress)
            .having(Column::Id.count().gte(threshold as i64))
            .into_tuple::<(String, i64)>()
            .all(&self.db)
            .await?;

        Ok(results
            .into_iter()
            .map(|(ip, count)| (ip, count as u64))
            .collect())
    }

    /// 不審なアクティビティを検出（IPごとの失敗回数と最終試行時刻を含む）
    #[allow(dead_code)] // Repository method for security monitoring
    pub async fn find_suspicious_ips_with_details(
        &self,
        threshold: u64,
        hours: i64,
    ) -> AppResult<Vec<(String, u64, DateTime<Utc>)>> {
        let since = Utc::now() - chrono::Duration::hours(hours);

        // 一度すべてのデータを取得してからメモリ上で集計する
        let all_attempts = Entity::find()
            .filter(Column::Success.eq(false))
            .filter(Column::AttemptedAt.gte(since))
            .all(&self.db)
            .await?;

        // IPアドレスごとに集計
        let mut ip_map: std::collections::HashMap<String, (u64, DateTime<Utc>)> =
            std::collections::HashMap::new();
        for attempt in all_attempts {
            let entry = ip_map
                .entry(attempt.ip_address.clone())
                .or_insert((0, attempt.attempted_at));
            entry.0 += 1;
            if attempt.attempted_at > entry.1 {
                entry.1 = attempt.attempted_at;
            }
        }

        // 閾値以上のものだけを結果として返す
        let results: Vec<(String, i64, DateTime<Utc>)> = ip_map
            .into_iter()
            .filter(|(_, (count, _))| *count >= threshold)
            .map(|(ip, (count, last))| (ip, count as i64, last))
            .collect();

        Ok(results
            .into_iter()
            .map(|(ip, count, last)| (ip, count as u64, last))
            .collect())
    }
}
