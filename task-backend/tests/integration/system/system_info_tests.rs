use axum::body::to_bytes;
use common::app_helper::setup_full_app;
use reqwest::StatusCode;
use serde_json::Value;
use tower::ServiceExt;

mod common {
    pub use crate::common::*;
}

#[tokio::test]
async fn test_system_info_success() {
    let (app, _schema, _db) = setup_full_app().await;
    let admin = common::auth_helper::authenticate_as_admin(&app).await;

    let response = app
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            "/admin/system/info",
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

    assert!(data.get("environment").is_some());
    assert!(data.get("is_test").is_some());
    assert!(data.get("is_production").is_some());
    assert!(data.get("is_development").is_some());

    assert_eq!(data.get("environment").unwrap().as_str().unwrap(), "test");
    assert!(data.get("is_test").unwrap().as_bool().unwrap());
    assert!(!data.get("is_production").unwrap().as_bool().unwrap());
}

#[tokio::test]
async fn test_system_info_environment_detection() {
    let (app, _schema, _db) = setup_full_app().await;
    let admin = common::auth_helper::authenticate_as_admin(&app).await;

    let response = app
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            "/admin/system/info",
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

    let environment = data.get("environment").unwrap().as_str().unwrap();
    let is_test = data.get("is_test").unwrap().as_bool().unwrap();
    let is_production = data.get("is_production").unwrap().as_bool().unwrap();
    let is_development = data.get("is_development").unwrap().as_bool().unwrap();

    match environment {
        "test" => {
            assert!(is_test);
            assert!(!is_production);
            assert!(!is_development);
        }
        "production" => {
            assert!(!is_test);
            assert!(is_production);
            assert!(!is_development);
        }
        "development" => {
            assert!(!is_test);
            assert!(!is_production);
            assert!(is_development);
        }
        _ => panic!("Unknown environment: {}", environment),
    }
}

#[tokio::test]
async fn test_system_info_requires_admin_auth() {
    let (app, _schema, _db) = setup_full_app().await;

    let response = app
        .oneshot(common::auth_helper::create_request(
            "GET",
            "/admin/system/info",
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_system_info_non_admin_forbidden() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = common::auth_helper::create_and_authenticate_user(&app).await;

    let response = app
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            "/admin/system/info",
            &user.access_token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
