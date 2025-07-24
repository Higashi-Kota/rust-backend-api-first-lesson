// task-backend/src/middleware/authorization.rs

use crate::domain::role_model::RoleWithPermissions;
use crate::utils::error_helper::{forbidden_error, unauthorized_error};
use crate::utils::permission::{PermissionChecker, ResourceContext};
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

/// 統一権限チェックミドルウェア
#[derive(Clone)]
pub struct RequirePermission {
    pub resource: &'static str,
    pub action: Action,
}

/// リソースに対するアクション
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    View,
    Create,
    Update,
    Delete,
    Admin,
}

impl RequirePermission {
    pub fn new(resource: &'static str, action: Action) -> Self {
        Self { resource, action }
    }
}

/// リソース名定数
pub mod resources {
    pub const TASK: &str = "task";
    pub const TEAM: &str = "team";
    pub const USER: &str = "user";
    pub const ORGANIZATION: &str = "organization";
    pub const ROLE: &str = "role";
    pub const ANALYTICS: &str = "analytics";
    pub const PAYMENT: &str = "payment";
    pub const SUBSCRIPTION: &str = "subscription";
    pub const AUDIT_LOG: &str = "audit_log";
    pub const GDPR: &str = "gdpr";
    pub const INVITATION: &str = "invitation";
    pub const SECURITY: &str = "security";
}

/// 権限チェックミドルウェアマクロ
#[macro_export]
macro_rules! require_permission {
    ($resource:expr, $action:expr) => {{
        use axum::middleware::from_fn_with_state;
        use $crate::middleware::authorization::{check_permission_with_state, RequirePermission};

        let permission = RequirePermission::new($resource, $action);
        from_fn_with_state(permission, check_permission_with_state)
    }};
}

/// 管理者権限チェック用のヘルパー関数
pub fn admin_permission_middleware() -> impl Fn(
    Request,
    Next,
)
    -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Response>> + Send>>
       + Clone
       + Send
       + 'static {
    permission_middleware(resources::ROLE, Action::Admin)
}

/// 状態を持つ権限チェックミドルウェア関数
pub async fn check_permission_with_state(
    axum::extract::State(permission): axum::extract::State<RequirePermission>,
    req: Request,
    next: Next,
) -> Result<Response, Response> {
    check_permission(permission, req, next).await
}

/// 権限チェックミドルウェア用のヘルパー関数
pub fn permission_middleware(
    resource: &'static str,
    action: Action,
) -> impl Fn(
    Request,
    Next,
)
    -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Response>> + Send>>
       + Clone
       + Send
       + 'static {
    move |req, next| {
        let permission = RequirePermission::new(resource, action);
        Box::pin(check_permission(permission, req, next))
    }
}

/// 権限チェックミドルウェア関数
pub async fn check_permission(
    permission: RequirePermission,
    mut req: Request,
    next: Next,
) -> Result<Response, Response> {
    // 認証ユーザー情報の取得
    let auth_user = req
        .extensions()
        .get::<crate::middleware::auth::AuthenticatedUserWithRole>()
        .cloned();

    let auth_user = match auth_user {
        Some(user) => user,
        None => {
            return Err(unauthorized_error(
                "No authenticated user found",
                "authorization::check_permission",
                "Authentication required",
            )
            .into_response());
        }
    };

    // リソース固有のコンテキスト取得
    let context = extract_resource_context(&req, &auth_user.claims.user_id, permission.resource);

    // 権限チェック実行
    // ロール情報が無い場合はエラー
    let role = match &auth_user.claims.role {
        Some(role) => role,
        None => {
            return Err(unauthorized_error(
                "No role information in token",
                "authorization::check_permission",
                "Authorization information is incomplete",
            )
            .into_response());
        }
    };

    let has_permission = check_resource_permission(
        role,
        permission.resource,
        permission.action,
        auth_user.claims.user_id,
        context.as_ref(),
    );

    if !has_permission {
        return Err(forbidden_error(
            &format!(
                "User {} does not have {:?} permission for resource {}",
                auth_user.claims.user_id, permission.action, permission.resource
            ),
            &format!("authorization::check_permission::{}", permission.resource),
            "You don't have permission to perform this action",
        )
        .into_response());
    }

    // 権限情報を拡張として追加
    req.extensions_mut().insert(PermissionContext {
        user_id: auth_user.claims.user_id,
        role: role.clone(),
        resource: permission.resource,
        action: permission.action,
    });

    Ok(next.run(req).await)
}

