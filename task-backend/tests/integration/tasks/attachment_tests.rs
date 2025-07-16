// tests/integration/tasks/attachment_tests.rs

use axum::body::{to_bytes, Body};
use axum::http::header;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::{
    auth_helper::create_and_authenticate_user, test_data::create_test_task_for_user,
};

/// ヘルパー関数：マルチパートリクエストを作成
fn create_multipart_request(
    method: &str,
    path: &str,
    token: &str,
    file_name: &str,
    file_content: &[u8],
    content_type: &str,
) -> Request<Body> {
    let boundary = "----boundary";

    // Build multipart body with proper binary data handling
    let mut body = Vec::new();

    // Add boundary
    body.extend_from_slice(b"------boundary\r\n");

    // Add content disposition
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"");
    body.extend_from_slice(file_name.as_bytes());
    body.extend_from_slice(b"\"\r\n");

    // Add content type
    body.extend_from_slice(b"Content-Type: ");
    body.extend_from_slice(content_type.as_bytes());
    body.extend_from_slice(b"\r\n\r\n");

    // Add file content (binary data)
    body.extend_from_slice(file_content);

    // Add closing boundary
    body.extend_from_slice(b"\r\n------boundary--\r\n");

    Request::builder()
        .method(method)
        .uri(path)
        .header("Authorization", format!("Bearer {}", token))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
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

/// 正常系テスト: 画像ファイルのアップロード成功
#[tokio::test]
async fn test_upload_image_success() {
    // Arrange: テストの前提条件を設定
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    // 小さなPNG画像のバイナリデータ（1x1ピクセルの透明PNG）
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x62, 0x00,
        0x00, 0x00, 0x02, 0x00, 0x01, 0xE5, 0x27, 0xDE, 0xFC, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
        0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    // Act: テスト対象の操作を実行
    let request = create_multipart_request(
        "POST",
        &format!("/tasks/{}/attachments", task.id),
        &user.access_token,
        "test_image.png",
        &png_data,
        "image/png",
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert: 期待される結果を確認
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["attachment"]["file_name"], "test_image.png");
    assert_eq!(json["data"]["attachment"]["mime_type"], "image/png");
    assert_eq!(json["data"]["attachment"]["task_id"], task.id.to_string());
    assert!(json["data"]["attachment"]["id"].is_string());
}

/// 異常系テスト: 許可されていないファイルタイプ
#[tokio::test]
async fn test_upload_unsupported_file_type_rejected() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    let exe_content = b"MZ\x90\x00"; // EXE file header

    // Act
    let request = create_multipart_request(
        "POST",
        &format!("/tasks/{}/attachments", task.id),
        &user.access_token,
        "program.exe",
        exe_content,
        "application/x-msdownload",
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], false);
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("File type"));
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("not allowed"));
}

/// 異常系テスト: ファイルサイズ制限超過
#[tokio::test]
async fn test_upload_file_size_exceeded() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    // 6MBのダミーデータ（Freeプランの5MB制限を超える）
    let large_data = vec![0u8; 6 * 1024 * 1024];

    // Act
    let request = create_multipart_request(
        "POST",
        &format!("/tasks/{}/attachments", task.id),
        &user.access_token,
        "large_image.jpg",
        &large_data,
        "image/jpeg",
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], false);
    assert!(json["error"]["message"].as_str().unwrap().contains("size"));
}

/// 権限エラーテスト: 他のユーザーのタスクにアップロード
#[tokio::test]
async fn test_upload_to_other_users_task_forbidden() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user1 = create_and_authenticate_user(&app).await;
    let user2 = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user1).await; // user1のタスク

    let png_data = vec![0x89, 0x50, 0x4E, 0x47]; // 簡略化したPNGデータ

    // Act: user2がuser1のタスクにアップロードを試みる
    let request = create_multipart_request(
        "POST",
        &format!("/tasks/{}/attachments", task.id),
        &user2.access_token, // user2のトークン
        "test.png",
        &png_data,
        "image/png",
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], false);
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("permission"));
}

/// 正常系テスト: 添付ファイル一覧の取得
#[tokio::test]
async fn test_list_attachments_success() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    // 2つのファイルをアップロード
    let png_data = vec![0x89, 0x50, 0x4E, 0x47];
    for i in 1..=2 {
        let request = create_multipart_request(
            "POST",
            &format!("/tasks/{}/attachments", task.id),
            &user.access_token,
            &format!("image{}.png", i),
            &png_data,
            "image/png",
        );
        app.clone().oneshot(request).await.unwrap();
    }

    // Act: 一覧を取得
    let request = create_request(
        "GET",
        &format!("/tasks/{}/attachments", task.id),
        &user.access_token,
        None,
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);

    // Check the response structure
    let data = json
        .get("data")
        .unwrap_or_else(|| panic!("Missing 'data' field in response: {}", json));

    // The response should have a paginated structure with 'items' and 'pagination'
    let items = data
        .get("items")
        .and_then(|d| d.as_array())
        .unwrap_or_else(|| panic!("Missing or invalid 'items' array in response: {}", json));

    assert_eq!(items.len(), 2);

    let pagination = data
        .get("pagination")
        .unwrap_or_else(|| panic!("Missing 'pagination' field in response: {}", json));

    assert_eq!(
        pagination.get("total_count").and_then(|t| t.as_i64()),
        Some(2)
    );
}

