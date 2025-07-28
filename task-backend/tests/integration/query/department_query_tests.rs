// tests/integration/department_query_tests.rs
use axum::{body, http::StatusCode};
use serde_json::json;
use task_backend::api::dto::organization_hierarchy_dto::{
    DepartmentResponseDto, DepartmentSearchQuery,
};
use task_backend::shared::types::PaginatedResponse;
use task_backend::types::ApiResponse;
use tower::ServiceExt;

use crate::common::app_helper::setup_full_app;
use crate::common::auth_helper::create_and_authenticate_admin;
use crate::common::request::create_request;

#[tokio::test]
async fn test_department_search_pagination() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin_token = create_and_authenticate_admin(&app).await;

    // 組織を作成
    let org_data = json!({
        "name": "Test Org for Departments",
        "description": "Organization for department tests",
        "subscription_tier": "free"
    });

    let org_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/organizations",
            &admin_token,
            &org_data,
        ))
        .await
        .unwrap();

    assert_eq!(
        org_response.status(),
        StatusCode::CREATED,
        "Failed to create organization"
    );
    let org_body = body::to_bytes(org_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&org_body).unwrap();
    let org_data = org_api_response.data.unwrap();
    let org_id = org_data["id"].as_str().unwrap();

    println!("Created organization with ID: {}", org_id);

    // まず組織が本当に存在するか確認
    let get_org_response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/organizations/{}", org_id),
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    println!("GET organization status: {:?}", get_org_response.status());
    if get_org_response.status() != StatusCode::OK {
        let body = body::to_bytes(get_org_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_msg = String::from_utf8_lossy(&body);
        println!("Failed to GET organization: {}", error_msg);
    }

    // デバッグ: まずルートが存在するか確認
    let routes_test_response = app
        .clone()
        .oneshot(create_request("GET", "/", &admin_token, &()))
        .await
        .unwrap();
    println!("Root route status: {:?}", routes_test_response.status());

    // 異なるパスパラメータ形式を試す
    let test_urls = vec![
        format!("/organizations/{}/departments", org_id),
        format!("/organizations/{}/hierarchy", org_id), // これは組織ルーターで定義されている
    ];

    for test_url in test_urls {
        println!("Testing GET request to: {}", test_url);
        let test_response = app
            .clone()
            .oneshot(create_request("GET", &test_url, &admin_token, &()))
            .await
            .unwrap();
        println!("Response status: {:?}", test_response.status());

        if test_response.status() == StatusCode::NOT_FOUND {
            let body = body::to_bytes(test_response.into_body(), usize::MAX)
                .await
                .unwrap();
            let error_msg = String::from_utf8_lossy(&body);
            println!("404 Response body: {}", error_msg);
        }
    }

    // テスト: 部門を作成してみる
    let test_dept_data = json!({
        "name": "Test Department",
        "description": "Test description"
    });

    let test_dept_url = format!("/organizations/{}/departments", org_id);
    println!("\nTesting department creation at URL: {}", test_dept_url);

    let test_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            &test_dept_url,
            &admin_token,
            &test_dept_data,
        ))
        .await
        .unwrap();

    let test_status = test_response.status();
    if test_status != StatusCode::OK {
        let body = body::to_bytes(test_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_msg = String::from_utf8_lossy(&body);
        println!(
            "Department creation failed with status {:?}: {}",
            test_status, error_msg
        );
        panic!("Failed to create test department");
    }

    // 複数の部門を作成
    for i in 0..15 {
        let dept_data = json!({
            "name": format!("Department {}", i),
            "description": format!("Description for dept {}", i)
        });

        let dept_url = format!("/organizations/{}/departments", org_id);

        let dept_response = app
            .clone()
            .oneshot(create_request("POST", &dept_url, &admin_token, &dept_data))
            .await
            .unwrap();
        assert_eq!(
            dept_response.status(),
            StatusCode::OK,
            "Failed to create department {}",
            i
        );
    }

    // Act: ページネーションのテスト
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/organizations/{}/departments?page=1&per_page=10", org_id),
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<DepartmentResponseDto>> =
        serde_json::from_slice(&body).unwrap();
    let depts = api_response.data.unwrap();

    // Assert
    assert!(depts.items.len() <= 10);
    assert_eq!(depts.pagination.page, 1);
    assert_eq!(depts.pagination.per_page, 10);
    assert!(depts.pagination.total_count >= 15);
}

