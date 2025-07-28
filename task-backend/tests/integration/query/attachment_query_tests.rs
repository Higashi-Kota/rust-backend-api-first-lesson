// tests/integration/attachment_query_tests.rs
use axum::{body, http::StatusCode};
use serde_json::json;
use task_backend::api::dto::attachment_dto::AttachmentDto;
use task_backend::api::dto::attachment_query_dto::AttachmentSearchQuery;
use task_backend::shared::types::PaginatedResponse;
use task_backend::types::ApiResponse;
use tower::ServiceExt;

use crate::common::app_helper::setup_full_app;
use crate::common::auth_helper::create_and_authenticate_user;
use crate::common::request::create_request;

#[tokio::test]
async fn test_attachment_search_pagination() {
    // Arrange
    let (app, _schema, db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // タスクを作成
    let task_data = json!({
        "title": "Task with attachments",
        "description": "Test task for attachments"
    });

    let task_response = app
        .clone()
        .oneshot(create_request("POST", "/tasks", &user.token, &task_data))
        .await
        .unwrap();

    assert_eq!(task_response.status(), StatusCode::CREATED);
    let task_body = body::to_bytes(task_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&task_body).unwrap();
    let task_data = task_api_response.data.unwrap();
    let task_id = task_data["id"].as_str().unwrap();

    // 複数の添付ファイルを作成
    let repo = task_backend::repository::attachment_repository::AttachmentRepository::new(
        db.connection.clone(),
    );

    for i in 0..15 {
        let attachment_data =
            task_backend::repository::attachment_repository::CreateAttachmentDto {
                task_id: uuid::Uuid::parse_str(task_id).unwrap(),
                uploaded_by: user.user_id,
                file_name: format!("file_{}.pdf", i),
                file_size: 1024 * (i + 1) as i64,
                mime_type: "application/pdf".to_string(),
                storage_key: format!("storage/file_{}.pdf", i),
            };

        repo.create(attachment_data).await.unwrap();
    }

    // Act: ページネーションのテスト
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/tasks/{}/attachments?page=1&per_page=10", task_id),
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<AttachmentDto>> =
        serde_json::from_slice(&body).unwrap();
    let attachments = api_response.data.unwrap();

    // Assert
    assert!(attachments.items.len() <= 10);
    assert_eq!(attachments.pagination.page, 1);
    assert_eq!(attachments.pagination.per_page, 10);
    assert!(attachments.pagination.total_count >= 15);
}

#[tokio::test]
async fn test_attachment_sort_by_file_name() {
    // Arrange
    let (app, _schema, db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // タスクを作成
    let task_data = json!({
        "title": "Task for sort test",
        "description": "Test task"
    });

    let task_response = app
        .clone()
        .oneshot(create_request("POST", "/tasks", &user.token, &task_data))
        .await
        .unwrap();

    let task_body = body::to_bytes(task_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&task_body).unwrap();
    let task_data = task_api_response.data.unwrap();
    let task_id = task_data["id"].as_str().unwrap();

    // 特定の名前の添付ファイルを作成
    let repo = task_backend::repository::attachment_repository::AttachmentRepository::new(
        db.connection.clone(),
    );
    let file_names = ["alpha.pdf", "charlie.doc", "bravo.txt", "delta.png"];

    for file_name in file_names.iter() {
        let attachment_data =
            task_backend::repository::attachment_repository::CreateAttachmentDto {
                task_id: uuid::Uuid::parse_str(task_id).unwrap(),
                uploaded_by: user.user_id,
                file_name: (*file_name).to_string(),
                file_size: 1024,
                mime_type: "application/octet-stream".to_string(),
                storage_key: format!("storage/{}", file_name),
            };

        repo.create(attachment_data).await.unwrap();
    }

    // Act: file_name昇順でソート
    let response_asc = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/tasks/{}/attachments?sort_by=file_name&sort_order=asc",
                task_id
            ),
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_asc.status(), StatusCode::OK);
    let body_asc = body::to_bytes(response_asc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_asc: ApiResponse<PaginatedResponse<AttachmentDto>> =
        serde_json::from_slice(&body_asc).unwrap();
    let attachments_asc = api_response_asc.data.unwrap();

    // file_name降順でソート
    let response_desc = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/tasks/{}/attachments?sort_by=file_name&sort_order=desc",
                task_id
            ),
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_desc.status(), StatusCode::OK);
    let body_desc = body::to_bytes(response_desc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_desc: ApiResponse<PaginatedResponse<AttachmentDto>> =
        serde_json::from_slice(&body_desc).unwrap();
    let attachments_desc = api_response_desc.data.unwrap();

    // Assert: file_nameがソートされているか確認
    assert!(!attachments_asc.items.is_empty());
    assert!(!attachments_desc.items.is_empty());

    // 昇順の場合
    for i in 1..attachments_asc.items.len() {
        assert!(attachments_asc.items[i - 1].file_name <= attachments_asc.items[i].file_name);
    }

    // 降順の場合
    for i in 1..attachments_desc.items.len() {
        assert!(attachments_desc.items[i - 1].file_name >= attachments_desc.items[i].file_name);
    }
}

