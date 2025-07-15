// tests/integration/gdpr/consent_management_tests.rs

use crate::common::{app_helper, auth_helper};
use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_get_consent_status_no_prior_consents() {
    // Arrange: Set up app and create user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Act: Get consent status
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/gdpr/users/{}/consents", user.id),
        &user.access_token,
        None,
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify response
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response["success"].as_bool().unwrap());

    let data = &response["data"];
    assert_eq!(response["data"]["user_id"], user.id.to_string());

    let consents = data["consents"].as_array().unwrap();
    assert_eq!(consents.len(), 4); // Four consent types

    // Verify all consents are not granted by default
    for consent in consents {
        assert_eq!(consent["is_granted"], false);
        assert!(consent["granted_at"].is_null());
        assert!(consent["display_name"].is_string());
        assert!(consent["description"].is_string());
    }

    // Verify data processing consent is marked as required
    let data_processing_consent = consents
        .iter()
        .find(|c| c["consent_type"] == "DataProcessing")
        .unwrap();
    assert_eq!(data_processing_consent["is_required"], true);
}

#[tokio::test]
async fn test_update_multiple_consents() {
    // Arrange: Set up app and create user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    let consent_updates = json!({
        "consents": {
            "DataProcessing": true,
            "Marketing": true,
            "Analytics": false,
            "ThirdPartySharing": false
        },
        "reason": "Initial consent setup"
    });

    // Act: Update consents
    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/gdpr/users/{}/consents", user.id),
        &user.access_token,
        Some(serde_json::to_string(&consent_updates).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify response
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response["success"].as_bool().unwrap());

    let data = &response["data"];
    let consents = data["consents"].as_array().unwrap();

    // Verify consent states
    let marketing_consent = consents
        .iter()
        .find(|c| c["consent_type"] == "Marketing")
        .unwrap();
    assert_eq!(marketing_consent["is_granted"], true);
    assert!(marketing_consent["granted_at"].is_string());

    let analytics_consent = consents
        .iter()
        .find(|c| c["consent_type"] == "Analytics")
        .unwrap();
    assert_eq!(analytics_consent["is_granted"], false);
    assert!(analytics_consent["revoked_at"].is_string());
}

#[tokio::test]
async fn test_update_single_consent() {
    // Arrange: Set up app and create user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // First, grant all consents
    let initial_consents = json!({
        "consents": {
            "DataProcessing": true,
            "Marketing": true,
            "Analytics": true,
            "ThirdPartySharing": true
        }
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/gdpr/users/{}/consents", user.id),
        &user.access_token,
        Some(serde_json::to_string(&initial_consents).unwrap()),
    );
    let _ = app.clone().oneshot(req).await.unwrap();

    // Act: Revoke single consent
    let single_update = json!({
        "consent_type": "Marketing",
        "is_granted": false,
        "reason": "Too many emails"
    });

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/gdpr/users/{}/consents/single", user.id),
        &user.access_token,
        Some(serde_json::to_string(&single_update).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify response
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let consents = response["data"]["consents"].as_array().unwrap();

    // Verify only marketing consent was revoked
    let marketing_consent = consents
        .iter()
        .find(|c| c["consent_type"] == "Marketing")
        .unwrap();
    assert_eq!(marketing_consent["is_granted"], false);

    // Verify others remain granted
    let analytics_consent = consents
        .iter()
        .find(|c| c["consent_type"] == "Analytics")
        .unwrap();
    assert_eq!(analytics_consent["is_granted"], true);
}

#[tokio::test]
async fn test_cannot_revoke_required_consent() {
    // Arrange: Set up app and create user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    let consent_updates = json!({
        "consents": {
            "DataProcessing": false, // This is required and cannot be revoked
            "Marketing": false
        }
    });

    // Act: Try to revoke required consent
    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/gdpr/users/{}/consents", user.id),
        &user.access_token,
        Some(serde_json::to_string(&consent_updates).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should fail validation
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["success"], false);
    let error_message = response["error"]["message"]["message"]
        .as_str()
        .or_else(|| response["error"]["message"].as_str())
        .unwrap_or("");
    assert!(error_message.contains("Data processing consent is required"));
}

