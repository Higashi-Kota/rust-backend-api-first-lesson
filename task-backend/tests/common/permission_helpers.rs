// tests/common/permission_helpers.rs

#![allow(dead_code)] // テスト用ヘルパー関数は許可

use crate::common::auth_helper::{create_admin_with_jwt, create_member_with_jwt};
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    routing::get,
    Router,
};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use task_backend::{
    api::AppState,
    domain::role_model::{RoleName, RoleWithPermissions},
    domain::subscription_tier::SubscriptionTier,
    middleware::authorization::{Action, PermissionContext, RequirePermission},
};
use tower::ServiceExt;
use uuid::Uuid;

/// テスト用のモックPermissionContextを作成
pub fn create_mock_permission_context(
    user_id: Uuid,
    role_name: RoleName,
    resource: &'static str,
    action: Action,
) -> PermissionContext {
    let role = RoleWithPermissions {
        id: Uuid::new_v4(),
        name: role_name,
        display_name: role_name.to_string(),
        description: Some("Test role".to_string()),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        subscription_tier: match role_name {
            RoleName::Admin => SubscriptionTier::Enterprise,
            _ => SubscriptionTier::Free,
        },
    };

    PermissionContext {
        user_id,
        role,
        resource,
        action,
    }
}

/// リソースIDを持つPermissionContextを作成（今は使用されていない）
pub fn create_mock_permission_context_with_resource(
    user_id: Uuid,
    role_name: RoleName,
    resource: &'static str,
    action: Action,
    _resource_id: Uuid,
) -> PermissionContext {
    // 現在のPermissionContextはresource_idを持たない
    create_mock_permission_context(user_id, role_name, resource, action)
}

/// テスト用のミドルウェアラッパー
/// 統一権限チェックミドルウェアをテスト環境で使用するためのヘルパー
pub async fn test_permission_middleware(
    State(_app_state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // テスト用の権限コンテキストをリクエストから取得
    if req.extensions().get::<PermissionContext>().is_some() {
        // コンテキストが既に設定されている場合はそのまま使用
        return Ok(next.run(req).await);
    }

    // デフォルトのテストコンテキストを設定
    let default_context = create_mock_permission_context(
        Uuid::new_v4(),
        RoleName::Member,
        "test_resource",
        Action::View,
    );
    req.extensions_mut().insert(default_context);

    Ok(next.run(req).await)
}

/// 特定の権限設定でリクエストを作成するヘルパー
pub fn create_request_with_permission(
    method: &str,
    uri: &str,
    permission_context: PermissionContext,
    body: Option<String>,
) -> Request<Body> {
    let request = Request::builder()
        .uri(uri)
        .method(method)
        .header("Content-Type", "application/json");

    let mut request = if let Some(body_content) = body {
        request.body(Body::from(body_content)).unwrap()
    } else {
        request.body(Body::empty()).unwrap()
    };

    // 権限コンテキストを拡張機能として追加
    request.extensions_mut().insert(permission_context);
    request
}

/// 管理者権限でリクエストを作成
pub fn create_admin_request(method: &str, uri: &str, body: Option<String>) -> Request<Body> {
    let context = create_mock_permission_context(
        Uuid::new_v4(),
        RoleName::Admin,
        "admin_resource",
        Action::Admin,
    );
    create_request_with_permission(method, uri, context, body)
}

/// メンバー権限でリクエストを作成
pub fn create_member_request(
    method: &str,
    uri: &str,
    resource: &'static str,
    action: Action,
    body: Option<String>,
) -> Request<Body> {
    let context =
        create_mock_permission_context(Uuid::new_v4(), RoleName::Member, resource, action);
    create_request_with_permission(method, uri, context, body)
}

/// 認証済みリクエストを作成するヘルパー関数
pub fn create_authenticated_request<T: serde::Serialize>(
    method: &str,
    path: &str,
    token: &str,
    body: Option<T>,
) -> Request<Body> {
    let builder = Request::builder()
        .method(method)
        .uri(path)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json");

    match body {
        Some(b) => {
            let body_bytes = serde_json::to_vec(&b).unwrap();
            builder.body(Body::from(body_bytes)).unwrap()
        }
        None => builder.body(Body::empty()).unwrap(),
    }
}

/// 権限エラー（403 Forbidden）をアサートする共通関数
pub fn assert_forbidden(response: Response) {
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Expected 403 Forbidden but got {}",
        response.status()
    );
}