/// 正常系テスト: 添付ファイルのダウンロード
#[tokio::test]
async fn test_download_attachment_success() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    // ファイルをアップロード
    let upload_request = create_multipart_request(
        "POST",
        &format!("/tasks/{}/attachments", task.id),
        &user.access_token,
        "download_test.png",
        &png_data,
        "image/png",
    );

    let upload_response = app.clone().oneshot(upload_request).await.unwrap();
    let upload_body = to_bytes(upload_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let upload_json: Value = serde_json::from_slice(&upload_body).unwrap();
    let attachment_id = upload_json["data"]["attachment"]["id"].as_str().unwrap();

    // Act: ダウンロード
    let request = create_request(
        "GET",
        &format!("/attachments/{}", attachment_id),
        &user.access_token,
        None,
    );

    let response = app.oneshot(request).await.unwrap();

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
        .contains("download_test.png"));

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), &png_data);
}

/// 正常系テスト: 添付ファイルの削除
#[tokio::test]
async fn test_delete_attachment_success() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    let png_data = vec![0x89, 0x50, 0x4E, 0x47];

    // ファイルをアップロード
    let upload_request = create_multipart_request(
        "POST",
        &format!("/tasks/{}/attachments", task.id),
        &user.access_token,
        "delete_test.png",
        &png_data,
        "image/png",
    );

    let upload_response = app.clone().oneshot(upload_request).await.unwrap();
    let upload_body = to_bytes(upload_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let upload_json: Value = serde_json::from_slice(&upload_body).unwrap();
    let attachment_id = upload_json["data"]["attachment"]["id"].as_str().unwrap();

    // Act: 削除
    let request = create_request(
        "DELETE",
        &format!("/attachments/{}", attachment_id),
        &user.access_token,
        None,
    );

    let response = app.clone().oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    // 削除後のダウンロード試行
    let download_request = create_request(
        "GET",
        &format!("/attachments/{}", attachment_id),
        &user.access_token,
        None,
    );

    let download_response = app.oneshot(download_request).await.unwrap();
    assert_eq!(download_response.status(), StatusCode::NOT_FOUND);
}

/// 権限エラーテスト: 他のユーザーの添付ファイルを削除
#[tokio::test]
async fn test_delete_other_users_attachment_forbidden() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user1 = create_and_authenticate_user(&app).await;
    let user2 = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user1).await;

    let png_data = vec![0x89, 0x50, 0x4E, 0x47];

    // user1がファイルをアップロード
    let upload_request = create_multipart_request(
        "POST",
        &format!("/tasks/{}/attachments", task.id),
        &user1.access_token,
        "forbidden_test.png",
        &png_data,
        "image/png",
    );

    let upload_response = app.clone().oneshot(upload_request).await.unwrap();
    let upload_body = to_bytes(upload_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let upload_json: Value = serde_json::from_slice(&upload_body).unwrap();
    let attachment_id = upload_json["data"]["attachment"]["id"].as_str().unwrap();

    // Act: user2が削除を試みる
    let request = create_request(
        "DELETE",
        &format!("/attachments/{}", attachment_id),
        &user2.access_token,
        None,
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// 異常系テスト: 存在しない添付ファイルのダウンロード
#[tokio::test]
async fn test_download_nonexistent_attachment() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let nonexistent_id = Uuid::new_v4();

    // Act
    let request = create_request(
        "GET",
        &format!("/attachments/{}", nonexistent_id),
        &user.access_token,
        None,
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// 正常系テスト: PDFファイルのアップロード
#[tokio::test]
async fn test_upload_pdf_success() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    // Simple PDF header (minimal valid PDF)
    let pdf_data = b"%PDF-1.4\n%%EOF";

    // Act
    let request = create_multipart_request(
        "POST",
        &format!("/tasks/{}/attachments", task.id),
        &user.access_token,
        "document.pdf",
        pdf_data,
        "application/pdf",
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["attachment"]["file_name"], "document.pdf");
    assert_eq!(json["data"]["attachment"]["mime_type"], "application/pdf");
}

/// 正常系テスト: CSVファイルのアップロード
#[tokio::test]
async fn test_upload_csv_success() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    let csv_data = b"name,age\nJohn,30\nJane,25";

    // Act
    let request = create_multipart_request(
        "POST",
        &format!("/tasks/{}/attachments", task.id),
        &user.access_token,
        "data.csv",
        csv_data,
        "text/csv",
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["attachment"]["file_name"], "data.csv");
    assert_eq!(json["data"]["attachment"]["mime_type"], "text/csv");
}

/// 正常系テスト: Wordファイル（.docx）のアップロード
#[tokio::test]
async fn test_upload_docx_success() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    // DOCX file signature (PK..)
    let docx_data = vec![0x50, 0x4B, 0x03, 0x04];

    // Act
    let request = create_multipart_request(
        "POST",
        &format!("/tasks/{}/attachments", task.id),
        &user.access_token,
        "report.docx",
        &docx_data,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["attachment"]["file_name"], "report.docx");
}

