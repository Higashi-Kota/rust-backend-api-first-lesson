use axum::body::to_bytes;
use common::app_helper::setup_full_app;
use reqwest::StatusCode;
use serde_json::Value;
use tower::ServiceExt;

mod common {
    pub use crate::common::*;
}

#[tokio::test]
async fn test_stripe_config_success() {
    let (app, _schema, _db) = setup_full_app().await;
    let admin = common::auth_helper::authenticate_as_admin(&app).await;

    let response = app
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            "/admin/payments/config",
            &admin.access_token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json.get("success").unwrap().as_bool().unwrap());

    let data = json.get("data").unwrap();
    assert!(data.get("publishable_key").is_some());
    assert!(data.get("is_test_mode").is_some());
}

#[tokio::test]
async fn test_stripe_config_test_mode_detection() {
    let (app, _schema, _db) = setup_full_app().await;
    let admin = common::auth_helper::authenticate_as_admin(&app).await;

    let response = app
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            "/admin/payments/config",
            &admin.access_token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let data = json.get("data").unwrap();
    let is_test_mode = data.get("is_test_mode").unwrap().as_bool().unwrap();

    // 開発モードかテストキーを使用している場合はtrue
    assert!(is_test_mode);
}

#[tokio::test]
async fn test_stripe_config_requires_admin_auth() {
    let (app, _schema, _db) = setup_full_app().await;

    // 認証トークンなしでアクセス
    let response = app
        .oneshot(common::auth_helper::create_request(
            "GET",
            "/admin/payments/config",
            None,
        ))
        .await
        .unwrap();

    // 認証なしではアクセスできないことを確認
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_stripe_config_non_admin_forbidden() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = common::auth_helper::create_and_authenticate_user(&app).await;

    let response = app
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            "/admin/payments/config",
            &user.access_token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