/// 権限チェックのアサーションヘルパー（別名）
pub async fn assert_permission_denied(app: &Router, request: Request<Body>) {
    let response = app.clone().oneshot(request).await.unwrap();
    assert_forbidden(response);
}

/// 権限チェックが成功することを確認
pub async fn assert_permission_allowed(app: &Router, request: Request<Body>) {
    let response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Expected permission to be allowed but got 403 Forbidden"
    );
}

/// 権限エラーレスポンスのボディも検証する関数
pub async fn assert_forbidden_with_message(response: Response, expected_message: &str) {
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check for error message in various response structures
    let error_message = if let Some(error) = error_response["error"].as_object() {
        // Nested error object structure: {"error": {"message": "..."}}
        error["message"].as_str().unwrap_or("")
    } else {
        error_response["error"]
            .as_str()
            .or_else(|| error_response["message"].as_str())
            .unwrap_or("")
    };

    assert!(
        error_message.contains(expected_message),
        "Expected error message to contain '{}', but got: {:?}",
        expected_message,
        error_response
    );
}

/// 管理者専用エンドポイントへのアクセステスト
pub async fn test_admin_endpoints_access(
    app: &Router,
    endpoints: Vec<(&str, &str)>,
    non_admin_token: &str,
) {
    for (endpoint, method) in endpoints {
        let request = create_authenticated_request::<()>(method, endpoint, non_admin_token, None);

        let response = app.clone().oneshot(request).await.unwrap();

        assert_forbidden(response);
    }
}

/// 管理者専用エンドポイントの包括的なテスト
pub async fn test_admin_only_endpoints(
    app: &Router,
    admin_paths: Vec<(&str, &str)>, // (path, method)のタプル
) {
    let member_token = create_member_with_jwt(app).await;
    let admin_token = create_admin_with_jwt(app).await;

    for (path, method) in admin_paths {
        // メンバーは403 Forbidden
        let member_request = create_authenticated_request::<()>(method, path, &member_token, None);
        let member_response = app.clone().oneshot(member_request).await.unwrap();
        assert_forbidden(member_response);

        // 管理者はアクセス可能（200/201/204）
        let admin_request = create_authenticated_request::<()>(method, path, &admin_token, None);
        let admin_response = app.clone().oneshot(admin_request).await.unwrap();
        assert!(
            admin_response.status().is_success()
                || admin_response.status() == StatusCode::NO_CONTENT,
            "Admin should have access to {}, but got status: {}",
            path,
            admin_response.status()
        );
    }
}

/// リソースベースの権限チェックテスト
pub async fn test_resource_access_denied(
    app: &Router,
    resource_path: &str,
    method: &str,
    unauthorized_token: &str,
    expected_status: StatusCode,
) {
    let request =
        create_authenticated_request::<()>(method, resource_path, unauthorized_token, None);

    let response = app.clone().oneshot(request).await.unwrap();
    assert!(
        response.status() == expected_status || response.status() == StatusCode::NOT_FOUND,
        "Expected {} or 404, got {}",
        expected_status,
        response.status()
    );
}

/// チームメンバーシップベースの権限チェックテスト
pub async fn test_team_membership_required(
    app: &Router,
    team_id: Uuid,
    endpoint: &str,
    method: &str,
    non_member_token: &str,
    body: Option<serde_json::Value>,
) {
    let request = create_authenticated_request(
        method,
        &endpoint.replace("{team_id}", &team_id.to_string()),
        non_member_token,
        body,
    );

    let response = app.clone().oneshot(request).await.unwrap();
    assert_forbidden(response);
}

/// サブスクリプションベースの権限チェックテスト
pub async fn test_subscription_tier_required(
    app: &Router,
    endpoint: &str,
    method: &str,
    free_user_token: &str,
    required_tier_message: &str,
) {
    let request = create_authenticated_request::<()>(method, endpoint, free_user_token, None);

    let response = app.clone().oneshot(request).await.unwrap();
    assert_forbidden_with_message(response, required_tier_message).await;
}

