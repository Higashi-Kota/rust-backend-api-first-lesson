// task-backend/src/utils/permission.rs

use crate::domain::permission::PermissionResult;
use crate::domain::role_model::{RoleName, RoleWithPermissions};
use crate::domain::subscription_tier::SubscriptionTier;
use uuid::Uuid;

/// 統合された権限チェック機能
pub struct PermissionChecker;

impl PermissionChecker {
    /// 管理者権限があるかチェック
    pub fn is_admin(role: &RoleWithPermissions) -> bool {
        role.is_admin() && role.is_active
    }

    /// 一般ユーザー権限があるかチェック
    #[allow(dead_code)]
    pub fn is_member(role: &RoleWithPermissions) -> bool {
        role.is_member() && role.is_active
    }

    /// 指定されたユーザーIDにアクセス権限があるかチェック
    #[allow(dead_code)]
    pub fn can_access_user(
        role: &RoleWithPermissions,
        requesting_user_id: Uuid,
        target_user_id: Uuid,
    ) -> bool {
        if !role.is_active {
            return false;
        }

        // 自分自身のデータには常にアクセス可能
        if requesting_user_id == target_user_id {
            return true;
        }

        // 管理者は他のユーザーのデータにもアクセス可能
        Self::is_admin(role)
    }

    /// リソースの作成権限があるかチェック
    #[allow(dead_code)]
    pub fn can_create_resource(role: &RoleWithPermissions, resource_type: &str) -> bool {
        if !role.is_active {
            return false;
        }

        match resource_type {
            "user" => Self::is_admin(role),
            "role" => Self::is_admin(role),
            "task" => true, // 全ロールでタスク作成可能
            _ => false,
        }
    }

    /// リソースの編集権限があるかチェック
    #[allow(dead_code)]
    pub fn can_update_resource(
        role: &RoleWithPermissions,
        resource_type: &str,
        owner_id: Option<Uuid>,
        requesting_user_id: Uuid,
    ) -> bool {
        if !role.is_active {
            return false;
        }

        match resource_type {
            "user" => {
                // 自分のユーザー情報は編集可能、管理者は全ユーザー編集可能
                if let Some(owner) = owner_id {
                    owner == requesting_user_id || Self::is_admin(role)
                } else {
                    Self::is_admin(role)
                }
            }
            "role" => Self::is_admin(role),
            "task" => {
                // 自分のタスクは編集可能、管理者は全タスク編集可能
                if let Some(owner) = owner_id {
                    owner == requesting_user_id || Self::is_admin(role)
                } else {
                    Self::is_admin(role)
                }
            }
            _ => false,
        }
    }

    /// リソースの削除権限があるかチェック
    #[allow(dead_code)]
    pub fn can_delete_resource(
        role: &RoleWithPermissions,
        resource_type: &str,
        owner_id: Option<Uuid>,
        requesting_user_id: Uuid,
    ) -> bool {
        if !role.is_active {
            return false;
        }

        match resource_type {
            "user" => Self::is_admin(role),
            "role" => Self::is_admin(role),
            "task" => {
                // 自分のタスクは削除可能、管理者は全タスク削除可能
                if let Some(owner) = owner_id {
                    owner == requesting_user_id || Self::is_admin(role)
                } else {
                    Self::is_admin(role)
                }
            }
            _ => false,
        }
    }

    /// リソースの表示権限があるかチェック
    #[allow(dead_code)]
    pub fn can_view_resource(
        role: &RoleWithPermissions,
        resource_type: &str,
        owner_id: Option<Uuid>,
        requesting_user_id: Uuid,
    ) -> bool {
        if !role.is_active {
            return false;
        }

        match resource_type {
            "user" => Self::can_access_user(
                role,
                requesting_user_id,
                owner_id.unwrap_or(requesting_user_id),
            ),
            "role" => Self::is_admin(role),
            "task" => {
                // 自分のタスクは表示可能、管理者は全タスク表示可能
                if let Some(owner) = owner_id {
                    owner == requesting_user_id || Self::is_admin(role)
                } else {
                    Self::is_admin(role)
                }
            }
            _ => false,
        }
    }

