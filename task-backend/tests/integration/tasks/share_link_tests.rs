// tests/integration/tasks/share_link_tests.rs

use axum::body::{to_bytes, Body};
use axum::http::header;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::{
    auth_helper::create_and_authenticate_user, test_data::create_test_task_for_user,
};

/// ヘルパー関数：ファイルをアップロードして添付ファイルIDを取得
async fn upload_test_file(
    app: &axum::Router,
    user: &crate::common::auth_helper::TestUser,
    task_id: Uuid,
) -> Uuid {
    let boundary = "----boundary";
    // Small PNG image (1x1 pixel transparent PNG)
    let file_content = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x62, 0x00,
        0x00, 0x00, 0x02, 0x00, 0x01, 0xE5, 0x27, 0xDE, 0xFC, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
        0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let file_name = "share_test.png";

    let mut body = Vec::new();
    body.extend_from_slice(b"------boundary\r\n");
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"");
    body.extend_from_slice(file_name.as_bytes());
    body.extend_from_slice(b"\"\r\n");
    body.extend_from_slice(b"Content-Type: image/png\r\n\r\n");
    body.extend_from_slice(&file_content);
    body.extend_from_slice(b"\r\n------boundary--\r\n");

    let request = Request::builder()
        .method("POST")
        .uri(format!("/tasks/{}/attachments", task_id))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    if status != StatusCode::CREATED {
        panic!("File upload failed with status {}: {}", status, json);
    }

    assert_eq!(status, StatusCode::CREATED);

    json["data"]["attachment"]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

/// ヘルパー関数：通常のリクエストを作成
fn create_request(method: &str, path: &str, token: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("Authorization", format!("Bearer {}", token));

    if let Some(json_body) = body {
        builder = builder.header("Content-Type", "application/json");
        builder
            .body(Body::from(serde_json::to_string(&json_body).unwrap()))
            .unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    }
}

/// 正常系テスト: 共有リンクの作成
#[tokio::test]
async fn test_create_share_link_success() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;
    let attachment_id = upload_test_file(&app, &user, task.id).await;

    // Act: 共有リンクを作成
    let request_body = serde_json::json!({
        "description": "Test share link",
        "expires_in_hours": 24,
        "max_access_count": 10
    });

    let request = create_request(
        "POST",
        &format!("/attachments/{}/share-links", attachment_id),
        &user.access_token,
        Some(request_body),
    );

    let response = app.clone().oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["share_link"]["id"].is_string());
    assert_eq!(
        json["data"]["share_link"]["attachment_id"],
        attachment_id.to_string()
    );
    assert_eq!(json["data"]["share_link"]["description"], "Test share link");
    assert_eq!(json["data"]["share_link"]["max_access_count"], 10);
    assert_eq!(json["data"]["share_link"]["current_access_count"], 0);
    assert_eq!(json["data"]["share_link"]["is_revoked"], false);
    assert!(json["data"]["share_link"]["share_url"]
        .as_str()
        .unwrap()
        .contains("/share/"));
}

/// 正常系テスト: 共有リンクのデフォルト値
#[tokio::test]
async fn test_create_share_link_with_defaults() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;
    let attachment_id = upload_test_file(&app, &user, task.id).await;

    // Act: 最小限のパラメータで共有リンクを作成
    let request_body = serde_json::json!({});

    let request = create_request(
        "POST",
        &format!("/attachments/{}/share-links", attachment_id),
        &user.access_token,
        Some(request_body),
    );

    let response = app.clone().oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["share_link"]["description"].is_null());
    assert!(json["data"]["share_link"]["max_access_count"].is_null());
}

/// 正常系テスト: 共有リンクでのダウンロード
#[tokio::test]
async fn test_download_via_share_link_success() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;
    let attachment_id = upload_test_file(&app, &user, task.id).await;

    // 共有リンクを作成
    let create_request = create_request(
        "POST",
        &format!("/attachments/{}/share-links", attachment_id),
        &user.access_token,
        Some(serde_json::json!({"expires_in_hours": 1})),
    );

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_json: Value = serde_json::from_slice(&create_body).unwrap();
    let share_token = create_json["data"]["share_link"]["share_token"]
        .as_str()
        .unwrap();

    // Act: 共有リンクでダウンロード（認証なし）
    let download_request = Request::builder()
        .method("GET")
        .uri(format!("/share/{}", share_token))
        .header("User-Agent", "Mozilla/5.0 Test Browser")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(download_request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/png"
    );
    assert!(response
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap()
        .contains("share_test.png"));

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    // Verify it's a PNG by checking the first few bytes
    assert_eq!(
        &body[0..8],
        &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
    );
}

