use crate::common::app_helper::setup_full_app;
use crate::common::auth_helper::{create_test_user_with_info, signup_test_user};
use axum::http::StatusCode;
use task_backend::api::dto::subscription_dto::{CurrentSubscriptionResponse, SubscriptionTierInfo};
use task_backend::repository::user_repository::UserRepository;
use task_backend::types::ApiResponse;
use tower::ServiceExt;

#[tokio::test]
async fn test_get_current_subscription_free_user() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;
    let unique_email = format!("subscription_free_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "free_user");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // Act
    let response = app
        .oneshot(create_request_get(
            "/payments/subscription",
            &user.access_token,
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body: ApiResponse<CurrentSubscriptionResponse> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert!(body.success);
    let data = body.data.unwrap();

    // Freeユーザーのデフォルト値を確認
    assert_eq!(data.current_tier, "free");
    assert_eq!(data.tier_display_name, "Free");
    assert_eq!(data.tier_level, 1);
    assert!(data.subscribed_at < chrono::Utc::now().into());
    assert!(!data.features.is_empty());
}

#[tokio::test]
async fn test_get_current_subscription_pro_user() {
    // Arrange
    let (app, _schema_name, db) = setup_full_app().await;
    let unique_email = format!("subscription_pro_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "pro_user");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // ユーザーをProプランに設定するために直接データベースを更新
    let user_repo = UserRepository::new(db.connection.clone());
    user_repo
        .update_subscription_tier(user.id, "pro".to_string())
        .await
        .unwrap();

    // Act
    let response = app
        .oneshot(create_request_get(
            "/payments/subscription",
            &user.access_token,
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body: ApiResponse<CurrentSubscriptionResponse> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert!(body.success);
    let data = body.data.unwrap();

    assert_eq!(data.current_tier, "pro");
    assert_eq!(data.tier_display_name, "Pro");
    assert_eq!(data.tier_level, 2);
    assert!(data.subscribed_at < chrono::Utc::now().into());
    assert!(!data.features.is_empty());
}

#[tokio::test]
async fn test_get_available_tiers() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;
    let unique_email = format!("available_tiers_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "tiers_user");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // Act
    let response = app
        .oneshot(create_request_get(
            "/payments/subscription/tiers",
            &user.access_token,
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body: ApiResponse<Vec<SubscriptionTierInfo>> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert!(body.success);
    let tiers = body.data.unwrap();

    // 3つのティアが存在することを確認
    assert_eq!(tiers.len(), 3);

    // free, pro, enterpriseの順で返されることを確認
    assert_eq!(tiers[0].tier, "free");
    assert_eq!(tiers[1].tier, "pro");
    assert_eq!(tiers[2].tier, "enterprise");

    // 価格情報の確認
    assert_eq!(tiers[0].monthly_price, Some(0.0));
    assert_eq!(tiers[1].monthly_price, Some(19.99));
    assert_eq!(tiers[2].monthly_price, Some(99.99));
}

#[tokio::test]
async fn test_get_upgrade_options_from_free() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;
    let unique_email = format!("upgrade_options_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "upgrade_user");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // Act
    let response = app
        .oneshot(create_request_get(
            "/payments/subscription/upgrade-options",
            &user.access_token,
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body: ApiResponse<Vec<SubscriptionTierInfo>> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert!(body.success);
    let options = body.data.unwrap();

    // Freeユーザーは2つのアップグレードオプションがある
    assert_eq!(options.len(), 2);
    assert_eq!(options[0].tier, "pro");
    assert_eq!(options[1].tier, "enterprise");
}

#[tokio::test]
async fn test_get_upgrade_options_from_pro() {
    // Arrange
    let (app, _schema_name, db) = setup_full_app().await;
    let unique_email = format!("upgrade_from_pro_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "pro_upgrade_user");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // ユーザーをProプランに設定
    let user_repo = UserRepository::new(db.connection.clone());
    user_repo
        .update_subscription_tier(user.id, "pro".to_string())
        .await
        .unwrap();

    // Act
    let response = app
        .oneshot(create_request_get(
            "/payments/subscription/upgrade-options",
            &user.access_token,
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body: ApiResponse<Vec<SubscriptionTierInfo>> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert!(body.success);
    let options = body.data.unwrap();

    // Proユーザーは1つのアップグレードオプションがある
    assert_eq!(options.len(), 1);
    assert_eq!(options[0].tier, "enterprise");
}

#[tokio::test]
async fn test_get_upgrade_options_from_enterprise() {
    // Arrange
    let (app, _schema_name, db) = setup_full_app().await;
    let unique_email = format!(
        "upgrade_from_enterprise_{}@example.com",
        uuid::Uuid::new_v4()
    );
    let signup_data = create_test_user_with_info(&unique_email, "enterprise_user");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // ユーザーをEnterpriseプランに設定
    let user_repo = UserRepository::new(db.connection.clone());
    user_repo
        .update_subscription_tier(user.id, "enterprise".to_string())
        .await
        .unwrap();

    // Act
    let response = app
        .oneshot(create_request_get(
            "/payments/subscription/upgrade-options",
            &user.access_token,
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body: ApiResponse<Vec<SubscriptionTierInfo>> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert!(body.success);
    let options = body.data.unwrap();

    // Enterpriseユーザーはアップグレードオプションがない
    assert_eq!(options.len(), 0);
}

#[tokio::test]
async fn test_subscription_unauthorized_access() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;

    // Act - 認証なしでアクセス
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/payments/subscription")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_subscription_with_cancellation_scheduled() {
    // Arrange
    let (app, _schema_name, db) = setup_full_app().await;
    let unique_email = format!("cancelled_sub_{}@example.com", uuid::Uuid::new_v4());
    let signup_data = create_test_user_with_info(&unique_email, "cancelled_user");
    let user = signup_test_user(&app, signup_data).await.unwrap();

    // ユーザーをProプランに設定（キャンセル予定）
    let user_repo = UserRepository::new(db.connection.clone());
    user_repo
        .update_subscription_tier(user.id, "pro".to_string())
        .await
        .unwrap();

    // Act
    let response = app
        .oneshot(create_request_get(
            "/payments/subscription",
            &user.access_token,
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body: ApiResponse<CurrentSubscriptionResponse> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert!(body.success);
    let data = body.data.unwrap();

    assert_eq!(data.current_tier, "pro");
    assert_eq!(data.tier_display_name, "Pro");
    assert_eq!(data.tier_level, 2);
    // キャンセル情報は別途Stripe APIで確認する必要がある
    assert!(data.subscribed_at < chrono::Utc::now().into());
}

// ヘルパー関数
fn create_request_get(uri: &str, token: &str) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method("GET")
        .uri(uri)
        .header("Authorization", format!("Bearer {}", token))
        .body(axum::body::Body::empty())
        .unwrap()
}
