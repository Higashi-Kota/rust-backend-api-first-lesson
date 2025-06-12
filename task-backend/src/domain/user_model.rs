// src/domain/user_model.rs
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait, DbErr, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    #[sea_orm(unique)]
    pub email: String,

    #[sea_orm(unique)]
    pub username: String,

    #[serde(skip_serializing)] // パスワードハッシュは絶対にシリアライズしない
    pub password_hash: String,

    pub is_active: bool,

    pub email_verified: bool,

    pub last_login_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_many = "crate::domain::task_model::Entity",
        from = "Column::Id",
        to = "crate::domain::task_model::Column::UserId"
    )]
    Tasks,

    #[sea_orm(has_many = "crate::domain::refresh_token_model::Entity")]
    RefreshTokens,

    #[sea_orm(has_many = "crate::domain::password_reset_token_model::Entity")]
    PasswordResetTokens,
}

// リレーション実装
impl Related<crate::domain::task_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tasks.def()
    }
}

impl Related<crate::domain::refresh_token_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RefreshTokens.def()
    }
}

impl Related<crate::domain::password_reset_token_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PasswordResetTokens.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            is_active: Set(true),       // デフォルトでアクティブ
            email_verified: Set(false), // デフォルトで未認証
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

// ユーザー用の便利メソッド実装
impl Model {
    /// ユーザーがアクティブかつメール認証済みかチェック
    pub fn is_fully_active(&self) -> bool {
        self.is_active && self.email_verified
    }

    /// ユーザーが認証可能な状態かチェック
    pub fn can_authenticate(&self) -> bool {
        self.is_active
    }

    /// パスワードハッシュを除いたセーフなユーザー情報を取得
    pub fn to_safe_user(&self) -> SafeUser {
        SafeUser {
            id: self.id,
            email: self.email.clone(),
            username: self.username.clone(),
            is_active: self.is_active,
            email_verified: self.email_verified,
            last_login_at: self.last_login_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// パスワードハッシュを含まないセーフなユーザー表現
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SafeUser {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Model> for SafeUser {
    fn from(user: Model) -> Self {
        user.to_safe_user()
    }
}

/// JWT のクレーム用のユーザー情報
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserClaims {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub email_verified: bool,
}

impl From<Model> for UserClaims {
    fn from(user: Model) -> Self {
        Self {
            user_id: user.id,
            username: user.username,
            email: user.email,
            is_active: user.is_active,
            email_verified: user.email_verified,
        }
    }
}

impl From<SafeUser> for UserClaims {
    fn from(user: SafeUser) -> Self {
        Self {
            user_id: user.id,
            username: user.username,
            email: user.email,
            is_active: user.is_active,
            email_verified: user.email_verified,
        }
    }
}

impl From<UserClaims> for SafeUser {
    fn from(claims: UserClaims) -> Self {
        Self {
            id: claims.user_id,
            email: claims.email,
            username: claims.username,
            is_active: claims.is_active,
            email_verified: claims.email_verified,
            last_login_at: None, // Claims don't contain login time
            // For claims conversion, we'll use current time as placeholders
            // In a real scenario, you'd want to fetch from database
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
