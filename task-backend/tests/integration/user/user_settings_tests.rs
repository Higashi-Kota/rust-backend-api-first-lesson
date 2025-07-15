// tests/integration/user/user_settings_tests.rs

use crate::common::{app_helper, auth_helper};
use axum::{body, http::StatusCode};
use task_backend::api::dto::user_dto::{UpdateUserSettingsRequest, UserSettingsResponse};
use task_backend::types::ApiResponse;
use tower::ServiceExt;

#[tokio::test]
async fn test_get_user_settings() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // Create and authenticate user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Get user settings
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/users/settings",
        &user.access_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: ApiResponse<UserSettingsResponse> = serde_json::from_slice(&body).unwrap();
    let settings = response.data.unwrap();

    // Verify default settings
    assert_eq!(settings.preferences.language, "ja");
    assert_eq!(settings.preferences.timezone, "Asia/Tokyo");
}

#[tokio::test]
async fn test_update_user_settings() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // Create and authenticate user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Update settings
    let update_request = UpdateUserSettingsRequest {
        language: Some("en".to_string()),
        timezone: Some("UTC".to_string()),
        notifications_enabled: Some(false),
        email_notifications: None,
        ui_preferences: None,
    };

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        "/users/settings",
        &user.access_token,
        Some(serde_json::to_string(&update_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_delete_user_settings() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Create test user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // First ensure the user has settings by getting them (which creates default settings)
    let get_req = auth_helper::create_authenticated_request(
        "GET",
        "/users/settings",
        &user.access_token,
        None,
    );
    let _ = app.clone().oneshot(get_req).await.unwrap();

    // Delete user settings as admin
    let req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/admin/users/{}/settings", user.id),
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();

    // Debug output
    if status != StatusCode::NO_CONTENT {
        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let error_response: serde_json::Value = serde_json::from_slice(&body).unwrap_or_else(
            |_| serde_json::json!({"raw": String::from_utf8_lossy(&body).to_string()}),
        );
        println!("Delete settings response: {:?}", error_response);
        panic!("Expected 204, got {}", status);
    }

    // Should return 204 No Content for successful deletion
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_invalid_settings_update() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // Create and authenticate user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Try to update with invalid timezone
    let update_request = UpdateUserSettingsRequest {
        language: Some("en".to_string()),
        timezone: Some("Invalid/Timezone".to_string()),
        notifications_enabled: None,
        email_notifications: None,
        ui_preferences: None,
    };

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        "/users/settings",
        &user.access_token,
        Some(serde_json::to_string(&update_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert!(res.status().is_client_error() || res.status().is_server_error());
}