#[tokio::test]
async fn test_attachment_sort_by_file_size() {
    // Arrange
    let (app, _schema, db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // タスクを作成
    let task_data = json!({
        "title": "Task for size sort test",
        "description": "Test task"
    });

    let task_response = app
        .clone()
        .oneshot(create_request("POST", "/tasks", &user.token, &task_data))
        .await
        .unwrap();

    let task_body = body::to_bytes(task_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&task_body).unwrap();
    let task_data = task_api_response.data.unwrap();
    let task_id = task_data["id"].as_str().unwrap();

    // 異なるサイズの添付ファイルを作成
    let repo = task_backend::repository::attachment_repository::AttachmentRepository::new(
        db.connection.clone(),
    );
    let sizes = [512, 2048, 1024, 4096];

    for (i, size) in sizes.iter().enumerate() {
        let attachment_data =
            task_backend::repository::attachment_repository::CreateAttachmentDto {
                task_id: uuid::Uuid::parse_str(task_id).unwrap(),
                uploaded_by: user.user_id,
                file_name: format!("file_{}.pdf", i),
                file_size: *size,
                mime_type: "application/pdf".to_string(),
                storage_key: format!("storage/file_{}.pdf", i),
            };

        repo.create(attachment_data).await.unwrap();
    }

    // Act: file_size昇順でソート
    let response_asc = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/tasks/{}/attachments?sort_by=file_size&sort_order=asc",
                task_id
            ),
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_asc.status(), StatusCode::OK);
    let body_asc = body::to_bytes(response_asc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_asc: ApiResponse<PaginatedResponse<AttachmentDto>> =
        serde_json::from_slice(&body_asc).unwrap();
    let attachments_asc = api_response_asc.data.unwrap();

    // Assert: file_sizeが昇順でソートされているか確認
    assert!(attachments_asc.items.len() >= 4);
    for i in 1..attachments_asc.items.len() {
        assert!(attachments_asc.items[i - 1].file_size <= attachments_asc.items[i].file_size);
    }
}

