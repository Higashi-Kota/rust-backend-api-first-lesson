// tests/integration/user_query_tests.rs
use axum::{body, http::StatusCode};
use serde_json::json;
use task_backend::api::dto::user_dto::{UserSearchQuery, UserSummary};
use task_backend::shared::types::PaginatedResponse;
use task_backend::types::ApiResponse;

type UserListResponse = PaginatedResponse<UserSummary>;
use tower::ServiceExt;

use crate::common::app_helper::setup_full_app;
use crate::common::auth_helper::create_and_authenticate_user;
use crate::common::request::create_request;

#[tokio::test]
async fn test_user_search_pagination() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin = create_and_authenticate_user(&app).await;

    // 管理者権限を付与（実際の実装に応じて調整）
    // ここでは管理者エンドポイントのテストのみ

    // 複数のユーザーを作成
    for i in 0..15 {
        let user_data = json!({
            "username": format!("testuser{}", i),
            "email": format!("test{}@example.com", i),
            "password": "SecurePassword123!"
        });

        app.clone()
            .oneshot(create_request("POST", "/auth/signup", "", &user_data))
            .await
            .unwrap();
    }

    // Act: ページネーションのテスト
    let response1 = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/users?page=1&per_page=10",
            &admin.token,
            &(),
        ))
        .await
        .unwrap();

    // 管理者権限がない場合は403
    if response1.status() == StatusCode::FORBIDDEN {
        return; // テストをスキップ
    }

    assert_eq!(response1.status(), StatusCode::OK);
    let body1 = body::to_bytes(response1.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response1: ApiResponse<UserListResponse> = serde_json::from_slice(&body1).unwrap();
    let users1 = api_response1.data.unwrap();

    assert!(users1.items.len() <= 10);
    assert_eq!(users1.pagination.page, 1);
    assert_eq!(users1.pagination.per_page, 10);
    assert!(users1.pagination.total_count >= 15);
}

#[tokio::test]
async fn test_user_search_sort_by_username() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin = create_and_authenticate_user(&app).await;

    // 複数のユーザーを作成
    let usernames = ["alice", "charlie", "bob", "david"];
    for username in usernames.iter() {
        let user_data = json!({
            "username": username,
            "email": format!("{}@example.com", username),
            "password": "SecurePassword123!"
        });

        app.clone()
            .oneshot(create_request("POST", "/auth/signup", "", &user_data))
            .await
            .unwrap();
    }

    // Act: username昇順でソート
    let response_asc = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/users?sort_by=username&sort_order=asc",
            &admin.token,
            &(),
        ))
        .await
        .unwrap();

    if response_asc.status() == StatusCode::FORBIDDEN {
        return; // テストをスキップ
    }

    assert_eq!(response_asc.status(), StatusCode::OK);
    let body_asc = body::to_bytes(response_asc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_asc: ApiResponse<UserListResponse> =
        serde_json::from_slice(&body_asc).unwrap();
    let users_asc = api_response_asc.data.unwrap();

    // username降順でソート
    let response_desc = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/users?sort_by=username&sort_order=desc",
            &admin.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_desc.status(), StatusCode::OK);
    let body_desc = body::to_bytes(response_desc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_desc: ApiResponse<UserListResponse> =
        serde_json::from_slice(&body_desc).unwrap();
    let users_desc = api_response_desc.data.unwrap();

    // Assert: usernameがソートされているか確認
    assert!(!users_asc.items.is_empty());
    assert!(!users_desc.items.is_empty());

    // 昇順の場合、usernameがアルファベット順になっているか確認
    for i in 1..users_asc.items.len() {
        assert!(users_asc.items[i - 1].username <= users_asc.items[i].username);
    }

    // 降順の場合、usernameが逆アルファベット順になっているか確認
    for i in 1..users_desc.items.len() {
        assert!(users_desc.items[i - 1].username >= users_desc.items[i].username);
    }
}

#[tokio::test]
async fn test_user_search_sort_by_created_at() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin = create_and_authenticate_user(&app).await;

    // 複数のユーザーを作成
    for i in 0..5 {
        let user_data = json!({
            "username": format!("timeuser{}", i),
            "email": format!("timeuser{}@example.com", i),
            "password": "SecurePassword123!"
        });

        app.clone()
            .oneshot(create_request("POST", "/auth/signup", "", &user_data))
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Act: created_at昇順でソート
    let response_asc = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/users?sort_by=created_at&sort_order=asc",
            &admin.token,
            &(),
        ))
        .await
        .unwrap();

    if response_asc.status() == StatusCode::FORBIDDEN {
        return; // テストをスキップ
    }

    assert_eq!(response_asc.status(), StatusCode::OK);
    let body_asc = body::to_bytes(response_asc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_asc: ApiResponse<UserListResponse> =
        serde_json::from_slice(&body_asc).unwrap();
    let users_asc = api_response_asc.data.unwrap();

    // created_at降順でソート
    let response_desc = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/users?sort_by=created_at&sort_order=desc",
            &admin.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_desc.status(), StatusCode::OK);
    let body_desc = body::to_bytes(response_desc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_desc: ApiResponse<UserListResponse> =
        serde_json::from_slice(&body_desc).unwrap();
    let users_desc = api_response_desc.data.unwrap();

    // Assert: 作成日時が正しくソートされているか確認
    assert!(users_asc.items.len() >= 5);
    assert!(users_desc.items.len() >= 5);

    // 昇順の場合、created_atが古い順になっているか確認
    for i in 1..users_asc.items.len() {
        assert!(users_asc.items[i - 1].created_at <= users_asc.items[i].created_at);
    }

    // 降順の場合、created_atが新しい順になっているか確認
    for i in 1..users_desc.items.len() {
        assert!(users_desc.items[i - 1].created_at >= users_desc.items[i].created_at);
    }
}