/// 正常系テスト: 共有リンク一覧の取得
#[tokio::test]
async fn test_list_share_links_success() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;
    let attachment_id = upload_test_file(&app, &user, task.id).await;

    // 2つの共有リンクを作成
    for i in 1..=2 {
        let request = create_request(
            "POST",
            &format!("/attachments/{}/share-links", attachment_id),
            &user.access_token,
            Some(serde_json::json!({
                "description": format!("Share link {}", i)
            })),
        );
        app.clone().oneshot(request).await.unwrap();
    }

    // Act: 一覧を取得
    let request = create_request(
        "GET",
        &format!("/attachments/{}/share-links", attachment_id),
        &user.access_token,
        None,
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["total"], 2);
    assert_eq!(json["data"]["share_links"].as_array().unwrap().len(), 2);
}

/// 正常系テスト: 共有リンクの無効化
#[tokio::test]
async fn test_revoke_share_link_success() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;
    let attachment_id = upload_test_file(&app, &user, task.id).await;

    // 共有リンクを作成
    let share_link_request = create_request(
        "POST",
        &format!("/attachments/{}/share-links", attachment_id),
        &user.access_token,
        Some(serde_json::json!({})),
    );

    let create_response = app.clone().oneshot(share_link_request).await.unwrap();
    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_json: Value = serde_json::from_slice(&create_body).unwrap();
    let share_link_id = create_json["data"]["share_link"]["id"].as_str().unwrap();
    let share_token = create_json["data"]["share_link"]["share_token"]
        .as_str()
        .unwrap();

    // Act: 共有リンクを無効化
    let revoke_request = create_request(
        "DELETE",
        &format!("/share-links/{}", share_link_id),
        &user.access_token,
        None,
    );

    let response = app.clone().oneshot(revoke_request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    // 無効化後のダウンロード試行
    let download_request = Request::builder()
        .method("GET")
        .uri(format!("/share/{}", share_token))
        .body(Body::empty())
        .unwrap();

    let download_response = app.oneshot(download_request).await.unwrap();
    assert_eq!(download_response.status(), StatusCode::FORBIDDEN);

    let body = to_bytes(download_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("revoked"));
}

/// 異常系テスト: アクセス回数制限
#[tokio::test]
async fn test_share_link_access_count_limit() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;
    let attachment_id = upload_test_file(&app, &user, task.id).await;

    // アクセス回数制限付きの共有リンクを作成
    let create_request = create_request(
        "POST",
        &format!("/attachments/{}/share-links", attachment_id),
        &user.access_token,
        Some(serde_json::json!({
            "max_access_count": 2
        })),
    );

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_json: Value = serde_json::from_slice(&create_body).unwrap();
    let share_token = create_json["data"]["share_link"]["share_token"]
        .as_str()
        .unwrap();

    // Act: 制限回数までアクセス
    for i in 1..=2 {
        let request = Request::builder()
            .method("GET")
            .uri(format!("/share/{}", share_token))
            .header("X-Real-IP", format!("192.168.1.{}", i))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // 制限を超えてアクセス
    let request = Request::builder()
        .method("GET")
        .uri(format!("/share/{}", share_token))
        .header("X-Real-IP", "192.168.1.3")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("maximum access count"));
}

/// 異常系テスト: 存在しない共有トークン
#[tokio::test]
async fn test_download_via_invalid_share_token() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;

    // Act
    let request = Request::builder()
        .method("GET")
        .uri("/share/invalid_token_12345")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Invalid share link"));
}

/// 権限エラーテスト: 他のユーザーの添付ファイルに共有リンクを作成
#[tokio::test]
async fn test_create_share_link_forbidden() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user1 = create_and_authenticate_user(&app).await;
    let user2 = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user1).await;
    let attachment_id = upload_test_file(&app, &user1, task.id).await;

    // Act: user2がuser1の添付ファイルに共有リンクを作成しようとする
    let request = create_request(
        "POST",
        &format!("/attachments/{}/share-links", attachment_id),
        &user2.access_token,
        Some(serde_json::json!({})),
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// 権限エラーテスト: 他のユーザーの共有リンクを無効化
#[tokio::test]
async fn test_revoke_other_users_share_link_forbidden() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user1 = create_and_authenticate_user(&app).await;
    let user2 = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user1).await;
    let attachment_id = upload_test_file(&app, &user1, task.id).await;

    // user1が共有リンクを作成
    let share_link_request = create_request(
        "POST",
        &format!("/attachments/{}/share-links", attachment_id),
        &user1.access_token,
        Some(serde_json::json!({})),
    );

    let create_response = app.clone().oneshot(share_link_request).await.unwrap();
    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_json: Value = serde_json::from_slice(&create_body).unwrap();
    let share_link_id = create_json["data"]["share_link"]["id"].as_str().unwrap();

    // Act: user2が無効化を試みる
    let request = create_request(
        "DELETE",
        &format!("/share-links/{}", share_link_id),
        &user2.access_token,
        None,
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