/// リソース固有のコンテキストを抽出
fn extract_resource_context(
    req: &Request,
    requesting_user_id: &Uuid,
    resource: &str,
) -> Option<ResourceContext> {
    // パスパラメータからリソースIDを取得
    let path = req.uri().path();

    match resource {
        resources::TASK => {
            // /tasks/{task_id} or /teams/{team_id}/tasks/{task_id}
            if let Some(task_id) = extract_uuid_from_path(path, "tasks") {
                // タスク所有者の取得はサービス層で行う
                return Some(ResourceContext::new(
                    resource,
                    *requesting_user_id,
                    None,
                    Some(task_id),
                ));
            }
        }
        resources::TEAM => {
            // /teams/{team_id}
            if let Some(team_id) = extract_uuid_from_path(path, "teams") {
                return Some(ResourceContext::new(
                    resource,
                    *requesting_user_id,
                    None,
                    Some(team_id),
                ));
            }
        }
        resources::USER => {
            // /users/{user_id}
            if let Some(user_id) = extract_uuid_from_path(path, "users") {
                return Some(ResourceContext::for_user(*requesting_user_id, user_id));
            }
        }
        resources::ORGANIZATION => {
            // /organizations/{organization_id}
            if let Some(org_id) = extract_uuid_from_path(path, "organizations") {
                return Some(ResourceContext::new(
                    resource,
                    *requesting_user_id,
                    None,
                    Some(org_id),
                ));
            }
        }
        _ => {}
    }

    None
}

/// パスからUUIDを抽出するヘルパー関数
fn extract_uuid_from_path(path: &str, resource_name: &str) -> Option<Uuid> {
    let segments: Vec<&str> = path.split('/').collect();

    for (i, segment) in segments.iter().enumerate() {
        if *segment == resource_name && i + 1 < segments.len() {
            if let Ok(uuid) = segments[i + 1].parse::<Uuid>() {
                return Some(uuid);
            }
        }
    }

    None
}

/// リソースと権限の組み合わせをチェック
fn check_resource_permission(
    role: &RoleWithPermissions,
    resource: &str,
    action: Action,
    requesting_user_id: Uuid,
    context: Option<&ResourceContext>,
) -> bool {
    // 管理者は全権限を持つ
    if PermissionChecker::is_admin(role) {
        return true;
    }

    match (resource, action) {
        // タスク権限
        (resources::TASK, Action::View) => {
            if let Some(ctx) = context {
                PermissionChecker::can_view_resource(
                    role,
                    resource,
                    ctx.owner_id,
                    requesting_user_id,
                )
            } else {
                true // タスク一覧は全員が閲覧可能
            }
        }
        (resources::TASK, Action::Create) => PermissionChecker::can_create_resource(role, resource),
        (resources::TASK, Action::Update) => {
            if let Some(ctx) = context {
                PermissionChecker::can_update_resource(
                    role,
                    resource,
                    ctx.owner_id,
                    requesting_user_id,
                )
            } else {
                false
            }
        }
        (resources::TASK, Action::Delete) => {
            if let Some(ctx) = context {
                PermissionChecker::can_delete_resource(
                    role,
                    resource,
                    ctx.owner_id,
                    requesting_user_id,
                )
            } else {
                false
            }
        }

        // チーム権限
        (resources::TEAM, Action::View) => true, // 誰でも閲覧可能（詳細はサービス層でチェック）
        (resources::TEAM, Action::Create) => PermissionChecker::can_create_resource(role, resource),
        (resources::TEAM, Action::Update | Action::Delete) => {
            // チームメンバーシップは非同期でチェックする必要があるため、
            // ミドルウェアでは基本的な権限のみチェックし、
            // 詳細な権限チェックはサービス層で実施
            // メンバー以上のロールを持っているユーザーのみ許可
            matches!(
                role.name,
                crate::domain::role_model::RoleName::Member
                    | crate::domain::role_model::RoleName::Admin
            )
        }

        // ユーザー権限
        (resources::USER, Action::View) => {
            if let Some(ctx) = context {
                PermissionChecker::can_access_user(
                    role,
                    requesting_user_id,
                    ctx.target_user_id.unwrap_or(requesting_user_id),
                )
            } else {
                PermissionChecker::can_list_users(role)
            }
        }
        (resources::USER, Action::Create) => PermissionChecker::can_create_resource(role, resource),
        (resources::USER, Action::Update) => {
            if let Some(ctx) = context {
                PermissionChecker::can_update_resource(
                    role,
                    resource,
                    ctx.target_user_id,
                    requesting_user_id,
                )
            } else {
                false
            }
        }
        (resources::USER, Action::Delete) => {
            PermissionChecker::can_delete_resource(role, resource, None, requesting_user_id)
        }

        // 組織権限
        (resources::ORGANIZATION, Action::View) => true, // 誰でも閲覧可能（詳細はサービス層でチェック）
        (resources::ORGANIZATION, Action::Create) => {
            // メンバー以上のロールが組織を作成可能
            PermissionChecker::can_create_resource(role, resource)
        }
        (resources::ORGANIZATION, Action::Update | Action::Delete) => {
            // 組織メンバーシップは非同期でチェックする必要があるため、
            // ミドルウェアでは基本的な権限のみチェックし、
            // 詳細な権限チェックはサービス層で実施
            // メンバー以上のロールを持っているユーザーのみ許可
            matches!(
                role.name,
                crate::domain::role_model::RoleName::Member
                    | crate::domain::role_model::RoleName::Admin
            )
        }

        // 管理者専用リソース
        (
            resources::ROLE | resources::ANALYTICS | resources::PAYMENT | resources::SUBSCRIPTION,
            _,
        ) => PermissionChecker::is_admin(role),

        // デフォルト: 権限なし
        _ => false,
    }
}