    /// 管理機能へのアクセス権限があるかチェック
    #[allow(dead_code)]
    pub fn can_access_admin_features(role: &RoleWithPermissions) -> bool {
        Self::is_admin(role)
            || role
                .subscription_tier
                .is_at_least(&SubscriptionTier::Enterprise)
    }

    /// 他のユーザーのデータを一覧表示する権限があるかチェック
    #[allow(dead_code)]
    pub fn can_list_users(role: &RoleWithPermissions) -> bool {
        Self::is_admin(role) || role.subscription_tier.is_at_least(&SubscriptionTier::Pro)
    }

    /// システム設定へのアクセス権限があるかチェック
    #[allow(dead_code)]
    pub fn can_access_system_settings(role: &RoleWithPermissions) -> bool {
        Self::is_admin(role)
    }

    /// 動的権限チェック（CLAUDE.md設計の実装）
    #[allow(dead_code)]
    pub fn check_dynamic_permission(
        role: &RoleWithPermissions,
        resource: &str,
        action: &str,
        target_user_id: Option<Uuid>,
    ) -> PermissionResult {
        role.can_perform_action(resource, action, target_user_id)
    }

    /// サブスクリプションベースの権限チェック
    #[allow(dead_code)]
    pub fn check_subscription_access(
        role: &RoleWithPermissions,
        required_tier: SubscriptionTier,
    ) -> bool {
        role.subscription_tier.is_at_least(&required_tier)
    }

    /// 特権ベースの機能アクセスチェック
    #[allow(dead_code)]
    pub fn check_privilege_access(
        role: &RoleWithPermissions,
        resource: &str,
        action: &str,
        feature: &str,
    ) -> bool {
        if let Some(privilege) = role.get_subscription_privilege(resource, action) {
            if let Some(quota) = privilege.quota {
                quota.has_feature(feature)
            } else {
                // クォータがない場合は無制限アクセス
                true
            }
        } else {
            false
        }
    }

