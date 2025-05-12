// tests/integration/api_tests.rs
use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use task_backend::{
    api::{
        dto::task_dto::{CreateTaskDto, TaskDto, UpdateTaskDto},
        handlers::task_handler::{task_router, AppState},
    },
    service::task_service::TaskService,
};
use tower::ServiceExt;
use uuid::Uuid;

use crate::common;

// APIテスト用ヘルパー関数 - /health エンドポイントを追加
async fn setup_app() -> Router {
    let db = common::db::TestDatabase::new().await;
    // connection を clone して使用
    let service = std::sync::Arc::new(TaskService::new(db.connection.clone()));
    let app_state = AppState {
        task_service: service,
    };

    task_router(app_state)
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = setup_app().await;

    let req = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn test_create_task_endpoint() {
    let app = setup_app().await;

    let task_dto = common::create_test_task();

    let req = Request::builder()
        .uri("/tasks")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&task_dto).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let task: TaskDto = serde_json::from_slice(&body).unwrap();

    assert_eq!(task.title, "Test Task");
    assert_eq!(task.status, "todo");
    assert!(common::is_valid_uuid(&task));
}

#[tokio::test]
async fn test_get_task_endpoint() {
    let app = setup_app().await;

    // まずタスクを作成
    let task_dto = common::create_test_task();

    let create_req = Request::builder()
        .uri("/tasks")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&task_dto).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let created_task: TaskDto = serde_json::from_slice(&body).unwrap();

    // 作成したタスクを取得
    let get_req = Request::builder()
        .uri(format!("/tasks/{}", created_task.id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(get_req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let task: TaskDto = serde_json::from_slice(&body).unwrap();

    assert_eq!(task.id, created_task.id);
    assert_eq!(task.title, "Test Task");
}

#[tokio::test]
async fn test_list_tasks_endpoint() {
    let app = setup_app().await;

    // いくつかのタスクを作成
    for i in 1..=3 {
        let task_dto = common::create_test_task_with_title(&format!("API Task {}", i));

        let create_req = Request::builder()
            .uri("/tasks")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&task_dto).unwrap()))
            .unwrap();

        app.clone().oneshot(create_req).await.unwrap();
    }

    // タスク一覧を取得
    let req = Request::builder()
        .uri("/tasks")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let tasks: Vec<TaskDto> = serde_json::from_slice(&body).unwrap();

    assert!(tasks.len() >= 3);
}

#[tokio::test]
async fn test_update_task_endpoint() {
    let app = setup_app().await;

    // まずタスクを作成
    let task_dto = common::create_test_task();

    let create_req = Request::builder()
        .uri("/tasks")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&task_dto).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(create_req).await.unwrap();
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let created_task: TaskDto = serde_json::from_slice(&body).unwrap();

    // 更新データを準備
    let update_dto = UpdateTaskDto {
        title: Some("Updated via API".to_string()),
        status: Some("in_progress".to_string()),
        description: None,
        due_date: None,
    };

    // タスクを更新
    let update_req = Request::builder()
        .uri(format!("/tasks/{}", created_task.id))
        .method("PATCH")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&update_dto).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(update_req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let updated_task: TaskDto = serde_json::from_slice(&body).unwrap();

    assert_eq!(updated_task.id, created_task.id);
    assert_eq!(updated_task.title, "Updated via API");
    assert_eq!(updated_task.status, "in_progress");
    // descriptionは更新していないので元の値が保持されているはず
    assert_eq!(updated_task.description, created_task.description);
}

