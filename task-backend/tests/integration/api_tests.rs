// tests/integration/api_tests.rs の全体的な修正コード

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
    Router,
};
use sea_orm::{ConnectionTrait, Statement}; // ConnectionTraitをインポート
use task_backend::{
    api::{
        dto::task_dto::TaskDto, // 未使用のDTOを削除
        handlers::task_handler::{task_router, AppState},
    },
    service::task_service::TaskService,
};
use tokio::sync::OnceCell;
use tower::ServiceExt;
// uuid::Uuid のインポートも削除（未使用）

use crate::common;

// APIテスト用のルーターを一度だけ初期化する
static APP: OnceCell<Router> = OnceCell::const_new();

// APIテスト用ヘルパー関数
async fn get_test_app() -> &'static Router {
    APP.get_or_init(|| async {
        // テストデータベースを作成
        let db = common::db::TestDatabase::new().await;

        // スキーマコンテキストの設定を確認・記録（デバッグ用）
        let schema_query = "SHOW search_path;";
        let schema_result = db
            .connection
            .query_one(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                schema_query.to_string(),
            ))
            .await
            .unwrap();

        println!("APIテスト用のsearch_path設定: {:?}", schema_result);

        // テストスキーマで検索パスを明示的に設定（念のため）
        let schema_name = db.get_schema_name();
        let set_search_path = format!("SET search_path TO \"{}\";", schema_name);
        db.connection
            .execute(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                set_search_path,
            ))
            .await
            .expect("search pathの設定に失敗");

        // TaskServiceを作成
        let service = std::sync::Arc::new(TaskService::new(db.connection.clone()));
        let app_state = AppState {
            task_service: service,
        };

        // Routerを作成して返す
        task_router(app_state)
    })
    .await
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = get_test_app().await;

    let req = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn test_create_task_endpoint() {
    let app = get_test_app().await;

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

// 他のテストも同様に修正

// ... 他のすべてのテストを同様に修正 ...
