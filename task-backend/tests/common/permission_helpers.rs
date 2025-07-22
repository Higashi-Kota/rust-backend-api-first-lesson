use axum::body::Body;
use axum::http::{Request, StatusCode};
use sea_orm::ActiveValue;
use task_backend::domain::user_model::UserActiveModel;
use task_backend::middleware::authorization::{Action, PermissionContext};
use tower::ServiceExt;
use uuid::Uuid;

/// テスト用のPermissionContextを作成するヘルパー関数
pub fn create_test_permission_context(
    user_id: Uuid,
    is_admin: bool,
    resource: &str,
    action: Action,
) -> PermissionContext {
    PermissionContext::new(user_id, is_admin, resource.to_string(), action)
}

/// モックPermissionContextを作成するヘルパー関数（管理者用）
pub fn create_admin_permission_context(user_id: Uuid) -> PermissionContext {
    PermissionContext::new(user_id, true, "admin".to_string(), Action::Admin)
}

/// モックPermissionContextを作成するヘルパー関数（一般ユーザー用）
pub fn create_user_permission_context(
    user_id: Uuid,
    resource: &str,
    action: Action,
) -> PermissionContext {
    PermissionContext::new(user_id, false, resource.to_string(), action)
}

/// 権限チェックをテストするためのリクエストビルダー
pub fn create_request_with_permission_context(
    method: &str,
    path: &str,
    context: PermissionContext,
    body: impl serde::Serialize,
) -> Request<Body> {
    let body_bytes = serde_json::to_vec(&body).unwrap();

    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json")
        .body(Body::from(body_bytes))
        .unwrap();

    // PermissionContextをリクエストの拡張に追加
    request.extensions_mut().insert(context);

    request
}

/// 権限エラーレスポンスを検証するヘルパー関数
pub async fn assert_permission_denied(response: axum::response::Response) {
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(error_response["error"]
        .as_str()
        .unwrap()
        .contains("Permission denied"));
}

/// 管理者権限が必要なエンドポイントをテストするヘルパー関数
pub async fn test_admin_only_endpoint<F>(
    app: axum::Router,
    path: &str,
    method: &str,
    create_request_body: F,
) where
    F: Fn() -> serde_json::Value,
{
    // 管理者ユーザーでのテスト
    let admin_id = Uuid::new_v4();
    let admin_context = create_admin_permission_context(admin_id);
    let admin_request =
        create_request_with_permission_context(method, path, admin_context, create_request_body());

    let admin_response = app.clone().oneshot(admin_request).await.unwrap();
    assert_ne!(admin_response.status(), StatusCode::FORBIDDEN);

    // 一般ユーザーでのテスト（アクセス拒否を期待）
    let user_id = Uuid::new_v4();
    let user_context = create_user_permission_context(user_id, "admin", Action::View);
    let user_request =
        create_request_with_permission_context(method, path, user_context, create_request_body());

    let user_response = app.oneshot(user_request).await.unwrap();
    assert_permission_denied(user_response).await;
}

/// リソースベース権限チェックをテストするヘルパー関数
pub async fn test_resource_based_permission<F>(
    app: axum::Router,
    path: &str,
    method: &str,
    resource: &str,
    action: Action,
    owner_id: Uuid,
    create_request_body: F,
) where
    F: Fn() -> serde_json::Value,
{
    // リソースの所有者でのテスト（アクセス許可を期待）
    let owner_context = create_user_permission_context(owner_id, resource, action.clone());
    let owner_request =
        create_request_with_permission_context(method, path, owner_context, create_request_body());

    let owner_response = app.clone().oneshot(owner_request).await.unwrap();
    assert_ne!(owner_response.status(), StatusCode::FORBIDDEN);

    // 他のユーザーでのテスト（アクセス拒否を期待）
    let other_user_id = Uuid::new_v4();
    let other_context = create_user_permission_context(other_user_id, resource, action.clone());
    let other_request =
        create_request_with_permission_context(method, path, other_context, create_request_body());

    let other_response = app.clone().oneshot(other_request).await.unwrap();
    assert_permission_denied(other_response).await;

    // 管理者でのテスト（アクセス許可を期待）
    let admin_id = Uuid::new_v4();
    let admin_context = create_test_permission_context(admin_id, true, resource, action);
    let admin_request =
        create_request_with_permission_context(method, path, admin_context, create_request_body());

    let admin_response = app.oneshot(admin_request).await.unwrap();
    assert_ne!(admin_response.status(), StatusCode::FORBIDDEN);
}

/// テスト用のユーザーを作成し、適切な権限コンテキストを設定するヘルパー
pub struct TestUserWithPermission {
    pub user_id: Uuid,
    pub email: String,
    pub is_admin: bool,
    pub context: PermissionContext,
}

impl TestUserWithPermission {
    pub fn new_admin() -> Self {
        let user_id = Uuid::new_v4();
        let email = format!("admin_{}@test.com", user_id);
        let context = create_admin_permission_context(user_id);

        Self {
            user_id,
            email,
            is_admin: true,
            context,
        }
    }

    pub fn new_user(resource: &str, action: Action) -> Self {
        let user_id = Uuid::new_v4();
        let email = format!("user_{}@test.com", user_id);
        let context = create_user_permission_context(user_id, resource, action);

        Self {
            user_id,
            email,
            is_admin: false,
            context,
        }
    }

    pub fn to_active_model(&self) -> UserActiveModel {
        UserActiveModel {
            id: ActiveValue::Set(self.user_id),
            email: ActiveValue::Set(self.email.clone()),
            is_admin: ActiveValue::Set(self.is_admin),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_permission_contexts() {
        let user_id = Uuid::new_v4();

        // 管理者コンテキストのテスト
        let admin_context = create_admin_permission_context(user_id);
        assert_eq!(admin_context.user_id, user_id);
        assert!(admin_context.is_admin);
        assert_eq!(admin_context.resource, "admin");
        assert_eq!(admin_context.action, Action::Admin);

        // 一般ユーザーコンテキストのテスト
        let user_context = create_user_permission_context(user_id, "task", Action::View);
        assert_eq!(user_context.user_id, user_id);
        assert!(!user_context.is_admin);
        assert_eq!(user_context.resource, "task");
        assert_eq!(user_context.action, Action::View);
    }

    #[test]
    fn test_test_user_creation() {
        let admin_user = TestUserWithPermission::new_admin();
        assert!(admin_user.is_admin);
        assert!(admin_user.context.is_admin);

        let normal_user = TestUserWithPermission::new_user("team", Action::Create);
        assert!(!normal_user.is_admin);
        assert!(!normal_user.context.is_admin);
        assert_eq!(normal_user.context.resource, "team");
        assert_eq!(normal_user.context.action, Action::Create);
    }
}
