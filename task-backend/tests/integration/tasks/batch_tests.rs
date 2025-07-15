// tests/integration/tasks/batch_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper, test_data};

#[tokio::test]
async fn test_batch_create_tasks_with_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // バッチ作成データ
    let batch_create_payload = json!({
        "tasks": [
            {
                "title": "Batch Auth Task 1",
                "description": "First batch task with auth",
                "status": "todo"
            },
            {
                "title": "Batch Auth Task 2",
                "description": "Second batch task with auth",
                "status": "todo"
            },
            {
                "title": "Batch Auth Task 3",
                "description": "Third batch task with auth",
                "status": "in_progress"
            }
        ]
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks/batch/create",
        &user.access_token,
        Some(batch_create_payload.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    // 作成されたタスクの検証
    assert!(result["data"]["created_tasks"].is_array());
    let created_tasks = result["data"]["created_tasks"].as_array().unwrap();
    assert_eq!(created_tasks.len(), 3);

    // 全てのタスクが正しいユーザーIDを持つことを確認
    for task in created_tasks {
        assert!(task["id"].is_string());
        assert_eq!(task["user_id"], user.id.to_string());
        assert!(task["title"].as_str().unwrap().contains("Batch Auth Task"));
    }

    // 作成数の確認
    assert_eq!(result["data"]["created_count"], 3);
}

#[tokio::test]
async fn test_batch_create_tasks_without_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 認証なしでバッチ作成試行
    let batch_create_payload = json!({
        "tasks": [
            {
                "title": "Unauthorized Batch Task",
                "description": "This should fail",
                "status": "todo"
            }
        ]
    });

    let req = Request::builder()
        .uri("/tasks/batch/create")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(batch_create_payload.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    // Error response format
    assert!(!error["success"].as_bool().unwrap());
    assert!(error["data"].is_null());
    assert!(error["error"].is_object());
    assert_eq!(error["error"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn test_batch_update_tasks_with_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // まず複数のタスクを作成
    let mut task_ids = Vec::new();
    for i in 1..=3 {
        let task_data = test_data::create_test_task_with_title(&format!("Batch Update Task {}", i));
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);

        let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
            .await
            .unwrap();
        let created_task: Value = serde_json::from_slice(&create_body).unwrap();
        task_ids.push(created_task["data"]["id"].as_str().unwrap().to_string());
    }

    // バッチ更新
    let batch_update_payload = json!({
        "tasks": [
            {
                "id": task_ids[0],
                "title": "Updated Batch Task 1",
                "status": "in_progress"
            },
            {
                "id": task_ids[1],
                "title": "Updated Batch Task 2",
                "status": "completed"
            },
            {
                "id": task_ids[2],
                "status": "in_progress"
            }
        ]
    });

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        "/tasks/batch/update",
        &user.access_token,
        Some(batch_update_payload.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["data"]["updated_count"], 3);

    // 更新されたことを確認
    for (i, task_id) in task_ids.iter().enumerate() {
        let get_req = auth_helper::create_authenticated_request(
            "GET",
            &format!("/tasks/{}", task_id),
            &user.access_token,
            None,
        );

        let get_res = app.clone().oneshot(get_req).await.unwrap();
        assert_eq!(get_res.status(), StatusCode::OK);

        let get_body = body::to_bytes(get_res.into_body(), usize::MAX)
            .await
            .unwrap();
        let task: Value = serde_json::from_slice(&get_body).unwrap();

        if i < 2 {
            assert!(task["data"]["title"]
                .as_str()
                .unwrap()
                .contains("Updated Batch Task"));
        }
        assert_ne!(task["data"]["status"], "todo"); // 全て更新されている
    }
}

#[tokio::test]
async fn test_batch_update_tasks_of_another_user() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 複数ユーザーを作成
    let users = auth_helper::setup_multiple_users(&app, 2).await.unwrap();
    let user1 = &users[0];
    let user2 = &users[1];

    // user1でタスク作成
    let task_data = test_data::create_test_task();
    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user1.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_task: Value = serde_json::from_slice(&create_body).unwrap();
    let task_id = created_task["data"]["id"].as_str().unwrap();

    // user2でバッチ更新試行（user1のタスクを含む）
    let batch_update_payload = json!({
        "tasks": [
            {
                "id": task_id,
                "title": "Unauthorized Update",
                "status": "completed"
            }
        ]
    });

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        "/tasks/batch/update",
        &user2.access_token,
        Some(batch_update_payload.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();

    // 他のユーザーのタスクを含む場合、部分的に失敗またはエラー
    assert!(
        status == StatusCode::OK
            || status == StatusCode::BAD_REQUEST
            || status == StatusCode::FORBIDDEN
    );

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    if status == StatusCode::OK {
        // 更新数が0であることを確認
        assert_eq!(result["data"]["updated_count"], 0);
    }
}

#[tokio::test]
async fn test_batch_delete_tasks_with_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // まず複数のタスクを作成
    let mut task_ids = Vec::new();
    for i in 1..=3 {
        let task_data = test_data::create_test_task_with_title(&format!("Batch Delete Task {}", i));
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);

        let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
            .await
            .unwrap();
        let created_task: Value = serde_json::from_slice(&create_body).unwrap();
        task_ids.push(created_task["data"]["id"].as_str().unwrap().to_string());
    }

    // バッチ削除
    let batch_delete_payload = json!({
        "ids": task_ids
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks/batch/delete",
        &user.access_token,
        Some(batch_delete_payload.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["data"]["deleted_count"], 3);

    // 削除されたことを確認
    for task_id in &task_ids {
        let get_req = auth_helper::create_authenticated_request(
            "GET",
            &format!("/tasks/{}", task_id),
            &user.access_token,
            None,
        );

        let get_res = app.clone().oneshot(get_req).await.unwrap();
        assert_eq!(get_res.status(), StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn test_batch_delete_tasks_of_another_user() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 複数ユーザーを作成
    let users = auth_helper::setup_multiple_users(&app, 2).await.unwrap();
    let user1 = &users[0];
    let user2 = &users[1];

    // user1でタスク作成
    let task_data = test_data::create_test_task();
    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user1.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_task: Value = serde_json::from_slice(&create_body).unwrap();
    let task_id = created_task["data"]["id"].as_str().unwrap();

    // user2でバッチ削除試行（user1のタスクを含む）
    let batch_delete_payload = json!({
        "ids": [task_id]
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks/batch/delete",
        &user2.access_token,
        Some(batch_delete_payload.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    // 他のユーザーのタスクは削除されない
    assert_eq!(result["data"]["deleted_count"], 0);

    // 元のユーザーではまだアクセス可能であることを確認
    let get_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/tasks/{}", task_id),
        &user1.access_token,
        None,
    );

    let get_res = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_batch_operations_with_invalid_data() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 無効なデータでバッチ作成試行
    let invalid_batch_payload = json!({
        "tasks": [
            {
                "title": "",  // 空のタイトル
                "description": "Invalid task",
                "status": "todo"
            },
            {
                "title": "Valid Task",
                "description": "This is valid",
                "status": "invalid_status"  // 無効なステータス
            }
        ]
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks/batch/create",
        &user.access_token,
        Some(invalid_batch_payload.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    // 無効なステータス値による422エラーまたは400エラー（enum解析失敗）
    assert!(
        res.status() == StatusCode::UNPROCESSABLE_ENTITY || res.status() == StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_batch_operations_empty_arrays() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 空の配列でバッチ作成
    let empty_batch_payload = json!({
        "tasks": []
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks/batch/create",
        &user.access_token,
        Some(empty_batch_payload.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    // 空の配列の処理は実装による（成功またはバリデーションエラー）
    assert!(res.status() == StatusCode::CREATED || res.status() == StatusCode::BAD_REQUEST);

    if res.status() == StatusCode::CREATED {
        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(result["data"]["created_count"], 0);
    }
}

#[tokio::test]
async fn test_batch_operations_malformed_json() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 不正なJSONでバッチ作成試行
    let malformed_json = r#"{"tasks": [invalid json}"#;

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks/batch/create",
        &user.access_token,
        Some(malformed_json.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();

    // Try to parse as JSON, but handle the case where it might not be JSON
    match serde_json::from_slice::<Value>(&body) {
        Ok(error) => {
            assert!(
                error["error"]["code"] == "PARSE_ERROR"
                    || (error["error"]["code"] == "VALIDATION_ERROR"
                        || error["error"]["code"] == "VALIDATION_ERRORS")
                    || error["error"]["code"] == "BAD_REQUEST"
            );
        }
        Err(_) => {
            // Not JSON, which is also acceptable for malformed requests
            // Just verify we got a 400 status (which we already checked above)
        }
    }
}

#[tokio::test]
async fn test_batch_operations_large_dataset() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 大量のタスクでバッチ作成
    let mut tasks = Vec::new();
    for i in 1..=50 {
        tasks.push(json!({
            "title": format!("Large Batch Task {}", i),
            "description": format!("Task number {} in large batch", i),
            "status": "todo"
        }));
    }

    let large_batch_payload = json!({
        "tasks": tasks
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks/batch/create",
        &user.access_token,
        Some(large_batch_payload.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();

    // 大量データの処理は実装による（成功、制限エラー、タイムアウトなど）
    assert!(
        status == StatusCode::CREATED
            || status == StatusCode::BAD_REQUEST
            || status == StatusCode::REQUEST_TIMEOUT
    );

    if status == StatusCode::CREATED {
        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(result["data"]["created_count"], 50);
    }
}
