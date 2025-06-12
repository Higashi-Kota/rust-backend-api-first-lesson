// src/domain/refresh_token_model.rs
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait, DbErr, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "refresh_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    pub user_id: Uuid,

    #[serde(skip_serializing)] // トークンハッシュは絶対にシリアライズしない
    pub token_hash: String,

    pub expires_at: DateTime<Utc>,

    pub is_revoked: bool,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
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

// リレーション実装
impl Related<crate::domain::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            is_revoked: Set(false), // デフォルトで有効
            ..ActiveModelTrait::default()
        }
    }

    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert {
            // 更新の場合のみ updated_at を更新
            self.updated_at = Set(Utc::now());
        }
        Ok(self)
    }
}

// リフレッシュトークン用の便利メソッド実装
impl Model {
    /// トークンが有効かどうかをチェック
    pub fn is_valid(&self) -> bool {
        !self.is_revoked && self.expires_at > Utc::now()
    }

    /// トークンが期限切れかどうかをチェック
    pub fn is_expired(&self) -> bool {
        self.expires_at <= Utc::now()
    }

    /// トークンが無効化されているかどうかをチェック
    pub fn is_revoked(&self) -> bool {
        self.is_revoked
    }

    /// トークンの有効期限までの残り時間を取得（秒）
    pub fn time_to_expiry(&self) -> Option<i64> {
        if self.is_expired() {
            None
        } else {
            Some((self.expires_at - Utc::now()).num_seconds())
        }
    }

    /// トークンが指定された時間内に期限切れになるかチェック
    pub fn expires_within(&self, duration: chrono::Duration) -> bool {
        self.expires_at <= Utc::now() + duration
    }
}

/// リフレッシュトークンの作成用構造体
#[derive(Debug, Clone)]
pub struct CreateRefreshToken {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

impl From<CreateRefreshToken> for ActiveModel {
    fn from(create_token: CreateRefreshToken) -> Self {
        Self {
            user_id: Set(create_token.user_id),
            token_hash: Set(create_token.token_hash),
            expires_at: Set(create_token.expires_at),
            ..Self::new()
        }
    }
}

/// セキュリティ上安全なリフレッシュトークン情報（ハッシュ値を除く）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SafeRefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub is_revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Model> for SafeRefreshToken {
    fn from(token: Model) -> Self {
        Self {
            id: token.id,
            user_id: token.user_id,
            expires_at: token.expires_at,
            is_revoked: token.is_revoked,
            created_at: token.created_at,
            updated_at: token.updated_at,
        }
    }
}

/// リフレッシュトークンの統計情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenStats {
    pub total_tokens: u64,
    pub active_tokens: u64,
    pub expired_tokens: u64,
    pub revoked_tokens: u64,
}

/// リフレッシュトークンのクリーンアップ結果
#[derive(Debug, Clone)]
pub struct CleanupResult {
    pub deleted_count: u64,
}

/// トークンローテーション用の結果
#[derive(Debug, Clone)]
pub struct TokenRotationResult {
    pub old_token_revoked: bool,
    pub new_token_created: bool,
}

/// リフレッシュトークンの設定
#[derive(Debug, Clone)]
pub struct RefreshTokenConfig {
    /// トークンの有効期間（デフォルト7日）
    pub validity_duration: chrono::Duration,
    /// 自動クリーンアップの閾値（デフォルト30日）
    pub cleanup_threshold: chrono::Duration,
    /// ユーザーあたりの最大トークン数（デフォルト5）
    pub max_tokens_per_user: u32,
}

impl Default for RefreshTokenConfig {
    fn default() -> Self {
        Self {
            validity_duration: chrono::Duration::days(7),
            cleanup_threshold: chrono::Duration::days(30),
            max_tokens_per_user: 5,
        }
    }
}
