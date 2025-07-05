// tests/integration/gdpr/data_export_tests.rs

use crate::common::{app_helper, auth_helper};
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

async fn create_test_data_for_user(
    app: &axum::Router,
    user: &auth_helper::TestUser,
) -> (Vec<Uuid>, Vec<Uuid>) {
    // Create tasks
    let mut task_ids = Vec::new();
    for i in 0..3 {
        let task_data = json!({
            "title": format!("Test Task {}", i),
            "description": format!("Description for task {}", i),
            "status": if i % 2 == 0 { "todo" } else { "in_progress" },
            "due_date": if i == 0 { Some((Utc::now() + Duration::days(7)).to_rfc3339()) } else { None }
        });

        let req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
        // Try to get ID from data field first, then directly from response
        let id_str = response["data"]["id"]
            .as_str()
            .or_else(|| response["id"].as_str())
            .unwrap_or_else(|| panic!("Task creation failed, no id in response: {:?}", response));
        task_ids.push(Uuid::parse_str(id_str).unwrap());
    }

    // Create team (only 1 for Free tier)
    let mut team_ids = Vec::new();
    let team_data = json!({
        "name": "Test Team",
        "description": "Team for GDPR testing"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &user.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // Try to get ID from data field first, then directly from response
    let id_str = response["data"]["id"]
        .as_str()
        .or_else(|| response["id"].as_str())
        .unwrap_or_else(|| panic!("Team creation failed, no id in response: {:?}", response));
    team_ids.push(Uuid::parse_str(id_str).unwrap());

    (task_ids, team_ids)
}

#[tokio::test]
async fn test_export_user_data_minimal() {
    // Arrange: Set up app and create user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    let export_request = json!({
        "include_tasks": false,
        "include_teams": false,
        "include_subscription_history": false,
        "include_activity_logs": false
    });

    // Act: Export user data
    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/gdpr/users/{}/export", user.id),
        &user.access_token,
        Some(serde_json::to_string(&export_request).unwrap()),
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

    // Verify user data is always included
    let user_data = &data["user_data"];
    assert_eq!(user_data["id"], user.id.to_string());
    assert_eq!(user_data["email"], user.email);
    assert_eq!(user_data["username"], user.username);
    assert_eq!(user_data["is_active"], true);
    assert!(user_data["role_name"].is_string());
    assert!(user_data["subscription_tier"].is_string());

    // Verify optional data is not included
    assert!(data["tasks"].is_null());
    assert!(data["teams"].is_null());
    assert!(data["subscription_history"].is_null());
    assert!(data["activity_logs"].is_null());

    // Verify export timestamp
    assert!(data["exported_at"].is_string());
}

