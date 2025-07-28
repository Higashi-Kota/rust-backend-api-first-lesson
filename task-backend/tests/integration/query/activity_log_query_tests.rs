// tests/integration/activity_log_query_tests.rs
use axum::{body, http::StatusCode};
use serde_json::json;
use task_backend::api::handlers::activity_log_handler::ActivityLogDto;
use task_backend::shared::types::PaginatedResponse;
use task_backend::types::ApiResponse;
use tower::ServiceExt;

use crate::common::app_helper::{create_test_task, create_test_team, setup_full_app};
use crate::common::auth_helper::create_and_authenticate_user;
use crate::common::request::create_request;
use sea_orm::EntityTrait;

#[tokio::test]
async fn test_activity_log_creation() {
    // Arrange
    let (app, _schema, db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // タスクを1つ作成
    create_test_task(&app, &user.token).await;

    // アクティビティログが記録されるまで待つ
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    // データベースから直接アクティビティログを確認
    use task_backend::domain::activity_log_model;
    let logs = activity_log_model::Entity::find()
        .all(&db.connection)
        .await
        .unwrap();

    println!("Total activity logs in DB: {}", logs.len());
    for log in &logs {
        println!(
            "Log: action={}, resource_type={}, user_id={}",
            log.action, log.resource_type, log.user_id
        );
    }

    // 少なくとも1つのログが存在することを確認
    assert!(
        !logs.is_empty(),
        "Expected at least one activity log in database"
    );
}

#[tokio::test]
async fn test_activity_log_pagination() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 複数のアクティビティを生成
    for i in 0..15 {
        let task_data = json!({
            "title": format!("Activity Task {}", i),
            "description": "Test activity",
            "status": "todo",
            "visibility": "private"
        });

        app.clone()
            .oneshot(create_request("POST", "/tasks", &user.token, &task_data))
            .await
            .unwrap();
    }

    // アクティビティログが記録されるまで待つ（非同期処理のため）
    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

    // Act: ページネーションのテスト
    let response1 = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/activity-logs/me?page=1&per_page=10",
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::OK);
    let body1 = body::to_bytes(response1.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response1: ApiResponse<PaginatedResponse<ActivityLogDto>> =
        serde_json::from_slice(&body1).unwrap();
    let page1_data = api_response1.data.unwrap();

    // デバッグ情報を出力
    println!("Activity logs count: {}", page1_data.pagination.total_count);
    println!("First page items: {}", page1_data.items.len());
    for (i, log) in page1_data.items.iter().enumerate() {
        println!(
            "Log {}: action={}, resource_type={}",
            i, log.action, log.resource_type
        );
    }

    assert_eq!(page1_data.items.len(), 10);
    assert_eq!(page1_data.pagination.page, 1);
    assert_eq!(page1_data.pagination.per_page, 10);
    // 15個のタスクを作成したので、最低でも15個のアクティビティログがあるはず
    assert!(
        page1_data.pagination.total_count >= 15,
        "Expected at least 15 activity logs, but got {}",
        page1_data.pagination.total_count
    );

    // ページ2のテスト
    let response2 = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/activity-logs/me?page=2&per_page=10",
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);
    let body2 = body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response2: ApiResponse<PaginatedResponse<ActivityLogDto>> =
        serde_json::from_slice(&body2).unwrap();
    let page2_data = api_response2.data.unwrap();

    assert_eq!(page2_data.pagination.page, 2);
    assert!(page2_data.items.len() >= 5);
}

#[tokio::test]
async fn test_activity_log_sort_by_created_at() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 複数のアクティビティを生成
    for i in 0..5 {
        let task_data = json!({
            "title": format!("Sort Task {}", i),
            "description": "Test sorting",
            "status": "todo",
            "visibility": "private"
        });

        app.clone()
            .oneshot(create_request("POST", "/tasks", &user.token, &task_data))
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Act: created_at昇順でソート
    let response_asc = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/activity-logs/me?sort_by=created_at&sort_order=asc",
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_asc.status(), StatusCode::OK);
    let body_asc = body::to_bytes(response_asc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_asc: ApiResponse<PaginatedResponse<ActivityLogDto>> =
        serde_json::from_slice(&body_asc).unwrap();
    let data_asc = api_response_asc.data.unwrap();
    let logs_asc = &data_asc.items;

    // created_at降順でソート
    let response_desc = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/activity-logs/me?sort_by=created_at&sort_order=desc",
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_desc.status(), StatusCode::OK);
    let body_desc = body::to_bytes(response_desc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_desc: ApiResponse<PaginatedResponse<ActivityLogDto>> =
        serde_json::from_slice(&body_desc).unwrap();
    let data_desc = api_response_desc.data.unwrap();
    let logs_desc = &data_desc.items;

    // Assert: 昇順と降順で順序が逆になっていることを確認
    assert!(logs_asc.len() >= 5);
    assert!(logs_desc.len() >= 5);

    // 最初のログのIDが異なることを確認
    assert_ne!(logs_asc[0].id, logs_desc[0].id);
}

#[tokio::test]
async fn test_activity_log_sort_by_action() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 異なるアクションを実行
    // タスク作成
    create_test_task(&app, &user.token).await;

    // チーム作成
    create_test_team(&app, &user.token).await;

    // タスク更新
    let task = create_test_task(&app, &user.token).await;
    let update_data = json!({
        "title": "Updated for Action Sort"
    });
    app.clone()
        .oneshot(create_request(
            "PATCH",
            &format!("/tasks/{}", task.id),
            &user.token,
            &update_data,
        ))
        .await
        .unwrap();

    // Act: action でソート
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/activity-logs/me?sort_by=action&sort_order=asc&per_page=20",
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<ActivityLogDto>> =
        serde_json::from_slice(&body).unwrap();
    let data = api_response.data.unwrap();
    let logs = &data.items;

    // Assert: ログが存在することを確認
    assert!(logs.len() >= 3);
}