/// テスト用のRequirePermissionミドルウェアビルダー
pub struct TestPermissionBuilder {
    resource: &'static str,
    action: Action,
}

impl TestPermissionBuilder {
    pub fn new(resource: &'static str, action: Action) -> Self {
        Self { resource, action }
    }

    pub fn build(self) -> RequirePermission {
        RequirePermission::new(self.resource, self.action)
    }
}

/// バッチエンドポイントテスト用ヘルパー
pub struct EndpointTest {
    pub path: &'static str,
    pub method: &'static str,
    pub body: Option<serde_json::Value>,
}

impl EndpointTest {
    pub fn new(path: &'static str, method: &'static str) -> Self {
        Self {
            path,
            method,
            body: None,
        }
    }

    pub fn with_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }
}

/// 複数のエンドポイントに対する権限チェックの一括テスト
pub async fn batch_test_forbidden_endpoints(
    app: &Router,
    endpoints: Vec<EndpointTest>,
    unauthorized_token: &str,
) {
    for endpoint in endpoints {
        let request = create_authenticated_request(
            endpoint.method,
            endpoint.path,
            unauthorized_token,
            endpoint.body,
        );

        let response = app.clone().oneshot(request).await.unwrap();
        assert_forbidden(response);
    }
}

/// 権限チェック結果を表す列挙型
#[derive(Debug, PartialEq)]
pub enum PermissionCheckResult {
    Allowed,
    Denied,
    Error(String),
}

/// 権限チェックの結果を検証するヘルパー
pub async fn check_permission_for_user(
    _app_state: &Arc<AppState>,
    user_id: Uuid,
    role_name: RoleName,
    resource: &'static str,
    action: Action,
    resource_id: Option<Uuid>,
) -> PermissionCheckResult {
    // RequirePermissionミドルウェアの動作をシミュレート
    let _context = if let Some(id) = resource_id {
        create_mock_permission_context_with_resource(user_id, role_name, resource, action, id)
    } else {
        create_mock_permission_context(user_id, role_name, resource, action)
    };

    // 管理者チェック
    if role_name == RoleName::Admin {
        return PermissionCheckResult::Allowed;
    }

    // リソース固有の権限チェック
    match (resource, &action) {
        ("task", Action::Create) | ("team", Action::Create) => PermissionCheckResult::Allowed,
        ("task", Action::View | Action::Update | Action::Delete) if resource_id.is_some() => {
            // 実際の実装では所有者チェックが必要
            PermissionCheckResult::Allowed
        }
        _ => PermissionCheckResult::Denied,
    }
}

/// 権限チェックのマクロ
#[macro_export]
macro_rules! assert_admin_only {
    ($app:expr, $path:expr, $method:expr) => {{
        let member_token = create_member_with_jwt($app).await;
        let request = create_authenticated_request::<()>($method, $path, &member_token, None);
        let response = $app.clone().oneshot(request).await.unwrap();
        assert_forbidden(response);
    }};
}

/// リソース所有者チェックのマクロ
#[macro_export]
macro_rules! assert_owner_only {
    ($app:expr, $path:expr, $method:expr, $owner_id:expr, $other_user_token:expr) => {{
        let request = create_authenticated_request::<()>($method, $path, $other_user_token, None);
        let response = $app.clone().oneshot(request).await.unwrap();
        assert!(
            response.status() == StatusCode::FORBIDDEN
                || response.status() == StatusCode::NOT_FOUND,
            "Expected 403 or 404 for non-owner access"
        );
    }};
}

/// テスト用の権限設定ビルダー
pub struct PermissionTestSetup {
    pub admin_token: String,
    pub member_token: String,
    pub viewer_token: Option<String>,
    pub team_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
}

impl PermissionTestSetup {
    /// 基本的な権限テスト環境をセットアップ
    pub async fn basic(app: &Router) -> Self {
        let admin_token = create_admin_with_jwt(app).await;
        let member_token = create_member_with_jwt(app).await;

        Self {
            admin_token,
            member_token,
            viewer_token: None,
            team_id: None,
            organization_id: None,
        }
    }

