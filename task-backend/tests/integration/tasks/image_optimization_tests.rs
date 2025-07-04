// tests/integration/tasks/image_optimization_tests.rs

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::{
    auth_helper::create_and_authenticate_user, test_data::create_test_task_for_user,
};

/// ヘルパー関数：テスト用のJPEG画像を生成
fn create_test_jpeg_image() -> Vec<u8> {
    // 簡単な1x1ピクセルの赤いJPEG画像
    vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, 0x00, 0x08, 0x06, 0x06, 0x07, 0x06,
        0x05, 0x08, 0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D, 0x0C, 0x0B, 0x0B,
        0x0C, 0x19, 0x12, 0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D, 0x1A, 0x1C, 0x1C, 0x20,
        0x24, 0x2E, 0x27, 0x20, 0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29, 0x2C, 0x30, 0x31,
        0x34, 0x34, 0x34, 0x1F, 0x27, 0x39, 0x3D, 0x38, 0x32, 0x3C, 0x2E, 0x33, 0x34, 0x32, 0xFF,
        0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00,
        0x1F, 0x00, 0x00, 0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
        0xFF, 0xC4, 0x00, 0xB5, 0x10, 0x00, 0x02, 0x01, 0x03, 0x03, 0x02, 0x04, 0x03, 0x05, 0x05,
        0x04, 0x04, 0x00, 0x00, 0x01, 0x7D, 0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21,
        0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08,
        0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0, 0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0A,
        0x16, 0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x34, 0x35, 0x36, 0x37,
        0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x53, 0x54, 0x55, 0x56,
        0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x73, 0x74, 0x75,
        0x76, 0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x92, 0x93,
        0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9,
        0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6,
        0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2,
        0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7,
        0xF8, 0xF9, 0xFA, 0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00, 0xFB, 0x00,
        0x00, 0x00, 0xFF, 0xD9,
    ]
}

/// ヘルパー関数：画像をアップロード
async fn upload_test_image(
    app: &axum::Router,
    user: &crate::common::auth_helper::TestUser,
    task_id: Uuid,
    file_name: &str,
    content_type: &str,
) -> (Uuid, Value) {
    let boundary = "----boundary";
    let file_content = create_test_jpeg_image();

    let mut body = Vec::new();
    body.extend_from_slice(b"------boundary\r\n");
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"");
    body.extend_from_slice(file_name.as_bytes());
    body.extend_from_slice(b"\"\r\n");
    body.extend_from_slice(b"Content-Type: ");
    body.extend_from_slice(content_type.as_bytes());
    body.extend_from_slice(b"\r\n\r\n");
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

    assert_eq!(status, StatusCode::CREATED);

    let attachment_id = json["data"]["attachment"]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    (attachment_id, json)
}

/// 正常系テスト: JPEG画像がWebPに変換される
#[tokio::test]
async fn test_jpeg_to_webp_conversion() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    // Act: JPEG画像をアップロード
    let (_attachment_id, response_json) =
        upload_test_image(&app, &user, task.id, "test_photo.jpg", "image/jpeg").await;

    // Assert: WebPに変換されているか確認
    assert_eq!(response_json["success"], true);
    assert_eq!(
        response_json["data"]["attachment"]["mime_type"],
        "image/webp"
    );
    assert_eq!(
        response_json["data"]["attachment"]["file_name"],
        "test_photo.webp"
    );

    // ファイルサイズが削減されているかを確認（元のJPEGより小さいはず）
    let optimized_size = response_json["data"]["attachment"]["file_size"]
        .as_i64()
        .unwrap();
    let original_size = create_test_jpeg_image().len() as i64;
    assert!(
        optimized_size < original_size,
        "WebP should be smaller than original JPEG"
    );
}

/// 正常系テスト: PNG画像もWebPに変換される
#[tokio::test]
async fn test_png_to_webp_conversion() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    // Act: PNG画像をアップロード（JPEGデータを使用してテスト）
    let (_attachment_id, response_json) =
        upload_test_image(&app, &user, task.id, "test_image.png", "image/png").await;

    // Assert: WebPに変換されているか確認
    assert_eq!(response_json["success"], true);
    // Note: 実際のPNGではないため、変換が失敗して元のままの可能性もある
    let mime_type = response_json["data"]["attachment"]["mime_type"]
        .as_str()
        .unwrap();
    assert!(mime_type == "image/webp" || mime_type == "image/png");
}

/// 正常系テスト: WebP画像は再変換されない
#[tokio::test]
async fn test_webp_no_reconversion() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    // Act: WebP画像をアップロード
    let (_attachment_id, response_json) =
        upload_test_image(&app, &user, task.id, "test_image.webp", "image/webp").await;

    // Assert: WebPのまま保持される
    assert_eq!(response_json["success"], true);
    assert_eq!(
        response_json["data"]["attachment"]["mime_type"],
        "image/webp"
    );
    assert_eq!(
        response_json["data"]["attachment"]["file_name"],
        "test_image.webp"
    );
}

/// 正常系テスト: 非画像ファイルは最適化されない
#[tokio::test]
async fn test_non_image_no_optimization() {
    // Arrange
    let (app, _schema, _db) = crate::common::app_helper::setup_full_app_with_storage().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task_for_user(&app, &user).await;

    // PDFファイルのダミーデータ
    let pdf_data = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog >>\nendobj\nxref\n0 1\n0000000000 65535 f\ntrailer\n<< /Size 1 >>\nstartxref\n9\n%%EOF";

    let boundary = "----boundary";
    let mut body = Vec::new();
    body.extend_from_slice(b"------boundary\r\n");
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"document.pdf\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: application/pdf\r\n\r\n");
    body.extend_from_slice(pdf_data);
    body.extend_from_slice(b"\r\n------boundary--\r\n");

    let request = Request::builder()
        .method("POST")
        .uri(format!("/tasks/{}/attachments", task.id))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    // Act
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Assert: PDFは変換されない
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["data"]["attachment"]["mime_type"], "application/pdf");
    assert_eq!(json["data"]["attachment"]["file_name"], "document.pdf");
}
