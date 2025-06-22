// src/repository/refresh_token_repository.rs

use crate::db;
use crate::domain::refresh_token_model::{
    self, ActiveModel as RefreshTokenActiveModel, CleanupResult, CreateRefreshToken,
    Entity as RefreshTokenEntity, RefreshTokenStats, TokenRotationResult,
};
use chrono::Utc;
use sea_orm::entity::*;
use sea_orm::{query::*, DbConn, DbErr, DeleteResult, Set};
use sea_orm::{Condition, Order, QueryFilter, QueryOrder};
use uuid::Uuid;

pub struct RefreshTokenRepository {
    db: DbConn,
    schema: Option<String>,
}

#[allow(dead_code)]
impl RefreshTokenRepository {
    pub fn new(db: DbConn) -> Self {
        Self { db, schema: None }
    }

    pub fn with_schema(db: DbConn, schema: String) -> Self {
        Self {
            db,
            schema: Some(schema),
        }
    }

    // スキーマを設定するヘルパーメソッド
    async fn prepare_connection(&self) -> Result<(), DbErr> {
        if let Some(schema) = &self.schema {
            db::set_schema(&self.db, schema).await?;
        }
        Ok(())
    }

    // --- 基本CRUD操作 ---

    /// リフレッシュトークンをIDで検索
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<refresh_token_model::Model>, DbErr> {
        self.prepare_connection().await?;
        RefreshTokenEntity::find_by_id(id).one(&self.db).await
    }

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

    /// ユーザーの有効なリフレッシュトークンを取得
    pub async fn find_active_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<refresh_token_model::Model>, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();

        RefreshTokenEntity::find()
            .filter(
                Condition::all()
                    .add(refresh_token_model::Column::UserId.eq(user_id))
                    .add(refresh_token_model::Column::IsRevoked.eq(false))
                    .add(refresh_token_model::Column::ExpiresAt.gt(now)),
            )
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

    /// リフレッシュトークンを無効化
    pub async fn revoke_token(
        &self,
        id: Uuid,
    ) -> Result<Option<refresh_token_model::Model>, DbErr> {
        self.prepare_connection().await?;

        let token = match RefreshTokenEntity::find_by_id(id).one(&self.db).await? {
            Some(t) => t,
            None => return Ok(None),
        };

        let mut active_model: RefreshTokenActiveModel = token.into();
        active_model.is_revoked = Set(true);

        Ok(Some(active_model.update(&self.db).await?))
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

    /// リフレッシュトークンを削除
    pub async fn delete(&self, id: Uuid) -> Result<DeleteResult, DbErr> {
        self.prepare_connection().await?;
        RefreshTokenEntity::delete_by_id(id).exec(&self.db).await
    }

    // --- トークンローテーション ---

    /// トークンローテーション（古いトークンを無効化し、新しいトークンを作成）
    pub async fn rotate_token(
        &self,
        old_token_hash: &str,
        new_token: CreateRefreshToken,
    ) -> Result<Option<TokenRotationResult>, DbErr> {
        self.prepare_connection().await?;

        // トランザクション開始
        let txn = self.db.begin().await?;

        // 古いトークンを無効化
        let old_token_revoked = if let Some(old_token) = RefreshTokenEntity::find()
            .filter(refresh_token_model::Column::TokenHash.eq(old_token_hash))
            .one(&txn)
            .await?
        {
            let mut active_model: RefreshTokenActiveModel = old_token.into();
            active_model.is_revoked = Set(true);
            active_model.update(&txn).await?;
            true
        } else {
            false
        };

        // 新しいトークンを作成
        let new_token_model = RefreshTokenActiveModel {
            user_id: Set(new_token.user_id),
            token_hash: Set(new_token.token_hash),
            expires_at: Set(new_token.expires_at),
            ..RefreshTokenActiveModel::new()
        };

        let created_token = new_token_model.insert(&txn).await?;
        let new_token_created = created_token.id != Uuid::nil();

        // トランザクションコミット
        txn.commit().await?;

        Ok(Some(TokenRotationResult {
            old_token_revoked,
            new_token_created,
        }))
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

    /// 古いトークンを削除（指定日数より古い）
    pub async fn cleanup_old_tokens(&self, days_old: i64) -> Result<CleanupResult, DbErr> {
        self.prepare_connection().await?;
        let cutoff_date = Utc::now() - chrono::Duration::days(days_old);

        let delete_result = RefreshTokenEntity::delete_many()
            .filter(refresh_token_model::Column::CreatedAt.lt(cutoff_date))
            .exec(&self.db)
            .await?;

        Ok(CleanupResult {
            deleted_count: delete_result.rows_affected,
        })
    }

    /// ユーザーのトークン数制限を強制（古いトークンから削除）
    pub async fn enforce_user_token_limit(
        &self,
        user_id: Uuid,
        max_tokens: u32,
    ) -> Result<CleanupResult, DbErr> {
        self.prepare_connection().await?;

        let all_tokens = RefreshTokenEntity::find()
            .filter(refresh_token_model::Column::UserId.eq(user_id))
            .order_by(refresh_token_model::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await?;

        if all_tokens.len() <= max_tokens as usize {
            return Ok(CleanupResult { deleted_count: 0 });
        }

        // 制限を超えた古いトークンを削除
        let tokens_to_delete = &all_tokens[max_tokens as usize..];
        let mut deleted_count = 0;

        for token in tokens_to_delete {
            RefreshTokenEntity::delete_by_id(token.id)
                .exec(&self.db)
                .await?;
            deleted_count += 1;
        }

        Ok(CleanupResult { deleted_count })
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

    /// ユーザー別のアクティブトークン数を取得
    pub async fn get_user_active_token_count(&self, user_id: Uuid) -> Result<u64, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();

        RefreshTokenEntity::find()
            .filter(
                Condition::all()
                    .add(refresh_token_model::Column::UserId.eq(user_id))
                    .add(refresh_token_model::Column::IsRevoked.eq(false))
                    .add(refresh_token_model::Column::ExpiresAt.gt(now)),
            )
            .count(&self.db)
            .await
    }
}

// --- DTOと関連構造体 ---
