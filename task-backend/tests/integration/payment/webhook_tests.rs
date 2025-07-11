use crate::common::app_helper::setup_full_app;
use crate::common::auth_helper::{create_test_user_with_info, signup_test_user};
use crate::common::stripe_helper::{
    create_checkout_completed_payload, create_payment_failed_payload,
    create_subscription_deleted_payload, generate_test_webhook_signature, is_stripe_test_mode,
};
use axum::http::StatusCode;
use serde_json::json;
use task_backend::core::subscription_tier::SubscriptionTier;
use task_backend::features::user::repositories::user::UserRepository;
use tower::ServiceExt;

#[tokio::test]
async fn test_webhook_missing_signature() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;

    let webhook_payload = json!({
        "type": "checkout.session.completed",
        "data": {
            "object": {
                "id": "cs_test_123",
                "metadata": {
                    "user_id": "00000000-0000-0000-0000-000000000000",
                    "tier": "pro"
                }
            }
        }
    });

    // Act - 署名なしでWebhookリクエスト
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/webhooks/stripe")
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(webhook_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_webhook_invalid_signature() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;

    let webhook_payload = json!({
        "type": "checkout.session.completed",
        "data": {
            "object": {
                "id": "cs_test_123",
                "metadata": {
                    "user_id": "00000000-0000-0000-0000-000000000000",
                    "tier": "pro"
                }
            }
        }
    });

    // Act - 無効な署名でWebhookリクエスト
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/webhooks/stripe")
                .header("Content-Type", "application/json")
                .header("stripe-signature", "invalid-signature")
                .body(axum::body::Body::from(webhook_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    // 開発モードではスキップされるため、OKが返る
    // Stripeテストモードでは署名検証が行われるため、BAD_REQUESTになる
    if is_stripe_test_mode() {
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    } else {
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_webhook_checkout_completed() {
    // Arrange
    let (app, _schema_name, db) = setup_full_app().await;
    let unique_email = format!("webhook_checkout_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "webhook_user");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // チェックアウト完了イベントのペイロードを作成
    let payload = create_checkout_completed_payload(
        "cs_test_123",
        "cus_test_123",
        "sub_test_123",
        &SubscriptionTier::Pro,
    );

    // 署名を生成
    let _signature = generate_test_webhook_signature(&payload);

    // メタデータにuser_idを追加
    let mut event: serde_json::Value = serde_json::from_str(&payload).unwrap();
    event["data"]["object"]["metadata"]["user_id"] = json!(user.id.to_string());
    let final_payload = event.to_string();
    let final_signature = generate_test_webhook_signature(&final_payload);

    // Act
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/webhooks/stripe")
                .header("Content-Type", "application/json")
                .header("stripe-signature", final_signature)
                .body(axum::body::Body::from(final_payload))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    // データベースでユーザーのサブスクリプションが更新されているか確認
    let user_repo = UserRepository::new(db.connection.clone());
    let updated_user = user_repo.find_by_id(user.id).await.unwrap().unwrap();

    // Stripeテストモードではサブスクリプションが更新される
    assert_eq!(updated_user.subscription_tier, "pro");

    // Webhookは顧客IDを設定しない（チェックアウト時に設定される）
    assert!(updated_user.stripe_customer_id.is_none());
}

#[tokio::test]
async fn test_webhook_subscription_deleted() {
    // Arrange
    let (app, _schema_name, db) = setup_full_app().await;
    let unique_email = format!("webhook_deleted_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "deleted_user");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // ユーザーをProプランに設定してStripe顧客IDを設定
    let user_repo = UserRepository::new(db.connection.clone());
    user_repo
        .update_subscription_tier(user.id, "pro".to_string())
        .await
        .unwrap();
    user_repo
        .update_stripe_customer_id(user.id, "cus_test_123")
        .await
        .unwrap();

    // サブスクリプション削除イベントのペイロードを作成
    let payload = create_subscription_deleted_payload("sub_test_123", "cus_test_123");
    let signature = generate_test_webhook_signature(&payload);

    // Act
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/webhooks/stripe")
                .header("Content-Type", "application/json")
                .header("stripe-signature", signature)
                .body(axum::body::Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    // データベースでユーザーのサブスクリプションがFreeに戻っているか確認
    let updated_user = user_repo.find_by_id(user.id).await.unwrap().unwrap();
    // 開発モードではWebhookがスキップされるため、サブスクリプションは変更されない
    if std::env::var("PAYMENT_DEVELOPMENT_MODE").unwrap_or_default() == "true" {
        assert_eq!(updated_user.subscription_tier, "pro"); // 元のままpro
    } else {
        assert_eq!(updated_user.subscription_tier, "free");
    }
}

#[tokio::test]
async fn test_webhook_payment_failed() {
    // Arrange
    let (app, _schema_name, db) = setup_full_app().await;
    let unique_email = format!("webhook_failed_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "failed_user");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // ユーザーをProプランに設定してStripe顧客IDを設定
    let user_repo = UserRepository::new(db.connection.clone());
    user_repo
        .update_subscription_tier(user.id, "pro".to_string())
        .await
        .unwrap();
    user_repo
        .update_stripe_customer_id(user.id, "cus_test_123")
        .await
        .unwrap();

    // 支払い失敗イベントのペイロードを作成
    let payload = create_payment_failed_payload("in_test_123", "cus_test_123", "sub_test_123");
    let signature = generate_test_webhook_signature(&payload);

    // Act
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/webhooks/stripe")
                .header("Content-Type", "application/json")
                .header("stripe-signature", signature)
                .body(axum::body::Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    // 支払い失敗時はサブスクリプションは維持される（猶予期間）
    let updated_user = user_repo.find_by_id(user.id).await.unwrap().unwrap();
    assert_eq!(updated_user.subscription_tier, "pro");
}

#[tokio::test]
async fn test_webhook_duplicate_event() {
    // Arrange
    let (app, _schema_name, db) = setup_full_app().await;
    let unique_email = format!("webhook_duplicate_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "duplicate_user");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // 同じイベントIDで2回送信
    let event_id = format!("evt_test_{}", uuid::Uuid::new_v4());
    let mut payload: serde_json::Value = serde_json::from_str(&create_checkout_completed_payload(
        "cs_test_123",
        "cus_test_123",
        "sub_test_123",
        &SubscriptionTier::Enterprise,
    ))
    .unwrap();

    // イベントIDを固定
    payload["id"] = json!(event_id.clone());
    payload["data"]["object"]["metadata"]["user_id"] = json!(user.id.to_string());

    let payload_string = payload.to_string();
    let signature = generate_test_webhook_signature(&payload_string);

    // Act - 1回目の送信
    let response1 = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/webhooks/stripe")
                .header("Content-Type", "application/json")
                .header("stripe-signature", signature.clone())
                .body(axum::body::Body::from(payload_string.clone()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::OK);

    // Act - 2回目の送信（重複）
    let response2 = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/webhooks/stripe")
                .header("Content-Type", "application/json")
                .header("stripe-signature", signature)
                .body(axum::body::Body::from(payload_string))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert - 2回目は同じ階層への変更となるため400エラー
    assert_eq!(response2.status(), StatusCode::BAD_REQUEST);

    // データベースで重複処理されていないことを確認
    let user_repo = UserRepository::new(db.connection.clone());
    let updated_user = user_repo.find_by_id(user.id).await.unwrap().unwrap();
    // 開発モードではWebhookがスキップされるため、サブスクリプションは変更されない
    if std::env::var("PAYMENT_DEVELOPMENT_MODE").unwrap_or_default() == "true" {
        assert_eq!(updated_user.subscription_tier, "free"); // 元のままfree
    } else {
        assert_eq!(updated_user.subscription_tier, "enterprise");
    }
}

#[tokio::test]
async fn test_webhook_unknown_event_type() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;

    // 未知のイベントタイプ（オブジェクトは有効なものを使用）
    let payload = crate::common::stripe_helper::create_test_webhook_payload(
        "unknown.event.type",
        json!({
            "id": "cus_test_123",
            "object": "customer",
            "email": "test@example.com"
        }),
    );
    let signature = generate_test_webhook_signature(&payload);

    // Act
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/webhooks/stripe")
                .header("Content-Type", "application/json")
                .header("stripe-signature", signature)
                .body(axum::body::Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert - 未知のイベントも正常に処理される（無視される）
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_webhook_malformed_json() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;

    let malformed_payload = "{invalid json";
    let signature = generate_test_webhook_signature(malformed_payload);

    // Act
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/webhooks/stripe")
                .header("Content-Type", "application/json")
                .header("stripe-signature", signature)
                .body(axum::body::Body::from(malformed_payload))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    // 開発モードではWebhookがスキップされるため、JSONパースエラーも発生しない
    if std::env::var("PAYMENT_DEVELOPMENT_MODE").unwrap_or_default() == "true" {
        assert_eq!(response.status(), StatusCode::OK);
    } else {
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn test_webhook_no_authentication_required() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;

    // Webhookエンドポイントは認証不要
    let webhook_payload = json!({
        "type": "checkout.session.completed",
        "data": {
            "object": {
                "id": "cs_test_123"
            }
        }
    });

    // Act - 認証ヘッダーなしでもアクセス可能
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/webhooks/stripe")
                .header("Content-Type", "application/json")
                .header("stripe-signature", "test-signature")
                .body(axum::body::Body::from(webhook_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert - 署名は無効でも、認証エラーではない
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}
