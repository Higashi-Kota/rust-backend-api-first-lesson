// src/domain/refresh_token_model.rs

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

    #[sea_orm(nullable)]
    pub last_used_at: Option<DateTime<Utc>>,

    pub use_count: i32,

    #[sea_orm(nullable)]
    pub ip_address: Option<String>,

    #[sea_orm(nullable)]
    pub user_agent: Option<String>,

    #[sea_orm(nullable)]
    pub device_type: Option<String>,

    #[sea_orm(nullable)]
    pub geolocation_country: Option<String>,

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
            use_count: Set(0),      // デフォルトは0回
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

/// 全トークン無効化の結果
#[derive(Debug, Clone)]
pub struct RevokeAllResult {
    pub revoked_count: u64,
    pub affected_users: u64,
}