#[tokio::test]
async fn test_department_sort_by_name() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin_token = create_and_authenticate_admin(&app).await;

    // 組織を作成
    let org_data = json!({
        "name": "Test Org for Sort",
        "description": "Organization for sort tests",
        "subscription_tier": "free"
    });

    let org_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/organizations",
            &admin_token,
            &org_data,
        ))
        .await
        .unwrap();

    let org_body = body::to_bytes(org_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&org_body).unwrap();
    let org_data = org_api_response.data.unwrap();
    let org_id = org_data["id"].as_str().unwrap();

    // 特定の名前の部門を作成
    let names = ["Alpha Dept", "Charlie Dept", "Bravo Dept", "Delta Dept"];
    for name in names.iter() {
        let dept_data = json!({
            "name": name,
            "description": format!("Description for {}", name)
        });

        app.clone()
            .oneshot(create_request(
                "POST",
                &format!("/organizations/{}/departments", org_id),
                &admin_token,
                &dept_data,
            ))
            .await
            .unwrap();
    }

    // Act: name昇順でソート
    let response_asc = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/organizations/{}/departments?sort_by=name&sort_order=asc",
                org_id
            ),
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_asc.status(), StatusCode::OK);
    let body_asc = body::to_bytes(response_asc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_asc: ApiResponse<PaginatedResponse<DepartmentResponseDto>> =
        serde_json::from_slice(&body_asc).unwrap();
    let depts_asc = api_response_asc.data.unwrap();

    // name降順でソート
    let response_desc = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/organizations/{}/departments?sort_by=name&sort_order=desc",
                org_id
            ),
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_desc.status(), StatusCode::OK);
    let body_desc = body::to_bytes(response_desc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_desc: ApiResponse<PaginatedResponse<DepartmentResponseDto>> =
        serde_json::from_slice(&body_desc).unwrap();
    let depts_desc = api_response_desc.data.unwrap();

    // Assert: nameがソートされているか確認
    assert!(!depts_asc.items.is_empty());
    assert!(!depts_desc.items.is_empty());

    // 昇順の場合
    for i in 1..depts_asc.items.len() {
        assert!(depts_asc.items[i - 1].name <= depts_asc.items[i].name);
    }

    // 降順の場合
    for i in 1..depts_desc.items.len() {
        assert!(depts_desc.items[i - 1].name >= depts_desc.items[i].name);
    }
}

#[tokio::test]
async fn test_department_sort_by_created_at() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin_token = create_and_authenticate_admin(&app).await;

    // 組織を作成
    let org_data = json!({
        "name": "Test Org for Time Sort",
        "description": "Organization for time sort tests",
        "subscription_tier": "free"
    });

    let org_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/organizations",
            &admin_token,
            &org_data,
        ))
        .await
        .unwrap();

    let org_body = body::to_bytes(org_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&org_body).unwrap();
    let org_data = org_api_response.data.unwrap();
    let org_id = org_data["id"].as_str().unwrap();

    // 時間差をつけて部門を作成
    for i in 0..5 {
        let dept_data = json!({
            "name": format!("Time Dept {}", i),
            "description": format!("Description {}", i)
        });

        app.clone()
            .oneshot(create_request(
                "POST",
                &format!("/organizations/{}/departments", org_id),
                &admin_token,
                &dept_data,
            ))
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Act: created_at昇順でソート
    let response_asc = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/organizations/{}/departments?sort_by=created_at&sort_order=asc",
                org_id
            ),
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_asc.status(), StatusCode::OK);
    let body_asc = body::to_bytes(response_asc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_asc: ApiResponse<PaginatedResponse<DepartmentResponseDto>> =
        serde_json::from_slice(&body_asc).unwrap();
    let depts_asc = api_response_asc.data.unwrap();

    // created_at降順でソート
    let response_desc = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/organizations/{}/departments?sort_by=created_at&sort_order=desc",
                org_id
            ),
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_desc.status(), StatusCode::OK);
    let body_desc = body::to_bytes(response_desc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_desc: ApiResponse<PaginatedResponse<DepartmentResponseDto>> =
        serde_json::from_slice(&body_desc).unwrap();
    let depts_desc = api_response_desc.data.unwrap();

    // Assert
    assert!(depts_asc.items.len() >= 5);
    assert!(depts_desc.items.len() >= 5);

    // 昇順の場合
    for i in 1..depts_asc.items.len() {
        assert!(depts_asc.items[i - 1].created_at <= depts_asc.items[i].created_at);
    }

    // 降順の場合
    for i in 1..depts_desc.items.len() {
        assert!(depts_desc.items[i - 1].created_at >= depts_desc.items[i].created_at);
    }
}

