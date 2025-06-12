// src/repository/user_repository.rs
#![allow(dead_code)]

use crate::db;
use crate::domain::user_model::{self, ActiveModel as UserActiveModel, Entity as UserEntity};
use sea_orm::entity::*;
use sea_orm::{query::*, DbConn, DbErr, DeleteResult, Set};
use sea_orm::{Condition, Order, PaginatorTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

pub struct UserRepository {
    db: DbConn,
    schema: Option<String>,
}

impl UserRepository {
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

    /// ユーザーをIDで検索
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<user_model::Model>, DbErr> {
        self.prepare_connection().await?;
        UserEntity::find_by_id(id).one(&self.db).await
    }

    /// ユーザーをメールアドレスで検索
    pub async fn find_by_email(&self, email: &str) -> Result<Option<user_model::Model>, DbErr> {
        self.prepare_connection().await?;
        UserEntity::find()
            .filter(user_model::Column::Email.eq(email))
            .one(&self.db)
            .await
    }

    /// ユーザーをユーザー名で検索
    pub async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<user_model::Model>, DbErr> {
        self.prepare_connection().await?;
        UserEntity::find()
            .filter(user_model::Column::Username.eq(username))
            .one(&self.db)
            .await
    }

    /// メールアドレスまたはユーザー名でユーザーを検索
    pub async fn find_by_email_or_username(
        &self,
        identifier: &str,
    ) -> Result<Option<user_model::Model>, DbErr> {
        self.prepare_connection().await?;
        UserEntity::find()
            .filter(
                Condition::any()
                    .add(user_model::Column::Email.eq(identifier))
                    .add(user_model::Column::Username.eq(identifier)),
            )
            .one(&self.db)
            .await
    }

    /// 全ユーザーを取得
    pub async fn find_all(&self) -> Result<Vec<user_model::Model>, DbErr> {
        self.prepare_connection().await?;
        UserEntity::find()
            .order_by(user_model::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await
    }

    /// アクティブユーザーのみを取得
    pub async fn find_active_users(&self) -> Result<Vec<user_model::Model>, DbErr> {
        self.prepare_connection().await?;
        UserEntity::find()
            .filter(user_model::Column::IsActive.eq(true))
            .order_by(user_model::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await
    }

    /// ページネーション付きでユーザーを取得
    pub async fn find_all_paginated(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<user_model::Model>, u64), DbErr> {
        self.prepare_connection().await?;

        let page_size = std::cmp::min(page_size, 100); // 最大100件
        let offset = (page - 1) * page_size;

        let users = UserEntity::find()
            .order_by(user_model::Column::CreatedAt, Order::Desc)
            .limit(page_size)
            .offset(offset)
            .all(&self.db)
            .await?;

        let total_count = UserEntity::find().count(&self.db).await?;

        Ok((users, total_count))
    }

    /// ユーザーを作成
    pub async fn create(&self, create_user: CreateUser) -> Result<user_model::Model, DbErr> {
        self.prepare_connection().await?;

        let new_user = UserActiveModel {
            email: Set(create_user.email),
            username: Set(create_user.username),
            password_hash: Set(create_user.password_hash),
            is_active: Set(create_user.is_active.unwrap_or(true)),
            email_verified: Set(create_user.email_verified.unwrap_or(false)),
            ..Default::default()
        };

        new_user.insert(&self.db).await
    }

    /// ユーザーを更新
    pub async fn update(
        &self,
        id: Uuid,
        update_user: UpdateUser,
    ) -> Result<Option<user_model::Model>, DbErr> {
        self.prepare_connection().await?;

        let user = match UserEntity::find_by_id(id).one(&self.db).await? {
            Some(u) => u,
            None => return Ok(None),
        };

        let mut active_model: UserActiveModel = user.into();
        let mut changed = false;

        if let Some(email) = update_user.email {
            active_model.email = Set(email);
            changed = true;
        }

        if let Some(username) = update_user.username {
            active_model.username = Set(username);
            changed = true;
        }

        if let Some(password_hash) = update_user.password_hash {
            active_model.password_hash = Set(password_hash);
            changed = true;
        }

        if let Some(is_active) = update_user.is_active {
            active_model.is_active = Set(is_active);
            changed = true;
        }

        if let Some(email_verified) = update_user.email_verified {
            active_model.email_verified = Set(email_verified);
            changed = true;
        }

        if changed {
            Ok(Some(active_model.update(&self.db).await?))
        } else {
            Ok(Some(active_model.try_into_model()?))
        }
    }

    /// ユーザーを削除
    pub async fn delete(&self, id: Uuid) -> Result<DeleteResult, DbErr> {
        self.prepare_connection().await?;
        UserEntity::delete_by_id(id).exec(&self.db).await
    }

    // --- 特殊な操作 ---

    /// メールアドレスの重複チェック
    pub async fn is_email_taken(&self, email: &str) -> Result<bool, DbErr> {
        self.prepare_connection().await?;
        let count = UserEntity::find()
            .filter(user_model::Column::Email.eq(email))
            .count(&self.db)
            .await?;
        Ok(count > 0)
    }

    /// ユーザー名の重複チェック
    pub async fn is_username_taken(&self, username: &str) -> Result<bool, DbErr> {
        self.prepare_connection().await?;
        let count = UserEntity::find()
            .filter(user_model::Column::Username.eq(username))
            .count(&self.db)
            .await?;
        Ok(count > 0)
    }

    /// メールアドレスまたはユーザー名の重複チェック（指定IDを除く）
    pub async fn is_email_or_username_taken_excluding_user(
        &self,
        email: &str,
        username: &str,
        exclude_user_id: Uuid,
    ) -> Result<(bool, bool), DbErr> {
        self.prepare_connection().await?;

        let email_taken = UserEntity::find()
            .filter(
                Condition::all()
                    .add(user_model::Column::Email.eq(email))
                    .add(user_model::Column::Id.ne(exclude_user_id)),
            )
            .count(&self.db)
            .await?
            > 0;

        let username_taken = UserEntity::find()
            .filter(
                Condition::all()
                    .add(user_model::Column::Username.eq(username))
                    .add(user_model::Column::Id.ne(exclude_user_id)),
            )
            .count(&self.db)
            .await?
            > 0;

        Ok((email_taken, username_taken))
    }

    /// ユーザーのアクティブ状態を更新
    pub async fn update_active_status(
        &self,
        id: Uuid,
        is_active: bool,
    ) -> Result<Option<user_model::Model>, DbErr> {
        self.prepare_connection().await?;

        let user = match UserEntity::find_by_id(id).one(&self.db).await? {
            Some(u) => u,
            None => return Ok(None),
        };

        let mut active_model: UserActiveModel = user.into();
        active_model.is_active = Set(is_active);

        Ok(Some(active_model.update(&self.db).await?))
    }

    /// メール認証状態を更新
    pub async fn update_email_verified_status(
        &self,
        id: Uuid,
        email_verified: bool,
    ) -> Result<Option<user_model::Model>, DbErr> {
        self.prepare_connection().await?;

        let user = match UserEntity::find_by_id(id).one(&self.db).await? {
            Some(u) => u,
            None => return Ok(None),
        };

        let mut active_model: UserActiveModel = user.into();
        active_model.email_verified = Set(email_verified);

        Ok(Some(active_model.update(&self.db).await?))
    }

    /// パスワードハッシュを更新
    pub async fn update_password_hash(
        &self,
        id: Uuid,
        password_hash: String,
    ) -> Result<Option<user_model::Model>, DbErr> {
        self.prepare_connection().await?;

        let user = match UserEntity::find_by_id(id).one(&self.db).await? {
            Some(u) => u,
            None => return Ok(None),
        };

        let mut active_model: UserActiveModel = user.into();
        active_model.password_hash = Set(password_hash);

        Ok(Some(active_model.update(&self.db).await?))
    }

    /// ユーザー名のみを更新
    pub async fn update_username(
        &self,
        id: Uuid,
        username: String,
    ) -> Result<Option<user_model::Model>, DbErr> {
        self.prepare_connection().await?;

        let user = match UserEntity::find_by_id(id).one(&self.db).await? {
            Some(u) => u,
            None => return Ok(None),
        };

        let mut active_model: UserActiveModel = user.into();
        active_model.username = Set(username);

        Ok(Some(active_model.update(&self.db).await?))
    }

    /// メールアドレスのみを更新
    pub async fn update_email(
        &self,
        id: Uuid,
        email: String,
    ) -> Result<Option<user_model::Model>, DbErr> {
        self.prepare_connection().await?;

        let user = match UserEntity::find_by_id(id).one(&self.db).await? {
            Some(u) => u,
            None => return Ok(None),
        };

        let mut active_model: UserActiveModel = user.into();
        active_model.email = Set(email);

        Ok(Some(active_model.update(&self.db).await?))
    }

    /// メール認証済みにマーク
    pub async fn mark_email_verified(&self, id: Uuid) -> Result<Option<user_model::Model>, DbErr> {
        self.update_email_verified_status(id, true).await
    }

    /// 最後のログイン時間を更新
    pub async fn update_last_login(&self, id: Uuid) -> Result<Option<user_model::Model>, DbErr> {
        self.prepare_connection().await?;

        let user = match UserEntity::find_by_id(id).one(&self.db).await? {
            Some(u) => u,
            None => return Ok(None),
        };

        let mut active_model: UserActiveModel = user.into();
        active_model.last_login_at = Set(Some(chrono::Utc::now()));

        Ok(Some(active_model.update(&self.db).await?))
    }

    /// ユーザー統計を取得
    pub async fn get_user_stats(&self) -> Result<UserStats, DbErr> {
        self.prepare_connection().await?;

        let total_users = UserEntity::find().count(&self.db).await?;

        let active_users = UserEntity::find()
            .filter(user_model::Column::IsActive.eq(true))
            .count(&self.db)
            .await?;

        let verified_users = UserEntity::find()
            .filter(user_model::Column::EmailVerified.eq(true))
            .count(&self.db)
            .await?;

        Ok(UserStats {
            total_users,
            active_users,
            verified_users,
            inactive_users: total_users - active_users,
            unverified_users: total_users - verified_users,
        })
    }
}

// --- DTOと関連構造体 ---

/// ユーザー作成用構造体
#[derive(Debug, Clone)]
pub struct CreateUser {
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub is_active: Option<bool>,
    pub email_verified: Option<bool>,
}

/// ユーザー更新用構造体
#[derive(Debug, Clone, Default)]
pub struct UpdateUser {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password_hash: Option<String>,
    pub is_active: Option<bool>,
    pub email_verified: Option<bool>,
}

/// ユーザー統計情報
#[derive(Debug, Clone)]
pub struct UserStats {
    pub total_users: u64,
    pub active_users: u64,
    pub verified_users: u64,
    pub inactive_users: u64,
    pub unverified_users: u64,
}