/// 権限コンテキスト（ミドルウェア処理後に利用可能）
#[derive(Clone, Debug)]
#[allow(dead_code)] // Public API for downstream consumers
pub struct PermissionContext {
    pub user_id: Uuid,
    pub role: RoleWithPermissions,
    pub resource: &'static str,
    pub action: Action,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::role_model::RoleName;
    use crate::domain::subscription_tier::SubscriptionTier;
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

    #[test]
    fn test_extract_uuid_from_path() {
        let task_id = Uuid::new_v4();
        let path = format!("/tasks/{}", task_id);
        assert_eq!(extract_uuid_from_path(&path, "tasks"), Some(task_id));

        let team_id = Uuid::new_v4();
        let path = format!("/teams/{}/tasks", team_id);
        assert_eq!(extract_uuid_from_path(&path, "teams"), Some(team_id));

        let path = "/tasks";
        assert_eq!(extract_uuid_from_path(path, "tasks"), None);
    }

    #[test]
    fn test_check_resource_permission_admin() {
        let admin_role = create_test_admin_role();
        let user_id = Uuid::new_v4();

        // 管理者は全権限を持つ
        assert!(check_resource_permission(
            &admin_role,
            resources::TASK,
            Action::Create,
            user_id,
            None
        ));
        assert!(check_resource_permission(
            &admin_role,
            resources::TEAM,
            Action::Delete,
            user_id,
            None
        ));
        assert!(check_resource_permission(
            &admin_role,
            resources::USER,
            Action::Update,
            user_id,
            None
        ));
    }

    #[test]
    fn test_check_resource_permission_member() {
        let member_role = create_test_member_role();
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();

        // メンバーはタスク作成可能
        assert!(check_resource_permission(
            &member_role,
            resources::TASK,
            Action::Create,
            user_id,
            None
        ));

        // メンバーは自分のタスクのみ編集可能
        let context = ResourceContext::new(resources::TASK, user_id, None, Some(user_id));
        assert!(check_resource_permission(
            &member_role,
            resources::TASK,
            Action::Update,
            user_id,
            Some(&context)
        ));

        // メンバーは他人のタスクを編集不可
        let context = ResourceContext::new(resources::TASK, user_id, None, Some(other_user_id));
        assert!(!check_resource_permission(
            &member_role,
            resources::TASK,
            Action::Update,
            user_id,
            Some(&context)
        ));

        // メンバーはロール管理不可
        assert!(!check_resource_permission(
            &member_role,
            resources::ROLE,
            Action::Create,
            user_id,
            None
        ));
    }
}
