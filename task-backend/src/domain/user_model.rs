// src/domain/user_model.rs

use super::role_model::RoleWithPermissions;
use crate::core::permission::{PermissionResult, PermissionScope};
use crate::core::subscription_tier::SubscriptionTier;
use crate::utils::permission::{PermissionChecker, PermissionType, ResourceContext};
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

    pub role_id: Uuid,

    pub subscription_tier: String,

    #[sea_orm(unique, nullable)]
    pub stripe_customer_id: Option<String>,

    pub last_login_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_many = "crate::features::task::domain::task_model::Entity",
        from = "Column::Id",
        to = "crate::features::task::domain::task_model::Column::UserId"
    )]
    Tasks,

    #[sea_orm(has_many = "crate::domain::refresh_token_model::Entity")]
    RefreshTokens,

    #[sea_orm(has_many = "crate::domain::password_reset_token_model::Entity")]
    PasswordResetTokens,

    #[sea_orm(
        belongs_to = "crate::domain::role_model::Entity",
        from = "Column::RoleId",
        to = "crate::domain::role_model::Column::Id"
    )]
    Role,

    #[sea_orm(
        has_many = "crate::features::subscription::models::history::Entity",
        from = "Column::Id",
        to = "crate::features::subscription::models::history::Column::UserId"
    )]
    SubscriptionHistory,
}