#[tokio::test]
async fn test_department_filter_by_active_status() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin_token = create_and_authenticate_admin(&app).await;

    // 組織を作成
    let org_data = json!({
        "name": "Test Org for Active Filter",
        "description": "Organization for active filter tests",
        "subscription_tier": "free"
    });

    let org_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/organizations",
            &admin_token,
            &org_data,
        ))
        .await
        .unwrap();

    let org_body = body::to_bytes(org_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&org_body).unwrap();
    let org_data = org_api_response.data.unwrap();
    let org_id = org_data["id"].as_str().unwrap();

    // Act: アクティブな部門のみをフィルタ（デフォルト）
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/organizations/{}/departments?active_only=true", org_id),
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<DepartmentResponseDto>> =
        serde_json::from_slice(&body).unwrap();
    let depts = api_response.data.unwrap();

    // Assert: すべての部門がアクティブであることを確認
    for dept in &depts.items {
        assert!(dept.is_active);
    }
}

#[tokio::test]
async fn test_department_search_with_search_term() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin_token = create_and_authenticate_admin(&app).await;

    // 組織を作成
    let org_data = json!({
        "name": "Test Org for Search",
        "description": "Organization for search tests",
        "subscription_tier": "free"
    });

    let org_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/organizations",
            &admin_token,
            &org_data,
        ))
        .await
        .unwrap();

    let org_body = body::to_bytes(org_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&org_body).unwrap();
    let org_data = org_api_response.data.unwrap();
    let org_id = org_data["id"].as_str().unwrap();

    // 検索用の部門を作成
    let dept_data = json!({
        "name": "SearchTarget Department",
        "description": "This is a searchable department"
    });

    app.clone()
        .oneshot(create_request(
            "POST",
            &format!("/organizations/{}/departments", org_id),
            &admin_token,
            &dept_data,
        ))
        .await
        .unwrap();

    // Act: 検索キーワードでフィルタ
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/organizations/{}/departments?search=SearchTarget", org_id),
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<DepartmentResponseDto>> =
        serde_json::from_slice(&body).unwrap();
    let depts = api_response.data.unwrap();

    // Assert
    assert!(!depts.items.is_empty());
    assert!(depts.items.iter().any(|d| d.name.contains("SearchTarget")));
}

#[tokio::test]
async fn test_department_sort_by_path() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin_token = create_and_authenticate_admin(&app).await;

    // 組織を作成
    let org_data = json!({
        "name": "Test Org for Path Sort",
        "description": "Organization for path sort tests",
        "subscription_tier": "free"
    });

    let org_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/organizations",
            &admin_token,
            &org_data,
        ))
        .await
        .unwrap();

    let org_body = body::to_bytes(org_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&org_body).unwrap();
    let org_data = org_api_response.data.unwrap();
    let org_id = org_data["id"].as_str().unwrap();

    // Act: path（hierarchy_path）でソート
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/organizations/{}/departments?sort_by=path&sort_order=asc",
                org_id
            ),
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<DepartmentResponseDto>> =
        serde_json::from_slice(&body).unwrap();
    let depts = api_response.data.unwrap();

    // Assert: pathでソートされていることを確認
    for i in 1..depts.items.len() {
        assert!(depts.items[i - 1].hierarchy_path <= depts.items[i].hierarchy_path);
    }
}

#[tokio::test]
async fn test_department_invalid_sort_field() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin_token = create_and_authenticate_admin(&app).await;

    // 組織を作成
    let org_data = json!({
        "name": "Test Org for Invalid Sort",
        "description": "Organization for invalid sort tests",
        "subscription_tier": "free"
    });

    let org_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/organizations",
            &admin_token,
            &org_data,
        ))
        .await
        .unwrap();

    let org_body = body::to_bytes(org_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&org_body).unwrap();
    let org_data = org_api_response.data.unwrap();
    let org_id = org_data["id"].as_str().unwrap();

    // Act: 無効なソートフィールドを指定
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/organizations/{}/departments?sort_by=invalid_field&sort_order=asc",
                org_id
            ),
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    // Assert: 正常に動作し、デフォルトのソート（created_at desc）が適用される
    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<DepartmentResponseDto>> =
        serde_json::from_slice(&body).unwrap();
    assert!(api_response.data.is_some());
}

#[tokio::test]
async fn test_department_all_sort_fields() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin_token = create_and_authenticate_admin(&app).await;

    // 組織を作成
    let org_data = json!({
        "name": "Test Org for All Fields",
        "description": "Organization for all fields tests",
        "subscription_tier": "free"
    });

    let org_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/organizations",
            &admin_token,
            &org_data,
        ))
        .await
        .unwrap();

    let org_body = body::to_bytes(org_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&org_body).unwrap();
    let org_data = org_api_response.data.unwrap();
    let org_id = org_data["id"].as_str().unwrap();

    // すべての許可されたソートフィールドをテスト
    let allowed_fields = DepartmentSearchQuery::allowed_sort_fields();

    for field in allowed_fields {
        // Act: 各フィールドでソート
        let response = app
            .clone()
            .oneshot(create_request(
                "GET",
                &format!(
                    "/organizations/{}/departments?sort_by={}&sort_order=asc",
                    org_id, field
                ),
                &admin_token,
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
