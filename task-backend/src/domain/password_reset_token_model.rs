// src/domain/password_reset_token_model.rs
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait, DbErr, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "password_reset_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    pub user_id: Uuid,

    #[serde(skip_serializing)] // トークンハッシュは絶対にシリアライズしない
    pub token_hash: String,

    pub expires_at: DateTime<Utc>,

    pub is_used: bool,

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
            is_used: Set(false), // デフォルトで未使用
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

// パスワードリセットトークン用の便利メソッド実装
impl Model {
    /// トークンが有効かどうかをチェック（未使用かつ未期限切れ）
    pub fn is_valid(&self) -> bool {
        !self.is_used && self.expires_at > Utc::now()
    }

    /// トークンが期限切れかどうかをチェック
    pub fn is_expired(&self) -> bool {
        self.expires_at <= Utc::now()
    }

    /// トークンが使用済みかどうかをチェック
    pub fn is_used(&self) -> bool {
        self.is_used
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

    /// トークンの作成からの経過時間（秒）
    pub fn age_in_seconds(&self) -> i64 {
        (Utc::now() - self.created_at).num_seconds()
    }

    /// トークンが使用可能な状態かチェック（詳細な理由付き）
    pub fn can_be_used(&self) -> Result<(), TokenValidationError> {
        if self.is_used {
            return Err(TokenValidationError::AlreadyUsed);
        }

        if self.is_expired() {
            return Err(TokenValidationError::Expired);
        }

        Ok(())
    }
}

/// パスワードリセットトークンの作成用構造体
#[derive(Debug, Clone)]
pub struct CreatePasswordResetToken {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

impl From<CreatePasswordResetToken> for ActiveModel {
    fn from(create_token: CreatePasswordResetToken) -> Self {
        Self {
            user_id: Set(create_token.user_id),
            token_hash: Set(create_token.token_hash),
            expires_at: Set(create_token.expires_at),
            ..Self::new()
        }
    }
}

/// セキュリティ上安全なパスワードリセットトークン情報（ハッシュ値を除く）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SafePasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub is_used: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Model> for SafePasswordResetToken {
    fn from(token: Model) -> Self {
        Self {
            id: token.id,
            user_id: token.user_id,
            expires_at: token.expires_at,
            is_used: token.is_used,
            created_at: token.created_at,
            updated_at: token.updated_at,
        }
    }
}

/// トークン検証エラーの種類
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenValidationError {
    /// トークンが既に使用済み
    AlreadyUsed,
    /// トークンが期限切れ
    Expired,
    /// トークンが見つからない
    NotFound,
    /// トークンのフォーマットが無効
    InvalidFormat,
}

impl std::fmt::Display for TokenValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenValidationError::AlreadyUsed => write!(f, "Token has already been used"),
            TokenValidationError::Expired => write!(f, "Token has expired"),
            TokenValidationError::NotFound => write!(f, "Token not found"),
            TokenValidationError::InvalidFormat => write!(f, "Invalid token format"),
        }
    }
}

impl std::error::Error for TokenValidationError {}

/// パスワードリセットトークンの統計情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetTokenStats {
    pub total_tokens: u64,
    pub active_tokens: u64,
    pub expired_tokens: u64,
    pub used_tokens: u64,
}

/// パスワードリセットトークンのクリーンアップ結果
#[derive(Debug, Clone)]
pub struct CleanupResult {
    pub deleted_expired_count: u64,
    pub deleted_used_count: u64,
    pub total_deleted: u64,
}

/// パスワードリセットトークンの設定
#[derive(Debug, Clone)]
pub struct PasswordResetTokenConfig {
    /// トークンの有効期間（デフォルト1時間）
    pub validity_duration: chrono::Duration,
    /// 自動クリーンアップの閾値（デフォルト24時間）
    pub cleanup_threshold: chrono::Duration,
    /// ユーザーあたりの最大アクティブトークン数（デフォルト3）
    pub max_active_tokens_per_user: u32,
    /// トークンの最小長（デフォルト32バイト）
    pub min_token_length: usize,
}

impl Default for PasswordResetTokenConfig {
    fn default() -> Self {
        Self {
            validity_duration: chrono::Duration::hours(1),
            cleanup_threshold: chrono::Duration::hours(24),
            max_active_tokens_per_user: 3,
            min_token_length: 32,
        }
    }
}

/// パスワードリセットの結果
#[derive(Debug, Clone)]
pub struct PasswordResetResult {
    pub token_id: Uuid,
    pub user_id: Uuid,
    pub reset_successful: bool,
    pub token_invalidated: bool,
}

/// パスワードリセット要求の結果
#[derive(Debug, Clone)]
pub struct PasswordResetRequestResult {
    pub token_id: Uuid,
    pub user_id: Uuid,
    pub token_created: bool,
    pub old_tokens_invalidated: u64,
}
