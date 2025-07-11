// task-backend/src/features/security/models/role.rs
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ロールエンティティ
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "roles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    #[sea_orm(unique)]
    pub name: String,

    pub display_name: String,

    #[sea_orm(nullable)]
    pub description: Option<String>,

    pub is_active: bool,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // TODO: Phase 19でUserモデルがfeatures/authに移行後に更新
    // #[sea_orm(has_many = "crate::domain::user_model::Entity")]
    // Users,
}

// TODO: Phase 19でUserモデルとの関連を定義
// impl Related<crate::domain::user_model::Entity> for Entity {
//     fn to() -> RelationDef {
//         Relation::Users.def()
//     }
// }

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
use crate::core::permission::Permission;
use crate::core::permission::{PermissionQuota, PermissionResult, PermissionScope, Privilege};
use crate::core::subscription_tier::SubscriptionTier;

/// ロール名を表すenum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleName {
    Admin,
    Member,
}

impl RoleName {
    /// ロール名を文字列として取得
    pub fn as_str(&self) -> &'static str {
        match self {
            RoleName::Admin => "admin",
            RoleName::Member => "member",
        }
    }

    /// 文字列からロール名を解析
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(RoleName::Admin),
            "member" => Some(RoleName::Member),
            _ => None,
        }
    }

    /// 管理者権限があるかチェック
    #[allow(dead_code)] // Utility method for role type checking
    pub fn is_admin(&self) -> bool {
        matches!(self, RoleName::Admin)
    }

    /// 一般ユーザー権限があるかチェック（管理者も含む）
    #[allow(dead_code)] // Utility method for role type checking
    pub fn is_member(&self) -> bool {
        matches!(self, RoleName::Member | RoleName::Admin)
    }

    /// 権限レベルを数値で取得（高いほど強い権限）
    #[allow(dead_code)] // Model utility method
    pub fn permission_level(&self) -> u8 {
        match self {
            RoleName::Admin => 100,
            RoleName::Member => 10,
        }
    }
}

impl std::fmt::Display for RoleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for RoleName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s).ok_or_else(|| format!("Invalid role name: {}", s))
    }
}

/// ロールWithアクセス権限チェック機能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleWithPermissions {
    pub id: Uuid,
    pub name: RoleName,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// サブスクリプション階層（デフォルトはFree）
    pub subscription_tier: SubscriptionTier,
}

#[allow(dead_code)] // TODO: Will be used when advanced permission features are integrated
impl RoleWithPermissions {
    /// Modelから変換
    pub fn from_model(model: Model) -> Result<Self, String> {
        let role_name = RoleName::from_str(&model.name)
            .ok_or_else(|| format!("Invalid role name in database: {}", model.name))?;

        Ok(Self {
            id: model.id,
            name: role_name,
            display_name: model.display_name,
            description: model.description,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            subscription_tier: SubscriptionTier::Free, // デフォルト値
        })
    }

    /// Modelから変換（サブスクリプション階層指定）
    pub fn from_model_with_subscription(
        model: Model,
        subscription_tier: SubscriptionTier,
    ) -> Result<Self, String> {
        let role_name = RoleName::from_str(&model.name)
            .ok_or_else(|| format!("Invalid role name in database: {}", model.name))?;

        Ok(Self {
            id: model.id,
            name: role_name,
            display_name: model.display_name,
            description: model.description,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            subscription_tier,
        })
    }

    /// 管理者権限があるかチェック（内部実装）
    pub fn is_admin(&self) -> bool {
        self.name.is_admin() && self.is_active
    }

    /// 一般ユーザー権限があるかチェック（内部実装）
    pub fn is_member(&self) -> bool {
        self.name.is_member() && self.is_active
    }

    /// 指定されたユーザーIDにアクセス権限があるかチェック（内部実装）
    pub fn can_access_user(&self, requesting_user_id: Uuid, target_user_id: Uuid) -> bool {
        if !self.is_active {
            return false;
        }
        if requesting_user_id == target_user_id {
            return true;
        }
        self.is_admin()
    }

    /// リソースの作成権限があるかチェック（内部実装）
    pub fn can_create_resource(&self, resource_type: &str) -> bool {
        if !self.is_active {
            return false;
        }
        match resource_type {
            "user" => self.is_admin(),
            "role" => self.is_admin(),
            "task" => true,
            _ => false,
        }
    }

    /// リソースの表示権限があるかチェック（新機能）
    pub fn can_view_resource(
        &self,
        resource_type: &str,
        owner_id: Option<Uuid>,
        requesting_user_id: Uuid,
    ) -> bool {
        if !self.is_active {
            return false;
        }
        match resource_type {
            "user" => {
                self.can_access_user(requesting_user_id, owner_id.unwrap_or(requesting_user_id))
            }
            "role" => self.is_admin(),
            "task" => {
                if let Some(owner) = owner_id {
                    owner == requesting_user_id || self.is_admin()
                } else {
                    self.is_admin()
                }
            }
            _ => false,
        }
    }