/// 正常系テスト: Excelファイル（.xlsx）のアップロード
#[tokio::test]
async fn test_upload_xlsx_success() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    // XLSX file signature (PK..)
    let xlsx_data = vec![0x50, 0x4B, 0x03, 0x04];

    // Act
    let request = create_multipart_request(
        "POST",
        &format!("/tasks/{}/attachments", task.id),
        &user.access_token,
        "spreadsheet.xlsx",
        &xlsx_data,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["attachment"]["file_name"], "spreadsheet.xlsx");
}

/// 正常系テスト: 署名付きダウンロードURL生成
#[tokio::test]
async fn test_generate_download_url_success() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    // まずファイルをアップロード
    let png_data = vec![0x89, 0x50, 0x4E, 0x47];
    let upload_request = create_multipart_request(
        "POST",
        &format!("/tasks/{}/attachments", task.id),
        &user.access_token,
        "test.png",
        &png_data,
        "image/png",
    );

    let upload_response = app.clone().oneshot(upload_request).await.unwrap();
    let upload_body = to_bytes(upload_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let upload_json: Value = serde_json::from_slice(&upload_body).unwrap();
    let attachment_id = upload_json["data"]["attachment"]["id"].as_str().unwrap();

    // Act: 署名付きURLを生成
    let request = create_request(
        "GET",
        &format!(
            "/attachments/{}/download-url?expires_in_seconds=3600",
            attachment_id
        ),
        &user.access_token,
        None,
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["download_url"].is_string());
    assert_eq!(json["data"]["expires_in_seconds"], 3600);
    assert!(json["data"]["expires_at"].is_number());

    // URLが正しい形式かチェック
    let download_url = json["data"]["download_url"].as_str().unwrap();
    assert!(download_url.contains("X-Amz-Algorithm") || download_url.contains("?"));
}

/// 正常系テスト: 署名付きURLのデフォルト有効期限
#[tokio::test]
async fn test_generate_download_url_default_expiry() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    // ファイルをアップロード
    let png_data = vec![0x89, 0x50, 0x4E, 0x47];
    let upload_request = create_multipart_request(
        "POST",
        &format!("/tasks/{}/attachments", task.id),
        &user.access_token,
        "test.png",
        &png_data,
        "image/png",
    );

    let upload_response = app.clone().oneshot(upload_request).await.unwrap();
    let upload_body = to_bytes(upload_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let upload_json: Value = serde_json::from_slice(&upload_body).unwrap();
    let attachment_id = upload_json["data"]["attachment"]["id"].as_str().unwrap();

    // Act: 有効期限を指定せずに署名付きURLを生成
    let request = create_request(
        "GET",
        &format!("/attachments/{}/download-url", attachment_id),
        &user.access_token,
        None,
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["expires_in_seconds"], 3600); // デフォルト1時間
}

/// 異常系テスト: 存在しない添付ファイルの署名付きURL生成
#[tokio::test]
async fn test_generate_download_url_not_found() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let nonexistent_id = Uuid::new_v4();

    // Act
    let request = create_request(
        "GET",
        &format!("/attachments/{}/download-url", nonexistent_id),
        &user.access_token,
        None,
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// 権限エラーテスト: 他のユーザーの添付ファイルの署名付きURL生成
#[tokio::test]
async fn test_generate_download_url_forbidden() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user1 = create_and_authenticate_user(&app).await;
    let user2 = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user1).await;

    // user1がファイルをアップロード
    let png_data = vec![0x89, 0x50, 0x4E, 0x47];
    let upload_request = create_multipart_request(
        "POST",
        &format!("/tasks/{}/attachments", task.id),
        &user1.access_token,
        "test.png",
        &png_data,
        "image/png",
    );

    let upload_response = app.clone().oneshot(upload_request).await.unwrap();
    let upload_body = to_bytes(upload_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let upload_json: Value = serde_json::from_slice(&upload_body).unwrap();
    let attachment_id = upload_json["data"]["attachment"]["id"].as_str().unwrap();

    // Act: user2が署名付きURLの生成を試みる
    let request = create_request(
        "GET",
        &format!("/attachments/{}/download-url", attachment_id),
        &user2.access_token,
        None,
    );

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
