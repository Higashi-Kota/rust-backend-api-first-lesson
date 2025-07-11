// task-backend/src/utils/permission.rs

use crate::core::permission::PermissionScope;
use crate::core::subscription_tier::SubscriptionTier;
use crate::features::security::models::role::{RoleName, RoleWithPermissions};
use uuid::Uuid;

/// 統合された権限チェック機能
pub struct PermissionChecker;

impl PermissionChecker {
    /// 管理者権限があるかチェック
    pub fn is_admin(role: &RoleWithPermissions) -> bool {
        role.is_admin() && role.is_active
    }

    /// 権限スコープが要求されたスコープを満たしているかチェック
    pub fn check_scope(current_scope: &PermissionScope, required_scope: &PermissionScope) -> bool {
        // current_scopeがrequired_scope以上の権限を持っているかチェック
        current_scope.includes(required_scope)
    }

    /// 一般ユーザー権限があるかチェック
    pub fn is_member(role: &RoleWithPermissions) -> bool {
        role.is_member() && role.is_active
    }

    /// 指定されたユーザーIDにアクセス権限があるかチェック
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
    pub fn can_create_resource(role: &RoleWithPermissions, resource_type: &str) -> bool {
        if !role.is_active {
            return false;
        }

        match resource_type {
            "user" => Self::is_admin(role),
            "role" => Self::is_admin(role),
            "task" => true, // 全ロールでタスク作成可能
            "team" => true, // 全ロールでチーム作成可能
            _ => false,
        }
    }

    /// リソースの編集権限があるかチェック
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
            "team" => {
                // チームオーナーは編集可能、管理者は全チーム編集可能
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
            "team" => {
                // チームオーナーは削除可能、管理者は全チーム削除可能
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
            "team" => {
                // チームメンバーは表示可能、管理者は全チーム表示可能
                // TODO: チームメンバーかどうかの確認が必要
                if let Some(_owner) = owner_id {
                    true // 一時的に全ユーザーがチームを表示可能
                } else {
                    Self::is_admin(role)
                }
            }
            _ => false,
        }
    }

    /// 管理機能へのアクセス権限があるかチェック
    #[allow(dead_code)] // Utility method for permission checks
    pub fn can_access_admin_features(role: &RoleWithPermissions) -> bool {
        Self::is_admin(role)
            || role
                .subscription_tier
                .is_at_least(&SubscriptionTier::Enterprise)
    }

    /// 他のユーザーのデータを一覧表示する権限があるかチェック
    pub fn can_list_users(role: &RoleWithPermissions) -> bool {
        Self::is_admin(role) || role.subscription_tier.is_at_least(&SubscriptionTier::Pro)
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
    pub fn for_user(requesting_user_id: Uuid, target_user_id: Uuid) -> Self {
        Self::new(
            "user",
            requesting_user_id,
            Some(target_user_id),
            Some(target_user_id),
        )
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

        assert!(PermissionChecker::is_member(&admin_role));
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

        let task_context = ResourceContext::new("task", user_id, None, Some(target_id));
        assert_eq!(task_context.resource_type, "task");
        assert_eq!(task_context.requesting_user_id, user_id);
        assert_eq!(task_context.owner_id, Some(target_id));
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
    }
}