#[tokio::test]
async fn test_get_consent_history() {
    // Arrange: Set up app and create user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create consent history by updating consents multiple times
    let updates = vec![
        json!({
            "consents": {
                "DataProcessing": true,
                "Marketing": true,
                "Analytics": true
            },
            "reason": "Initial signup"
        }),
        json!({
            "consents": {
                "Marketing": false
            },
            "reason": "Too many emails"
        }),
        json!({
            "consents": {
                "Marketing": true,
                "ThirdPartySharing": true
            },
            "reason": "Opted back in"
        }),
    ];

    for update in updates {
        let req = auth_helper::create_authenticated_request(
            "PUT",
            &format!("/gdpr/users/{}/consents", user.id),
            &user.access_token,
            Some(serde_json::to_string(&update).unwrap()),
        );
        let _ = app.clone().oneshot(req).await.unwrap();
    }

    // Act: Get consent history
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/gdpr/users/{}/consents/history", user.id),
        &user.access_token,
        None,
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify response
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response["success"].as_bool().unwrap());

    let data = &response["data"];
    assert_eq!(response["data"]["user_id"], user.id.to_string());

    let history = data["history"].as_array().unwrap();
    assert!(history.len() >= 3); // At least 3 consent changes from our updates

    // Verify history entries have required fields
    for entry in history {
        assert!(entry["id"].is_string());
        assert!(entry["consent_type"].is_string());
        assert!(
            entry["action"].as_str().unwrap() == "granted"
                || entry["action"].as_str().unwrap() == "revoked"
        );
        assert!(entry["is_granted"].is_boolean());
        assert!(entry["timestamp"].is_string());
    }
}

#[tokio::test]
async fn test_user_cannot_access_other_user_consents() {
    // Arrange: Set up app and create two users
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user1 = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let user2_data = auth_helper::create_test_user_with_info("other@example.com", "OtherUser");
    let user2 = auth_helper::signup_test_user(&app, user2_data)
        .await
        .unwrap();

    // Act: User1 tries to access User2's consents
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/gdpr/users/{}/consents", user2.id),
        &user1.access_token,
        None,
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should be forbidden
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_can_view_any_user_consents() {
    // Arrange: Set up app with admin and regular user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;
    let user_data = auth_helper::create_test_user_with_info("user@example.com", "RegularUser");
    let user = auth_helper::signup_test_user(&app, user_data)
        .await
        .unwrap();

    // Act: Admin accesses user's consents
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/gdpr/users/{}/consents", user.id),
        &admin_token,
        None,
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should be allowed
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response["success"].as_bool().unwrap());
    assert_eq!(response["data"]["user_id"], user.id.to_string());
}

#[tokio::test]
async fn test_consent_updates_are_idempotent() {
    // Arrange: Set up app and create user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    let consent_updates = json!({
        "consents": {
            "DataProcessing": true,
            "Marketing": true
        }
    });

    // Act: Update consents twice with same values
    for _ in 0..2 {
        let req = auth_helper::create_authenticated_request(
            "PUT",
            &format!("/gdpr/users/{}/consents", user.id),
            &user.access_token,
            Some(serde_json::to_string(&consent_updates).unwrap()),
        );
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    // Get consent history
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/gdpr/users/{}/consents/history", user.id),
        &user.access_token,
        None,
    );
    let res = app.clone().oneshot(req).await.unwrap();

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Assert: History should show updates even if values didn't change
    let history = response["data"]["history"].as_array().unwrap();
    assert!(history.len() >= 2);
}

#[tokio::test]
async fn test_consent_validation_errors() {
    // Arrange: Set up app and create user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Test 1: Invalid consent type in single update
    let invalid_single = json!({
        "consent_type": "InvalidType",
        "is_granted": true
    });

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/gdpr/users/{}/consents/single", user.id),
        &user.access_token,
        Some(serde_json::to_string(&invalid_single).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Test 2: Empty consents map
    let empty_consents = json!({
        "consents": {}
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/gdpr/users/{}/consents", user.id),
        &user.access_token,
        Some(serde_json::to_string(&empty_consents).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    // Should succeed as empty update is valid
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_consent_status_reflects_latest_updates() {
    // Arrange: Set up app and create user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Update consents
    let consent_updates = json!({
        "consents": {
            "DataProcessing": true,
            "Marketing": true,
            "Analytics": false
        }
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/gdpr/users/{}/consents", user.id),
        &user.access_token,
        Some(serde_json::to_string(&consent_updates).unwrap()),
    );
    let _ = app.clone().oneshot(req).await.unwrap();

    // Act: Get current consent status
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/gdpr/users/{}/consents", user.id),
        &user.access_token,
        None,
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify status matches updates
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let consents = response["data"]["consents"].as_array().unwrap();

    let marketing_consent = consents
        .iter()
        .find(|c| c["consent_type"] == "Marketing")
        .unwrap();
    assert_eq!(marketing_consent["is_granted"], true);

    let analytics_consent = consents
        .iter()
        .find(|c| c["consent_type"] == "Analytics")
        .unwrap();
    assert_eq!(analytics_consent["is_granted"], false);
}