    /// ロール名ベースの基本的な権限チェック（JWTから詳細ロール情報がない場合のフォールバック）
    pub fn check_permission_by_role_name(
        role_name: &str,
        permission_type: PermissionType,
        resource_context: Option<ResourceContext>,
    ) -> bool {
        let role_type = match role_name.to_lowercase().as_str() {
            "admin" => RoleName::Admin,
            "member" => RoleName::Member,
            _ => return false,
        };

        match permission_type {
            PermissionType::IsAdmin => role_type.is_admin(),
            PermissionType::IsMember => role_type.is_member(),
            PermissionType::CanAccessUser => {
                if let Some(ResourceContext {
                    requesting_user_id,
                    target_user_id: Some(target_id),
                    ..
                }) = resource_context
                {
                    requesting_user_id == target_id || role_type.is_admin()
                } else {
                    role_type.is_admin()
                }
            }
            PermissionType::CanCreateResource => {
                if let Some(ResourceContext { resource_type, .. }) = resource_context {
                    match resource_type.as_str() {
                        "user" | "role" => role_type.is_admin(),
                        "task" => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            PermissionType::CanDeleteResource => {
                if let Some(ResourceContext {
                    resource_type,
                    requesting_user_id,
                    owner_id,
                    ..
                }) = resource_context
                {
                    match resource_type.as_str() {
                        "user" | "role" => role_type.is_admin(),
                        "task" => {
                            if let Some(owner) = owner_id {
                                requesting_user_id == owner || role_type.is_admin()
                            } else {
                                role_type.is_admin()
                            }
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }
        }
    }
}

/// 権限タイプの列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PermissionType {
    IsAdmin,
    IsMember,
    CanAccessUser,
    CanCreateResource,
    CanDeleteResource,
}

/// リソースコンテキスト（権限チェック時に必要な情報）
#[derive(Debug, Clone)]
pub struct ResourceContext {
    pub resource_type: String,
    pub requesting_user_id: Uuid,
    pub target_user_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
}

impl ResourceContext {
    #[allow(dead_code)]
    pub fn new(
        resource_type: &str,
        requesting_user_id: Uuid,
        target_user_id: Option<Uuid>,
        owner_id: Option<Uuid>,
    ) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            requesting_user_id,
            target_user_id,
            owner_id,
        }
    }

    /// ユーザーリソース用のコンテキストを作成
    #[allow(dead_code)]
    pub fn for_user(requesting_user_id: Uuid, target_user_id: Uuid) -> Self {
        Self::new(
            "user",
            requesting_user_id,
            Some(target_user_id),
            Some(target_user_id),
        )
    }

    /// タスクリソース用のコンテキストを作成
    #[allow(dead_code)]
    pub fn for_task(requesting_user_id: Uuid, task_owner_id: Option<Uuid>) -> Self {
        Self::new("task", requesting_user_id, None, task_owner_id)
    }

    /// ロールリソース用のコンテキストを作成
    #[allow(dead_code)]
    pub fn for_role(requesting_user_id: Uuid) -> Self {
        Self::new("role", requesting_user_id, None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_admin_role() -> RoleWithPermissions {
        RoleWithPermissions {
            id: Uuid::new_v4(),
            name: RoleName::Admin,
            display_name: "Administrator".to_string(),
            description: Some("Test admin role".to_string()),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            subscription_tier: SubscriptionTier::Enterprise,
        }
    }

    fn create_test_member_role() -> RoleWithPermissions {
        RoleWithPermissions {
            id: Uuid::new_v4(),
            name: RoleName::Member,
            display_name: "Member".to_string(),
            description: Some("Test member role".to_string()),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            subscription_tier: SubscriptionTier::Free,
        }
    }

    fn create_test_inactive_role() -> RoleWithPermissions {
        RoleWithPermissions {
            id: Uuid::new_v4(),
            name: RoleName::Admin,
            display_name: "Inactive Admin".to_string(),
            description: Some("Test inactive admin role".to_string()),
            is_active: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            subscription_tier: SubscriptionTier::Enterprise,
        }
    }

    fn create_test_pro_member_role() -> RoleWithPermissions {
        RoleWithPermissions {
            id: Uuid::new_v4(),
            name: RoleName::Member,
            display_name: "Pro Member".to_string(),
            description: Some("Test pro member role".to_string()),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            subscription_tier: SubscriptionTier::Pro,
        }
    }

    #[test]
    fn test_is_admin() {
        let admin_role = create_test_admin_role();
        let member_role = create_test_member_role();
        let inactive_role = create_test_inactive_role();

        assert!(PermissionChecker::is_admin(&admin_role));
        assert!(!PermissionChecker::is_admin(&member_role));
        assert!(!PermissionChecker::is_admin(&inactive_role));
    }

    #[test]
    fn test_is_member() {
        let admin_role = create_test_admin_role();
        let member_role = create_test_member_role();
        let inactive_role = create_test_inactive_role();

        assert!(!PermissionChecker::is_member(&admin_role));
        assert!(PermissionChecker::is_member(&member_role));
        assert!(!PermissionChecker::is_member(&inactive_role));
    }

    #[test]
    fn test_can_access_user() {
        let admin_role = create_test_admin_role();
        let member_role = create_test_member_role();
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();

        // 管理者は他のユーザーにアクセス可能
        assert!(PermissionChecker::can_access_user(
            &admin_role,
            user_id,
            other_user_id
        ));

        // 一般ユーザーは自分自身にのみアクセス可能
        assert!(PermissionChecker::can_access_user(
            &member_role,
            user_id,
            user_id
        ));
        assert!(!PermissionChecker::can_access_user(
            &member_role,
            user_id,
            other_user_id
        ));
    }

    #[test]
    fn test_can_create_resource() {
        let admin_role = create_test_admin_role();
        let member_role = create_test_member_role();

        // 管理者は全リソース作成可能
        assert!(PermissionChecker::can_create_resource(&admin_role, "user"));
        assert!(PermissionChecker::can_create_resource(&admin_role, "role"));
        assert!(PermissionChecker::can_create_resource(&admin_role, "task"));

        // 一般ユーザーはタスクのみ作成可能
        assert!(!PermissionChecker::can_create_resource(
            &member_role,
            "user"
        ));
        assert!(!PermissionChecker::can_create_resource(
            &member_role,
            "role"
        ));
        assert!(PermissionChecker::can_create_resource(&member_role, "task"));
    }

    #[test]
    fn test_can_delete_resource() {
        let admin_role = create_test_admin_role();
        let member_role = create_test_member_role();
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();

        // 管理者は全リソース削除可能
        assert!(PermissionChecker::can_delete_resource(
            &admin_role,
            "user",
            Some(other_user_id),
            user_id
        ));
        assert!(PermissionChecker::can_delete_resource(
            &admin_role,
            "role",
            None,
            user_id
        ));
        assert!(PermissionChecker::can_delete_resource(
            &admin_role,
            "task",
            Some(other_user_id),
            user_id
        ));

        // 一般ユーザーは自分のタスクのみ削除可能
        assert!(!PermissionChecker::can_delete_resource(
            &member_role,
            "user",
            Some(user_id),
            user_id
        ));
        assert!(!PermissionChecker::can_delete_resource(
            &member_role,
            "role",
            None,
            user_id
        ));
        assert!(PermissionChecker::can_delete_resource(
            &member_role,
            "task",
            Some(user_id),
            user_id
        ));
        assert!(!PermissionChecker::can_delete_resource(
            &member_role,
            "task",
            Some(other_user_id),
            user_id
        ));
    }

    #[test]
    fn test_permission_by_role_name() {
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();

        // 管理者ロール名での権限チェック
        assert!(PermissionChecker::check_permission_by_role_name(
            "admin",
            PermissionType::IsAdmin,
            None
        ));

        assert!(PermissionChecker::check_permission_by_role_name(
            "admin",
            PermissionType::CanAccessUser,
            Some(ResourceContext::for_user(user_id, other_user_id))
        ));

        // 一般ユーザーロール名での権限チェック
        assert!(!PermissionChecker::check_permission_by_role_name(
            "member",
            PermissionType::IsAdmin,
            None
        ));

        assert!(!PermissionChecker::check_permission_by_role_name(
            "member",
            PermissionType::CanAccessUser,
            Some(ResourceContext::for_user(user_id, other_user_id))
        ));

        assert!(PermissionChecker::check_permission_by_role_name(
            "member",
            PermissionType::CanAccessUser,
            Some(ResourceContext::for_user(user_id, user_id))
        ));
    }

    #[test]
    fn test_resource_context() {
        let user_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();

        let context = ResourceContext::for_user(user_id, target_id);
        assert_eq!(context.resource_type, "user");
        assert_eq!(context.requesting_user_id, user_id);
        assert_eq!(context.target_user_id, Some(target_id));

        let task_context = ResourceContext::for_task(user_id, Some(target_id));
        assert_eq!(task_context.resource_type, "task");
        assert_eq!(task_context.requesting_user_id, user_id);
        assert_eq!(task_context.owner_id, Some(target_id));
    }

    #[test]
    fn test_dynamic_permission_check() {
        let admin_role = create_test_admin_role();
        let member_role = create_test_member_role();
        let pro_member = create_test_pro_member_role();

        // Admin can access all tasks
        let result =
            PermissionChecker::check_dynamic_permission(&admin_role, "tasks", "read", None);
        assert!(result.is_allowed());

        // Member can access own tasks
        let result =
            PermissionChecker::check_dynamic_permission(&member_role, "tasks", "read", None);
        assert!(result.is_allowed());

        // Member cannot access role management
        let result =
            PermissionChecker::check_dynamic_permission(&member_role, "roles", "create", None);
        assert!(result.is_denied());

        // Pro member has enhanced features
        let result =
            PermissionChecker::check_dynamic_permission(&pro_member, "tasks", "read", None);
        assert!(result.is_allowed());
    }

    #[test]
    fn test_subscription_access() {
        let admin_role = create_test_admin_role();
        let member_role = create_test_member_role();
        let pro_member = create_test_pro_member_role();

        // Enterprise tier access
        assert!(PermissionChecker::check_subscription_access(
            &admin_role,
            SubscriptionTier::Enterprise
        ));
        assert!(!PermissionChecker::check_subscription_access(
            &member_role,
            SubscriptionTier::Enterprise
        ));
        assert!(!PermissionChecker::check_subscription_access(
            &pro_member,
            SubscriptionTier::Enterprise
        ));

        // Pro tier access
        assert!(PermissionChecker::check_subscription_access(
            &admin_role,
            SubscriptionTier::Pro
        ));
        assert!(!PermissionChecker::check_subscription_access(
            &member_role,
            SubscriptionTier::Pro
        ));
        assert!(PermissionChecker::check_subscription_access(
            &pro_member,
            SubscriptionTier::Pro
        ));

        // Free tier access
        assert!(PermissionChecker::check_subscription_access(
            &admin_role,
            SubscriptionTier::Free
        ));
        assert!(PermissionChecker::check_subscription_access(
            &member_role,
            SubscriptionTier::Free
        ));
        assert!(PermissionChecker::check_subscription_access(
            &pro_member,
            SubscriptionTier::Free
        ));
    }

    #[test]
    fn test_privilege_access() {
        let pro_member = create_test_pro_member_role();
        let free_member = create_test_member_role();

        // Pro member has advanced features for task reading
        assert!(PermissionChecker::check_privilege_access(
            &pro_member,
            "tasks",
            "read",
            "advanced_filter"
        ));
        assert!(PermissionChecker::check_privilege_access(
            &pro_member,
            "tasks",
            "read",
            "export"
        ));
        assert!(!PermissionChecker::check_privilege_access(
            &pro_member,
            "tasks",
            "read",
            "unlimited_access"
        ));

        // Free member has basic features only
        assert!(!PermissionChecker::check_privilege_access(
            &free_member,
            "tasks",
            "read",
            "advanced_filter"
        ));
        assert!(!PermissionChecker::check_privilege_access(
            &free_member,
            "tasks",
            "read",
            "export"
        ));
        assert!(PermissionChecker::check_privilege_access(
            &free_member,
            "tasks",
            "read",
            "basic_access"
        ));
    }

    #[test]
    fn test_enhanced_admin_features() {
        let admin_role = create_test_admin_role();
        let pro_member = create_test_pro_member_role();
        let free_member = create_test_member_role();

        // Admin features (now includes Enterprise subscription)
        assert!(PermissionChecker::can_access_admin_features(&admin_role));
        assert!(!PermissionChecker::can_access_admin_features(&pro_member));
        assert!(!PermissionChecker::can_access_admin_features(&free_member));

        // User listing (now includes Pro subscription)
        assert!(PermissionChecker::can_list_users(&admin_role));
        assert!(PermissionChecker::can_list_users(&pro_member));
        assert!(!PermissionChecker::can_list_users(&free_member));

        // System settings (still admin only)
        assert!(PermissionChecker::can_access_system_settings(&admin_role));
        assert!(!PermissionChecker::can_access_system_settings(&pro_member));
        assert!(!PermissionChecker::can_access_system_settings(&free_member));
    }
}