#[tokio::test]
async fn test_export_user_data_with_tasks() {
    // Arrange: Set up app, create user and tasks
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let (task_ids, _) = create_test_data_for_user(&app, &user).await;

    let export_request = json!({
        "include_tasks": true,
        "include_teams": false,
        "include_subscription_history": false,
        "include_activity_logs": false
    });

    // Act: Export user data
    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/gdpr/users/{}/export", user.id),
        &user.access_token,
        Some(serde_json::to_string(&export_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify response includes tasks
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let tasks = response["data"]["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), task_ids.len());

    // Verify task data
    for task in tasks {
        assert!(task["id"].is_string());
        assert!(task["title"].as_str().unwrap().starts_with("Test Task"));
        assert!(task["description"].is_string());
        assert!(task["status"].is_string());
        assert!(task["created_at"].is_string());
        assert!(task["updated_at"].is_string());
    }
}

#[tokio::test]
async fn test_export_user_data_with_teams() {
    // Arrange: Set up app, create user and teams
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let (_, team_ids) = create_test_data_for_user(&app, &user).await;

    let export_request = json!({
        "include_tasks": false,
        "include_teams": true,
        "include_subscription_history": false,
        "include_activity_logs": false
    });

    // Act: Export user data
    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/gdpr/users/{}/export", user.id),
        &user.access_token,
        Some(serde_json::to_string(&export_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify response includes teams
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let teams = response["data"]["teams"].as_array().unwrap();
    assert_eq!(teams.len(), team_ids.len());

    // Verify team data
    for team in teams {
        assert!(team["id"].is_string());
        assert!(team["name"].as_str().unwrap().starts_with("Test Team"));
        assert!(team["description"].is_string());
        assert_eq!(team["role_in_team"], "owner"); // User is owner of teams they create
        assert!(team["joined_at"].is_string());
    }
}

#[tokio::test]
async fn test_export_user_data_complete() {
    // Arrange: Set up app, create user with all data types
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let (task_ids, team_ids) = create_test_data_for_user(&app, &user).await;

    // Create subscription history by upgrading
    let upgrade_request = json!({
        "new_tier": "pro"
    });
    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/users/{}/subscription/upgrade", user.id),
        &user.access_token,
        Some(serde_json::to_string(&upgrade_request).unwrap()),
    );
    let _ = app.clone().oneshot(req).await.unwrap();

    let export_request = json!({
        "include_tasks": true,
        "include_teams": true,
        "include_subscription_history": true,
        "include_activity_logs": true
    });

    // Act: Export all user data
    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/gdpr/users/{}/export", user.id),
        &user.access_token,
        Some(serde_json::to_string(&export_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify complete export
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let data = &response["data"];

    // Verify all sections are present
    assert!(data["user_data"].is_object());
    assert!(data["tasks"].is_array());
    assert!(data["teams"].is_array());
    assert!(data["subscription_history"].is_array());

    // Verify counts
    assert_eq!(data["tasks"].as_array().unwrap().len(), task_ids.len());
    assert_eq!(data["teams"].as_array().unwrap().len(), team_ids.len());

    // Verify subscription history
    let history = data["subscription_history"].as_array().unwrap();
    assert!(!history.is_empty()); // At least one upgrade
    assert_eq!(history[0]["previous_tier"], "free");
    assert_eq!(history[0]["new_tier"], "pro");
}

#[tokio::test]
async fn test_user_cannot_export_other_user_data() {
    // Arrange: Set up app and create two users
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user1 = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let user2_data = auth_helper::create_test_user_with_info("other@example.com", "OtherUser");
    let user2 = auth_helper::signup_test_user(&app, user2_data)
        .await
        .unwrap();

    let export_request = json!({
        "include_tasks": true,
        "include_teams": true,
        "include_subscription_history": true,
        "include_activity_logs": true
    });

    // Act: User1 tries to export User2's data
    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/gdpr/users/{}/export", user2.id),
        &user1.access_token,
        Some(serde_json::to_string(&export_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should be forbidden
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_can_export_any_user_data() {
    // Arrange: Set up app with admin and regular user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;
    let user_data = auth_helper::create_test_user_with_info("user@example.com", "RegularUser");
    let user = auth_helper::signup_test_user(&app, user_data)
        .await
        .unwrap();

    // Create some data for the user
    let (_, _) = create_test_data_for_user(&app, &user).await;

    let export_request = json!({
        "include_tasks": true,
        "include_teams": true,
        "include_subscription_history": false,
        "include_activity_logs": false
    });

    // Act: Admin exports user's data
    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/admin/gdpr/users/{}/export", user.id),
        &admin_token,
        Some(serde_json::to_string(&export_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should be allowed
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response["success"].as_bool().unwrap());
    assert_eq!(response["data"]["user_data"]["id"], user.id.to_string());
}

#[tokio::test]
async fn test_export_nonexistent_user() {
    // Arrange: Set up app and admin
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;
    let nonexistent_id = Uuid::new_v4();

    let export_request = json!({
        "include_tasks": false,
        "include_teams": false,
        "include_subscription_history": false,
        "include_activity_logs": false
    });

    // Act: Try to export nonexistent user's data
    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/admin/gdpr/users/{}/export", nonexistent_id),
        &admin_token,
        Some(serde_json::to_string(&export_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should return not found
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_export_includes_all_task_fields() {
    // Arrange: Set up app and create user with detailed task
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create a task with all fields
    let task_data = json!({
        "title": "Detailed Task",
        "description": "This task has all fields filled",
        "status": "in_progress",
        "due_date": (Utc::now() + Duration::days(30)).to_rfc3339()
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let task_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = task_response["data"]["id"]
        .as_str()
        .or_else(|| task_response["id"].as_str())
        .expect("Task ID not found in response");

    let export_request = json!({
        "include_tasks": true,
        "include_teams": false,
        "include_subscription_history": false,
        "include_activity_logs": false
    });

    // Act: Export data
    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/gdpr/users/{}/export", user.id),
        &user.access_token,
        Some(serde_json::to_string(&export_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify task fields
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let tasks = response["data"]["tasks"].as_array().unwrap();
    let exported_task = tasks.iter().find(|t| t["id"] == task_id).unwrap();

    assert_eq!(exported_task["title"], "Detailed Task");
    assert_eq!(
        exported_task["description"],
        "This task has all fields filled"
    );
    assert_eq!(exported_task["status"], "in_progress");
    assert!(exported_task["due_date"].is_string());
    assert!(exported_task["created_at"].is_string());
    assert!(exported_task["updated_at"].is_string());
}

#[tokio::test]
async fn test_export_preserves_data_integrity() {
    // Arrange: Set up app and create user with specific data
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create tasks with specific titles
    let task_titles = vec!["First Task", "Second Task", "Third Task"];
    for title in &task_titles {
        let task_data = json!({
            "title": title,
            "description": format!("Description for {}", title),
            "status": "todo"
        });

        let req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );
        let _ = app.clone().oneshot(req).await.unwrap();
    }

    let export_request = json!({
        "include_tasks": true,
        "include_teams": false,
        "include_subscription_history": false,
        "include_activity_logs": false
    });

    // Act: Export data
    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/gdpr/users/{}/export", user.id),
        &user.access_token,
        Some(serde_json::to_string(&export_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify all tasks are exported correctly
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let tasks = response["data"]["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), task_titles.len());

    // Verify all titles are present
    let exported_titles: Vec<String> = tasks
        .iter()
        .map(|t| t["title"].as_str().unwrap().to_string())
        .collect();

    for title in &task_titles {
        assert!(exported_titles.iter().any(|t| t == title));
    }
}
