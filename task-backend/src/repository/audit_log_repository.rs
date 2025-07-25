// src/repository/audit_log_repository.rs
use crate::db;
use crate::domain::audit_log_model::{
    self, ActiveModel as AuditLogActiveModel, Entity as AuditLogEntity, Model as AuditLogModel,
};
use sea_orm::{entity::*, query::*, DbConn, DbErr};
use uuid::Uuid;

pub struct AuditLogRepository {
    db: DbConn,
    schema: Option<String>,
}

impl AuditLogRepository {
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

    // 監査ログの作成
    pub async fn create(&self, audit_log: AuditLogActiveModel) -> Result<AuditLogModel, DbErr> {
        self.prepare_connection().await?;
        audit_log.insert(&self.db).await
    }

    // ユーザーの監査ログを取得
    pub async fn find_by_user(
        &self,
        user_id: Uuid,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<AuditLogModel>, DbErr> {
        self.prepare_connection().await?;
        AuditLogEntity::find()
            .filter(audit_log_model::Column::UserId.eq(user_id))
            .order_by_desc(audit_log_model::Column::CreatedAt)
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await
    }

    // チームの監査ログを取得
    pub async fn find_by_team(
        &self,
        team_id: Uuid,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<AuditLogModel>, DbErr> {
        self.prepare_connection().await?;
        AuditLogEntity::find()
            .filter(audit_log_model::Column::TeamId.eq(team_id))
            .order_by_desc(audit_log_model::Column::CreatedAt)
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await
    }

    // 総件数を取得（ページネーション用）
    pub async fn count_by_user(&self, user_id: Uuid) -> Result<u64, DbErr> {
        self.prepare_connection().await?;
        AuditLogEntity::find()
            .filter(audit_log_model::Column::UserId.eq(user_id))
            .count(&self.db)
            .await
    }

    pub async fn count_by_team(&self, team_id: Uuid) -> Result<u64, DbErr> {
        self.prepare_connection().await?;
        AuditLogEntity::find()
            .filter(audit_log_model::Column::TeamId.eq(team_id))
            .count(&self.db)
            .await
    }

    // 古いログの削除（保持期間を過ぎたもの）
    pub async fn delete_old_logs(
        &self,
        before_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, DbErr> {
        self.prepare_connection().await?;
        let result = AuditLogEntity::delete_many()
            .filter(audit_log_model::Column::CreatedAt.lt(before_date))
            .exec(&self.db)
            .await?;
        Ok(result.rows_affected)
    }
}

impl Clone for AuditLogRepository {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            schema: self.schema.clone(),
        }
    }
}
