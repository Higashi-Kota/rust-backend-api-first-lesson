use axum::http::StatusCode;
use task_backend::log_with_context;

#[tokio::test]
async fn test_structured_logging_macro() {
    // 構造化ログマクロの基本的な動作をテスト

    // コンテキストなしのログ
    log_with_context!(tracing::Level::INFO, "Test message without context");

    // コンテキスト付きのログ
    let user_id = uuid::Uuid::new_v4();
    let task_id = uuid::Uuid::new_v4();

    log_with_context!(
        tracing::Level::INFO,
        "Test message with context",
        "user_id" => user_id,
        "task_id" => task_id,
        "operation" => "test"
    );

    // エラーレベルのログ
    let error_message = "Test error";
    log_with_context!(
        tracing::Level::ERROR,
        "Error occurred during test",
        "error" => error_message,
        "user_id" => user_id
    );

    // 警告レベルのログ
    log_with_context!(
        tracing::Level::WARN,
        "Warning during test",
        "warning_type" => "test_warning"
    );

    // デバッグレベルのログ
    log_with_context!(
        tracing::Level::DEBUG,
        "Debug information",
        "debug_data" => "test_data"
    );
}

#[tokio::test]
async fn test_logging_middleware_integration() {
    use axum::{routing::get, Router};
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    // テスト用のルーターを作成
    let app = Router::new()
        .route("/test", get(|| async { "Test response" }))
        .route(
            "/error",
            get(|| async { (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error") }),
        )
        .layer(axum::middleware::from_fn(
            task_backend::logging::logging_middleware,
        ))
        .layer(axum::middleware::from_fn(
            task_backend::logging::inject_request_context,
        ));

    // テストサーバーを起動
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = TcpListener::bind(addr).await.unwrap();
    let actual_addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });

    // テストリクエストを送信
    let client = reqwest::Client::new();

    // 正常なリクエスト
    let response = client
        .get(format!("http://{}/test", actual_addr))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // エラーを返すリクエスト
    let response = client
        .get(format!("http://{}/error", actual_addr))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 500);

    // 存在しないパス
    let response = client
        .get(format!("http://{}/not-found", actual_addr))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_request_context_generation() {
    use axum::{extract::Extension, routing::get, Router};
    use task_backend::logging::RequestContext;

    // RequestContextが正しく生成されることを確認
    let app = Router::new()
        .route(
            "/context-test",
            get(|Extension(context): Extension<RequestContext>| async move {
                // RequestContextの各フィールドが設定されていることを確認
                assert!(!context.request_id.is_empty());
                assert_eq!(context.path, "/context-test");
                assert_eq!(context.method, "GET");
                assert_eq!(context.user_id, None); // 初期状態ではNone

                "Context test passed"
            }),
        )
        .layer(axum::middleware::from_fn(
            task_backend::logging::inject_request_context,
        ));

    // テストサーバーを起動
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let actual_addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });

    // テストリクエストを送信
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/context-test", actual_addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "Context test passed");
}
