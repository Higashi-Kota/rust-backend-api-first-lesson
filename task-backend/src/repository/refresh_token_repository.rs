// src/repository/refresh_token_repository.rs

use crate::db;
use crate::domain::refresh_token_model::{
    self, ActiveModel as RefreshTokenActiveModel, CleanupResult, CreateRefreshToken,
    Entity as RefreshTokenEntity, RefreshTokenStats, RevokeAllResult,
};
use chrono::Utc;
use sea_orm::entity::*;
use sea_orm::{query::*, DbConn, DbErr, Set};
use sea_orm::{Condition, Order, QueryFilter, QueryOrder};
use uuid::Uuid;

pub struct RefreshTokenRepository {
    db: DbConn,
    schema: Option<String>,
}

impl RefreshTokenRepository {
    pub fn new(db: DbConn) -> Self {
        Self { db, schema: None }
    }

    // スキーマを設定するヘルパーメソッド
    async fn prepare_connection(&self) -> Result<(), DbErr> {
        if let Some(schema) = &self.schema {
            db::set_schema(&self.db, schema).await?;
        }
        Ok(())
    }

    // --- 基本CRUD操作 ---

    /// リフレッシュトークンをトークンハッシュで検索
    pub async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<refresh_token_model::Model>, DbErr> {
        self.prepare_connection().await?;
        RefreshTokenEntity::find()
            .filter(refresh_token_model::Column::TokenHash.eq(token_hash))
            .one(&self.db)
            .await
    }

    /// 有効なリフレッシュトークンをトークンハッシュで検索
    pub async fn find_valid_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<refresh_token_model::Model>, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();

