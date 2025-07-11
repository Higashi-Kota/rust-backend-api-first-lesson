// src/repository/refresh_token_repository.rs

use crate::db;
use crate::features::auth::models::refresh_token::{
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
    ) -> Result<Option<crate::features::auth::models::refresh_token::Model>, DbErr> {
        self.prepare_connection().await?;
        RefreshTokenEntity::find()
            .filter(crate::features::auth::models::refresh_token::Column::TokenHash.eq(token_hash))
            .one(&self.db)
            .await
    }

    /// 有効なリフレッシュトークンをトークンハッシュで検索
    pub async fn find_valid_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<crate::features::auth::models::refresh_token::Model>, DbErr> {
        self.prepare_connection().await?;
        let now = Utc::now();

        RefreshTokenEntity::find()
            .filter(
                Condition::all()
                    .add(
                        crate::features::auth::models::refresh_token::Column::TokenHash
                            .eq(token_hash),
                    )
                    .add(crate::features::auth::models::refresh_token::Column::IsRevoked.eq(false))
                    .add(crate::features::auth::models::refresh_token::Column::ExpiresAt.gt(now)),
            )
            .one(&self.db)
            .await
    }

    /// ユーザーの全リフレッシュトークンを取得
    pub async fn find_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::features::auth::models::refresh_token::Model>, DbErr> {
        self.prepare_connection().await?;
        RefreshTokenEntity::find()
            .filter(crate::features::auth::models::refresh_token::Column::UserId.eq(user_id))
            .order_by(
                crate::features::auth::models::refresh_token::Column::CreatedAt,
                Order::Desc,
            )
            .all(&self.db)
            .await
    }

    /// リフレッシュトークンを作成
    pub async fn create(
        &self,
        create_token: CreateRefreshToken,
    ) -> Result<crate::features::auth::models::refresh_token::Model, DbErr> {
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
            .filter(crate::features::auth::models::refresh_token::Column::ExpiresAt.lt(now))
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
            .filter(crate::features::auth::models::refresh_token::Column::IsRevoked.eq(true))
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
                    .add(crate::features::auth::models::refresh_token::Column::IsRevoked.eq(false))
                    .add(crate::features::auth::models::refresh_token::Column::ExpiresAt.gt(now)),
            )
            .count(&self.db)
            .await?;

        let expired_tokens = RefreshTokenEntity::find()
            .filter(crate::features::auth::models::refresh_token::Column::ExpiresAt.lte(now))
            .count(&self.db)
            .await?;

        let revoked_tokens = RefreshTokenEntity::find()
            .filter(crate::features::auth::models::refresh_token::Column::IsRevoked.eq(true))
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
            .filter(crate::features::auth::models::refresh_token::Column::IsRevoked.eq(false))
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
                    .add(crate::features::auth::models::refresh_token::Column::IsRevoked.eq(false))
                    .add(
                        crate::features::auth::models::refresh_token::Column::UserId
                            .ne(exclude_user_id),
                    ),
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
                    .add(crate::features::auth::models::refresh_token::Column::IsRevoked.eq(false))
                    .add(crate::features::auth::models::refresh_token::Column::ExpiresAt.gt(now)),
            )
            .order_by(
                crate::features::auth::models::refresh_token::Column::CreatedAt,
                Order::Asc,
            )
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
                    .add(crate::features::auth::models::refresh_token::Column::IsRevoked.eq(false))
                    .add(crate::features::auth::models::refresh_token::Column::ExpiresAt.gt(now)),
            )
            .order_by(
                crate::features::auth::models::refresh_token::Column::CreatedAt,
                Order::Desc,
            )
            .one(&self.db)
            .await?;

        Ok(newest_token.map(|token| (now - token.created_at).num_hours()))
    }

    /// 平均セッション継続時間（分）を計算
    pub async fn get_average_session_duration_minutes(&self) -> Result<f64, DbErr> {
        self.prepare_connection().await?;

        let tokens = RefreshTokenEntity::find()
            .filter(crate::features::auth::models::refresh_token::Column::LastUsedAt.is_not_null())
            .all(&self.db)
            .await?;

        if tokens.is_empty() {
            return Ok(0.0);
        }

        let total_duration_minutes: i64 = tokens
            .iter()
            .filter_map(|token| {
                token
                    .last_used_at
                    .map(|last_used| (last_used - token.created_at).num_minutes())
            })
            .sum();

        Ok(total_duration_minutes as f64 / tokens.len() as f64)
    }

    /// 地理情報別のセッション分布を取得
    pub async fn get_geographic_distribution(&self) -> Result<Vec<(String, u64, u64)>, DbErr> {
        self.prepare_connection().await?;

        use sea_orm::sea_query::Expr;
        use sea_orm::QuerySelect;

        // 国別にグループ化してカウント
        let result = RefreshTokenEntity::find()
            .filter(
                Condition::all()
                    .add(crate::features::auth::models::refresh_token::Column::IsRevoked.eq(false))
                    .add(
                        crate::features::auth::models::refresh_token::Column::GeolocationCountry
                            .is_not_null(),
                    ),
            )
            .select_only()
            .column(crate::features::auth::models::refresh_token::Column::GeolocationCountry)
            .column_as(
                Expr::col(crate::features::auth::models::refresh_token::Column::Id).count(),
                "session_count",
            )
            .column_as(
                Expr::col(crate::features::auth::models::refresh_token::Column::UserId)
                    .count_distinct(),
                "unique_users",
            )
            .group_by(crate::features::auth::models::refresh_token::Column::GeolocationCountry)
            .into_tuple::<(Option<String>, i64, i64)>()
            .all(&self.db)
            .await?;

        Ok(result
            .into_iter()
            .filter_map(|(country, count, users)| country.map(|c| (c, count as u64, users as u64)))
            .collect())
    }

    /// デバイスタイプ別のセッション分布を取得
    pub async fn get_device_distribution(&self) -> Result<Vec<(String, u64, u64)>, DbErr> {
        self.prepare_connection().await?;

        use sea_orm::sea_query::Expr;
        use sea_orm::QuerySelect;

        // デバイスタイプ別にグループ化してカウント
        let result = RefreshTokenEntity::find()
            .filter(
                Condition::all()
                    .add(crate::features::auth::models::refresh_token::Column::IsRevoked.eq(false))
                    .add(
                        crate::features::auth::models::refresh_token::Column::DeviceType
                            .is_not_null(),
                    ),
            )
            .select_only()
            .column(crate::features::auth::models::refresh_token::Column::DeviceType)
            .column_as(
                Expr::col(crate::features::auth::models::refresh_token::Column::Id).count(),
                "session_count",
            )
            .column_as(
                Expr::col(crate::features::auth::models::refresh_token::Column::UserId)
                    .count_distinct(),
                "unique_users",
            )
            .group_by(crate::features::auth::models::refresh_token::Column::DeviceType)
            .into_tuple::<(Option<String>, i64, i64)>()
            .all(&self.db)
            .await?;

        Ok(result
            .into_iter()
            .filter_map(|(device, count, users)| device.map(|d| (d, count as u64, users as u64)))
            .collect())
    }

    /// ピーク時の同時セッション数を取得
    pub async fn get_peak_concurrent_sessions(&self, hours: i64) -> Result<u64, DbErr> {
        self.prepare_connection().await?;
        let since = Utc::now() - chrono::Duration::hours(hours);

        let count = RefreshTokenEntity::find()
            .filter(
                Condition::all()
                    .add(crate::features::auth::models::refresh_token::Column::IsRevoked.eq(false))
                    .add(
                        crate::features::auth::models::refresh_token::Column::LastUsedAt.gte(since),
                    ),
            )
            .count(&self.db)
            .await?;

        Ok(count)
    }
}

// --- DTOと関連構造体 ---