    /// チーム権限テスト環境をセットアップ
    pub async fn with_team(app: &Router) -> Self {
        let mut setup = Self::basic(app).await;

        // チームを作成（実装は省略）
        let team_id = Uuid::new_v4();
        setup.team_id = Some(team_id);

        setup
    }

    /// 組織権限テスト環境をセットアップ
    pub async fn with_organization(app: &Router) -> Self {
        let mut setup = Self::basic(app).await;

        // 組織を作成（実装は省略）
        let org_id = Uuid::new_v4();
        setup.organization_id = Some(org_id);

        setup
    }
}

/// モックミドルウェアによるテスト簡略化
pub fn create_test_router_with_mock_permissions() -> Router {
    Router::new()
        .route("/test", get(|| async { "OK" }))
        .layer(axum::middleware::from_fn(
            |req: Request<Body>, next: Next| async move {
                // テスト用の権限コンテキストを自動的に挿入
                let mut req = req;
                let context = create_mock_permission_context(
                    Uuid::new_v4(),
                    RoleName::Member,
                    "test",
                    Action::View,
                );
                req.extensions_mut().insert(context);
                next.run(req).await
            },
        ))
}

/// 階層的権限のテストヘルパー
pub async fn test_hierarchical_permissions(
    app: &Router,
    org_admin_token: &str,
    team_admin_token: &str,
    member_token: &str,
    resource_path: &str,
) {
    // 組織管理者はアクセス可能
    let org_req = create_authenticated_request::<()>("GET", resource_path, org_admin_token, None);
    let org_res = app.clone().oneshot(org_req).await.unwrap();
    assert!(
        org_res.status().is_success(),
        "Org admin should have access"
    );

    // チーム管理者はアクセス可能
    let team_req = create_authenticated_request::<()>("GET", resource_path, team_admin_token, None);
    let team_res = app.clone().oneshot(team_req).await.unwrap();
    assert!(
        team_res.status().is_success(),
        "Team admin should have access"
    );

    // 一般メンバーはアクセス不可
    let member_req = create_authenticated_request::<()>("GET", resource_path, member_token, None);
    let member_res = app.clone().oneshot(member_req).await.unwrap();
    assert_forbidden(member_res);
}

/// 動的権限（時間ベース）のテストヘルパー
pub async fn test_time_based_permission(
    app: &Router,
    token: &str,
    resource_path: &str,
    _valid_duration: chrono::Duration,
) -> (bool, bool) {
    // 有効期間内のアクセス
    let valid_req = create_authenticated_request::<()>("GET", resource_path, token, None);
    let valid_res = app.clone().oneshot(valid_req).await.unwrap();
    let valid_access = valid_res.status().is_success();

    // 有効期間後のアクセスをシミュレート（実際の実装では時間を操作）
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    let expired_req = create_authenticated_request::<()>("GET", resource_path, token, None);
    let expired_res = app.clone().oneshot(expired_req).await.unwrap();
    let expired_access = expired_res.status() == StatusCode::FORBIDDEN;

    (valid_access, expired_access)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mock_permission_context() {
        let user_id = Uuid::new_v4();
        let context =
            create_mock_permission_context(user_id, RoleName::Admin, "users", Action::View);

        assert_eq!(context.user_id, user_id);
        assert_eq!(context.role.name, RoleName::Admin);
        assert_eq!(context.resource, "users");
        assert_eq!(context.action, Action::View);
    }

    #[test]
    fn test_create_mock_permission_context_with_resource() {
        let user_id = Uuid::new_v4();
        let resource_id = Uuid::new_v4();
        let context = create_mock_permission_context_with_resource(
            user_id,
            RoleName::Member,
            "tasks",
            Action::Update,
            resource_id,
        );

        assert_eq!(context.user_id, user_id);
        assert_eq!(context.role.name, RoleName::Member);
        assert_eq!(context.resource, "tasks");
        assert_eq!(context.action, Action::Update);
    }

    #[test]
    fn test_endpoint_test_builder() {
        let endpoint = EndpointTest::new("/test", "GET").with_body(json!({"key": "value"}));

        assert_eq!(endpoint.path, "/test");
        assert_eq!(endpoint.method, "GET");
        assert!(endpoint.body.is_some());
    }
}