    /// 動的権限チェック
    pub fn can_perform_action(
        &self,
        resource: &str,
        action: &str,
        target_user_id: Option<Uuid>,
    ) -> PermissionResult {
        if !self.is_active {
            return PermissionResult::Denied {
                reason: "User role is inactive".to_string(),
            };
        }

        let base_permission = self.get_base_permission(resource, action, target_user_id);
        if let Some((scope, _reason)) = base_permission {
            let subscription_privilege = self.get_subscription_privilege(resource, action);
            PermissionResult::Allowed {
                privilege: subscription_privilege,
                scope,
            }
        } else {
            PermissionResult::Denied {
                reason: format!("Access denied for {} {} action", resource, action),
            }
        }
    }

    /// 基本権限を取得
    fn get_base_permission(
        &self,
        resource: &str,
        action: &str,
        target_user_id: Option<Uuid>,
    ) -> Option<(PermissionScope, String)> {
        match (resource, action) {
            ("tasks", "read") => {
                if self.is_admin() {
                    Some((
                        PermissionScope::Global,
                        "Admin can read all tasks".to_string(),
                    ))
                } else {
                    Some((
                        PermissionScope::Own,
                        "Member can read own tasks".to_string(),
                    ))
                }
            }
            ("tasks", "write" | "create") => {
                if self.is_admin() {
                    Some((
                        PermissionScope::Global,
                        "Admin can write all tasks".to_string(),
                    ))
                } else {
                    Some((
                        PermissionScope::Own,
                        "Member can write own tasks".to_string(),
                    ))
                }
            }
            ("tasks", "delete") => {
                if self.is_admin() {
                    Some((
                        PermissionScope::Global,
                        "Admin can delete all tasks".to_string(),
                    ))
                } else {
                    Some((
                        PermissionScope::Own,
                        "Member can delete own tasks".to_string(),
                    ))
                }
            }
            ("users", "read") => {
                if self.is_admin() {
                    Some((
                        PermissionScope::Global,
                        "Admin can read all users".to_string(),
                    ))
                } else {
                    target_user_id.map(|_target_id| {
                        (
                            PermissionScope::Own,
                            "Member can read own profile".to_string(),
                        )
                    })
                }
            }
            ("users", "write" | "update") => {
                if self.is_admin() {
                    Some((
                        PermissionScope::Global,
                        "Admin can update all users".to_string(),
                    ))
                } else {
                    Some((
                        PermissionScope::Own,
                        "Member can update own profile".to_string(),
                    ))
                }
            }
            ("roles", _) => {
                if self.is_admin() {
                    Some((
                        PermissionScope::Global,
                        "Admin can manage roles".to_string(),
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// サブスクリプションによる特権を取得
    pub fn get_subscription_privilege(&self, resource: &str, action: &str) -> Option<Privilege> {
        match (&self.subscription_tier, resource, action) {
            (SubscriptionTier::Free, "tasks", "read") => Some(Privilege::new(
                "basic_task_access",
                SubscriptionTier::Free,
                Some(PermissionQuota::limited(100, 10)),
            )),
            (SubscriptionTier::Pro, "tasks", "read") => Some(Privilege::new(
                "pro_task_access",
                SubscriptionTier::Pro,
                Some(PermissionQuota::new(
                    Some(10_000),
                    Some(100),
                    vec!["advanced_filter".to_string(), "export".to_string()],
                )),
            )),
            (SubscriptionTier::Enterprise, "tasks", "read") => Some(Privilege::new(
                "enterprise_task_access",
                SubscriptionTier::Enterprise,
                Some(PermissionQuota::unlimited()),
            )),
            (SubscriptionTier::Pro, "tasks", "write") => Some(Privilege::new(
                "pro_task_write",
                SubscriptionTier::Pro,
                Some(PermissionQuota::new(
                    Some(1_000),
                    Some(50),
                    vec!["batch_operations".to_string()],
                )),
            )),
            (SubscriptionTier::Enterprise, "tasks", "write") => Some(Privilege::new(
                "enterprise_task_write",
                SubscriptionTier::Enterprise,
                Some(PermissionQuota::unlimited()),
            )),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_name_conversion() {
        assert_eq!(RoleName::Admin.as_str(), "admin");
        assert_eq!(RoleName::Member.as_str(), "member");

        assert_eq!(RoleName::from_str("admin"), Some(RoleName::Admin));
        assert_eq!(RoleName::from_str("ADMIN"), Some(RoleName::Admin));
        assert_eq!(RoleName::from_str("member"), Some(RoleName::Member));
        assert_eq!(RoleName::from_str("invalid"), None);
    }

    #[test]
    fn test_role_checks() {
        assert!(RoleName::Admin.is_admin());
        assert!(RoleName::Admin.is_member());
        assert!(!RoleName::Member.is_admin());
        assert!(RoleName::Member.is_member());
    }

    #[test]
    fn test_subscription_tier() {
        assert_eq!(SubscriptionTier::Free.level(), 1);
        assert_eq!(SubscriptionTier::Pro.level(), 2);
        assert_eq!(SubscriptionTier::Enterprise.level(), 3);

        assert!(SubscriptionTier::Pro.is_at_least(&SubscriptionTier::Free));
        assert!(SubscriptionTier::Enterprise.is_at_least(&SubscriptionTier::Pro));
        assert!(!SubscriptionTier::Free.is_at_least(&SubscriptionTier::Pro));

        assert_eq!(SubscriptionTier::Free.as_str(), "free");
        assert_eq!(SubscriptionTier::Pro.as_str(), "pro");
        assert_eq!(SubscriptionTier::Enterprise.as_str(), "enterprise");

        assert_eq!(
            SubscriptionTier::from_str("free"),
            Some(SubscriptionTier::Free)
        );
        assert_eq!(
            SubscriptionTier::from_str("PRO"),
            Some(SubscriptionTier::Pro)
        );
        assert_eq!(SubscriptionTier::from_str("invalid"), None);
    }

    #[test]
    fn test_permission_scope() {
        assert_eq!(PermissionScope::Own.level(), 1);
        assert_eq!(PermissionScope::Team.level(), 2);
        assert_eq!(PermissionScope::Organization.level(), 3);
        assert_eq!(PermissionScope::Global.level(), 4);

        assert!(PermissionScope::Global.includes(&PermissionScope::Own));
        assert!(PermissionScope::Organization.includes(&PermissionScope::Team));
        assert!(!PermissionScope::Own.includes(&PermissionScope::Team));
    }

    #[test]
    fn test_permission_quota() {
        let quota = PermissionQuota::limited(100, 10);
        assert_eq!(quota.max_items, Some(100));
        assert_eq!(quota.rate_limit, Some(10));
        assert!(quota.has_feature("basic_access"));

        let unlimited = PermissionQuota::unlimited();
        assert_eq!(unlimited.max_items, None);
        assert_eq!(unlimited.rate_limit, None);
        assert!(unlimited.has_feature("unlimited_access"));
    }

    #[test]
    fn test_permission() {
        let permission = Permission::new("tasks", "read", PermissionScope::Own);
        assert!(permission.matches("tasks", "read"));
        assert!(!permission.matches("tasks", "write"));
        assert!(!permission.matches("users", "read"));
    }

    #[test]
    fn test_privilege() {
        let privilege = Privilege::new(
            "pro_access",
            SubscriptionTier::Pro,
            Some(PermissionQuota::limited(1000, 50)),
        );

        assert!(privilege.is_available_for_tier(&SubscriptionTier::Pro));
        assert!(privilege.is_available_for_tier(&SubscriptionTier::Enterprise));
        assert!(!privilege.is_available_for_tier(&SubscriptionTier::Free));
    }

    #[test]
    fn test_permission_result() {
        let allowed = PermissionResult::allowed(None, PermissionScope::Own);
        assert!(allowed.is_allowed());
        assert!(!allowed.is_denied());

        let denied = PermissionResult::denied("Access denied");
        assert!(!denied.is_allowed());
        assert!(denied.is_denied());
    }

    #[test]
    fn test_dynamic_permissions() {
        let admin_role = RoleWithPermissions {
            id: Uuid::new_v4(),
            name: RoleName::Admin,
            display_name: "Administrator".to_string(),
            description: Some("Test admin role".to_string()),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            subscription_tier: SubscriptionTier::Enterprise,
        };

        let result = admin_role.can_perform_action("tasks", "read", None);
        assert!(result.is_allowed());

        let member_role = RoleWithPermissions {
            id: Uuid::new_v4(),
            name: RoleName::Member,
            display_name: "Member".to_string(),
            description: Some("Test member role".to_string()),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            subscription_tier: SubscriptionTier::Free,
        };

        let result = member_role.can_perform_action("tasks", "read", None);
        assert!(result.is_allowed());

        let result = member_role.can_perform_action("roles", "create", None);
        assert!(result.is_denied());
    }

    #[test]
    fn test_subscription_privileges() {
        let free_role = RoleWithPermissions {
            id: Uuid::new_v4(),
            name: RoleName::Member,
            display_name: "Free User".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            subscription_tier: SubscriptionTier::Free,
        };

        let privilege = free_role.get_subscription_privilege("tasks", "read");
        assert!(privilege.is_some());
        let privilege = privilege.unwrap();
        assert_eq!(privilege.name, "basic_task_access");
        assert_eq!(privilege.subscription_tier, SubscriptionTier::Free);

        let pro_role = RoleWithPermissions {
            id: Uuid::new_v4(),
            name: RoleName::Member,
            display_name: "Pro User".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            subscription_tier: SubscriptionTier::Pro,
        };

        let privilege = pro_role.get_subscription_privilege("tasks", "read");
        assert!(privilege.is_some());
        let privilege = privilege.unwrap();
        assert_eq!(privilege.name, "pro_task_access");
        assert!(privilege.quota.is_some());
        let quota = privilege.quota.unwrap();
        assert!(quota.has_feature("advanced_filter"));
        assert!(quota.has_feature("export"));
    }
}
