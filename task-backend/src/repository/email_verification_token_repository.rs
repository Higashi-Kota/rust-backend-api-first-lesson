// task-backend/src/repository/email_verification_token_repository.rs

use crate::db::DbPool;
use crate::domain::email_verification_token_model::{
    ActiveModel, Column, CreateEmailVerificationToken, EmailVerificationResult, Entity, Model,
    TokenValidationError,
};
use crate::error::{AppError, AppResult};
use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct EmailVerificationTokenRepository {
    db: DbPool,
}

#[allow(dead_code)]
impl EmailVerificationTokenRepository {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// メール認証トークンを作成
    pub async fn create(&self, create_token: CreateEmailVerificationToken) -> AppResult<Model> {
        let new_token = ActiveModel {
            user_id: Set(create_token.user_id),
            token_hash: Set(create_token.token_hash),
            expires_at: Set(create_token.expires_at),
            is_used: Set(false),
            created_at: Set(Utc::now()),
            used_at: Set(None),
            ..Default::default()
        };

        let token = new_token.insert(&self.db).await?;
        Ok(token)
    }

    /// ユーザーIDでトークンを検索（最新のもの）
    pub async fn find_latest_by_user_id(&self, user_id: Uuid) -> AppResult<Option<Model>> {
        let token = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::IsUsed.eq(false))
            .order_by_desc(Column::CreatedAt)
            .one(&self.db)
            .await?;

        Ok(token)
    }

    /// トークンハッシュでトークンを検索
    pub async fn find_by_token_hash(&self, token_hash: &str) -> AppResult<Option<Model>> {
        let token = Entity::find()
            .filter(Column::TokenHash.eq(token_hash))
            .one(&self.db)
            .await?;

        Ok(token)
    }

    /// 有効なトークンを検索（期限切れでなく、未使用）
    pub async fn find_valid_by_token_hash(&self, token_hash: &str) -> AppResult<Option<Model>> {
        let now = Utc::now();
        let token = Entity::find()
            .filter(Column::TokenHash.eq(token_hash))
            .filter(Column::IsUsed.eq(false))
            .filter(Column::ExpiresAt.gt(now))
            .one(&self.db)
            .await?;

        Ok(token)
    }

    /// メール認証を実行（トランザクション内で実行）
    pub async fn execute_email_verification(
        &self,
        token_hash: &str,
    ) -> AppResult<Result<EmailVerificationResult, TokenValidationError>> {
        let db = &self.db;
        let token_hash = token_hash.to_string();

        let result = db
            .transaction::<_, EmailVerificationResult, DbErr>(|txn| {
                Box::pin(async move {
                    // トークンを検索
                    let token = Entity::find()
                        .filter(Column::TokenHash.eq(&token_hash))
                        .one(txn)
                        .await?;

                    let token = match token {
                        Some(token) => token,
                        None => return Err(DbErr::Custom("Token not found".to_string())),
                    };

                    // トークンの有効性を確認
                    if let Err(validation_error) = token.is_valid() {
                        let error_msg = match validation_error {
                            TokenValidationError::Expired => "Token has expired",
                            TokenValidationError::AlreadyUsed => "Token has already been used",
                            _ => "Token validation failed",
                        };
                        return Err(DbErr::Custom(error_msg.to_string()));
                    }

                    // トークンを使用済みに更新
                    let used_at = Utc::now();
                    let mut active_token: ActiveModel = token.clone().into();
                    active_token.is_used = Set(true);
                    active_token.used_at = Set(Some(used_at));
                    active_token.update(txn).await?;

                    Ok(EmailVerificationResult {
                        token_id: token.id,
                        user_id: token.user_id,
                        used_at,
                    })
                })
            })
            .await;

        match result {
            Ok(verification_result) => Ok(Ok(verification_result)),
            Err(db_err) => {
                let error_msg = db_err.to_string();
                if error_msg.contains("Token not found") {
                    Ok(Err(TokenValidationError::NotFound))
                } else if error_msg.contains("Token has expired") {
                    Ok(Err(TokenValidationError::Expired))
                } else if error_msg.contains("Token has already been used") {
                    Ok(Err(TokenValidationError::AlreadyUsed))
                } else {
                    Ok(Err(TokenValidationError::ValidationFailed(error_msg)))
                }
            }
        }
    }

    /// 新しいトークンを作成し、古いトークンを無効化
    pub async fn create_verification_request(
        &self,
        user_id: Uuid,
        token_hash: String,
        expires_at: chrono::DateTime<Utc>,
    ) -> AppResult<CreateVerificationRequestResult> {
        let db = &self.db;

        let result = db
            .transaction::<_, CreateVerificationRequestResult, DbErr>(|txn| {
                Box::pin(async move {
                    // 既存の未使用トークンを無効化
                    let existing_tokens = Entity::find()
                        .filter(Column::UserId.eq(user_id))
                        .filter(Column::IsUsed.eq(false))
                        .all(txn)
                        .await?;

                    let mut invalidated_count = 0;
                    for token in existing_tokens {
                        let mut active_token: ActiveModel = token.into();
                        active_token.is_used = Set(true);
                        active_token.used_at = Set(Some(Utc::now()));
                        active_token.update(txn).await?;
                        invalidated_count += 1;
                    }

                    // 新しいトークンを作成
                    let new_token = ActiveModel {
                        user_id: Set(user_id),
                        token_hash: Set(token_hash),
                        expires_at: Set(expires_at),
                        is_used: Set(false),
                        created_at: Set(Utc::now()),
                        used_at: Set(None),
                        ..Default::default()
                    };

                    let created_token = new_token.insert(txn).await?;

                    Ok(CreateVerificationRequestResult {
                        token_id: created_token.id,
                        old_tokens_invalidated: invalidated_count,
                    })
                })
            })
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(result)
    }

    /// 期限切れトークンの削除
    pub async fn cleanup_expired_tokens(&self, before_hours: u32) -> AppResult<CleanupResult> {
        let cutoff_time = Utc::now() - Duration::hours(before_hours as i64);

        let expired_tokens = Entity::find()
            .filter(Column::ExpiresAt.lt(cutoff_time))
            .all(&self.db)
            .await?;

        let deleted_count = expired_tokens.len();

        // 期限切れトークンを削除
        Entity::delete_many()
            .filter(Column::ExpiresAt.lt(cutoff_time))
            .exec(&self.db)
            .await?;

        Ok(CleanupResult {
            deleted_count: deleted_count as u32,
        })
    }

    /// すべてのトークンを削除（テスト用）
    pub async fn cleanup_all(&self, _keep_latest: u32) -> AppResult<CleanupResult> {
        let all_tokens = Entity::find().all(&self.db).await?;
        let deleted_count = all_tokens.len();

        Entity::delete_many().exec(&self.db).await?;

        Ok(CleanupResult {
            deleted_count: deleted_count as u32,
        })
    }

    /// ユーザーのトークン履歴を取得
    pub async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Vec<Model>> {
        let tokens = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::CreatedAt)
            .all(&self.db)
            .await?;

        Ok(tokens)
    }
}

/// メール認証リクエスト作成結果
#[derive(Debug, Clone)]
pub struct CreateVerificationRequestResult {
    pub token_id: Uuid,
    pub old_tokens_invalidated: u32,
}

/// クリーンアップ結果
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CleanupResult {
    pub deleted_count: u32,
}
