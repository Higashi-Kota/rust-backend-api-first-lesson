use crate::common::{
    app_helper::{create_request, setup_full_app},
    auth_helper::create_and_authenticate_user,
};
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde_json::json;
use task_backend::{domain::subscription_tier::SubscriptionTier, utils::email::EmailProvider};
use tower::ServiceExt;

#[tokio::test]
async fn test_subscription_cancellation_with_grace_period() {
    // Arrange
    let (app, _schema_name, db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // まずProにアップグレード
    let email_service = std::sync::Arc::new(
        task_backend::utils::email::EmailService::new(task_backend::utils::email::EmailConfig {
            provider: EmailProvider::Development,
            ..Default::default()
        })
        .unwrap(),
    );
    let subscription_service =
        task_backend::service::subscription_service::SubscriptionService::new(
            db.connection.clone(),
            email_service.clone(),
        );
    subscription_service
        .change_subscription_tier(
            user.id,
            SubscriptionTier::Pro.as_str().to_string(),
            None,
            Some("Test upgrade".to_string()),
        )
        .await
        .unwrap();

    // Stripeサブスクリプション情報を作成（請求期間終了時にキャンセル）
    let subscription_repo =
        task_backend::repository::stripe_subscription_repository::StripeSubscriptionRepository::new(
            db.connection.clone(),
        );
    let create_sub =
        task_backend::repository::stripe_subscription_repository::CreateStripeSubscription {
            user_id: user.id,
            stripe_subscription_id: "sub_test_123".to_string(),
            stripe_price_id: "price_test_pro".to_string(),
            status: "active".to_string(),
            current_period_start: Some(Utc::now() - Duration::days(15)),
            current_period_end: Some(Utc::now() + Duration::days(15)),
            cancel_at: Some(Utc::now() + Duration::days(15)),
            canceled_at: None,
        };
    subscription_repo.create(create_sub).await.unwrap();

    // Act - キャンセルWebhookをシミュレート（開発モード用の簡易ペイロード）
    let webhook_payload = r#"{
        "id": "evt_test_cancel",
        "object": "event",
        "api_version": "2023-10-16",
        "created": 1234567890,
        "data": {
            "object": {
                "id": "sub_test_123",
                "object": "subscription",
                "customer": "cus_test_123",
                "status": "active",
                "cancel_at_period_end": true,
                "current_period_start": 1234567890,
                "current_period_end": 1234567890,
                "automatic_tax": {
                    "enabled": false
                },
                "billing_cycle_anchor": 1234567890,
                "billing_thresholds": null,
                "cancel_at": null,
                "canceled_at": null,
                "collection_method": "charge_automatically",
                "created": 1234567890,
                "currency": "jpy",
                "days_until_due": null,
                "default_payment_method": null,
                "default_source": null,
                "default_tax_rates": [],
                "description": null,
                "discount": null,
                "ended_at": null,
                "items": {
                    "object": "list",
                    "data": [{
                        "id": "si_test",
                        "object": "subscription_item",
                        "billing_thresholds": null,
                        "created": 1234567890,
                        "metadata": {},
                        "price": {
                            "id": "price_test_pro",
                            "object": "price",
                            "active": true,
                            "billing_scheme": "per_unit",
                            "created": 1234567890,
                            "currency": "jpy",
                            "livemode": false,
                            "lookup_key": null,
                            "metadata": {},
                            "nickname": null,
                            "product": "prod_test",
                            "recurring": {
                                "aggregate_usage": null,
                                "interval": "month",
                                "interval_count": 1,
                                "usage_type": "licensed"
                            },
                            "tax_behavior": "unspecified",
                            "tiers_mode": null,
                            "transform_quantity": null,
                            "type": "recurring",
                            "unit_amount": 1000,
                            "unit_amount_decimal": "1000"
                        },
                        "quantity": 1,
                        "subscription": "sub_test_123",
                        "tax_rates": []
                    }],
                    "has_more": false,
                    "url": "/v1/subscription_items?subscription=sub_test_123"
                },
                "latest_invoice": null,
                "livemode": false,
                "metadata": {},
                "next_pending_invoice_item_invoice": null,
                "on_behalf_of": null,
                "pause_collection": null,
                "payment_settings": {
                    "payment_method_options": null,
                    "payment_method_types": null,
                    "save_default_payment_method": "off"
                },
                "pending_invoice_item_interval": null,
                "pending_setup_intent": null,
                "pending_update": null,
                "plan": null,
                "quantity": 1,
                "schedule": null,
                "start_date": 1234567890,
                "test_clock": null,
                "transfer_data": null,
                "trial_end": null,
                "trial_settings": {
                    "end_behavior": {
                        "missing_payment_method": "create_invoice"
                    }
                },
                "trial_start": null
            }
        },
        "type": "customer.subscription.deleted",
        "livemode": false,
        "pending_webhooks": 1,
        "request": {
            "id": null,
            "idempotency_key": null
        }
    }"#;

    // ユーザーのStripe顧客IDを設定
    let user_repo =
        task_backend::repository::user_repository::UserRepository::new(db.connection.clone());
    user_repo
        .update_stripe_customer_id(user.id, "cus_test_123")
        .await
        .unwrap();

    // 開発モードでwebhookを処理
    std::env::set_var("STRIPE_DEVELOPMENT_MODE", "true");

    let webhook_response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/webhooks/stripe")
                .method("POST")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(webhook_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(webhook_response.status(), StatusCode::OK);

    // Assert - ユーザーはまだProティアを維持
    let user_after = user_repo.find_by_id(user.id).await.unwrap().unwrap();
    assert_eq!(user_after.subscription_tier, "pro");

    // サブスクリプション情報を確認
    let sub_after = subscription_repo
        .find_by_stripe_subscription_id("sub_test_123")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(sub_after.status, "canceled");
}

#[tokio::test]
async fn test_team_creation_with_subscription_limits() {
    // Arrange
    let (app, _schema_name, db) = setup_full_app().await;
    let free_user = create_and_authenticate_user(&app).await;
    let pro_user = create_and_authenticate_user(&app).await;

    // Proユーザーをアップグレード
    let email_service = std::sync::Arc::new(
        task_backend::utils::email::EmailService::new(task_backend::utils::email::EmailConfig {
            provider: EmailProvider::Development,
            ..Default::default()
        })
        .unwrap(),
    );
    let subscription_service =
        task_backend::service::subscription_service::SubscriptionService::new(
            db.connection.clone(),
            email_service.clone(),
        );
    subscription_service
        .change_subscription_tier(
            pro_user.id,
            SubscriptionTier::Pro.as_str().to_string(),
            None,
            Some("Test upgrade".to_string()),
        )
        .await
        .unwrap();

    // Act & Assert - Freeユーザーは1チームまで作成可能
    let team_request = json!({
        "name": "Free Team 1",
        "description": "First team"
    });

    let response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/teams",
            &free_user.access_token,
            &team_request,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 2つ目のチーム作成は失敗
    let team_request2 = json!({
        "name": "Free Team 2",
        "description": "Second team"
    });

    let response2 = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/teams",
            &free_user.access_token,
            &team_request2,
        ))
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::FORBIDDEN);
    let body = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();
    let error: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(error["error"]["message"]
        .as_str()
        .unwrap()
        .contains("teams limit"));

    // Proユーザーは5チームまで作成可能
    for i in 1..=5 {
        let team_request = json!({
            "name": format!("Pro Team {}", i),
            "description": format!("Team number {}", i)
        });

        let response = app
            .clone()
            .oneshot(create_request(
                "POST",
                "/teams",
                &pro_user.access_token,
                &team_request,
            ))
            .await
            .unwrap();

        if response.status() != StatusCode::CREATED {
            // デバッグ情報を出力
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let error: serde_json::Value = serde_json::from_slice(&body).unwrap();
            eprintln!("Failed to create team {} for Pro user: {:?}", i, error);
            panic!("Team creation failed at iteration {}", i);
        }
    }

    // 6つ目のチーム作成は失敗
    let team_request6 = json!({
        "name": "Pro Team 6",
        "description": "Team number 6"
    });

    let response6 = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/teams",
            &pro_user.access_token,
            &team_request6,
        ))
        .await
        .unwrap();

    assert_eq!(response6.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_task_creation_with_subscription_limits() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;
    let free_user = create_and_authenticate_user(&app).await;

    // Act - Free tierは100タスクまで作成可能（テストでは数個作成）
    for i in 1..=5 {
        let task_request = json!({
            "title": format!("Task {}", i),
            "description": format!("Test task {}", i),
            "status": "todo"
        });

        let response = app
            .clone()
            .oneshot(create_request(
                "POST",
                "/tasks",
                &free_user.access_token,
                &task_request,
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // 実際の制限テストは単体テストで実施済み（100タスクの作成は統合テストには重い）
}

#[tokio::test]
async fn test_team_member_invitation_with_limits() {
    // Arrange
    let (app, _schema_name, _db) = setup_full_app().await;
    let owner = create_and_authenticate_user(&app).await;

    // チームを作成
    let team_request = json!({
        "name": "Test Team",
        "description": "Team for member limit test"
    });

    let team_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/teams",
            &owner.access_token,
            &team_request,
        ))
        .await
        .unwrap();

    assert_eq!(team_response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(team_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let team: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let team_id = team["data"]["id"].as_str().unwrap();

    // Act - Freeティアは5メンバーまで（オーナー含む）
    // すでにオーナーが1人いるので、4人まで招待可能
    for _i in 1..=4 {
        let member = create_and_authenticate_user(&app).await;
        let invite_request = json!({
            "user_id": member.id,
            "role": "Member"
        });

        let response = app
            .clone()
            .oneshot(create_request(
                "POST",
                &format!("/teams/{}/members", team_id),
                &owner.access_token,
                &invite_request,
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // 5人目の招待は失敗
    let member5 = create_and_authenticate_user(&app).await;
    let invite_request5 = json!({
        "user_id": member5.id,
        "role": "Member"
    });

    let response3 = app
        .clone()
        .oneshot(create_request(
            "POST",
            &format!("/teams/{}/members", team_id),
            &owner.access_token,
            &invite_request5,
        ))
        .await
        .unwrap();

    let status = response3.status();
    let body = axum::body::to_bytes(response3.into_body(), usize::MAX)
        .await
        .unwrap();
    let error: serde_json::Value = serde_json::from_slice(&body).unwrap();

    if status != StatusCode::FORBIDDEN {
        eprintln!(
            "Response for 5th member: status={}, body={:?}",
            status, error
        );
    }
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(error["error"]["message"]
        .as_str()
        .unwrap()
        .contains("team_members limit"));
}

#[tokio::test]
async fn test_subscription_period_end_downgrade() {
    // Arrange
    let (app, _schema_name, db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // Proにアップグレード
    let email_service = std::sync::Arc::new(
        task_backend::utils::email::EmailService::new(task_backend::utils::email::EmailConfig {
            provider: EmailProvider::Development,
            ..Default::default()
        })
        .unwrap(),
    );
    let subscription_service =
        task_backend::service::subscription_service::SubscriptionService::new(
            db.connection.clone(),
            email_service.clone(),
        );
    subscription_service
        .change_subscription_tier(
            user.id,
            SubscriptionTier::Pro.as_str().to_string(),
            None,
            Some("Test upgrade".to_string()),
        )
        .await
        .unwrap();

    // Stripeサブスクリプション情報を作成（キャンセル済み）
    let subscription_repo =
        task_backend::repository::stripe_subscription_repository::StripeSubscriptionRepository::new(
            db.connection.clone(),
        );
    let create_sub =
        task_backend::repository::stripe_subscription_repository::CreateStripeSubscription {
            user_id: user.id,
            stripe_subscription_id: "sub_test_ended".to_string(),
            stripe_price_id: "price_test_pro".to_string(),
            status: "canceled".to_string(),
            current_period_start: Some(Utc::now() - Duration::days(30)),
            current_period_end: Some(Utc::now() - Duration::days(1)),
            cancel_at: None,
            canceled_at: Some(Utc::now() - Duration::days(1)),
        };
    subscription_repo.create(create_sub).await.unwrap();

    // ユーザーのStripe顧客IDを設定
    let user_repo =
        task_backend::repository::user_repository::UserRepository::new(db.connection.clone());
    user_repo
        .update_stripe_customer_id(user.id, "cus_test_ended")
        .await
        .unwrap();

    // Act - 期間終了のWebhookをシミュレート
    let current_period_end = (Utc::now() - Duration::days(1)).timestamp();
    let webhook_payload = format!(
        r#"{{
        "id": "evt_test_ended",
        "object": "event",
        "api_version": "2023-10-16",
        "created": 1234567890,
        "data": {{
            "object": {{
                "id": "sub_test_ended",
                "object": "subscription",
                "customer": "cus_test_ended",
                "status": "canceled",
                "cancel_at_period_end": false,
                "current_period_start": 1234567890,
                "current_period_end": {},
                "automatic_tax": {{
                    "enabled": false
                }},
                "billing_cycle_anchor": 1234567890,
                "billing_thresholds": null,
                "cancel_at": null,
                "canceled_at": null,
                "collection_method": "charge_automatically",
                "created": 1234567890,
                "currency": "jpy",
                "days_until_due": null,
                "default_payment_method": null,
                "default_source": null,
                "default_tax_rates": [],
                "description": null,
                "discount": null,
                "ended_at": null,
                "items": {{
                    "object": "list",
                    "data": [{{
                        "id": "si_test",
                        "object": "subscription_item",
                        "billing_thresholds": null,
                        "created": 1234567890,
                        "metadata": {{}},
                        "price": {{
                            "id": "price_test_pro",
                            "object": "price",
                            "active": true,
                            "billing_scheme": "per_unit",
                            "created": 1234567890,
                            "currency": "jpy",
                            "livemode": false,
                            "lookup_key": null,
                            "metadata": {{}},
                            "nickname": null,
                            "product": "prod_test",
                            "recurring": {{
                                "aggregate_usage": null,
                                "interval": "month",
                                "interval_count": 1,
                                "usage_type": "licensed"
                            }},
                            "tax_behavior": "unspecified",
                            "tiers_mode": null,
                            "transform_quantity": null,
                            "type": "recurring",
                            "unit_amount": 1000,
                            "unit_amount_decimal": "1000"
                        }},
                        "quantity": 1,
                        "subscription": "sub_test_ended",
                        "tax_rates": []
                    }}],
                    "has_more": false,
                    "url": "/v1/subscription_items?subscription=sub_test_ended"
                }},
                "latest_invoice": null,
                "livemode": false,
                "metadata": {{}},
                "next_pending_invoice_item_invoice": null,
                "on_behalf_of": null,
                "pause_collection": null,
                "payment_settings": {{
                    "payment_method_options": null,
                    "payment_method_types": null,
                    "save_default_payment_method": "off"
                }},
                "pending_invoice_item_interval": null,
                "pending_setup_intent": null,
                "pending_update": null,
                "plan": null,
                "quantity": 1,
                "schedule": null,
                "start_date": 1234567890,
                "test_clock": null,
                "transfer_data": null,
                "trial_end": null,
                "trial_settings": {{
                    "end_behavior": {{
                        "missing_payment_method": "create_invoice"
                    }}
                }},
                "trial_start": null
            }}
        }},
        "type": "customer.subscription.updated",
        "livemode": false,
        "pending_webhooks": 1,
        "request": {{
            "id": null,
            "idempotency_key": null
        }}
    }}"#,
        current_period_end
    );

    std::env::set_var("STRIPE_DEVELOPMENT_MODE", "true");

    let webhook_response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/webhooks/stripe")
                .method("POST")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(webhook_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(webhook_response.status(), StatusCode::OK);

    // Assert - ユーザーはFreeティアに戻る
    let user_after = user_repo.find_by_id(user.id).await.unwrap().unwrap();
    assert_eq!(user_after.subscription_tier, "free");
}
