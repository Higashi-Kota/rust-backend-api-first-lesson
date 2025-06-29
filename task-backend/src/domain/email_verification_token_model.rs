// task-backend/src/domain/email_verification_token_model.rs

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "email_verification_tokens")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub is_used: bool,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::UserId",
        to = "crate::domain::user_model::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<crate::domain::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// メール認証トークンの検証エラー
#[derive(Error, Debug)]
pub enum TokenValidationError {
    #[error("Token not found")]
    NotFound,
    #[error("Token has expired")]
    Expired,
    #[error("Token has already been used")]
    AlreadyUsed,
    #[error("Token validation failed: {0}")]
    ValidationFailed(String),
}

/// メール認証トークン作成用の構造体
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CreateEmailVerificationToken {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

/// メール認証トークン検証結果
#[derive(Debug, Clone)]
pub struct EmailVerificationResult {
    pub token_id: Uuid,
    pub user_id: Uuid,
    #[allow(dead_code)]
    pub used_at: DateTime<Utc>,
}

impl Model {
    /// トークンが期限切れかどうかを確認
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// トークンが使用済みかどうかを確認
    pub fn is_used(&self) -> bool {
        self.is_used
    }

    /// トークンが有効かどうかを確認
    pub fn is_valid(&self) -> Result<(), TokenValidationError> {
        if self.is_used() {
            return Err(TokenValidationError::AlreadyUsed);
        }
        if self.is_expired() {
            return Err(TokenValidationError::Expired);
        }
        Ok(())
    }
}

/// メール認証トークンの情報（レスポンス用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationTokenInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub is_used: bool,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

impl From<Model> for EmailVerificationTokenInfo {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            expires_at: model.expires_at,
            is_used: model.is_used,
            created_at: model.created_at,
            used_at: model.used_at,
        }
    }
}
