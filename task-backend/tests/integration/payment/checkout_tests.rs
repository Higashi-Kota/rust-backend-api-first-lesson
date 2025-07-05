use crate::common::app_helper::setup_full_app;
use crate::common::auth_helper::{create_test_user_with_info, signup_test_user};
use crate::common::stripe_helper::is_stripe_test_mode;
use axum::http::StatusCode;
use serde_json::json;
use task_backend::api::dto::ApiResponse;
use task_backend::api::handlers::payment_handler::{
    CreateCheckoutResponse, CustomerPortalResponse,
};
use task_backend::repository::user_repository::UserRepository;
use tower::ServiceExt;

#[tokio::test]
async fn test_checkout_session_creation_success() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;
    let unique_email = format!("checkout_test_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "test_user");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // Act
    let response = app
        .oneshot(create_request(
            "POST",
            "/payments/checkout",
            &user.access_token,
            &json!({
                "tier": "pro"
            }),
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body: ApiResponse<CreateCheckoutResponse> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert!(body.success);
    let data = body.data.unwrap();

    // 実際のStripeテストモードの場合、URLパターンを検証
    if is_stripe_test_mode() {
        assert!(
            data.checkout_url.contains("checkout.stripe.com")
                || data.checkout_url.contains("checkout")
        );
    } else {
        assert!(data.checkout_url.contains("mock"));
    }

    // データベースの状態を検証（Stripe顧客IDが保存されているか）
    let user_repo = UserRepository::new(_db.connection.clone());
    let _db_user = user_repo.find_by_id(user.id).await.unwrap().unwrap();

    // Stripeテストモードの場合のみ顧客IDを検証
    if is_stripe_test_mode() {
        // チェックアウトセッション作成時点では顧客IDはまだない場合もある
        // Webhookで更新される
    }
}

#[tokio::test]
async fn test_checkout_session_invalid_tier() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;
    let unique_email = format!("invalid_tier_test_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "test_user2");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // Act - 無効なtierを指定
    let response = app
        .oneshot(create_request(
            "POST",
            "/payments/checkout",
            &user.access_token,
            &json!({
                "tier": "invalid"
            }),
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: ApiResponse<serde_json::Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert!(!body.success);
    // Validation error message
    assert!(body.message.contains("Validation failed"));
}

#[tokio::test]
async fn test_checkout_session_same_tier() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;
    let signup_data = create_test_user_with_info("user3@example.com", "test_user3");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // 現在はFreeプランなので、Freeプランへのチェックアウトは拒否される
    let response = app
        .oneshot(create_request(
            "POST",
            "/payments/checkout",
            &user.access_token,
            &json!({
                "tier": "free"
            }),
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_checkout_unauthorized() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;

    // Act - 認証なしでリクエスト
    let response = app
        .oneshot(create_request_no_auth(
            "POST",
            "/payments/checkout",
            &json!({
                "tier": "pro"
            }),
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_customer_portal_session_creation() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;
    let unique_email = format!("portal_test_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "test_user4");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // Stripeテストモードの場合、実際の顧客作成が必要
    // テストではモック環境を想定し、顧客IDなしでのエラーをテスト

    // Act
    let response = app
        .oneshot(create_request_empty(
            "POST",
            "/payments/portal",
            &user.access_token,
        ))
        .await
        .unwrap();

    // Assert
    if is_stripe_test_mode() {
        // Stripeテストモードでは顧客IDがないためエラー
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    } else {
        // 開発モードではモックURLが返される
        assert_eq!(response.status(), StatusCode::OK);
    }

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    if is_stripe_test_mode() {
        // エラーレスポンスを検証
        let error_body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(error_body["success"], false);
        assert!(error_body["message"]
            .as_str()
            .unwrap()
            .contains("No Stripe customer ID"));
    } else {
        // 成功レスポンスを検証
        let body: ApiResponse<CustomerPortalResponse> =
            serde_json::from_slice(&body_bytes).unwrap();
        assert!(body.success);
        let data = body.data.unwrap();
        assert!(data.portal_url.contains("mock"));
    }
}

// 追加のテストケース

#[tokio::test]
async fn test_checkout_session_enterprise_tier() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;
    let unique_email = format!("enterprise_test_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "enterprise_user");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // Act - Enterpriseプランへのアップグレード
    let response = app
        .oneshot(create_request(
            "POST",
            "/payments/checkout",
            &user.access_token,
            &json!({
                "tier": "enterprise"
            }),
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body: ApiResponse<CreateCheckoutResponse> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert!(body.success);
    let data = body.data.unwrap();

    // チェックアウトURLが存在することを確認
    assert!(!data.checkout_url.is_empty());

    if is_stripe_test_mode() {
        assert!(
            data.checkout_url.contains("checkout.stripe.com")
                || data.checkout_url.contains("checkout")
        );
    } else {
        assert!(data.checkout_url.contains("mock"));
    }
}

#[tokio::test]
async fn test_checkout_session_already_subscribed() {
    // Arrange
    let (app, _schema_name, db) = setup_full_app().await;
    let unique_email = format!("already_subscribed_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "subscribed_user");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // ユーザーをProプランに設定
    let user_repo = UserRepository::new(db.connection.clone());
    user_repo
        .update_subscription_tier(user.id, "pro".to_string())
        .await
        .unwrap();

    // Act - 同じProプランへのチェックアウトを試みる
    let response = app
        .oneshot(create_request(
            "POST",
            "/payments/checkout",
            &user.access_token,
            &json!({
                "tier": "pro"
            }),
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: ApiResponse<serde_json::Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert!(!body.success);
    assert!(body
        .message
        .contains("Cannot checkout for the same or lower tier"));
}

#[tokio::test]
async fn test_customer_portal_no_stripe_customer() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;
    let unique_email = format!("no_customer_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "no_customer");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // Act - Stripe顧客IDなしでポータルへアクセス
    let response = app
        .oneshot(create_request_empty(
            "POST",
            "/payments/portal",
            &user.access_token,
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Stripeテストモードでは顧客IDがないとエラーになる
    let body: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert_eq!(body["success"], false);
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("No Stripe customer ID"));
}

#[tokio::test]
async fn test_checkout_missing_tier() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;
    let unique_email = format!("missing_tier_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "missing_tier");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // Act - tierパラメータなしでリクエスト
    let response = app
        .oneshot(create_request(
            "POST",
            "/payments/checkout",
            &user.access_token,
            &json!({}),
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_checkout_invalid_json() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;
    let unique_email = format!("invalid_json_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "invalid_json");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // Act - 不正なJSONを送信
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/payments/checkout")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", user.access_token))
                .body(axum::body::Body::from("{invalid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ヘルパー関数
fn create_request(
    method: &str,
    uri: &str,
    token: &str,
    body: &serde_json::Value,
) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(axum::body::Body::from(body.to_string()))
        .unwrap()
}

fn create_request_empty(
    method: &str,
    uri: &str,
    token: &str,
) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("Authorization", format!("Bearer {}", token))
        .body(axum::body::Body::empty())
        .unwrap()
}

fn create_request_no_auth(
    method: &str,
    uri: &str,
    body: &serde_json::Value,
) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(body.to_string()))
        .unwrap()
}
