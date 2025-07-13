// src/repository/password_reset_token_repository.rs

use crate::db;
use crate::features::auth::models::password_reset_token::{
    ActiveModel as PasswordResetTokenActiveModel, CleanupResult,
    Entity as PasswordResetTokenEntity, PasswordResetTokenStats,
};
use chrono::Utc;
use sea_orm::entity::*;
use sea_orm::{query::*, DbConn, DbErr, Set};
use sea_orm::{Condition, Order, QueryFilter, QueryOrder};
use uuid::Uuid;

pub struct PasswordResetTokenRepository {
    db: DbConn,
    schema: Option<String>,
}

impl PasswordResetTokenRepository {
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

    /// パスワードリセットトークンをトークンハッシュで検索
    pub async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<crate::features::auth::models::password_reset_token::Model>, DbErr> {
        self.prepare_connection().await?;
        PasswordResetTokenEntity::find()
            .filter(
                crate::features::auth::models::password_reset_token::Column::TokenHash
                    .eq(token_hash),
            )
            .one(&self.db)
            .await
    }

    // --- 高レベル操作（Phase 1.1/1.2で必要な機能のみ）---

    /// 簡易パスワードリセット要求
    pub async fn create_reset_request(
        &self,
        user_id: Uuid,
        token_hash: String,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<bool, DbErr> {
        self.prepare_connection().await?;

        let new_token = PasswordResetTokenActiveModel {
            user_id: Set(user_id),
            token_hash: Set(token_hash),
            expires_at: Set(expires_at),
            ..PasswordResetTokenActiveModel::new()
        };

        new_token.insert(&self.db).await?;
        Ok(true)
    }

    /// 簡易パスワードリセット実行（user_idも返す）
    pub async fn execute_password_reset(
        &self,
        token_hash: &str,
    ) -> Result<Result<Uuid, String>, DbErr> {
        self.prepare_connection().await?;

        if let Some(token) = self.find_by_token_hash(token_hash).await? {
            if !token.is_used && token.expires_at > Utc::now() {
                let user_id = token.user_id;
                let mut active_model: PasswordResetTokenActiveModel = token.into();
                active_model.is_used = Set(true);
                active_model.update(&self.db).await?;
                Ok(Ok(user_id))
            } else {
                Ok(Err("Token expired or already used".to_string()))
            }
        } else {
            Ok(Err("Token not found".to_string()))
        }
    }

    // --- クリーンアップ機能 ---

    /// 期限切れのトークンを削除
    pub async fn cleanup_expired_tokens(&self) -> Result<CleanupResult, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();

        let delete_result = PasswordResetTokenEntity::delete_many()
            .filter(crate::features::auth::models::password_reset_token::Column::ExpiresAt.lt(now))
            .exec(&self.db)
            .await?;

        Ok(CleanupResult {
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
                    .add(
                        crate::features::auth::models::password_reset_token::Column::ExpiresAt
                            .lt(now),
                    )
                    .add(
                        crate::features::auth::models::password_reset_token::Column::IsUsed
                            .eq(true),
                    )
                    .add(
                        crate::features::auth::models::password_reset_token::Column::CreatedAt
                            .lt(cutoff_date),
                    ),
            )
            .exec(&self.db)
            .await?;

        Ok(CleanupResult {
            total_deleted: delete_result.rows_affected,
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
                    .add(
                        crate::features::auth::models::password_reset_token::Column::IsUsed
                            .eq(false),
                    )
                    .add(
                        crate::features::auth::models::password_reset_token::Column::ExpiresAt
                            .gt(now),
                    ),
            )
            .count(&self.db)
            .await?;

        let expired_tokens = PasswordResetTokenEntity::find()
            .filter(crate::features::auth::models::password_reset_token::Column::ExpiresAt.lte(now))
            .count(&self.db)
            .await?;

        let used_tokens = PasswordResetTokenEntity::find()
            .filter(crate::features::auth::models::password_reset_token::Column::IsUsed.eq(true))
            .count(&self.db)
            .await?;

        Ok(PasswordResetTokenStats {
            total_tokens,
            active_tokens,
            expired_tokens,
            used_tokens,
        })
    }

    /// 最近のパスワードリセット活動を取得
    pub async fn get_recent_reset_activity(
        &self,
        hours: i64,
    ) -> Result<Vec<crate::features::auth::models::password_reset_token::Model>, DbErr> {
        self.prepare_connection().await?;
        let since = Utc::now() - chrono::Duration::hours(hours);

        PasswordResetTokenEntity::find()
            .filter(
                crate::features::auth::models::password_reset_token::Column::CreatedAt.gt(since),
            )
            .order_by(
                crate::features::auth::models::password_reset_token::Column::CreatedAt,
                Order::Desc,
            )
            .all(&self.db)
            .await
    }

    /// 特定のユーザーの最近のリセット要求数を取得
    pub async fn count_recent_requests_by_user(
        &self,
        user_id: Uuid,
        minutes: i64,
    ) -> Result<u64, DbErr> {
        self.prepare_connection().await?;
        let since = Utc::now() - chrono::Duration::minutes(minutes);

        PasswordResetTokenEntity::find()
            .filter(crate::features::auth::models::password_reset_token::Column::UserId.eq(user_id))
            .filter(
                crate::features::auth::models::password_reset_token::Column::CreatedAt.gt(since),
            )
            .count(&self.db)
            .await
    }

    /// 今日のパスワードリセット要求数を取得
    pub async fn count_requests_today(&self) -> Result<u64, DbErr> {
        self.prepare_connection().await?;
        let today = Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();

        PasswordResetTokenEntity::find()
            .filter(
                crate::features::auth::models::password_reset_token::Column::CreatedAt.gte(today),
            )
            .count(&self.db)
            .await
    }

    /// 今週のパスワードリセット要求数を取得
    pub async fn count_requests_this_week(&self) -> Result<u64, DbErr> {
        self.prepare_connection().await?;
        let this_week = Utc::now() - chrono::Duration::days(7);

        PasswordResetTokenEntity::find()
            .filter(
                crate::features::auth::models::password_reset_token::Column::CreatedAt
                    .gte(this_week),
            )
            .count(&self.db)
            .await
    }

    /// 使用されたトークンの平均使用時間（分）を計算
    pub async fn get_average_usage_time_minutes(&self) -> Result<Option<f64>, DbErr> {
        self.prepare_connection().await?;

        // 使用されたトークンを取得
        let used_tokens = PasswordResetTokenEntity::find()
            .filter(crate::features::auth::models::password_reset_token::Column::IsUsed.eq(true))
            .all(&self.db)
            .await?;

        if used_tokens.is_empty() {
            return Ok(None);
        }

        // 各トークンの使用時間を計算（作成時刻から更新時刻までの差）
        let total_minutes: f64 = used_tokens
            .iter()
            .map(|token| (token.updated_at - token.created_at).num_minutes() as f64)
            .sum();

        Ok(Some(total_minutes / used_tokens.len() as f64))
    }
}

// --- DTOと関連構造体 ---
