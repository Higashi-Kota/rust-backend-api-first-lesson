// tests/integration/gdpr_compliance_test.rs

use crate::common::{app_helper, auth_helper};
use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_gdpr_export_user_data() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create a test user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create some tasks for the user
    let task_data = json!({
        "title": "Test Task for GDPR",
        "description": "This task will be exported"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // Export user data
    let export_request = json!({
        "include_tasks": true,
        "include_teams": true,
        "include_subscription_history": true,
        "include_activity_logs": true
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/gdpr/users/{}/export", user.id),
        &user.access_token,
        Some(serde_json::to_string(&export_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();

    // Debug output if not OK
    if status != StatusCode::OK {
        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let error_text = String::from_utf8_lossy(&body);
        eprintln!(
            "GDPR export error: Status: {}, Body: {}",
            status, error_text
        );
        panic!("Expected OK status, got: {:?}", status);
    }

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify export structure
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"]["user_data"].is_object());
    assert!(response["data"]["tasks"].is_array());
    assert!(response["data"]["teams"].is_array());
    assert!(response["data"]["subscription_history"].is_array());
    assert!(response["data"]["exported_at"].is_string());
}

#[tokio::test]
async fn test_gdpr_delete_user_data() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create a test user
    let signup_data = auth_helper::create_test_user_data();
    let user = auth_helper::signup_test_user(&app, signup_data)
        .await
        .unwrap();

    // Create some data for the user
    let task_data = json!({
        "title": "Task to be deleted",
        "description": "This will be removed"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    app.clone().oneshot(req).await.unwrap();

    // Request data deletion
    let delete_request = json!({
        "confirm_deletion": true,
        "reason": "User requested deletion"
    });

    let req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/gdpr/users/{}/delete", user.id),
        &user.access_token,
        Some(serde_json::to_string(&delete_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify deletion response
    assert!(response["success"].as_bool().unwrap());
    assert_eq!(response["data"]["user_id"], user.id.to_string());
    assert!(response["data"]["deleted_at"].is_string());
    assert!(response["data"]["deleted_records"]["tasks_count"].is_number());
    assert!(response["data"]["deleted_records"]["teams_count"].is_number());
    assert!(response["data"]["deleted_records"]["refresh_tokens_count"].is_number());

    // Verify user cannot login after deletion
    let signin_data = json!({
        "identifier": user.email,
        "password": "MyUniqueP@ssw0rd91"
    });

    let req = Request::builder()
        .uri("/auth/signin")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&signin_data).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_ne!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_gdpr_compliance_status() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create a test user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Check compliance status
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/gdpr/users/{}/status", user.id),
        &user.access_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify compliance status structure
    assert!(response["success"].as_bool().unwrap());
    assert_eq!(response["data"]["user_id"], user.id.to_string());
    assert_eq!(response["data"]["data_retention_days"], 90);
    assert_eq!(response["data"]["deletion_requested"], false);
}

#[tokio::test]
async fn test_admin_gdpr_export_any_user() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin and regular user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Admin exports user's data
    let export_request = json!({
        "include_tasks": true,
        "include_teams": true,
        "include_subscription_history": true,
        "include_activity_logs": true
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/admin/gdpr/users/{}/export", user.id),
        &admin_token,
        Some(serde_json::to_string(&export_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify admin can export any user's data
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"]["user_data"].is_object());
    assert!(response["data"]["exported_at"].is_string());
}

#[tokio::test]
async fn test_admin_gdpr_delete_user_data() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin and regular user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;
    let signup_data = auth_helper::create_test_user_data();
    let user = auth_helper::signup_test_user(&app, signup_data)
        .await
        .unwrap();

    // Admin deletes user's data
    let delete_request = json!({
        "confirm_deletion": true,
        "reason": "User requested account deletion via support"
    });

    let req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/admin/gdpr/users/{}/delete", user.id),
        &admin_token,
        Some(serde_json::to_string(&delete_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify admin deletion
    assert!(response["success"].as_bool().unwrap());
    assert_eq!(response["data"]["user_id"], user.id.to_string());
    assert!(response["data"]["deleted_at"].is_string());
}

#[tokio::test]
async fn test_gdpr_export_requires_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    let fake_user_id = Uuid::new_v4();
    let export_request = json!({
        "include_tasks": true,
        "include_teams": true,
        "include_subscription_history": true,
        "include_activity_logs": true
    });

    let req = Request::builder()
        .uri(format!("/gdpr/users/{}/export", fake_user_id))
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&export_request).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_gdpr_user_cannot_export_other_users_data() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create two users
    let user1 = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let user2_signup = auth_helper::create_test_user_with_info("user2@example.com", "user2");
    let user2 = auth_helper::signup_test_user(&app, user2_signup)
        .await
        .unwrap();

    // User1 tries to export user2's data
    let export_request = json!({
        "include_tasks": true,
        "include_teams": true,
        "include_subscription_history": true,
        "include_activity_logs": true
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/gdpr/users/{}/export", user2.id),
        &user1.access_token,
        Some(serde_json::to_string(&export_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();

    if status != StatusCode::FORBIDDEN {
        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let error_text = String::from_utf8_lossy(&body);
        eprintln!("Expected FORBIDDEN but got {}: {}", status, error_text);
    }

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_gdpr_deletion_requires_confirmation() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Try to delete without confirmation
    let delete_request = json!({
        "confirm_deletion": false,
        "reason": "Testing"
    });

    let req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/gdpr/users/{}/delete", user.id),
        &user.access_token,
        Some(serde_json::to_string(&delete_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Should indicate confirmation is required
    assert!(!response["success"].as_bool().unwrap());
    // Check if the error is in message or error field
    let error_msg = response["message"]
        .as_str()
        .or_else(|| response["error"].as_str())
        .unwrap_or("");
    assert!(error_msg.to_lowercase().contains("confirm"));
}
