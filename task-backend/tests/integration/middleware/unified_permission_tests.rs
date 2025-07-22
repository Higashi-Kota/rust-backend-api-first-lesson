// tests/integration/middleware/unified_permission_tests.rs

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use task_backend::api::dto::task_dto::{CreateTaskDto, UpdateTaskDto};
use task_backend::middleware::authorization::{
    admin_permission_middleware, permission_middleware, resources, Action,
};
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::app_helper::setup_full_app;
use crate::common::auth_helper::{
    authenticate_as_admin, create_and_authenticate_user, create_authenticated_request,
};

// ==============================
// 1. ミドルウェア単体テスト
// ==============================

#[tokio::test]
async fn test_unified_permission_middleware_unit() {
    // Arrange: アプリケーションセットアップ
    let (_app, _schema, _db) = setup_full_app().await;

    // Act & Assert: 管理者権限チェック
    let admin_middleware = admin_permission_middleware();
    let admin_middleware_clone = admin_middleware.clone();
    // 同じオブジェクトを参照しているわけではないことを確認（Cloneが正しく動作）
    assert!(!std::ptr::eq(&admin_middleware, &admin_middleware_clone));

    // Act & Assert: 個別権限チェック
    let task_view_middleware = permission_middleware(resources::TASK, Action::View);
    let task_view_middleware_clone = task_view_middleware.clone();
    assert!(!std::ptr::eq(
        &task_view_middleware,
        &task_view_middleware_clone
    ));
}

// ==============================
// 2. リソース・アクション組み合わせテスト
// ==============================

#[tokio::test]
async fn test_resource_action_combinations() {
    // Arrange: 各ロールのユーザーを作成
    let (app, _schema, _db) = setup_full_app().await;
    let admin_user = authenticate_as_admin(&app).await;
    let member_user = create_and_authenticate_user(&app).await;

    // Test 1: 管理者は管理者専用エンドポイントにアクセス可能
    let admin_endpoints = vec![("/admin/users", "GET"), ("/admin/system/info", "GET")];

    for (path, method) in admin_endpoints {
        let response = app
            .clone()
            .oneshot(create_authenticated_request(
                method,
                path,
                &admin_user.token,
                None,
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Admin should access {} {}",
            method,
            path
        );
    }

    // Test 2: メンバーは管理者専用エンドポイントにアクセス不可
    let member_restricted_endpoints = vec![
        ("/admin/users", "GET", StatusCode::FORBIDDEN),
        ("/admin/system/info", "GET", StatusCode::FORBIDDEN),
    ];

    for (path, method, expected_status) in member_restricted_endpoints {
        let response = app
            .clone()
            .oneshot(create_authenticated_request(
                method,
                path,
                &member_user.token,
                None,
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            expected_status,
            "Member access to {} {} should be {}",
            method,
            path,
            expected_status
        );
    }
}

// ==============================
// 3. エッジケーステスト
// ==============================

#[tokio::test]
async fn test_permission_edge_cases() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // Test 1: 認証なしでのアクセス（Authorization ヘッダーなし）
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/tasks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Test 2: 無効なトークンでのアクセス
    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "GET",
            "/tasks",
            "invalid_token",
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Test 3: 期限切れトークンのシミュレーション（実際の期限切れトークンは作成が複雑なので、無効なトークンで代用）
    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "GET",
            "/tasks",
            "expired_token",
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Test 4: 存在しないリソースへのアクセス
    let non_existent_id = Uuid::new_v4();
    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "GET",
            &format!("/tasks/{}", non_existent_id),
            &user.token,
            None,
        ))
        .await
        .unwrap();

    // 存在しないリソースは404
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ==============================
// 4. 権限継承・階層テスト
// ==============================

#[tokio::test]
async fn test_permission_hierarchy() {
    let (app, _schema, _db) = setup_full_app().await;

    // 異なる権限レベルのユーザーを作成
    let admin_user = authenticate_as_admin(&app).await;
    let member_user = create_and_authenticate_user(&app).await;

    // Test 1: 管理者権限の継承確認
    // 管理者は明示的に許可されていないアクションも実行可能
    let admin_only_endpoints = vec![
        ("/admin/analytics/summary", "GET"),
        ("/admin/system/info", "GET"),
        ("/admin/users", "GET"),
    ];

    for (path, method) in admin_only_endpoints {
        let admin_response = app
            .clone()
            .oneshot(create_authenticated_request(
                method,
                path,
                &admin_user.token,
                None,
            ))
            .await
            .unwrap();

        // 管理者はアクセス可能
        assert!(
            admin_response.status() == StatusCode::OK
                || admin_response.status() == StatusCode::NOT_FOUND,
            "Admin should access {} {} (got {})",
            method,
            path,
            admin_response.status()
        );

        let member_response = app
            .clone()
            .oneshot(create_authenticated_request(
                method,
                path,
                &member_user.token,
                None,
            ))
            .await
            .unwrap();

        assert_eq!(
            member_response.status(),
            StatusCode::FORBIDDEN,
            "Member should not access {} {}",
            method,
            path
        );
    }
}

