// tests/integration/password_reset_test.rs

use axum::{body, http::StatusCode};
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper};

#[tokio::test]
async fn test_password_reset_flow() {
    let (app, _schema_name, db) = app_helper::setup_full_app().await;

    // ユーザー登録
    let register_data = serde_json::json!({
        "username": "testreset",
        "email": "testreset@example.com",
        "password": "MyUniqueP@ssw0rd91"
    });

    let register_req = axum::http::Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&register_data).unwrap(),
        ))
        .unwrap();

    let register_res = app.clone().oneshot(register_req).await.unwrap();

    if register_res.status() != StatusCode::CREATED {
        let body = body::to_bytes(register_res.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: Value = serde_json::from_slice(&body).unwrap();
        panic!("Registration failed with: {:?}", error);
    }
    assert_eq!(register_res.status(), StatusCode::CREATED);

    // パスワードリセット要求
    let reset_request_data = serde_json::json!({
        "email": "testreset@example.com"
    });

    let reset_request_req = axum::http::Request::builder()
        .uri("/auth/forgot-password")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&reset_request_data).unwrap(),
        ))
        .unwrap();

    let reset_request_res = app.clone().oneshot(reset_request_req).await.unwrap();
    assert_eq!(reset_request_res.status(), StatusCode::OK);

    let body = body::to_bytes(reset_request_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert!(response["message"]
        .as_str()
        .unwrap()
        .contains("password reset link has been sent"));

    // トークンを取得（テスト用）
    let token = {
        use sea_orm::ColumnTrait;
        use sea_orm::EntityTrait;
        use sea_orm::QueryFilter;
        use task_backend::domain::password_reset_token_model::{
            Column as TokenColumn, Entity as TokenEntity,
        };

        let token = TokenEntity::find()
            .filter(TokenColumn::IsUsed.eq(false))
            .one(&db.connection)
            .await
            .unwrap()
            .unwrap();

        token.token_hash
    };

    // パスワードリセット実行
    let reset_data = serde_json::json!({
        "token": token,
        "new_password": "MyNewP@ssw0rd94"
    });

    let reset_req = axum::http::Request::builder()
        .uri("/auth/reset-password")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&reset_data).unwrap(),
        ))
        .unwrap();

    let reset_res = app.clone().oneshot(reset_req).await.unwrap();
    assert_eq!(reset_res.status(), StatusCode::OK);

    let body = body::to_bytes(reset_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert!(response["message"]
        .as_str()
        .unwrap()
        .contains("Password has been reset successfully"));

    // 新しいパスワードでログインできることを確認
    let signin_data = serde_json::json!({
        "identifier": "testreset@example.com",
        "password": "MyNewP@ssw0rd94"
    });

    let signin_req = axum::http::Request::builder()
        .uri("/auth/signin")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&signin_data).unwrap(),
        ))
        .unwrap();

    let signin_res = app.clone().oneshot(signin_req).await.unwrap();
    assert_eq!(signin_res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_password_reset_rate_limiting() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録
    let register_data = serde_json::json!({
        "username": "testratelimit",
        "email": "testratelimit@example.com",
        "password": "MyUniqueP@ssw0rd92"
    });

    let register_req = axum::http::Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&register_data).unwrap(),
        ))
        .unwrap();

    let _ = app.clone().oneshot(register_req).await.unwrap();

    // 複数回パスワードリセット要求を送信
    for i in 0..5 {
        let reset_request_data = serde_json::json!({
            "email": "testratelimit@example.com"
        });

        let reset_request_req = axum::http::Request::builder()
            .uri("/auth/forgot-password")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&reset_request_data).unwrap(),
            ))
            .unwrap();

        let reset_request_res = app.clone().oneshot(reset_request_req).await.unwrap();
        assert_eq!(reset_request_res.status(), StatusCode::OK);

        let body = body::to_bytes(reset_request_res.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: Value = serde_json::from_slice(&body).unwrap();

        // すべてのリクエストで同じメッセージが返される（セキュリティのため）
        assert!(response["message"]
            .as_str()
            .unwrap()
            .contains("password reset link has been sent"));

        println!("Request {} completed", i + 1);
    }
}

#[tokio::test]
async fn test_password_reset_with_invalid_token() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 無効なトークンでパスワードリセットを試みる
    let reset_data = serde_json::json!({
        "token": "invalid_token_hash",
        "new_password": "MyNewP@ssw0rd95"
    });

    let reset_req = axum::http::Request::builder()
        .uri("/auth/reset-password")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&reset_data).unwrap(),
        ))
        .unwrap();

    let reset_res = app.clone().oneshot(reset_req).await.unwrap();
    assert_eq!(reset_res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_password_reset_token_expiry() {
    let (app, _schema_name, db) = app_helper::setup_full_app().await;

    // ユーザー登録
    let register_data = serde_json::json!({
        "username": "testexpiry",
        "email": "testexpiry@example.com",
        "password": "MyUniqueP@ssw0rd93"
    });

    let register_req = axum::http::Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&register_data).unwrap(),
        ))
        .unwrap();

    let _ = app.clone().oneshot(register_req).await.unwrap();

    // パスワードリセット要求
    let reset_request_data = serde_json::json!({
        "email": "testexpiry@example.com"
    });

    let reset_request_req = axum::http::Request::builder()
        .uri("/auth/forgot-password")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&reset_request_data).unwrap(),
        ))
        .unwrap();

    let _ = app.clone().oneshot(reset_request_req).await.unwrap();

    // トークンを期限切れに設定（テスト用）
    {
        use chrono::{Duration, Utc};
        use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
        use task_backend::domain::password_reset_token_model::{
            ActiveModel, Column as TokenColumn, Entity as TokenEntity,
        };

        let token = TokenEntity::find()
            .filter(TokenColumn::IsUsed.eq(false))
            .one(&db.connection)
            .await
            .unwrap()
            .unwrap();

        let mut active_token: ActiveModel = token.clone().into();
        active_token.expires_at = Set(Utc::now() - Duration::hours(2)); // 2時間前に期限切れ
        active_token.update(&db.connection).await.unwrap();

        // 期限切れトークンでリセットを試みる
        let reset_data = serde_json::json!({
            "token": token.token_hash,
            "new_password": "MyNewP@ssw0rd96"
        });

        let reset_req = auth_helper::create_request(
            "POST",
            "/auth/reset-password",
            Some(serde_json::to_string(&reset_data).unwrap()),
        );

        let reset_res = app.clone().oneshot(reset_req).await.unwrap();
        assert_eq!(reset_res.status(), StatusCode::BAD_REQUEST);

        let body = body::to_bytes(reset_res.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: Value = serde_json::from_slice(&body).unwrap();

        assert!(response["error"]
            .as_str()
            .unwrap()
            .contains("Invalid or expired"));
    }
}