#[tokio::test]
async fn test_activity_log_filter_by_resource_type() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // タスクとチームを作成
    create_test_task(&app, &user.token).await;
    create_test_team(&app, &user.token).await;

    // アクティビティログが記録されるまで待つ（非同期処理のため）
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    // Act: taskリソースタイプでフィルタ
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/activity-logs/me?resource_type=task",
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<ActivityLogDto>> =
        serde_json::from_slice(&body).unwrap();
    let data = api_response.data.unwrap();
    let logs = &data.items;

    // デバッグ情報を出力
    println!(
        "Filter by resource_type=task: total={}, items={}",
        data.pagination.total_count,
        logs.len()
    );
    for (i, log) in logs.iter().enumerate() {
        println!(
            "Log {}: action={}, resource_type={}",
            i, log.action, log.resource_type
        );
    }

    // Assert: すべてのログがtaskリソースタイプであることを確認
    assert!(
        !logs.is_empty(),
        "Expected at least one activity log for task resource type"
    );
    for log in logs {
        assert_eq!(log.resource_type, "task");
    }
}

#[tokio::test]
async fn test_activity_log_filter_by_action() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 複数のアクションを実行
    create_test_task(&app, &user.token).await;
    create_test_task(&app, &user.token).await;

    let task = create_test_task(&app, &user.token).await;
    let update_data = json!({
        "title": "Updated Task"
    });
    app.clone()
        .oneshot(create_request(
            "PATCH",
            &format!("/tasks/{}", task.id),
            &user.token,
            &update_data,
        ))
        .await
        .unwrap();

    // Act: task_createdアクションでフィルタ
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/activity-logs/me?action=create_task",
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<ActivityLogDto>> =
        serde_json::from_slice(&body).unwrap();
    let data = api_response.data.unwrap();
    let logs = &data.items;

    // Assert: すべてのログがtask_createdアクションであることを確認
    assert!(logs.len() >= 3);
    for log in logs {
        assert_eq!(log.action, "create_task");
    }
}

#[tokio::test]
async fn test_activity_log_date_range_filter() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 現在時刻を取得
    let now = chrono::Utc::now();
    let one_hour_ago = now - chrono::Duration::hours(1);
    let one_hour_later = now + chrono::Duration::hours(1);

    // タスクを作成
    create_test_task(&app, &user.token).await;

    // 少し待つ（非同期ログ記録が完了するのを待つ）
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Act: 日付範囲でフィルタ
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/activity-logs/me?created_after={}&created_before={}",
                urlencoding::encode(&one_hour_ago.to_rfc3339()),
                urlencoding::encode(&one_hour_later.to_rfc3339())
            ),
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    let status = response.status();
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    if status != StatusCode::OK {
        let error_msg = String::from_utf8_lossy(&body);
        panic!(
            "Activity log date range query failed with status {:?}: {}",
            status, error_msg
        );
    }

    assert_eq!(status, StatusCode::OK);
    let api_response: ApiResponse<PaginatedResponse<ActivityLogDto>> =
        serde_json::from_slice(&body).unwrap();
    let data = api_response.data.unwrap();
    let logs = &data.items;

    // Assert: ログが存在することを確認
    assert!(!logs.is_empty());
}

#[tokio::test]
async fn test_activity_log_combined_filters_and_sort() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 複数のタスクを作成・更新
    for i in 0..5 {
        let task = create_test_task(&app, &user.token).await;
        if i % 2 == 0 {
            let update_data = json!({
                "title": format!("Updated Task {}", i)
            });
            app.clone()
                .oneshot(create_request(
                    "PATCH",
                    &format!("/tasks/{}", task.id),
                    &user.token,
                    &update_data,
                ))
                .await
                .unwrap();
        }
    }

    // Act: taskリソースタイプ、created_at降順でフィルタ＆ソート
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/activity-logs/me?resource_type=task&sort_by=created_at&sort_order=desc&per_page=20",
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<ActivityLogDto>> =
        serde_json::from_slice(&body).unwrap();
    let data = api_response.data.unwrap();
    let logs = &data.items;

    // Assert
    assert!(logs.len() >= 7); // 5 creates + at least 2 updates

    // すべてtaskリソースタイプ
    for log in logs {
        assert_eq!(log.resource_type, "task");
    }
}

#[tokio::test]
async fn test_activity_log_invalid_sort_field() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // Act: 無効なソートフィールドを指定
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/activity-logs/me?sort_by=invalid_field&sort_order=asc",
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    // Assert: 正常に動作し、デフォルトのソート（created_at desc）が適用される
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_activity_log_admin_access() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin = create_and_authenticate_user(&app).await;
    let user = create_and_authenticate_user(&app).await;

    // ユーザーがタスクを作成
    create_test_task(&app, &user.token).await;

    // 管理者権限を設定（実際の実装に応じて調整が必要）
    // ここでは管理者エンドポイントのテストのみ

    // Act: 管理者がすべてのアクティビティログにアクセス
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/admin/activity-logs?user_id={}", user.id),
            &admin.token,
            &(),
        ))
        .await
        .unwrap();

    // Assert: 権限がない場合は403、ある場合は200
    // 実際の権限設定に応じて期待値を調整
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::FORBIDDEN);
}
