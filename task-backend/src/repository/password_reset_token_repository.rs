// src/repository/password_reset_token_repository.rs

use crate::db;
use crate::domain::password_reset_token_model::{
    self, ActiveModel as PasswordResetTokenActiveModel, CleanupResult,
    Entity as PasswordResetTokenEntity, PasswordResetRequestResult, PasswordResetResult,
    PasswordResetTokenStats, TokenValidationError,
};
use chrono::{DateTime, Utc};
use sea_orm::entity::*;
use sea_orm::{query::*, DbConn, DbErr, DeleteResult, Set};
use sea_orm::{Condition, Order, QueryFilter, QueryOrder};
use uuid::Uuid;

pub struct PasswordResetTokenRepository {
    db: DbConn,
    schema: Option<String>,
}

#[allow(dead_code)]
impl PasswordResetTokenRepository {
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

    /// パスワードリセットトークンをIDで検索
    pub async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<password_reset_token_model::Model>, DbErr> {
        self.prepare_connection().await?;
        PasswordResetTokenEntity::find_by_id(id).one(&self.db).await
    }

    /// パスワードリセットトークンをトークンハッシュで検索
    pub async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<password_reset_token_model::Model>, DbErr> {
        self.prepare_connection().await?;
        PasswordResetTokenEntity::find()
            .filter(password_reset_token_model::Column::TokenHash.eq(token_hash))
            .one(&self.db)
            .await
    }

    /// 有効なパスワードリセットトークンをトークンハッシュで検索（未使用かつ未期限切れ）
    pub async fn find_valid_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<password_reset_token_model::Model>, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();