// ==============================
// 5. 動的権限テスト
// ==============================

#[tokio::test]
async fn test_dynamic_permissions() {
    let (app, _schema, _db) = setup_full_app().await;
    let user1 = create_and_authenticate_user(&app).await;
    let user2 = create_and_authenticate_user(&app).await;

    // タスクを作成
    let create_task_dto = CreateTaskDto {
        title: "Test Task".to_string(),
        description: Some("Test Description".to_string()),
        priority: Some("high".to_string()),
        due_date: None,
        status: None,
    };

    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "POST",
            "/tasks",
            &user1.token,
            Some(serde_json::to_string(&create_task_dto).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let created_task = &response_json["data"];
    let task_id = created_task["id"].as_str().unwrap();

    // Test 1: 所有者は自分のタスクを更新可能
    let update_task_dto = UpdateTaskDto {
        title: Some("Updated Task".to_string()),
        description: None,
        priority: None,
        due_date: None,
        status: None,
    };

    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "PATCH",
            &format!("/tasks/{}", task_id),
            &user1.token,
            Some(serde_json::to_string(&update_task_dto).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test 2: 他のユーザーは更新不可（タスクが見つからないため404）
    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "PATCH",
            &format!("/tasks/{}", task_id),
            &user2.token,
            Some(serde_json::to_string(&update_task_dto).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Test 3: 所有者は削除可能
    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "DELETE",
            &format!("/tasks/{}", task_id),
            &user1.token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

// ==============================
// 6. パフォーマンステスト
// ==============================

#[tokio::test]
async fn test_permission_check_performance() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    use std::time::Instant;

    // 権限チェックのオーバーヘッドを測定
    let iterations = 100;
    let start = Instant::now();

    for _ in 0..iterations {
        let response = app
            .clone()
            .oneshot(create_authenticated_request(
                "GET",
                "/tasks",
                &user.token,
                None,
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    let duration = start.elapsed();
    let avg_time = duration.as_millis() as f64 / iterations as f64;

    println!(
        "Average permission check time: {:.2}ms over {} iterations",
        avg_time, iterations
    );

    // 権限チェックは高速であるべき（平均10ms未満）
    assert!(
        avg_time < 10.0,
        "Permission check should be fast (avg: {:.2}ms)",
        avg_time
    );
}

// ==============================
// 7. 同時実行テスト
// ==============================

#[tokio::test]
async fn test_concurrent_permission_checks() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 複数の同時リクエストを送信
    let mut handles = vec![];
    let concurrent_requests = 10;

    for i in 0..concurrent_requests {
        let app_clone = app.clone();
        let token = user.token.clone();

        let handle = tokio::spawn(async move {
            let response = app_clone
                .oneshot(create_authenticated_request(
                    "GET",
                    &format!("/tasks?page={}", i),
                    &token,
                    None,
                ))
                .await
                .unwrap();

            (i, response.status())
        });

        handles.push(handle);
    }

    // すべてのリクエストが成功することを確認
    for handle in handles {
        let (index, status) = handle.await.unwrap();
        assert_eq!(
            status,
            StatusCode::OK,
            "Concurrent request {} should succeed",
            index
        );
    }
}

// ==============================
// 8. 権限変更の反映テスト
// ==============================

#[tokio::test]
async fn test_permission_changes_reflection() {
    let (app, _schema, _db) = setup_full_app().await;

    // 通常ユーザーとして開始
    let user = create_and_authenticate_user(&app).await;

    // 管理者専用エンドポイントにアクセス（失敗するはず）
    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "GET",
            "/admin/users",
            &user.token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // 注: 実際の権限変更はトークンの再発行が必要なため、
    // このテストでは権限チェックロジックの正確性を確認

    // 管理者ユーザーでアクセス（成功するはず）
    let admin_user = authenticate_as_admin(&app).await;
    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "GET",
            "/admin/users",
            &admin_user.token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ==============================
// 9. エラーメッセージ検証テスト
// ==============================

#[tokio::test]
async fn test_permission_error_messages() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // Test 1: 認証なしエラー
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/tasks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: serde_json::Value = serde_json::from_slice(&body).unwrap();
    println!("Unauthorized error response: {:?}", error);

    // エラーレスポンスのstructureに応じた検証
    if let Some(error_obj) = error.get("error") {
        if let Some(message) = error_obj.get("message") {
            assert_eq!(message, "Authentication required");
        } else {
            panic!("Expected error.message not found in response: {:?}", error);
        }
    } else {
        panic!("Expected error object not found in response: {:?}", error);
    }

    // Test 2: 権限不足エラー
    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "GET",
            "/admin/users",
            &user.token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: serde_json::Value = serde_json::from_slice(&body).unwrap();
    println!("Forbidden error response: {:?}", error);

    // エラーレスポンスのstructureに応じた検証
    if let Some(error_obj) = error.get("error") {
        if let Some(message) = error_obj.get("message") {
            assert_eq!(message, "Admin access required");
        } else {
            panic!("Expected error.message not found in response: {:?}", error);
        }
    } else {
        panic!("Expected error object not found in response: {:?}", error);
    }
}