#[tokio::test]
async fn test_delete_task_endpoint() {
    let app = setup_app().await;

    // まずタスクを作成
    let task_dto = common::create_test_task();

    let create_req = Request::builder()
        .uri("/tasks")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&task_dto).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(create_req).await.unwrap();
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let created_task: TaskDto = serde_json::from_slice(&body).unwrap();

    // タスクを削除
    let delete_req = Request::builder()
        .uri(format!("/tasks/{}", created_task.id))
        .method("DELETE")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(delete_req).await.unwrap();

    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // 削除されたことを確認
    let get_req = Request::builder()
        .uri(format!("/tasks/{}", created_task.id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(get_req).await.unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_batch_operations_endpoints() {
    let app = setup_app().await;

    // バッチ作成テスト
    let batch_create_payload = json!({
        "tasks": [
            {
                "title": "Batch API Task 1",
                "description": "First batch task",
                "status": "todo"
            },
            {
                "title": "Batch API Task 2",
                "description": "Second batch task",
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

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let create_result: Value = serde_json::from_slice(&body).unwrap();

    // 作成されたタスクのIDを取得
    let created_tasks = create_result["created_tasks"].as_array().unwrap();
    assert_eq!(created_tasks.len(), 2);

    let id1 = Uuid::parse_str(created_tasks[0]["id"].as_str().unwrap()).unwrap();
    let id2 = Uuid::parse_str(created_tasks[1]["id"].as_str().unwrap()).unwrap();

    // バッチ更新テスト
    let batch_update_payload = json!({
        "tasks": [
            {
                "id": id1.to_string(),
                "title": "Updated Batch API Task 1",
                "status": "in_progress"
            },
            {
                "id": id2.to_string(),
                "title": "Updated Batch API Task 2",
                "status": "in_progress"
            }
        ]
    });

    let req = Request::builder()
        .uri("/tasks/batch/update")
        .method("PATCH")
        .header("Content-Type", "application/json")
        .body(Body::from(batch_update_payload.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let update_result: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(update_result["updated_count"], 2);

    // 更新を確認
    let req1 = Request::builder()
        .uri(format!("/tasks/{}", id1))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let req2 = Request::builder()
        .uri(format!("/tasks/{}", id2))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    let res2 = app.clone().oneshot(req2).await.unwrap();

    assert_eq!(res1.status(), StatusCode::OK);
    assert_eq!(res2.status(), StatusCode::OK);

    let body1 = body::to_bytes(res1.into_body(), usize::MAX).await.unwrap();
    let body2 = body::to_bytes(res2.into_body(), usize::MAX).await.unwrap();

    let task1: TaskDto = serde_json::from_slice(&body1).unwrap();
    let task2: TaskDto = serde_json::from_slice(&body2).unwrap();

    assert_eq!(task1.title, "Updated Batch API Task 1");
    assert_eq!(task2.title, "Updated Batch API Task 2");
    assert_eq!(task1.status, "in_progress");
    assert_eq!(task2.status, "in_progress");

    // バッチ削除テスト
    let batch_delete_payload = json!({
        "ids": [id1.to_string(), id2.to_string()]
    });

    let req = Request::builder()
        .uri("/tasks/batch/delete")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(batch_delete_payload.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let delete_result: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(delete_result["deleted_count"], 2);

    // 削除を確認
    let req1 = Request::builder()
        .uri(format!("/tasks/{}", id1))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let req2 = Request::builder()
        .uri(format!("/tasks/{}", id2))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    let res2 = app.clone().oneshot(req2).await.unwrap();

    assert_eq!(res1.status(), StatusCode::NOT_FOUND);
    assert_eq!(res2.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_filter_endpoint() {
    let app = setup_app().await;

    // テストデータを作成
    let tasks = [
        CreateTaskDto {
            title: "Important API Task".to_string(),
            description: Some("High priority".to_string()),
            status: Some("todo".to_string()),
            due_date: None,
        },
        CreateTaskDto {
            title: "Regular API Task".to_string(),
            description: Some("Medium priority".to_string()),
            status: Some("in_progress".to_string()),
            due_date: None,
        },
        CreateTaskDto {
            title: "Another Important API Task".to_string(),
            description: Some("Also high priority".to_string()),
            status: Some("todo".to_string()),
            due_date: None,
        },
    ];

    for task in &tasks {
        let req = Request::builder()
            .uri("/tasks")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(task).unwrap()))
            .unwrap();

        app.clone().oneshot(req).await.unwrap();
    }

    // ステータスでフィルタリング
    let req = Request::builder()
        .uri("/tasks/filter?status=todo")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    let filtered_tasks = result["tasks"].as_array().unwrap();
    assert!(filtered_tasks.len() >= 2);

    // タイトルでフィルタリング
    let req = Request::builder()
        .uri("/tasks/filter?title_contains=Important")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    let filtered_tasks = result["tasks"].as_array().unwrap();
    assert!(filtered_tasks.len() >= 2);

    // 複合フィルタリング
    let req = Request::builder()
        .uri("/tasks/filter?status=todo&title_contains=Important")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    let filtered_tasks = result["tasks"].as_array().unwrap();
    assert!(!filtered_tasks.is_empty());

    // 無効なフィルター
    let req = Request::builder()
        .uri("/tasks/filter?status=nonexistent&title_contains=nonexistent")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    let filtered_tasks = result["tasks"].as_array().unwrap();
    assert!(filtered_tasks.is_empty());
}

#[tokio::test]
async fn test_pagination_endpoint() {
    let app = setup_app().await;

    // ページネーションテスト用のタスクを作成
    for i in 1..=12 {
        let task = CreateTaskDto {
            title: format!("Pagination API Task {}", i),
            description: Some("For pagination test".to_string()),
            status: Some("todo".to_string()),
            due_date: None,
        };

        let req = Request::builder()
            .uri("/tasks")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&task).unwrap()))
            .unwrap();

        app.clone().oneshot(req).await.unwrap();
    }

    // 1ページ目を取得
    let req = Request::builder()
        .uri("/tasks/paginated?page=1&page_size=5")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let page1_result: Value = serde_json::from_slice(&body).unwrap();

    let page1_tasks = page1_result["tasks"].as_array().unwrap();
    assert_eq!(page1_tasks.len(), 5);

    let pagination = &page1_result["pagination"];
    assert_eq!(pagination["current_page"], 1);
    assert_eq!(pagination["page_size"], 5);
    assert!(pagination["total_items"].as_u64().unwrap() >= 12);
    assert_eq!(pagination["has_next_page"], true);
    assert_eq!(pagination["has_previous_page"], false);

    // 2ページ目を取得
    let req = Request::builder()
        .uri("/tasks/paginated?page=2&page_size=5")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let page2_result: Value = serde_json::from_slice(&body).unwrap();

    let page2_tasks = page2_result["tasks"].as_array().unwrap();
    assert_eq!(page2_tasks.len(), 5);

    let pagination = &page2_result["pagination"];
    assert_eq!(pagination["current_page"], 2);
    assert_eq!(pagination["has_previous_page"], true);
}

#[tokio::test]
async fn test_validation() {
    let app = setup_app().await;

    // 空のタイトルを持つ無効なタスク
    let invalid_task = json!({
        "title": "",
        "description": "This should fail validation",
        "status": "todo"
    });

    let req = Request::builder()
        .uri("/tasks")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(invalid_task.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "validation_errors");
    assert!(!error["errors"].as_array().unwrap().is_empty());

    // 無効なステータス値を持つタスク
    let invalid_task = json!({
        "title": "Task with Invalid Status",
        "description": "This should fail validation",
        "status": "invalid_status"
    });

    let req = Request::builder()
        .uri("/tasks")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(invalid_task.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "validation_errors");
    assert!(!error["errors"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_not_found_error() {
    let app = setup_app().await;

    // 存在しないIDでタスクを取得
    let non_existent_id = Uuid::new_v4();
    let req = Request::builder()
        .uri(format!("/tasks/{}", non_existent_id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "not_found");
    assert!(error["error"]
        .as_str()
        .unwrap()
        .contains(&non_existent_id.to_string()));
}

#[tokio::test]
async fn test_invalid_uuid_error() {
    let app = setup_app().await;

    // 無効なUUIDでタスクを取得
    let req = Request::builder()
        .uri("/tasks/not-a-uuid")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "invalid_uuid");
}
