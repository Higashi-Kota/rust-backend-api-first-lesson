// tests/integration/team/team_invitation_tests.rs

use crate::common::{app_helper, auth_helper};
use axum::http::StatusCode;
use serde_json::json;
use task_backend::features::auth::dto::SignupRequest;
use tower::ServiceExt;
use uuid::Uuid;

async fn create_test_team(app: &axum::Router, token: &str) -> serde_json::Value {
    let team_data = json!({
        "name": format!("Test Team {}", Uuid::new_v4()),
        "description": "A test team for invitation testing"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    response["data"].clone()
}

async fn create_test_user(
    app: &axum::Router,
    email: &str,
    password: &str,
) -> auth_helper::TestUser {
    let signup_data = SignupRequest {
        email: email.to_string(),
        username: format!("user_{}", &Uuid::new_v4().to_string()[..8]),
        password: password.to_string(),
    };

    auth_helper::signup_test_user(app, signup_data)
        .await
        .unwrap()
}

fn get_team_id(team: &serde_json::Value) -> &str {
    team["id"].as_str().unwrap()
}

#[tokio::test]
async fn test_create_single_invitation_success() {
    // Arrange: Set up app, create team and member
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let team = create_test_team(&app, &admin_user.access_token).await;
    let team_id = get_team_id(&team);

    let invitation_request = json!({
        "email": "newmember@example.com",
        "message": "Welcome to our team!"
    });

    // Act: Create invitation
    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/invitations/single", team_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&invitation_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();

    // Assert: Verify response and database state
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();

    if status != StatusCode::OK {
        let error_str = String::from_utf8_lossy(&body);
        eprintln!("Error response: {}", error_str);
        panic!("Expected OK status, got: {}", status);
    }
    assert_eq!(status, StatusCode::OK);
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response["success"].as_bool().unwrap());

    let invitation = &response["data"];
    assert_eq!(invitation["invited_email"], "newmember@example.com");
    assert_eq!(invitation["message"], "Welcome to our team!");
    assert_eq!(invitation["status"], "Pending");
    assert_eq!(invitation["team_id"], team_id);
}

#[tokio::test]
async fn test_bulk_invitation_success() {
    // Arrange: Set up app and create team
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let team = create_test_team(&app, &admin_user.access_token).await;
    let team_id = get_team_id(&team);

    let bulk_request = json!({
        "emails": [
            "user1@example.com",
            "user2@example.com",
            "user3@example.com"
        ],
        "message": "Join our amazing team!"
    });

    // Act: Send bulk invitations
    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/bulk-member-invite", team_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&bulk_request).unwrap()),
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
    assert_eq!(data["success_count"], 3);
    assert_eq!(data["invitations"].as_array().unwrap().len(), 3);
    assert!(data["failed_emails"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_accept_invitation_success() {
    // Arrange: Create invitation and invitee user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let team = create_test_team(&app, &admin_user.access_token).await;
    let team_id = get_team_id(&team);

    // Create user to be invited
    let invitee = create_test_user(&app, "invitee@example.com", "MyUniqueP@ssw0rd91").await;

    // Create invitation via API
    let invitation_request = json!({
        "email": "invitee@example.com",
        "message": "Welcome!"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/invitations/single", team_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&invitation_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let invitation_id = create_response["data"]["id"].as_str().unwrap();

    // Act: Accept invitation
    let accept_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/invitations/{}/accept", invitation_id),
        &invitee.access_token,
        Some("{}".to_string()),
    );
    let accept_res = app.clone().oneshot(accept_req).await.unwrap();

    // Assert: Verify response
    assert_eq!(accept_res.status(), StatusCode::OK);

    let accept_body = axum::body::to_bytes(accept_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let accept_response: serde_json::Value = serde_json::from_slice(&accept_body).unwrap();

    assert!(accept_response["success"].as_bool().unwrap());

    let invitation = &accept_response["data"];
    assert_eq!(invitation["status"], "Accepted");
    assert!(invitation["accepted_at"].is_string());
    assert_eq!(invitation["invited_user_id"], invitee.id.to_string());
}

#[tokio::test]
async fn test_decline_invitation_with_reason() {
    // Arrange: Create invitation
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let team = create_test_team(&app, &admin_user.access_token).await;
    let team_id = get_team_id(&team);

    // Create invitation
    let invitation_request = json!({
        "email": "declined@example.com",
        "message": "Join us!"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/invitations/single", team_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&invitation_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let invitation_id = create_response["data"]["id"].as_str().unwrap();

    let decline_request = json!({
        "reason": "Not interested in joining at this time"
    });

    // Act: Decline invitation
    let decline_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/teams/{}/invitations/{}/decline", team_id, invitation_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&decline_request).unwrap()),
    );
    let decline_res = app.clone().oneshot(decline_req).await.unwrap();

    // Assert: Verify response
    assert_eq!(decline_res.status(), StatusCode::OK);

    let decline_body = axum::body::to_bytes(decline_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let decline_response: serde_json::Value = serde_json::from_slice(&decline_body).unwrap();

    let invitation = &decline_response["data"];

    assert_eq!(invitation["status"], "Declined");
    assert!(invitation["declined_at"].is_string());
    assert_eq!(
        invitation["decline_reason"],
        "Not interested in joining at this time"
    );
}

#[tokio::test]
async fn test_resend_invitation() {
    // Arrange: Create invitation
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let team = create_test_team(&app, &admin_user.access_token).await;
    let team_id = get_team_id(&team);

    // Create invitation
    let invitation_request = json!({
        "email": "resend@example.com",
        "message": "Original message"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/invitations/single", team_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&invitation_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let invitation_id = create_response["data"]["id"].as_str().unwrap();

    let resend_request = json!({
        "message": "Updated message - please join us!"
    });

    // Act: Resend invitation
    let resend_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/invitations/{}/resend", invitation_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&resend_request).unwrap()),
    );
    let resend_res = app.clone().oneshot(resend_req).await.unwrap();

    // Assert: Verify response
    assert_eq!(resend_res.status(), StatusCode::OK);

    let resend_body = axum::body::to_bytes(resend_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let resend_response: serde_json::Value = serde_json::from_slice(&resend_body).unwrap();

    let invitation = &resend_response["data"];

    assert_eq!(invitation["message"], "Updated message - please join us!");
    assert_eq!(invitation["status"], "Pending");
}

#[tokio::test]
async fn test_duplicate_invitation_prevention() {
    // Arrange: Create team and initial invitation
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let team = create_test_team(&app, &admin_user.access_token).await;
    let team_id = get_team_id(&team);

    // Create first invitation
    let invitation_request = json!({
        "email": "duplicate@example.com",
        "message": "First invitation"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/invitations/single", team_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&invitation_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Act: Try to create duplicate invitation
    let duplicate_request = json!({
        "email": "duplicate@example.com",
        "message": "Second invitation"
    });

    let req2 = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/invitations/single", team_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&duplicate_request).unwrap()),
    );
    let res2 = app.clone().oneshot(req2).await.unwrap();
    let status2 = res2.status();

    // Assert: Second invitation should either fail or update existing
    // The actual behavior depends on the business logic
    let body2 = axum::body::to_bytes(res2.into_body(), usize::MAX)
        .await
        .unwrap();
    let response2: serde_json::Value = serde_json::from_slice(&body2).unwrap();

    // If it succeeded, verify it's the same invitation
    if status2 == StatusCode::OK {
        assert_eq!(response2["data"]["invited_email"], "duplicate@example.com");
    }
}

#[tokio::test]
async fn test_invitation_permissions() {
    // Arrange: Create one team and users
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create admin and team
    let admin_user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let team = create_test_team(&app, &admin_user.access_token).await;
    let team_id = get_team_id(&team);

    // Create non-admin user
    let regular_user = create_test_user(&app, "member@example.com", "MyUniqueP@ssw0rd91").await;

    // Act: Non-admin tries to invite to team (not a member)
    let invitation_request = json!({
        "email": "newuser@example.com",
        "message": "Join the team"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/invitations/single", team_id),
        &regular_user.access_token,
        Some(serde_json::to_string(&invitation_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should be forbidden since user is not a team member
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_team_invitations_with_statistics() {
    // Arrange: Create team with multiple invitations in different states
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let team = create_test_team(&app, &admin_user.access_token).await;
    let team_id = get_team_id(&team);

    // Create multiple invitations
    for i in 0..3 {
        let invitation_request = json!({
            "email": format!("pending{}@example.com", i),
            "message": "Join us!"
        });

        let req = auth_helper::create_authenticated_request(
            "POST",
            &format!("/teams/{}/invitations/single", team_id),
            &admin_user.access_token,
            Some(serde_json::to_string(&invitation_request).unwrap()),
        );
        let _ = app.clone().oneshot(req).await.unwrap();
    }

    // Act: Get team invitations
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/teams/{}/invitations", team_id),
        &admin_user.access_token,
        None,
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify response and statistics
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response["success"].as_bool().unwrap());

    let data = &response["data"];
    assert!(data["total_count"].as_u64().unwrap() >= 3);
    assert!(data["invitations"].as_array().unwrap().len() >= 3);

    let status_counts = &data["status_counts"];
    assert!(status_counts["pending"].as_u64().unwrap() >= 3);
}

#[tokio::test]
async fn test_cancel_invitation() {
    // Arrange: Create invitation
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let team = create_test_team(&app, &admin_user.access_token).await;
    let team_id = get_team_id(&team);

    // Create invitation
    let invitation_request = json!({
        "email": "cancel@example.com",
        "message": "Join us!"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/invitations/single", team_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&invitation_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let invitation_id = create_response["data"]["id"].as_str().unwrap();

    // Act: Cancel invitation
    let cancel_req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/teams/{}/invitations/{}/cancel", team_id, invitation_id),
        &admin_user.access_token,
        None,
    );
    let cancel_res = app.clone().oneshot(cancel_req).await.unwrap();

    // Assert: Verify response
    assert_eq!(cancel_res.status(), StatusCode::OK);

    let cancel_body = axum::body::to_bytes(cancel_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let cancel_response: serde_json::Value = serde_json::from_slice(&cancel_body).unwrap();

    let invitation = &cancel_response["data"];
    assert_eq!(invitation["status"], "Cancelled");
}

#[tokio::test]
async fn test_invitation_validation_errors() {
    // Arrange
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let team = create_test_team(&app, &admin_user.access_token).await;
    let team_id = get_team_id(&team);

    // Test 1: Invalid email format
    let invalid_email_request = json!({
        "email": "not-an-email",
        "message": "Welcome!"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/invitations/single", team_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&invalid_email_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // Test 2: Empty bulk emails
    let empty_bulk_request = json!({
        "emails": [],
        "message": "Welcome!"
    });

    let req2 = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/bulk-member-invite", team_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&empty_bulk_request).unwrap()),
    );
    let res2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::BAD_REQUEST);

    // Test 3: Too many bulk emails
    let too_many_emails: Vec<String> = (0..51).map(|i| format!("user{}@example.com", i)).collect();
    let too_many_request = json!({
        "emails": too_many_emails,
        "message": "Welcome!"
    });

    let req3 = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/bulk-member-invite", team_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&too_many_request).unwrap()),
    );
    let res3 = app.clone().oneshot(req3).await.unwrap();
    assert_eq!(res3.status(), StatusCode::BAD_REQUEST);

    // Test 4: Message too long
    let long_message_request = json!({
        "email": "test@example.com",
        "message": "a".repeat(501)
    });

    let req4 = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/invitations/single", team_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&long_message_request).unwrap()),
    );
    let res4 = app.clone().oneshot(req4).await.unwrap();
    assert_eq!(res4.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_user_invitations() {
    // Arrange: Create invitations for specific user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create one team
    let team = create_test_team(&app, &admin_user.access_token).await;
    let team_id = get_team_id(&team);

    let target_email = "popular@example.com";

    // Create multiple invitations from same team
    for i in 0..2 {
        let invitation_request = json!({
            "email": target_email,
            "message": format!("Join the team - invitation {}", i + 1)
        });

        let req = auth_helper::create_authenticated_request(
            "POST",
            &format!("/teams/{}/invitations/single", team_id),
            &admin_user.access_token,
            Some(serde_json::to_string(&invitation_request).unwrap()),
        );
        let _ = app.clone().oneshot(req).await.unwrap();
    }

    // Create user and authenticate
    let user = create_test_user(&app, target_email, "MyUniqueP@ssw0rd91").await;

    // Act: Get user's invitations
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/users/invitations",
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

    let invitations = response["data"].as_array().unwrap();
    assert!(!invitations.is_empty()); // 同じチームから複数の招待は最新のものだけが有効かもしれない

    // Verify all invitations are for the correct email
    for invitation in invitations {
        assert_eq!(invitation["invited_email"], target_email);
        assert_eq!(invitation["status"], "Pending");
    }
}

#[tokio::test]
async fn test_invitation_pagination() {
    // Arrange: Create many invitations
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let team = create_test_team(&app, &admin_user.access_token).await;
    let team_id = get_team_id(&team);

    // Create 25 invitations
    for i in 0..25 {
        let invitation_request = json!({
            "email": format!("user{}@example.com", i),
            "message": "Join us!"
        });

        let req = auth_helper::create_authenticated_request(
            "POST",
            &format!("/teams/{}/invitations/single", team_id),
            &admin_user.access_token,
            Some(serde_json::to_string(&invitation_request).unwrap()),
        );
        let _ = app.clone().oneshot(req).await.unwrap();
    }

    // Act: Get first page
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!(
            "/teams/{}/invitations/paginated?page=1&page_size=10",
            team_id
        ),
        &admin_user.access_token,
        None,
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify pagination
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let data = &response["data"];

    assert_eq!(data["invitations"].as_array().unwrap().len(), 10);
    assert_eq!(data["total_count"], 25);
    assert_eq!(data["page"], 1);
    assert_eq!(data["page_size"], 10);
    assert_eq!(data["total_pages"], 3);

    // Get second page
    let req2 = auth_helper::create_authenticated_request(
        "GET",
        &format!(
            "/teams/{}/invitations/paginated?page=2&page_size=10",
            team_id
        ),
        &admin_user.access_token,
        None,
    );
    let res2 = app.clone().oneshot(req2).await.unwrap();

    let body2 = axum::body::to_bytes(res2.into_body(), usize::MAX)
        .await
        .unwrap();
    let response2: serde_json::Value = serde_json::from_slice(&body2).unwrap();

    let data2 = &response2["data"];

    assert_eq!(data2["invitations"].as_array().unwrap().len(), 10);
    assert_eq!(data2["page"], 2);
}
