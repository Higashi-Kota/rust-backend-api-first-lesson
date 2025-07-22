use crate::common::auth_helper::{create_admin_with_jwt, create_member_with_jwt};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

/// 権限エラー（403 Forbidden）をアサートする共通関数
pub fn assert_forbidden(response: Response) {
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Expected 403 Forbidden but got {}",
        response.status()
    );
}

/// 権限エラーレスポンスのボディも検証する関数
pub async fn assert_forbidden_with_message(response: Response, expected_message: &str) {
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(
        error_response["error"]
            .as_str()
            .unwrap_or("")
            .contains(expected_message),
        "Expected error message to contain '{}', but got: {:?}",
        expected_message,
        error_response["error"]
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_test_builder() {
        let endpoint = EndpointTest::new("/test", "GET").with_body(json!({"key": "value"}));

        assert_eq!(endpoint.path, "/test");
        assert_eq!(endpoint.method, "GET");
        assert!(endpoint.body.is_some());
    }
}
