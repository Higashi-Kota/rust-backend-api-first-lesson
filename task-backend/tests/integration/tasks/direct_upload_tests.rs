use axum::body::to_bytes;
use common::{
    app_helper::setup_full_app,
    auth_helper::{create_and_authenticate_user, create_authenticated_request, create_request},
};
use reqwest::StatusCode;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

mod common {
    pub use crate::common::*;
}

#[tokio::test]
async fn test_generate_upload_url_success() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // タスクを作成
    let create_task_body = json!({
        "title": "Test Task for Upload",
        "description": "Test description",
        "status": "todo"
    });

    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&create_task_body).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = json["data"]["id"].as_str().unwrap();

    // アップロードURL生成
    let upload_url_request = json!({
        "file_name": "test-document.pdf",
        "expires_in_seconds": 1800
    });

    let response = app
        .oneshot(create_authenticated_request(
            "POST",
            &format!("/tasks/{}/attachments/upload-url", task_id),
            &user.access_token,
            Some(serde_json::to_string(&upload_url_request).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert!(json["data"]["upload_url"].as_str().is_some());
    assert!(json["data"]["upload_key"].as_str().is_some());
    assert!(json["data"]["expires_at"].is_number());

    // upload_keyが正しい形式か確認
    let upload_key = json["data"]["upload_key"].as_str().unwrap();
    assert!(upload_key.starts_with(&format!("tasks/{}/attachments/", task_id)));
    assert!(upload_key.contains("test-document.pdf"));
}

#[tokio::test]
async fn test_generate_upload_url_invalid_task() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    let invalid_task_id = Uuid::new_v4();
    let upload_url_request = json!({
        "file_name": "test.jpg"
    });

    let response = app
        .oneshot(create_authenticated_request(
            "POST",
            &format!("/tasks/{}/attachments/upload-url", invalid_task_id),
            &user.access_token,
            Some(serde_json::to_string(&upload_url_request).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_generate_upload_url_unauthorized() {
    let (app, _schema, _db) = setup_full_app().await;

    let upload_url_request = json!({
        "file_name": "test.jpg"
    });

    let response = app
        .oneshot(create_request(
            "POST",
            &format!("/tasks/{}/attachments/upload-url", Uuid::new_v4()),
            Some(serde_json::to_string(&upload_url_request).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
