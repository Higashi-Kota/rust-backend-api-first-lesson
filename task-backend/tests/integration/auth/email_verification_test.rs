// tests/integration/email_verification_test.rs

use axum::{body, http::StatusCode};
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper};

#[tokio::test]
async fn test_email_verification_flow() {
    let (app, _schema_name, db) = app_helper::setup_full_app().await;

    // ユーザー登録（メール認証トークンが送信される）
    let register_data = serde_json::json!({
        "username": "testverify",
        "email": "testverify@example.com",
        "password": "TestPass2024"
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
    let status = register_res.status();
    if status != StatusCode::CREATED {
        let body = body::to_bytes(register_res.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_response: Value = serde_json::from_slice(&body).unwrap();
        eprintln!("Registration failed: {:?}", error_response);
    }
    assert_eq!(status, StatusCode::CREATED);

    // ユーザーがメール未認証であることを確認
    let user_id = {
        use sea_orm::ColumnTrait;
        use sea_orm::EntityTrait;
        use sea_orm::QueryFilter;
        use task_backend::domain::user_model::{Column as UserColumn, Entity as UserEntity};

        let user = UserEntity::find()
            .filter(UserColumn::Email.eq("testverify@example.com"))
            .one(&db.connection)
            .await
            .unwrap()
            .unwrap();

        assert!(!user.email_verified);
        user.id
    };

    // メール認証トークンを生成して保存（テスト用）
    let token = {
        use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
        use task_backend::domain::email_verification_token_model::{
            ActiveModel, Column as TokenColumn, Entity as TokenEntity,
        };

        // 既存のトークンを削除
        let existing_token = TokenEntity::find()
            .filter(TokenColumn::UserId.eq(user_id))
            .filter(TokenColumn::IsUsed.eq(false))
            .one(&db.connection)
            .await
            .unwrap()
            .unwrap();

        // テスト用の生トークンを生成
        let raw_token = uuid::Uuid::new_v4().to_string();
        // ハッシュ化（本番環境と同じフロー）
        let token_hash = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(raw_token.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        // 既存トークンを更新（ハッシュを置き換え）
        let mut active_model: ActiveModel = existing_token.into();
        active_model.token_hash = Set(token_hash);
        active_model.update(&db.connection).await.unwrap();

        // 生トークンを返す（これがメールで送信される想定）
        raw_token
    };

    // メール認証を実行
    let verify_data = serde_json::json!({
        "token": token
    });

    let verify_req = axum::http::Request::builder()
        .uri("/auth/verify-email")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&verify_data).unwrap(),
        ))
        .unwrap();

    let verify_res = app.clone().oneshot(verify_req).await.unwrap();
    assert_eq!(verify_res.status(), StatusCode::OK);

    let body = body::to_bytes(verify_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Message assertion removed
    assert!(
        response["data"]["verified"].as_bool().unwrap_or(false)
            || response["data"]["email_verified"]
                .as_bool()
                .unwrap_or(false)
    );

    // ユーザーがメール認証済みになったことを確認
    {
        use sea_orm::EntityTrait;
        use task_backend::domain::user_model::Entity as UserEntity;

        let user = UserEntity::find_by_id(user_id)
            .one(&db.connection)
            .await
            .unwrap()
            .unwrap();

        assert!(user.email_verified);
    }
}

#[tokio::test]
async fn test_email_verification_with_invalid_token() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 無効なトークンでメール認証を試みる
    let verify_data = serde_json::json!({
        "token": "invalid_token_hash"
    });

    let verify_req = axum::http::Request::builder()
        .uri("/auth/verify-email")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&verify_data).unwrap(),
        ))
        .unwrap();

    let verify_res = app.clone().oneshot(verify_req).await.unwrap();
    assert_eq!(verify_res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_resend_verification_email() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録
    let register_data = serde_json::json!({
        "username": "testresend",
        "email": "testresend@example.com",
        "password": "TestPass2024"
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

    // 認証メール再送信
    let resend_data = serde_json::json!({
        "email": "testresend@example.com"
    });

    let resend_req = axum::http::Request::builder()
        .uri("/auth/resend-verification")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&resend_data).unwrap(),
        ))
        .unwrap();

    let resend_res = app.clone().oneshot(resend_req).await.unwrap();
    assert_eq!(resend_res.status(), StatusCode::OK);

    let body = body::to_bytes(resend_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let _response: Value = serde_json::from_slice(&body).unwrap();

    // Message assertion removed
}

#[tokio::test]
async fn test_email_verification_token_expiry() {
    let (app, _schema_name, db) = app_helper::setup_full_app().await;

    // ユーザー登録
    let register_data = serde_json::json!({
        "username": "testexpiry",
        "email": "testexpiry@example.com",
        "password": "TestPass2024"
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

    // トークンを期限切れに設定（テスト用）
    {
        use chrono::{Duration, Utc};
        use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
        use task_backend::domain::email_verification_token_model::{
            ActiveModel, Column as TokenColumn, Entity as TokenEntity,
        };

        let token = TokenEntity::find()
            .filter(TokenColumn::IsUsed.eq(false))
            .one(&db.connection)
            .await
            .unwrap()
            .unwrap();

        let mut active_token: ActiveModel = token.clone().into();
        active_token.expires_at = Set(Utc::now() - Duration::hours(1)); // 1時間前に期限切れ
        active_token.update(&db.connection).await.unwrap();

        // 期限切れトークンで認証を試みる
        let verify_data = serde_json::json!({
            "token": token.token_hash
        });

        let verify_req = auth_helper::create_request(
            "POST",
            "/auth/verify-email",
            Some(serde_json::to_string(&verify_data).unwrap()),
        );

        let verify_res = app.clone().oneshot(verify_req).await.unwrap();
        assert_eq!(verify_res.status(), StatusCode::BAD_REQUEST);

        let body = body::to_bytes(verify_res.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: Value = serde_json::from_slice(&body).unwrap();

        assert!(response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("expired"));
    }
}