// リレーション実装
impl Related<crate::features::task::domain::task_model::Entity> for Entity {
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

impl Related<crate::domain::role_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Role.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            is_active: Set(true),                       // デフォルトでアクティブ
            email_verified: Set(false),                 // デフォルトで未認証
            subscription_tier: Set("free".to_string()), // デフォルトはFree階層
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
            role_id: self.role_id,
            subscription_tier: self.subscription_tier.clone(),
            last_login_at: self.last_login_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    /// ロール情報付きのセーフなユーザー情報を取得
    pub fn to_safe_user_with_role(&self, role: RoleWithPermissions) -> SafeUserWithRole {
        SafeUserWithRole {
            id: self.id,
            email: self.email.clone(),
            username: self.username.clone(),
            is_active: self.is_active,
            email_verified: self.email_verified,
            role,
            subscription_tier: self.subscription_tier.clone(),
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
    pub role_id: Uuid,
    pub subscription_tier: String,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// ロール情報付きのセーフなユーザー表現
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SafeUserWithRole {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub role: RoleWithPermissions,
    pub subscription_tier: String,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Model> for SafeUser {
    fn from(user: Model) -> Self {
        user.to_safe_user()
    }
}

/// JWT のクレーム用のユーザー情報（統合版）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserClaims {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub role_name: String,                   // ロール名（基本認証用）
    pub role: Option<RoleWithPermissions>,   // 詳細ロール情報（権限チェック用）
    pub subscription_tier: SubscriptionTier, // サブスクリプション階層
}

impl UserClaims {
    /// 管理者権限があるかチェック（統合版）
    pub fn is_admin(&self) -> bool {
        if let Some(ref role) = self.role {
            PermissionChecker::is_admin(role)
        } else {
            PermissionChecker::check_permission_by_role_name(
                &self.role_name,
                PermissionType::IsAdmin,
                None,
            )
        }
    }

    /// 一般ユーザー権限があるかチェック（統合版）
    pub fn is_member(&self) -> bool {
        if let Some(ref role) = self.role {
            PermissionChecker::is_member(role)
        } else {
            PermissionChecker::check_permission_by_role_name(
                &self.role_name,
                PermissionType::IsMember,
                None,
            )
        }
    }

    /// 他のユーザーのデータにアクセス権限があるかチェック（統合版）
    pub fn can_access_user(&self, target_user_id: Uuid) -> bool {
        if let Some(ref role) = self.role {
            PermissionChecker::can_access_user(role, self.user_id, target_user_id)
        } else {
            let context = ResourceContext::for_user(self.user_id, target_user_id);
            PermissionChecker::check_permission_by_role_name(
                &self.role_name,
                PermissionType::CanAccessUser,
                Some(context),
            )
        }
    }

    /// リソース作成権限があるかチェック（統合版）
    pub fn can_create_resource(&self, resource_type: &str) -> bool {
        if let Some(ref role) = self.role {
            PermissionChecker::can_create_resource(role, resource_type)
        } else {
            let context = ResourceContext::new(resource_type, self.user_id, None, None);
            PermissionChecker::check_permission_by_role_name(
                &self.role_name,
                PermissionType::CanCreateResource,
                Some(context),
            )
        }
    }

    /// リソース削除権限があるかチェック（統合版）
    pub fn can_delete_resource(&self, resource_type: &str, owner_id: Option<Uuid>) -> bool {
        if let Some(ref role) = self.role {
            PermissionChecker::can_delete_resource(role, resource_type, owner_id, self.user_id)
        } else {
            let context = ResourceContext::new(resource_type, self.user_id, None, owner_id);
            PermissionChecker::check_permission_by_role_name(
                &self.role_name,
                PermissionType::CanDeleteResource,
                Some(context),
            )
        }
    }

    /// リソースの編集権限があるかチェック（新機能）
    pub fn can_update_resource(&self, resource_type: &str, owner_id: Option<Uuid>) -> bool {
        if let Some(ref role) = self.role {
            PermissionChecker::can_update_resource(role, resource_type, owner_id, self.user_id)
        } else {
            // ロール名ベースの場合は基本的な権限チェック
            match resource_type {
                "user" => {
                    if let Some(owner) = owner_id {
                        self.user_id == owner || self.role_name == "admin"
                    } else {
                        self.role_name == "admin"
                    }
                }
                "role" => self.role_name == "admin",
                "task" => {
                    if let Some(owner) = owner_id {
                        self.user_id == owner || self.role_name == "admin"
                    } else {
                        self.role_name == "admin"
                    }
                }
                _ => false,
            }
        }
    }

    /// リソースの表示権限があるかチェック（新機能）
    pub fn can_view_resource(&self, resource_type: &str, owner_id: Option<Uuid>) -> bool {
        if let Some(ref role) = self.role {
            PermissionChecker::can_view_resource(role, resource_type, owner_id, self.user_id)
        } else {
            // ロール名ベースの基本的な権限チェック
            match resource_type {
                "user" => {
                    if let Some(target_id) = owner_id {
                        self.can_access_user(target_id)
                    } else {
                        self.role_name == "admin"
                    }
                }
                "role" => self.role_name == "admin",
                "task" => {
                    if let Some(owner) = owner_id {
                        self.user_id == owner || self.role_name == "admin"
                    } else {
                        self.role_name == "admin"
                    }
                }
                _ => false,
            }
        }
    }

    /// 動的権限チェック
    pub fn can_perform_action(
        &self,
        resource: &str,
        action: &str,
        target_user_id: Option<Uuid>,
    ) -> PermissionResult {
        if let Some(ref role) = self.role {
            role.can_perform_action(resource, action, target_user_id)
        } else {
            // ロール情報がない場合は基本的なチェック
            match (resource, action) {
                ("tasks", "read") => {
                    if self.role_name == "admin" {
                        PermissionResult::Allowed {
                            privilege: None,
                            scope: PermissionScope::Global,
                        }
                    } else {
                        PermissionResult::Allowed {
                            privilege: None,
                            scope: PermissionScope::Own,
                        }
                    }
                }
                ("tasks", "write" | "create" | "delete") => {
                    if self.role_name == "admin" {
                        PermissionResult::Allowed {
                            privilege: None,
                            scope: PermissionScope::Global,
                        }
                    } else {
                        PermissionResult::Allowed {
                            privilege: None,
                            scope: PermissionScope::Own,
                        }
                    }
                }
                ("users", "read") => {
                    if self.role_name == "admin" {
                        PermissionResult::Allowed {
                            privilege: None,
                            scope: PermissionScope::Global,
                        }
                    } else if target_user_id == Some(self.user_id) {
                        PermissionResult::Allowed {
                            privilege: None,
                            scope: PermissionScope::Own,
                        }
                    } else {
                        PermissionResult::Denied {
                            reason: "Cannot access other users".to_string(),
                        }
                    }
                }
                ("roles", _) => {
                    if self.role_name == "admin" {
                        PermissionResult::Allowed {
                            privilege: None,
                            scope: PermissionScope::Global,
                        }
                    } else {
                        PermissionResult::Denied {
                            reason: "Only admin can manage roles".to_string(),
                        }
                    }
                }
                _ => PermissionResult::Denied {
                    reason: "Unknown resource or action".to_string(),
                },
            }
        }
    }

    /// サブスクリプション階層を取得
    pub fn get_subscription_tier(&self) -> SubscriptionTier {
        self.subscription_tier
    }
}

impl SafeUserWithRole {
    /// JWTクレームに変換（統合版）
    pub fn to_simple_claims(&self) -> UserClaims {
        UserClaims {
            user_id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            is_active: self.is_active,
            email_verified: self.email_verified,
            role_name: self.role.name.as_str().to_string(),
            role: Some(self.role.clone()),
            subscription_tier: self.role.subscription_tier,
        }
    }

    /// 認証可能かチェック
    pub fn can_authenticate(&self) -> bool {
        self.is_active
    }
}

impl From<SafeUserWithRole> for UserClaims {
    fn from(user: SafeUserWithRole) -> Self {
        Self {
            user_id: user.id,
            username: user.username,
            email: user.email,
            is_active: user.is_active,
            email_verified: user.email_verified,
            role_name: user.role.name.as_str().to_string(),
            subscription_tier: user.role.subscription_tier,
            role: Some(user.role),
        }
    }
}

impl From<UserClaims> for SafeUserWithRole {
    fn from(claims: UserClaims) -> Self {
        let role = claims.role.unwrap_or_else(|| {
            // デフォルトのロールを作成（role_nameから推測）
            let role_name = match claims.role_name.as_str() {
                "admin" => crate::domain::role_model::RoleName::Admin,
                _ => crate::domain::role_model::RoleName::Member,
            };

            crate::domain::role_model::RoleWithPermissions {
                id: uuid::Uuid::new_v4(),
                name: role_name,
                display_name: claims.role_name.clone(),
                description: None,
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                subscription_tier: claims.subscription_tier,
            }
        });

        Self {
            id: claims.user_id,
            email: claims.email,
            username: claims.username,
            is_active: claims.is_active,
            email_verified: claims.email_verified,
            role,
            subscription_tier: claims.subscription_tier.to_string(),
            last_login_at: None, // JWTクレームにはログイン時刻は含まれない
            // 注意: created_atとupdated_atはJWTクレームに含まれないため、現在時刻を設定
            // 実際の値が必要な場合は、DBからユーザー情報を取得する必要がある
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl From<SafeUserWithRole> for SafeUser {
    fn from(user_with_role: SafeUserWithRole) -> Self {
        Self {
            id: user_with_role.id,
            email: user_with_role.email,
            username: user_with_role.username,
            is_active: user_with_role.is_active,
            email_verified: user_with_role.email_verified,
            role_id: user_with_role.role.id,
            subscription_tier: user_with_role.subscription_tier,
            last_login_at: user_with_role.last_login_at,
            created_at: user_with_role.created_at,
            updated_at: user_with_role.updated_at,
        }
    }
}

impl From<UserClaims> for SafeUser {
    fn from(claims: UserClaims) -> Self {
        // 注意: この変換は、JWTクレームから基本的なユーザー情報のみを取得します。
        // role_id, created_at, updated_at, last_login_at などの完全な情報は含まれていません。
        // 完全なユーザー情報が必要な場合は、UserRepositoryを使用してDBから取得してください。
        Self {
            id: claims.user_id,
            email: claims.email,
            username: claims.username,
            is_active: claims.is_active,
            email_verified: claims.email_verified,
            role_id: claims.role.as_ref().map_or_else(Uuid::nil, |r| r.id), // role情報がある場合はそれを使用、なければnil UUIDを返す
            subscription_tier: claims.subscription_tier.to_string(),
            last_login_at: None, // JWTクレームにはログイン時刻は含まれない
            // タイムスタンプは実際のDB値ではなく、現在時刻を使用
            // 実際の値が必要な場合は、DBからユーザー情報を取得する必要がある
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