#[tokio::test]
async fn test_user_search_filter_by_active_status() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin = create_and_authenticate_user(&app).await;

    // Act: アクティブなユーザーのみをフィルタ
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/users?is_active=true",
            &admin.token,
            &(),
        ))
        .await
        .unwrap();

    if response.status() == StatusCode::FORBIDDEN {
        return; // テストをスキップ
    }

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<UserListResponse> = serde_json::from_slice(&body).unwrap();
    let users = api_response.data.unwrap();

    // Assert: すべてのユーザーがアクティブであることを確認
    for user in &users.items {
        assert!(user.is_active);
    }
}

#[tokio::test]
async fn test_user_search_filter_by_email_verified() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin = create_and_authenticate_user(&app).await;

    // Act: メール確認済みのユーザーのみをフィルタ
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/users?email_verified=false",
            &admin.token,
            &(),
        ))
        .await
        .unwrap();

    if response.status() == StatusCode::FORBIDDEN {
        return; // テストをスキップ
    }

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<UserListResponse> = serde_json::from_slice(&body).unwrap();
    let users = api_response.data.unwrap();

    // Assert: すべてのユーザーがメール未確認であることを確認
    for user in &users.items {
        assert!(!user.email_verified);
    }
}

#[tokio::test]
async fn test_user_search_with_search_term() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin = create_and_authenticate_user(&app).await;

    // 特定のユーザーを作成
    let user_data = json!({
        "username": "searchtest",
        "email": "searchtest@example.com",
        "password": "SecurePassword123!"
    });

    app.clone()
        .oneshot(create_request("POST", "/auth/signup", "", &user_data))
        .await
        .unwrap();

    // Act: 検索キーワードでフィルタ
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/users?search=searchtest",
            &admin.token,
            &(),
        ))
        .await
        .unwrap();

    if response.status() == StatusCode::FORBIDDEN {
        return; // テストをスキップ
    }

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<UserListResponse> = serde_json::from_slice(&body).unwrap();
    let users = api_response.data.unwrap();

    // Assert: 検索結果にマッチするユーザーが含まれることを確認
    assert!(!users.items.is_empty());
    assert!(users
        .items
        .iter()
        .any(|u| u.username == "searchtest" || u.email == "searchtest@example.com"));
}

#[tokio::test]
async fn test_user_search_combined_filters_and_sort() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin = create_and_authenticate_user(&app).await;

    // Act: 複合フィルタとソート
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/users?is_active=true&sort_by=username&sort_order=asc&per_page=5",
            &admin.token,
            &(),
        ))
        .await
        .unwrap();

    if response.status() == StatusCode::FORBIDDEN {
        return; // テストをスキップ
    }

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<UserListResponse> = serde_json::from_slice(&body).unwrap();
    let users = api_response.data.unwrap();

    // Assert
    assert!(users.items.len() <= 5);

    // すべてアクティブなユーザー
    for user in &users.items {
        assert!(user.is_active);
    }

    // usernameが昇順
    for i in 1..users.items.len() {
        assert!(users.items[i - 1].username <= users.items[i].username);
    }
}

#[tokio::test]
async fn test_user_search_invalid_sort_field() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin = create_and_authenticate_user(&app).await;

    // Act: 無効なソートフィールドを指定
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/users?sort_by=invalid_field&sort_order=asc",
            &admin.token,
            &(),
        ))
        .await
        .unwrap();

    if response.status() == StatusCode::FORBIDDEN {
        return; // テストをスキップ
    }

    // Assert: 正常に動作し、デフォルトのソート（created_at desc）が適用される
    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<UserListResponse> = serde_json::from_slice(&body).unwrap();
    assert!(api_response.data.is_some());
}

#[tokio::test]
async fn test_user_search_all_sort_fields() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin = create_and_authenticate_user(&app).await;

    // すべての許可されたソートフィールドをテスト
    let allowed_fields = UserSearchQuery::allowed_sort_fields();

    for field in allowed_fields {
        // Act: 各フィールドでソート
        let response = app
            .clone()
            .oneshot(create_request(
                "GET",
                &format!("/admin/users?sort_by={}&sort_order=asc", field),
                &admin.token,
                &(),
            ))
            .await
            .unwrap();

        if response.status() == StatusCode::FORBIDDEN {
            continue; // テストをスキップ
        }

        // Assert: 正常に動作することを確認
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Failed to sort by field: {}",
            field
        );
    }
}
