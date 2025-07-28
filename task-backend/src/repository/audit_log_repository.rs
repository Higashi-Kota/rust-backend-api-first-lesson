// src/repository/audit_log_repository.rs
use crate::api::handlers::audit_log_handler::AuditLogQuery;
use crate::db;
use crate::domain::audit_log_model::{
    self, ActiveModel as AuditLogActiveModel, Entity as AuditLogEntity, Model as AuditLogModel,
};
use crate::types::{SortOrder, SortQuery};
use sea_orm::{entity::*, query::*, DbConn, DbErr, QueryOrder};
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
        sort: &SortQuery,
        created_after: Option<chrono::DateTime<chrono::Utc>>,
        created_before: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<AuditLogModel>, DbErr> {
        self.prepare_connection().await?;
        let mut query = AuditLogEntity::find().filter(audit_log_model::Column::UserId.eq(user_id));

        // 日付フィルタの適用
        if let Some(after) = created_after {
            query = query.filter(audit_log_model::Column::CreatedAt.gte(after));
        }
        if let Some(before) = created_before {
            query = query.filter(audit_log_model::Column::CreatedAt.lte(before));
        }

        // ソートの適用
        query = self.apply_sorting(query, sort);

        query.limit(limit).offset(offset).all(&self.db).await
    }

    // チームの監査ログを取得
    pub async fn find_by_team(
        &self,
        team_id: Uuid,
        limit: u64,
        offset: u64,
        sort: &SortQuery,
        created_after: Option<chrono::DateTime<chrono::Utc>>,
        created_before: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<AuditLogModel>, DbErr> {
        self.prepare_connection().await?;
        let mut query = AuditLogEntity::find().filter(audit_log_model::Column::TeamId.eq(team_id));

        // 日付フィルタの適用
        if let Some(after) = created_after {
            query = query.filter(audit_log_model::Column::CreatedAt.gte(after));
        }
        if let Some(before) = created_before {
            query = query.filter(audit_log_model::Column::CreatedAt.lte(before));
        }

        // ソートの適用
        query = self.apply_sorting(query, sort);

        query.limit(limit).offset(offset).all(&self.db).await
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

    // ソート適用ヘルパー
    fn apply_sorting(
        &self,
        mut query: Select<AuditLogEntity>,
        sort: &SortQuery,
    ) -> Select<AuditLogEntity> {
        if let Some(sort_by) = &sort.sort_by {
            let allowed_fields = AuditLogQuery::allowed_sort_fields();

            if allowed_fields.contains(&sort_by.as_str()) {
                match sort_by.as_str() {
                    "created_at" => {
                        query = match sort.sort_order {
                            SortOrder::Asc => {
                                query.order_by_asc(audit_log_model::Column::CreatedAt)
                            }
                            SortOrder::Desc => {
                                query.order_by_desc(audit_log_model::Column::CreatedAt)
                            }
                        };
                    }
                    "action" => {
                        query = match sort.sort_order {
                            SortOrder::Asc => query.order_by_asc(audit_log_model::Column::Action),
                            SortOrder::Desc => query.order_by_desc(audit_log_model::Column::Action),
                        };
                    }
                    "resource_type" => {
                        query = match sort.sort_order {
                            SortOrder::Asc => {
                                query.order_by_asc(audit_log_model::Column::ResourceType)
                            }
                            SortOrder::Desc => {
                                query.order_by_desc(audit_log_model::Column::ResourceType)
                            }
                        };
                    }
                    "user_id" => {
                        query = match sort.sort_order {
                            SortOrder::Asc => query.order_by_asc(audit_log_model::Column::UserId),
                            SortOrder::Desc => query.order_by_desc(audit_log_model::Column::UserId),
                        };
                    }
                    _ => {
                        // デフォルトは作成日時の降順
                        query = query.order_by_desc(audit_log_model::Column::CreatedAt);
                    }
                }
            } else {
                // 許可されていないフィールドの場合はデフォルト
                query = query.order_by_desc(audit_log_model::Column::CreatedAt);
            }
        } else {
            // sort_byが指定されていない場合はデフォルト
            query = query.order_by_desc(audit_log_model::Column::CreatedAt);
        }

        query
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
