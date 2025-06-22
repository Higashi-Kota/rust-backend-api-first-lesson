// src/repository/user_repository.rs

use crate::db;
use crate::domain::role_model::{self, Entity as RoleEntity, RoleWithPermissions};
use crate::domain::user_model::{
    self, ActiveModel as UserActiveModel, Entity as UserEntity, SafeUserWithRole,
};
use sea_orm::entity::*;
use sea_orm::{Condition, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use sea_orm::{DbConn, DbErr, DeleteResult, JoinType, Set};
use uuid::Uuid;

#[derive(Debug)]
pub struct UserRepository {
    db: DbConn,
    schema: Option<String>,
}

impl UserRepository {
    pub fn new(db: DbConn) -> Self {
        Self { db, schema: None }
    }

    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub async fn find_all(&self) -> Result<Vec<user_model::Model>, DbErr> {
        self.prepare_connection().await?;
        UserEntity::find()
            .order_by(user_model::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await
    }

    /// アクティブユーザーのみを取得
    #[allow(dead_code)]
    pub async fn find_active_users(&self) -> Result<Vec<user_model::Model>, DbErr> {
        self.prepare_connection().await?;
        UserEntity::find()
            .filter(user_model::Column::IsActive.eq(true))
            .order_by(user_model::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await
    }

    /// ページネーション付きでユーザーを取得
    #[allow(dead_code)]
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
            role_id: Set(create_user.role_id),
            subscription_tier: Set(create_user
                .subscription_tier
                .unwrap_or_else(|| "free".to_string())),
            is_active: Set(create_user.is_active.unwrap_or(true)),
            email_verified: Set(create_user.email_verified.unwrap_or(false)),
            ..Default::default()
        };

        new_user.insert(&self.db).await
    }

    /// ユーザーを更新
    #[allow(dead_code)]
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

        if let Some(role_id) = update_user.role_id {
            active_model.role_id = Set(role_id);
            changed = true;
        }

        if let Some(subscription_tier) = update_user.subscription_tier {
            active_model.subscription_tier = Set(subscription_tier);
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

    /// サブスクリプション階層のみを更新
    pub async fn update_subscription_tier(
        &self,
        id: Uuid,
        subscription_tier: String,
    ) -> Result<Option<user_model::Model>, DbErr> {
        self.prepare_connection().await?;

        let user = match UserEntity::find_by_id(id).one(&self.db).await? {
            Some(u) => u,
            None => return Ok(None),
        };

        let mut active_model: UserActiveModel = user.into();
        active_model.subscription_tier = Set(subscription_tier);

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
        self.update_email_verified(id, true).await
    }

    /// メール認証状態を更新
    pub async fn update_email_verified(
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
    #[allow(dead_code)]
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

    // --- ロール関連操作 ---

    /// ユーザーをロール情報と一緒に取得
    pub async fn find_by_id_with_role(&self, id: Uuid) -> Result<Option<SafeUserWithRole>, DbErr> {
        self.prepare_connection().await?;

        let result = UserEntity::find_by_id(id)
            .join(JoinType::InnerJoin, user_model::Relation::Role.def())
            .select_also(RoleEntity)
            .one(&self.db)
            .await?;

        match result {
            Some((user, Some(role))) => match RoleWithPermissions::from_model(role) {
                Ok(role_with_perms) => Ok(Some(user.to_safe_user_with_role(role_with_perms))),
                Err(_) => Err(DbErr::Custom("Invalid role data".to_string())),
            },
            _ => Ok(None),
        }
    }

    /// メールアドレスでユーザーをロール情報と一緒に取得
    #[allow(dead_code)]
    pub async fn find_by_email_with_role(
        &self,
        email: &str,
    ) -> Result<Option<SafeUserWithRole>, DbErr> {
        self.prepare_connection().await?;

        let result = UserEntity::find()
            .filter(user_model::Column::Email.eq(email))
            .join(JoinType::InnerJoin, user_model::Relation::Role.def())
            .select_also(RoleEntity)
            .one(&self.db)
            .await?;

        match result {
            Some((user, Some(role))) => match RoleWithPermissions::from_model(role) {
                Ok(role_with_perms) => Ok(Some(user.to_safe_user_with_role(role_with_perms))),
                Err(_) => Err(DbErr::Custom("Invalid role data".to_string())),
            },
            _ => Ok(None),
        }
    }

    /// ユーザー名でユーザーをロール情報と一緒に取得
    #[allow(dead_code)]
    pub async fn find_by_username_with_role(
        &self,
        username: &str,
    ) -> Result<Option<SafeUserWithRole>, DbErr> {
        self.prepare_connection().await?;

        let result = UserEntity::find()
            .filter(user_model::Column::Username.eq(username))
            .join(JoinType::InnerJoin, user_model::Relation::Role.def())
            .select_also(RoleEntity)
            .one(&self.db)
            .await?;

        match result {
            Some((user, Some(role))) => match RoleWithPermissions::from_model(role) {
                Ok(role_with_perms) => Ok(Some(user.to_safe_user_with_role(role_with_perms))),
                Err(_) => Err(DbErr::Custom("Invalid role data".to_string())),
            },
            _ => Ok(None),
        }
    }

    /// メールアドレスまたはユーザー名でユーザーをロール情報と一緒に取得
    #[allow(dead_code)]
    pub async fn find_by_email_or_username_with_role(
        &self,
        identifier: &str,
    ) -> Result<Option<SafeUserWithRole>, DbErr> {
        self.prepare_connection().await?;

        let result = UserEntity::find()
            .filter(
                Condition::any()
                    .add(user_model::Column::Email.eq(identifier))
                    .add(user_model::Column::Username.eq(identifier)),
            )
            .join(JoinType::InnerJoin, user_model::Relation::Role.def())
            .select_also(RoleEntity)
            .one(&self.db)
            .await?;

        match result {
            Some((user, Some(role))) => match RoleWithPermissions::from_model(role) {
                Ok(role_with_perms) => Ok(Some(user.to_safe_user_with_role(role_with_perms))),
                Err(_) => Err(DbErr::Custom("Invalid role data".to_string())),
            },
            _ => Ok(None),
        }
    }

    /// 全ユーザーをロール情報と一緒に取得
    pub async fn find_all_with_roles(&self) -> Result<Vec<SafeUserWithRole>, DbErr> {
        self.prepare_connection().await?;

        let results = UserEntity::find()
            .join(JoinType::InnerJoin, user_model::Relation::Role.def())
            .select_also(RoleEntity)
            .order_by(user_model::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await?;

        let mut users_with_roles = Vec::new();
        for (user, role_opt) in results {
            if let Some(role) = role_opt {
                match RoleWithPermissions::from_model(role) {
                    Ok(role_with_perms) => {
                        users_with_roles.push(user.to_safe_user_with_role(role_with_perms));
                    }
                    Err(_) => continue, // スキップして続行
                }
            }
        }

        Ok(users_with_roles)
    }

    /// ページネーション付きでユーザーをロール情報と一緒に取得
    #[allow(dead_code)]
    pub async fn find_all_with_roles_paginated(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<SafeUserWithRole>, u64), DbErr> {
        self.prepare_connection().await?;

        let page_size = std::cmp::min(page_size, 100); // 最大100件
        let offset = (page - 1) * page_size;

        let results = UserEntity::find()
            .join(JoinType::InnerJoin, user_model::Relation::Role.def())
            .select_also(RoleEntity)
            .order_by(user_model::Column::CreatedAt, Order::Desc)
            .limit(page_size)
            .offset(offset)
            .all(&self.db)
            .await?;

        let total_count = UserEntity::find().count(&self.db).await?;

        let mut users_with_roles = Vec::new();
        for (user, role_opt) in results {
            if let Some(role) = role_opt {
                match RoleWithPermissions::from_model(role) {
                    Ok(role_with_perms) => {
                        users_with_roles.push(user.to_safe_user_with_role(role_with_perms));
                    }
                    Err(_) => continue, // スキップして続行
                }
            }
        }

        Ok((users_with_roles, total_count))
    }

    /// 特定のロールを持つユーザーを取得
    pub async fn find_by_role_id(&self, role_id: Uuid) -> Result<Vec<user_model::Model>, DbErr> {
        self.prepare_connection().await?;

        UserEntity::find()
            .filter(user_model::Column::RoleId.eq(role_id))
            .order_by(user_model::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await
    }

    /// 特定のロール名を持つユーザーを取得
    #[allow(dead_code)]
    pub async fn find_by_role_name(&self, role_name: &str) -> Result<Vec<SafeUserWithRole>, DbErr> {
        self.prepare_connection().await?;

        let results = UserEntity::find()
            .join(JoinType::InnerJoin, user_model::Relation::Role.def())
            .filter(role_model::Column::Name.eq(role_name))
            .select_also(RoleEntity)
            .order_by(user_model::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await?;

        let mut users_with_roles = Vec::new();
        for (user, role_opt) in results {
            if let Some(role) = role_opt {
                match RoleWithPermissions::from_model(role) {
                    Ok(role_with_perms) => {
                        users_with_roles.push(user.to_safe_user_with_role(role_with_perms));
                    }
                    Err(_) => continue,
                }
            }
        }

        Ok(users_with_roles)
    }

    /// ユーザーのロールを更新
    pub async fn update_user_role(
        &self,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<Option<user_model::Model>, DbErr> {
        self.prepare_connection().await?;

        let user = match UserEntity::find_by_id(user_id).one(&self.db).await? {
            Some(u) => u,
            None => return Ok(None),
        };

        let mut active_model: UserActiveModel = user.into();
        active_model.role_id = Set(role_id);

        Ok(Some(active_model.update(&self.db).await?))
    }

    /// 管理者ユーザーを取得
    #[allow(dead_code)]
    pub async fn find_admin_users(&self) -> Result<Vec<SafeUserWithRole>, DbErr> {
        self.find_by_role_name("admin").await
    }

    /// 一般ユーザーを取得
    #[allow(dead_code)]
    pub async fn find_member_users(&self) -> Result<Vec<SafeUserWithRole>, DbErr> {
        self.find_by_role_name("member").await
    }

    /// 特定のサブスクリプション階層のユーザーを取得
    #[allow(dead_code)]
    pub async fn find_by_subscription_tier(
        &self,
        tier: &str,
    ) -> Result<Vec<user_model::Model>, DbErr> {
        self.prepare_connection().await?;

        UserEntity::find()
            .filter(user_model::Column::SubscriptionTier.eq(tier))
            .order_by(user_model::Column::CreatedAt, Order::Desc)
            .all(&self.db)
            .await
    }

    /// サブスクリプション階層別のユーザー統計を取得
    pub async fn get_subscription_tier_stats(&self) -> Result<Vec<SubscriptionTierStats>, DbErr> {
        self.prepare_connection().await?;

        let results = UserEntity::find().all(&self.db).await?;

        let mut tier_stats: std::collections::HashMap<String, SubscriptionTierStats> =
            std::collections::HashMap::new();

        for user in results {
            let stats =
                tier_stats
                    .entry(user.subscription_tier.clone())
                    .or_insert(SubscriptionTierStats {
                        tier: user.subscription_tier.clone(),
                        user_count: 0,
                        total_users: 0,
                        active_users: 0,
                        verified_users: 0,
                    });

            stats.user_count += 1;
            stats.total_users += 1;
            if user.is_active {
                stats.active_users += 1;
            }
            if user.email_verified {
                stats.verified_users += 1;
            }
        }

        Ok(tier_stats.into_values().collect())
    }

    /// ロール別ユーザー統計を取得
    #[allow(dead_code)]
    pub async fn get_user_stats_by_role(&self) -> Result<Vec<RoleUserStats>, DbErr> {
        self.prepare_connection().await?;

        let results = UserEntity::find()
            .join(JoinType::InnerJoin, user_model::Relation::Role.def())
            .select_also(RoleEntity)
            .all(&self.db)
            .await?;

        let mut role_stats: std::collections::HashMap<String, RoleUserStats> =
            std::collections::HashMap::new();

        for (user, role_opt) in results {
            if let Some(role) = role_opt {
                let stats = role_stats
                    .entry(role.name.clone())
                    .or_insert(RoleUserStats {
                        role_name: role.name.clone(),
                        role_display_name: role.display_name.clone(),
                        total_users: 0,
                        active_users: 0,
                        verified_users: 0,
                    });

                stats.total_users += 1;
                if user.is_active {
                    stats.active_users += 1;
                }
                if user.email_verified {
                    stats.verified_users += 1;
                }
            }
        }

        Ok(role_stats.into_values().collect())
    }

    /// ユーザー検索（管理者用）
    pub async fn search_users(
        &self,
        query: Option<String>,
        is_active: Option<bool>,
        email_verified: Option<bool>,
        page: i32,
        per_page: i32,
    ) -> Result<Vec<SafeUserWithRole>, DbErr> {
        self.prepare_connection().await?;

        let mut condition = Condition::all();

        if let Some(q) = query {
            let search_term = format!("%{}%", q);
            condition = condition.add(
                Condition::any()
                    .add(user_model::Column::Username.like(&search_term))
                    .add(user_model::Column::Email.like(&search_term)),
            );
        }

        if let Some(active) = is_active {
            condition = condition.add(user_model::Column::IsActive.eq(active));
        }

        if let Some(verified) = email_verified {
            condition = condition.add(user_model::Column::EmailVerified.eq(verified));
        }

        let page_size = std::cmp::min(per_page as u64, 100);
        let offset = ((page - 1) * per_page) as u64;

        let results = UserEntity::find()
            .filter(condition)
            .join(JoinType::InnerJoin, user_model::Relation::Role.def())
            .select_also(RoleEntity)
            .order_by(user_model::Column::CreatedAt, Order::Desc)
            .limit(page_size)
            .offset(offset)
            .all(&self.db)
            .await?;

        let mut users_with_roles = Vec::new();
        for (user, role_opt) in results {
            if let Some(role) = role_opt {
                match RoleWithPermissions::from_model(role) {
                    Ok(role_with_perms) => {
                        users_with_roles.push(user.to_safe_user_with_role(role_with_perms));
                    }
                    Err(_) => continue,
                }
            }
        }

        Ok(users_with_roles)
    }

    /// ユーザー検索の総件数を取得（管理者用）
    pub async fn count_users_by_filter(
        &self,
        query: Option<&str>,
        is_active: Option<bool>,
        email_verified: Option<bool>,
    ) -> Result<usize, DbErr> {
        self.prepare_connection().await?;

        let mut condition = Condition::all();

        if let Some(q) = query {
            let search_term = format!("%{}%", q);
            condition = condition.add(
                Condition::any()
                    .add(user_model::Column::Username.like(&search_term))
                    .add(user_model::Column::Email.like(&search_term)),
            );
        }

        if let Some(active) = is_active {
            condition = condition.add(user_model::Column::IsActive.eq(active));
        }

        if let Some(verified) = email_verified {
            condition = condition.add(user_model::Column::EmailVerified.eq(verified));
        }

        let count = UserEntity::find().filter(condition).count(&self.db).await?;

        Ok(count as usize)
    }
}

// --- DTOと関連構造体 ---

/// ユーザー作成用構造体
#[derive(Debug, Clone)]
pub struct CreateUser {
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub role_id: Uuid,
    pub subscription_tier: Option<String>,
    pub is_active: Option<bool>,
    pub email_verified: Option<bool>,
}

/// ユーザー更新用構造体
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct UpdateUser {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password_hash: Option<String>,
    pub role_id: Option<Uuid>,
    pub subscription_tier: Option<String>,
    pub is_active: Option<bool>,
    pub email_verified: Option<bool>,
}

/// ユーザー統計情報
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UserStats {
    pub total_users: u64,
    pub active_users: u64,
    pub verified_users: u64,
    pub inactive_users: u64,
    pub unverified_users: u64,
}

/// ロール別ユーザー統計情報
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RoleUserStats {
    pub role_name: String,
    pub role_display_name: String,
    pub total_users: u64,
    pub active_users: u64,
    pub verified_users: u64,
}

/// サブスクリプション階層別ユーザー統計情報
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionTierStats {
    pub tier: String,
    pub user_count: u64,
    pub total_users: u64,
    pub active_users: u64,
    pub verified_users: u64,
}