        RefreshTokenEntity::find()
            .filter(
                Condition::all()
                    .add(refresh_token_model::Column::TokenHash.eq(token_hash))
                    .add(refresh_token_model::Column::IsRevoked.eq(false))
                    .add(refresh_token_model::Column::ExpiresAt.gt(now)),
            )
            .one(&self.db)
            .await
    }

    /// ユーザーの全リフレッシュトークンを取得
    pub async fn find_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<refresh_token_model::Model>, DbErr> {
        self.prepare_connection().await?;
        RefreshTokenEntity::find()
            .filter(refresh_token_model::Column::UserId.eq(user_id))
            .order_by(refresh_token_model::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await
    }

    /// リフレッシュトークンを作成
    pub async fn create(
        &self,
        create_token: CreateRefreshToken,
    ) -> Result<refresh_token_model::Model, DbErr> {
        self.prepare_connection().await?;

        let new_token = RefreshTokenActiveModel {
            user_id: Set(create_token.user_id),
            token_hash: Set(create_token.token_hash),
            expires_at: Set(create_token.expires_at),
            ..Default::default()
        };

        new_token.insert(&self.db).await
    }

    /// トークンハッシュで無効化
    pub async fn revoke_by_token_hash(&self, token_hash: &str) -> Result<bool, DbErr> {
        self.prepare_connection().await?;

        let token = match self.find_by_token_hash(token_hash).await? {
            Some(t) => t,
            None => return Ok(false),
        };

        let mut active_model: RefreshTokenActiveModel = token.into();
        active_model.is_revoked = Set(true);
        active_model.update(&self.db).await?;

        Ok(true)
    }

    /// ユーザーの全リフレッシュトークンを無効化
    pub async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<u64, DbErr> {
        self.prepare_connection().await?;

        let tokens = self.find_by_user_id(user_id).await?;
        let mut revoked_count = 0;

        for token in tokens {
            if !token.is_revoked {
                let mut active_model: RefreshTokenActiveModel = token.into();
                active_model.is_revoked = Set(true);
                active_model.update(&self.db).await?;
                revoked_count += 1;
            }
        }

        Ok(revoked_count)
    }

    // --- トークンローテーション（Phase 1.1/1.2で必要な機能のみ）---

    /// 簡易トークンローテーション
    pub async fn rotate_token(
        &self,
        old_token_hash: &str,
        new_token: CreateRefreshToken,
    ) -> Result<Option<bool>, DbErr> {
        self.prepare_connection().await?;

        // 古いトークンを無効化
        self.revoke_by_token_hash(old_token_hash).await?;

        // 新しいトークンを作成
        self.create(new_token).await?;

        Ok(Some(true))
    }

    // --- クリーンアップ機能 ---

    /// 期限切れのトークンを削除
    pub async fn cleanup_expired_tokens(&self) -> Result<CleanupResult, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();

        let delete_result = RefreshTokenEntity::delete_many()
            .filter(refresh_token_model::Column::ExpiresAt.lt(now))
            .exec(&self.db)
            .await?;

        Ok(CleanupResult {
            deleted_count: delete_result.rows_affected,
        })
    }

    /// 無効化されたトークンを削除
    pub async fn cleanup_revoked_tokens(&self) -> Result<CleanupResult, DbErr> {
        self.prepare_connection().await?;

        let delete_result = RefreshTokenEntity::delete_many()
            .filter(refresh_token_model::Column::IsRevoked.eq(true))
            .exec(&self.db)
            .await?;

        Ok(CleanupResult {
            deleted_count: delete_result.rows_affected,
        })
    }

    // --- 統計情報 ---

    /// リフレッシュトークンの統計情報を取得
    pub async fn get_token_stats(&self) -> Result<RefreshTokenStats, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();

        let total_tokens = RefreshTokenEntity::find().count(&self.db).await?;

        let active_tokens = RefreshTokenEntity::find()
            .filter(
                Condition::all()
                    .add(refresh_token_model::Column::IsRevoked.eq(false))
                    .add(refresh_token_model::Column::ExpiresAt.gt(now)),
            )
            .count(&self.db)
            .await?;

        let expired_tokens = RefreshTokenEntity::find()
            .filter(refresh_token_model::Column::ExpiresAt.lte(now))
            .count(&self.db)
            .await?;

        let revoked_tokens = RefreshTokenEntity::find()
            .filter(refresh_token_model::Column::IsRevoked.eq(true))
            .count(&self.db)
            .await?;

        Ok(RefreshTokenStats {
            total_tokens,
            active_tokens,
            expired_tokens,
            revoked_tokens,
        })
    }

    /// 全ユーザーのトークンを無効化
    pub async fn revoke_all_tokens(&self) -> Result<RevokeAllResult, DbErr> {
        self.prepare_connection().await?;

        // 有効なトークンのみを対象
        let active_tokens = RefreshTokenEntity::find()
            .filter(refresh_token_model::Column::IsRevoked.eq(false))
            .all(&self.db)
            .await?;

        let mut revoked_count = 0;
        let mut affected_users = std::collections::HashSet::new();

        for token in active_tokens {
            let mut active_model: RefreshTokenActiveModel = token.clone().into();
            active_model.is_revoked = Set(true);
            active_model.update(&self.db).await?;
            revoked_count += 1;
            affected_users.insert(token.user_id);
        }

        Ok(RevokeAllResult {
            revoked_count,
            affected_users: affected_users.len() as u64,
        })
    }

    /// 特定ユーザーを除いて全トークンを無効化
    pub async fn revoke_all_tokens_except_user(
        &self,
        exclude_user_id: Uuid,
    ) -> Result<RevokeAllResult, DbErr> {
        self.prepare_connection().await?;

        // 指定ユーザー以外の有効なトークンを取得
        let active_tokens = RefreshTokenEntity::find()
            .filter(
                Condition::all()
                    .add(refresh_token_model::Column::IsRevoked.eq(false))
                    .add(refresh_token_model::Column::UserId.ne(exclude_user_id)),
            )
            .all(&self.db)
            .await?;

        let mut revoked_count = 0;
        let mut affected_users = std::collections::HashSet::new();

        for token in active_tokens {
            let mut active_model: RefreshTokenActiveModel = token.clone().into();
            active_model.is_revoked = Set(true);
            active_model.update(&self.db).await?;
            revoked_count += 1;
            affected_users.insert(token.user_id);
        }

        Ok(RevokeAllResult {
            revoked_count,
            affected_users: affected_users.len() as u64,
        })
    }

    /// 最古のアクティブトークンの年齢（日数）を取得
    pub async fn get_oldest_active_token_age_days(&self) -> Result<Option<i64>, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();

        let oldest_token = RefreshTokenEntity::find()
            .filter(
                Condition::all()
                    .add(refresh_token_model::Column::IsRevoked.eq(false))
                    .add(refresh_token_model::Column::ExpiresAt.gt(now)),
            )
            .order_by(refresh_token_model::Column::CreatedAt, Order::Asc)
            .one(&self.db)
            .await?;

        Ok(oldest_token.map(|token| (now - token.created_at).num_days()))
    }

    /// 最新のアクティブトークンの年齢（時間）を取得
    pub async fn get_newest_active_token_age_hours(&self) -> Result<Option<i64>, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();

        let newest_token = RefreshTokenEntity::find()
            .filter(
                Condition::all()
                    .add(refresh_token_model::Column::IsRevoked.eq(false))
                    .add(refresh_token_model::Column::ExpiresAt.gt(now)),
            )
            .order_by(refresh_token_model::Column::CreatedAt, Order::Desc)
            .one(&self.db)
            .await?;

        Ok(newest_token.map(|token| (now - token.created_at).num_hours()))
    }
}

// --- DTOと関連構造体 ---
