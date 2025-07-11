// task-backend/src/features/storage/repository/attachment_share_link_repository.rs

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::features::storage::models::attachment_share_link::{
    self, Entity as AttachmentShareLink,
};
use crate::features::storage::models::share_link_access_log;
use chrono::{DateTime, Utc};
use sea_orm::{
    entity::*, query::*, ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionTrait,
};
use uuid::Uuid;

pub struct AttachmentShareLinkRepository {
    db: DbPool,
}

impl AttachmentShareLinkRepository {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// 共有リンクを作成
    pub async fn create(
        &self,
        dto: CreateShareLinkDto,
    ) -> AppResult<crate::features::storage::models::attachment_share_link::Model> {
        let share_link = crate::features::storage::models::attachment_share_link::ActiveModel {
            id: Set(dto.id),
            attachment_id: Set(dto.attachment_id),
            created_by: Set(dto.created_by),
            share_token: Set(dto.share_token),
            description: Set(dto.description),
            expires_at: Set(dto.expires_at),
            max_access_count: Set(dto.max_access_count),
            current_access_count: Set(0),
            is_revoked: Set(false),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        share_link.insert(&self.db).await.map_err(AppError::DbErr)
    }

    /// トークンで共有リンクを検索
    pub async fn find_by_token(
        &self,
        token: &str,
    ) -> AppResult<Option<crate::features::storage::models::attachment_share_link::Model>> {
        AttachmentShareLink::find()
            .filter(
                crate::features::storage::models::attachment_share_link::Column::ShareToken
                    .eq(token),
            )
            .one(&self.db)
            .await
            .map_err(AppError::DbErr)
    }

    /// IDで共有リンクを検索
    pub async fn find_by_id(
        &self,
        id: Uuid,
    ) -> AppResult<Option<crate::features::storage::models::attachment_share_link::Model>> {
        AttachmentShareLink::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::DbErr)
    }

    /// 添付ファイルIDで共有リンク一覧を取得
    pub async fn find_by_attachment_id(
        &self,
        attachment_id: Uuid,
    ) -> AppResult<Vec<crate::features::storage::models::attachment_share_link::Model>> {
        AttachmentShareLink::find()
            .filter(
                crate::features::storage::models::attachment_share_link::Column::AttachmentId
                    .eq(attachment_id),
            )
            .filter(
                crate::features::storage::models::attachment_share_link::Column::IsRevoked
                    .eq(false),
            )
            .order_by_desc(
                crate::features::storage::models::attachment_share_link::Column::CreatedAt,
            )
            .all(&self.db)
            .await
            .map_err(AppError::DbErr)
    }

    /// 共有リンクを無効化
    pub async fn revoke(&self, id: Uuid) -> AppResult<()> {
        let share_link = AttachmentShareLink::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::DbErr)?
            .ok_or_else(|| AppError::NotFound("Share link not found".to_string()))?;

        let mut active_model: crate::features::storage::models::attachment_share_link::ActiveModel =
            share_link.into();
        active_model.is_revoked = Set(true);
        active_model.updated_at = Set(Utc::now());

        active_model
            .update(&self.db)
            .await
            .map_err(AppError::DbErr)?;

        Ok(())
    }

    /// アクセスカウントを増加し、ログを記録（トランザクション）
    pub async fn record_access(
        &self,
        share_link_id: Uuid,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<()> {
        let db: &DatabaseConnection = &self.db;

        let txn = db.begin().await.map_err(AppError::DbErr)?;

        // アクセスカウントを増加
        let share_link = AttachmentShareLink::find_by_id(share_link_id)
            .one(&txn)
            .await
            .map_err(AppError::DbErr)?
            .ok_or_else(|| AppError::NotFound("Share link not found".to_string()))?;

        let mut active_model: crate::features::storage::models::attachment_share_link::ActiveModel =
            share_link.into();
        active_model.current_access_count = Set(active_model.current_access_count.unwrap() + 1);
        active_model.updated_at = Set(Utc::now());

        active_model.update(&txn).await.map_err(AppError::DbErr)?;

        // アクセスログを記録
        let access_log = crate::features::storage::models::share_link_access_log::ActiveModel {
            id: Set(Uuid::new_v4()),
            share_link_id: Set(share_link_id),
            ip_address: Set(ip_address),
            user_agent: Set(user_agent),
            accessed_at: Set(Utc::now()),
        };

        access_log.insert(&txn).await.map_err(AppError::DbErr)?;

        txn.commit().await.map_err(AppError::DbErr)?;

        Ok(())
    }
}

pub struct CreateShareLinkDto {
    pub id: Uuid,
    pub attachment_id: Uuid,
    pub created_by: Uuid,
    pub share_token: String,
    pub description: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub max_access_count: Option<i32>,
}