        PasswordResetTokenEntity::find()
            .filter(
                Condition::all()
                    .add(password_reset_token_model::Column::TokenHash.eq(token_hash))
                    .add(password_reset_token_model::Column::IsUsed.eq(false))
                    .add(password_reset_token_model::Column::ExpiresAt.gt(now)),
            )
            .one(&self.db)
            .await
    }

    /// ユーザーの全パスワードリセットトークンを取得
    pub async fn find_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<password_reset_token_model::Model>, DbErr> {
        self.prepare_connection().await?;
        PasswordResetTokenEntity::find()
            .filter(password_reset_token_model::Column::UserId.eq(user_id))
            .order_by(password_reset_token_model::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await
    }

    /// ユーザーの有効なパスワードリセットトークンを取得
    pub async fn find_active_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<password_reset_token_model::Model>, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();

        PasswordResetTokenEntity::find()
            .filter(
                Condition::all()
                    .add(password_reset_token_model::Column::UserId.eq(user_id))
                    .add(password_reset_token_model::Column::IsUsed.eq(false))
                    .add(password_reset_token_model::Column::ExpiresAt.gt(now)),
            )
            .order_by(password_reset_token_model::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await
    }

    /// パスワードリセットトークンを作成
    pub async fn create(
        &self,
        create_token: CreatePasswordResetToken,
    ) -> Result<password_reset_token_model::Model, DbErr> {
        self.prepare_connection().await?;

        let new_token = PasswordResetTokenActiveModel {
            user_id: Set(create_token.user_id),
            token_hash: Set(create_token.token_hash),
            expires_at: Set(create_token.expires_at),
            ..Default::default()
        };

        new_token.insert(&self.db).await
    }

    /// パスワードリセットトークンを使用済みにマーク
    pub async fn mark_as_used(
        &self,
        id: Uuid,
    ) -> Result<Option<password_reset_token_model::Model>, DbErr> {
        self.prepare_connection().await?;

        let token = match PasswordResetTokenEntity::find_by_id(id)
            .one(&self.db)
            .await?
        {
            Some(t) => t,
            None => return Ok(None),
        };

        let mut active_model: PasswordResetTokenActiveModel = token.into();
        active_model.is_used = Set(true);

        Ok(Some(active_model.update(&self.db).await?))
    }

    /// トークンハッシュで使用済みにマーク
    pub async fn mark_as_used_by_token_hash(&self, token_hash: &str) -> Result<bool, DbErr> {
        self.prepare_connection().await?;

        let token = match self.find_by_token_hash(token_hash).await? {
            Some(t) => t,
            None => return Ok(false),
        };

        let mut active_model: PasswordResetTokenActiveModel = token.into();
        active_model.is_used = Set(true);
        active_model.update(&self.db).await?;

        Ok(true)
    }

    /// ユーザーの全パスワードリセットトークンを使用済みにマーク
    pub async fn mark_all_user_tokens_as_used(&self, user_id: Uuid) -> Result<u64, DbErr> {
        self.prepare_connection().await?;

        let tokens = self.find_by_user_id(user_id).await?;
        let mut marked_count = 0;

        for token in tokens {
            if !token.is_used {
                let mut active_model: PasswordResetTokenActiveModel = token.into();
                active_model.is_used = Set(true);
                active_model.update(&self.db).await?;
                marked_count += 1;
            }
        }

        Ok(marked_count)
    }

    /// パスワードリセットトークンを削除
    pub async fn delete(&self, id: Uuid) -> Result<DeleteResult, DbErr> {
        self.prepare_connection().await?;
        PasswordResetTokenEntity::delete_by_id(id)
            .exec(&self.db)
            .await
    }

    // --- 高レベル操作 ---

    /// パスワードリセット要求（新しいトークンを作成し、古いトークンを無効化）
    pub async fn create_reset_request(
        &self,
        user_id: Uuid,
        token_hash: String,
        expires_at: DateTime<Utc>,
    ) -> Result<PasswordResetRequestResult, DbErr> {
        self.prepare_connection().await?;

        // トランザクション開始
        let txn = self.db.begin().await?;

        // ユーザーの既存の有効なトークンを無効化
        let existing_tokens = PasswordResetTokenEntity::find()
            .filter(
                Condition::all()
                    .add(password_reset_token_model::Column::UserId.eq(user_id))
                    .add(password_reset_token_model::Column::IsUsed.eq(false))
                    .add(password_reset_token_model::Column::ExpiresAt.gt(Utc::now())),
            )
            .all(&txn)
            .await?;

        let mut old_tokens_invalidated = 0;
        for token in existing_tokens {
            let mut active_model: PasswordResetTokenActiveModel = token.into();
            active_model.is_used = Set(true);
            active_model.update(&txn).await?;
            old_tokens_invalidated += 1;
        }

        // 新しいトークンを作成
        let new_token = PasswordResetTokenActiveModel {
            user_id: Set(user_id),
            token_hash: Set(token_hash),
            expires_at: Set(expires_at),
            ..PasswordResetTokenActiveModel::new()
        };

        let created_token = new_token.insert(&txn).await?;

        // トランザクションコミット
        txn.commit().await?;

        Ok(PasswordResetRequestResult {
            token_id: created_token.id,
            user_id,
            token_created: true,
            old_tokens_invalidated,
        })
    }

    /// パスワードリセット実行（トークンを検証し、使用済みにマーク）
    pub async fn execute_password_reset(
        &self,
        token_hash: &str,
    ) -> Result<Result<PasswordResetResult, TokenValidationError>, DbErr> {
        self.prepare_connection().await?;

        let token = match self.find_by_token_hash(token_hash).await? {
            Some(t) => t,
            None => return Ok(Err(TokenValidationError::NotFound)),
        };

        // トークンの有効性をチェック
        if let Err(validation_error) = token.can_be_used() {
            return Ok(Err(validation_error));
        }

        // トークンを使用済みにマーク
        let mut active_model: PasswordResetTokenActiveModel = token.clone().into();
        active_model.is_used = Set(true);
        active_model.update(&self.db).await?;

        Ok(Ok(PasswordResetResult {
            token_id: token.id,
            user_id: token.user_id,
            reset_successful: true,
            token_invalidated: true,
        }))
    }

    // --- クリーンアップ機能 ---

    /// 期限切れのトークンを削除
    pub async fn cleanup_expired_tokens(&self) -> Result<CleanupResult, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();

        let delete_result = PasswordResetTokenEntity::delete_many()
            .filter(password_reset_token_model::Column::ExpiresAt.lt(now))
            .exec(&self.db)
            .await?;

        Ok(CleanupResult {
            deleted_expired_count: delete_result.rows_affected,
            deleted_used_count: 0,
            total_deleted: delete_result.rows_affected,
        })
    }

    /// 使用済みのトークンを削除
    pub async fn cleanup_used_tokens(&self) -> Result<CleanupResult, DbErr> {
        self.prepare_connection().await?;

        let delete_result = PasswordResetTokenEntity::delete_many()
            .filter(password_reset_token_model::Column::IsUsed.eq(true))
            .exec(&self.db)
            .await?;

        Ok(CleanupResult {
            deleted_expired_count: 0,
            deleted_used_count: delete_result.rows_affected,
            total_deleted: delete_result.rows_affected,
        })
    }

    /// 古いトークンを削除（指定時間より古い）
    pub async fn cleanup_old_tokens(&self, hours_old: i64) -> Result<CleanupResult, DbErr> {
        self.prepare_connection().await?;
        let cutoff_date = Utc::now() - chrono::Duration::hours(hours_old);

        let delete_result = PasswordResetTokenEntity::delete_many()
            .filter(password_reset_token_model::Column::CreatedAt.lt(cutoff_date))
            .exec(&self.db)
            .await?;

        Ok(CleanupResult {
            deleted_expired_count: 0,
            deleted_used_count: 0,
            total_deleted: delete_result.rows_affected,
        })
    }

    /// 包括的なクリーンアップ（期限切れ・使用済み・古いトークンを削除）
    pub async fn cleanup_all(&self, max_age_hours: i64) -> Result<CleanupResult, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();
        let cutoff_date = now - chrono::Duration::hours(max_age_hours);

        // 期限切れまたは使用済みまたは古いトークンを削除
        let delete_result = PasswordResetTokenEntity::delete_many()
            .filter(
                Condition::any()
                    .add(password_reset_token_model::Column::ExpiresAt.lt(now))
                    .add(password_reset_token_model::Column::IsUsed.eq(true))
                    .add(password_reset_token_model::Column::CreatedAt.lt(cutoff_date)),
            )
            .exec(&self.db)
            .await?;

        Ok(CleanupResult {
            deleted_expired_count: 0, // 正確な内訳は計算が複雑なため0に設定
            deleted_used_count: 0,
            total_deleted: delete_result.rows_affected,
        })
    }

    /// ユーザーのトークン数制限を強制（古いトークンから削除）
    pub async fn enforce_user_token_limit(
        &self,
        user_id: Uuid,
        max_tokens: u32,
    ) -> Result<CleanupResult, DbErr> {
        self.prepare_connection().await?;

        let all_tokens = PasswordResetTokenEntity::find()
            .filter(password_reset_token_model::Column::UserId.eq(user_id))
            .order_by(password_reset_token_model::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await?;

        if all_tokens.len() <= max_tokens as usize {
            return Ok(CleanupResult {
                deleted_expired_count: 0,
                deleted_used_count: 0,
                total_deleted: 0,
            });
        }

        // 制限を超えた古いトークンを削除
        let tokens_to_delete = &all_tokens[max_tokens as usize..];
        let mut deleted_count = 0;

        for token in tokens_to_delete {
            PasswordResetTokenEntity::delete_by_id(token.id)
                .exec(&self.db)
                .await?;
            deleted_count += 1;
        }

        Ok(CleanupResult {
            deleted_expired_count: 0,
            deleted_used_count: 0,
            total_deleted: deleted_count,
        })
    }

    // --- 統計情報 ---

    /// パスワードリセットトークンの統計情報を取得
    pub async fn get_token_stats(&self) -> Result<PasswordResetTokenStats, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();

        let total_tokens = PasswordResetTokenEntity::find().count(&self.db).await?;

        let active_tokens = PasswordResetTokenEntity::find()
            .filter(
                Condition::all()
                    .add(password_reset_token_model::Column::IsUsed.eq(false))
                    .add(password_reset_token_model::Column::ExpiresAt.gt(now)),
            )
            .count(&self.db)
            .await?;

        let expired_tokens = PasswordResetTokenEntity::find()
            .filter(password_reset_token_model::Column::ExpiresAt.lte(now))
            .count(&self.db)
            .await?;

        let used_tokens = PasswordResetTokenEntity::find()
            .filter(password_reset_token_model::Column::IsUsed.eq(true))
            .count(&self.db)
            .await?;

        Ok(PasswordResetTokenStats {
            total_tokens,
            active_tokens,
            expired_tokens,
            used_tokens,
        })
    }

    /// ユーザー別のアクティブトークン数を取得
    pub async fn get_user_active_token_count(&self, user_id: Uuid) -> Result<u64, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();

        PasswordResetTokenEntity::find()
            .filter(
                Condition::all()
                    .add(password_reset_token_model::Column::UserId.eq(user_id))
                    .add(password_reset_token_model::Column::IsUsed.eq(false))
                    .add(password_reset_token_model::Column::ExpiresAt.gt(now)),
            )
            .count(&self.db)
            .await
    }

    /// 最近のパスワードリセット活動を取得
    pub async fn get_recent_reset_activity(
        &self,
        hours: i64,
    ) -> Result<Vec<password_reset_token_model::Model>, DbErr> {
        self.prepare_connection().await?;
        let since = Utc::now() - chrono::Duration::hours(hours);

        PasswordResetTokenEntity::find()
            .filter(password_reset_token_model::Column::CreatedAt.gt(since))
            .order_by(password_reset_token_model::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await
    }
}

// --- DTOと関連構造体 ---

/// パスワードリセットトークン作成用構造体
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CreatePasswordResetToken {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}