#[tokio::test]
async fn test_attachment_filter_by_file_type() {
    // Arrange
    let (app, _schema, db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // タスクを作成
    let task_data = json!({
        "title": "Task for type filter test",
        "description": "Test task"
    });

    let task_response = app
        .clone()
        .oneshot(create_request("POST", "/tasks", &user.token, &task_data))
        .await
        .unwrap();

    let task_body = body::to_bytes(task_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&task_body).unwrap();
    let task_data = task_api_response.data.unwrap();
    let task_id = task_data["id"].as_str().unwrap();

    // 異なるファイルタイプの添付ファイルを作成
    let repo = task_backend::repository::attachment_repository::AttachmentRepository::new(
        db.connection.clone(),
    );
    let files = [
        ("document.pdf", "application/pdf"),
        ("image.png", "image/png"),
        (
            "spreadsheet.xlsx",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ),
        ("text.txt", "text/plain"),
    ];

    for (file_name, mime_type) in files.iter() {
        let attachment_data =
            task_backend::repository::attachment_repository::CreateAttachmentDto {
                task_id: uuid::Uuid::parse_str(task_id).unwrap(),
                uploaded_by: user.user_id,
                file_name: (*file_name).to_string(),
                file_size: 1024,
                mime_type: (*mime_type).to_string(),
                storage_key: format!("storage/{}", file_name),
            };

        repo.create(attachment_data).await.unwrap();
    }

    // Act: PDFファイルのみをフィルタ
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/tasks/{}/attachments?file_type=application/pdf", task_id),
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<AttachmentDto>> =
        serde_json::from_slice(&body).unwrap();
    let attachments = api_response.data.unwrap();

    // Assert: すべてPDFファイルであることを確認
    assert!(!attachments.items.is_empty());
    for attachment in &attachments.items {
        assert_eq!(attachment.mime_type, "application/pdf");
    }
}

#[tokio::test]
async fn test_attachment_filter_by_size_range() {
    // Arrange
    let (app, _schema, db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // タスクを作成
    let task_data = json!({
        "title": "Task for size range test",
        "description": "Test task"
    });

    let task_response = app
        .clone()
        .oneshot(create_request("POST", "/tasks", &user.token, &task_data))
        .await
        .unwrap();

    let task_body = body::to_bytes(task_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&task_body).unwrap();
    let task_data = task_api_response.data.unwrap();
    let task_id = task_data["id"].as_str().unwrap();

    // 様々なサイズの添付ファイルを作成
    let repo = task_backend::repository::attachment_repository::AttachmentRepository::new(
        db.connection.clone(),
    );
    let sizes = [100, 500, 1024, 2048, 5000, 10000];

    for (i, size) in sizes.iter().enumerate() {
        let attachment_data =
            task_backend::repository::attachment_repository::CreateAttachmentDto {
                task_id: uuid::Uuid::parse_str(task_id).unwrap(),
                uploaded_by: user.user_id,
                file_name: format!("file_{}.bin", i),
                file_size: *size,
                mime_type: "application/octet-stream".to_string(),
                storage_key: format!("storage/file_{}.bin", i),
            };

        repo.create(attachment_data).await.unwrap();
    }

    // Act: 1KB〜5KBのファイルのみをフィルタ
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/tasks/{}/attachments?min_size=1024&max_size=5000", task_id),
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<AttachmentDto>> =
        serde_json::from_slice(&body).unwrap();
    let attachments = api_response.data.unwrap();

    // Assert: サイズが範囲内であることを確認
    assert!(!attachments.items.is_empty());
    for attachment in &attachments.items {
        assert!(attachment.file_size >= 1024);
        assert!(attachment.file_size <= 5000);
    }
}

#[tokio::test]
async fn test_attachment_search_by_file_name() {
    // Arrange
    let (app, _schema, db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // タスクを作成
    let task_data = json!({
        "title": "Task for search test",
        "description": "Test task"
    });

    let task_response = app
        .clone()
        .oneshot(create_request("POST", "/tasks", &user.token, &task_data))
        .await
        .unwrap();

    let task_body = body::to_bytes(task_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&task_body).unwrap();
    let task_data = task_api_response.data.unwrap();
    let task_id = task_data["id"].as_str().unwrap();

    // 検索用の添付ファイルを作成
    let repo = task_backend::repository::attachment_repository::AttachmentRepository::new(
        db.connection.clone(),
    );

    let attachment_data = task_backend::repository::attachment_repository::CreateAttachmentDto {
        task_id: uuid::Uuid::parse_str(task_id).unwrap(),
        uploaded_by: user.user_id,
        file_name: "searchable_document.pdf".to_string(),
        file_size: 2048,
        mime_type: "application/pdf".to_string(),
        storage_key: "storage/searchable_document.pdf".to_string(),
    };

    repo.create(attachment_data).await.unwrap();

    // Act: 検索キーワードでフィルタ
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/tasks/{}/attachments?search=searchable", task_id),
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<AttachmentDto>> =
        serde_json::from_slice(&body).unwrap();
    let attachments = api_response.data.unwrap();

    // Assert
    assert!(!attachments.items.is_empty());
    assert!(attachments
        .items
        .iter()
        .any(|a| a.file_name.contains("searchable")));
}

#[tokio::test]
async fn test_attachment_invalid_sort_field() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // タスクを作成
    let task_data = json!({
        "title": "Task for invalid sort test",
        "description": "Test task"
    });

    let task_response = app
        .clone()
        .oneshot(create_request("POST", "/tasks", &user.token, &task_data))
        .await
        .unwrap();

    let task_body = body::to_bytes(task_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&task_body).unwrap();
    let task_data = task_api_response.data.unwrap();
    let task_id = task_data["id"].as_str().unwrap();

    // Act: 無効なソートフィールドを指定
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/tasks/{}/attachments?sort_by=invalid_field&sort_order=asc",
                task_id
            ),
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    // Assert: 正常に動作し、デフォルトのソート（created_at desc）が適用される
    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<AttachmentDto>> =
        serde_json::from_slice(&body).unwrap();
    assert!(api_response.data.is_some());
}

#[tokio::test]
async fn test_attachment_all_sort_fields() {
    // Arrange
    let (app, _schema, db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // タスクを作成
    let task_data = json!({
        "title": "Task for all fields test",
        "description": "Test task"
    });

    let task_response = app
        .clone()
        .oneshot(create_request("POST", "/tasks", &user.token, &task_data))
        .await
        .unwrap();

    let task_body = body::to_bytes(task_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&task_body).unwrap();
    let task_data = task_api_response.data.unwrap();
    let task_id = task_data["id"].as_str().unwrap();

    // テスト用の添付ファイルを作成
    let repo = task_backend::repository::attachment_repository::AttachmentRepository::new(
        db.connection.clone(),
    );
    let attachment_data = task_backend::repository::attachment_repository::CreateAttachmentDto {
        task_id: uuid::Uuid::parse_str(task_id).unwrap(),
        uploaded_by: user.user_id,
        file_name: "test.pdf".to_string(),
        file_size: 1024,
        mime_type: "application/pdf".to_string(),
        storage_key: "storage/test.pdf".to_string(),
    };

    repo.create(attachment_data).await.unwrap();

    // すべての許可されたソートフィールドをテスト
    let allowed_fields = AttachmentSearchQuery::allowed_sort_fields();

    for field in allowed_fields {
        // Act: 各フィールドでソート
        let response = app
            .clone()
            .oneshot(create_request(
                "GET",
                &format!(
                    "/tasks/{}/attachments?sort_by={}&sort_order=asc",
                    task_id, field
                ),
                &user.token,
                &(),
            ))
            .await
            .unwrap();

        // Assert: 正常に動作することを確認
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Failed to sort by field: {}",
            field
        );
    }
}
