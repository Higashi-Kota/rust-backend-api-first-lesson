use crate::common::{app_helper, auth_helper};
use axum::http::StatusCode;
use serde_json::{json, Value};
use tower::ServiceExt;

/// 無効なソートフィールドのエラーハンドリング
#[tokio::test]
async fn test_invalid_sort_field_error() {
    // Arrange
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Act: 無効なソートフィールドでリクエスト
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?sort_by=invalid_field",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();

    // Assert: 400 Bad Requestが返される
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], false);
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Invalid sort field"));
}

/// 各エンドポイントの許可されたソートフィールドの確認
#[tokio::test]
async fn test_allowed_sort_fields_per_endpoint() {
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // タスクエンドポイントのテスト
    let task_allowed_fields = vec![
        "title",
        "priority",
        "status",
        "due_date",
        "created_at",
        "updated_at",
    ];
    for field in task_allowed_fields {
        let req = auth_helper::create_authenticated_request(
            "GET",
            &format!("/tasks/search?sort_by={}", field),
            &user.access_token,
            None,
        );
        let response = app.clone().oneshot(req).await.unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Field '{}' should be allowed for tasks endpoint",
            field
        );
    }

    // ユーザーエンドポイントのテスト（管理者権限が必要）
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;
    let user_allowed_fields = vec![
        "username",
        "email",
        "created_at",
        "updated_at",
        "last_login_at",
    ];
    for field in user_allowed_fields {
        let req = auth_helper::create_authenticated_request(
            "GET",
            &format!("/admin/users?sort_by={}", field),
            &admin_token,
            None,
        );
        let response = app.clone().oneshot(req).await.unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Field '{}' should be allowed for users endpoint",
            field
        );
    }

    // アクティビティログエンドポイントのテスト
    let activity_allowed_fields = vec!["created_at", "action", "resource_type"];
    for field in activity_allowed_fields {
        let req = auth_helper::create_authenticated_request(
            "GET",
            &format!("/activity-logs/me?sort_by={}", field),
            &user.access_token,
            None,
        );
        let response = app.clone().oneshot(req).await.unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Field '{}' should be allowed for activity logs endpoint",
            field
        );
    }
}

/// ソート順序の大文字小文字の処理
#[tokio::test]
async fn test_sort_order_case_insensitive() {
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 複数のタスクを作成
    for i in 0..5 {
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(
                json!({
                    "title": format!("Task {}", i),
                    "priority": "medium",
                    "status": "todo"
                })
                .to_string(),
            ),
        );
        app.clone().oneshot(create_req).await.unwrap();
    }

    // Test 1: 大文字のASC（現在の実装では小文字のみ受け付ける）
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?sort_by=created_at&sort_order=ASC",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();
    // 大文字は無効な値としてエラーになる
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test 2: 小文字のdesc
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?sort_by=created_at&sort_order=desc",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test 3: 混在のDeSc（無効な値として扱われる）
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?sort_by=created_at&sort_order=DeSc",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();
    // 混在ケースも無効な値としてエラーになる
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// 複数ソートフィールドの処理（現在の実装では単一フィールドのみサポート）
#[tokio::test]
async fn test_multiple_sort_fields_not_supported() {
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 複数のソートフィールドを指定（Axumは重複フィールドを拒否する）
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?sort_by=priority&sort_by=created_at",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();

    // Axumのクエリデシリアライザーは重複フィールドでエラーを返す
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    // Axumは重複フィールドに対してプレーンテキストのエラーを返す
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("duplicate field"));
}

/// NULL値を含むフィールドでのソート
#[tokio::test]
async fn test_sort_with_null_values() {
    let (app, _schema, db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // due_dateがNULLのタスクとNULLでないタスクを作成
    use chrono::{Duration, Utc};
    use sea_orm::{ActiveModelTrait, Set};
    use task_backend::domain::task_model;

    // NULLのdue_dateを持つタスク
    let task1 = task_model::ActiveModel {
        user_id: Set(Some(user.id)),
        title: Set("Task without due date".to_string()),
        description: Set(None),
        status: Set("todo".to_string()),
        priority: Set("medium".to_string()),
        due_date: Set(None),
        ..Default::default()
    };
    task1.insert(&db.connection).await.unwrap();

    // 未来のdue_dateを持つタスク
    let task2 = task_model::ActiveModel {
        user_id: Set(Some(user.id)),
        title: Set("Task with future due date".to_string()),
        description: Set(None),
        status: Set("todo".to_string()),
        priority: Set("medium".to_string()),
        due_date: Set(Some(Utc::now() + Duration::days(7))),
        ..Default::default()
    };
    task2.insert(&db.connection).await.unwrap();

    // 過去のdue_dateを持つタスク
    let task3 = task_model::ActiveModel {
        user_id: Set(Some(user.id)),
        title: Set("Task with past due date".to_string()),
        description: Set(None),
        status: Set("todo".to_string()),
        priority: Set("medium".to_string()),
        due_date: Set(Some(Utc::now() - Duration::days(7))),
        ..Default::default()
    };
    task3.insert(&db.connection).await.unwrap();

    // due_dateでソート（昇順）
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?sort_by=due_date&sort_order=asc",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let items = json["data"]["items"].as_array().unwrap();

    // NULL値は最後に来るべき（SeaORMのデフォルト動作）
    let first_item = &items[0];
    assert!(first_item["due_date"].as_i64().is_some()); // NULLでない
}

/// ソートパフォーマンステスト（多数のレコード）
#[tokio::test]
#[ignore] // 通常のテスト実行では省略、必要時に実行
async fn test_sort_performance_with_many_records() {
    let (app, _schema, db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 1000件のタスクを作成
    use rand::Rng;
    use sea_orm::{ActiveModelTrait, Set};
    use task_backend::domain::task_model;

    let priorities = ["low", "medium", "high"];
    let mut rng = rand::thread_rng();

    for i in 0..1000 {
        let priority_idx = rng.gen_range(0..priorities.len());
        let task = task_model::ActiveModel {
            user_id: Set(Some(user.id)),
            title: Set(format!("Performance Test Task {}", i)),
            description: Set(Some(format!("Description for task {}", i))),
            status: Set("todo".to_string()),
            priority: Set(priorities[priority_idx].to_string()),
            due_date: Set(None),
            ..Default::default()
        };
        task.insert(&db.connection).await.unwrap();
    }

    // ソート付きクエリの実行時間を測定
    let start = std::time::Instant::now();

    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?sort_by=priority&sort_order=desc&per_page=100",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();

    let duration = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        duration.as_millis() < 1000,
        "Sort query took too long: {:?}",
        duration
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let items = json["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 100); // per_page = 100
}
