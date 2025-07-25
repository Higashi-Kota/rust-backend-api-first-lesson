// tests/integration/tasks/dynamic_permission_tests.rs
//
// 動的権限チェックのテスト
// サブスクリプションティアに基づく機能制限の検証

use crate::common::{app_helper, auth_helper};
use axum::{body, http::StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

/// Freeティアユーザーのタスク作成制限テスト
#[tokio::test]
async fn test_free_tier_task_limit() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let free_user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create 10 tasks (Free tier limit)
    for i in 1..=10 {
        let task_data = json!({
            "title": format!("Task {}", i),
            "description": "Test task"
        });

        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &free_user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let response = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Act - Try to create 11th task
    let task_data = json!({
        "title": "Task 11",
        "description": "This should fail for free tier"
    });

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &free_user.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let response = app.oneshot(create_req).await.unwrap();

    // Assert - Should be forbidden due to tier limit
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: Value = serde_json::from_slice(&body).unwrap();
    let message = error_response["error"]["message"]
        .as_str()
        .expect("Error message should be present");
    assert!(message.to_lowercase().contains("tasks limit"));
}

/// Freeティアユーザーのチーム作成制限テスト
#[tokio::test]
async fn test_free_tier_team_limit() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let free_user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create first team (Free tier allows 1 team)
    let team_data = json!({
        "name": format!("Team {}", Uuid::new_v4()),
        "description": "First team"
    });

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &free_user.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let response = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Act - Try to create second team
    let team2_data = json!({
        "name": format!("Team2 {}", Uuid::new_v4()),
        "description": "Second team - should fail"
    });

    let create_req2 = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &free_user.access_token,
        Some(serde_json::to_string(&team2_data).unwrap()),
    );

    let response2 = app.oneshot(create_req2).await.unwrap();

    // Assert - Should be forbidden due to tier limit
    assert_eq!(response2.status(), StatusCode::FORBIDDEN);

    let body = body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: Value = serde_json::from_slice(&body).unwrap();
    let message = error_response["error"]["message"]
        .as_str()
        .expect("Error message should be present");
    assert!(message.to_lowercase().contains("teams limit"));
}

/// 高度な機能へのアクセス制限テスト
#[tokio::test]
async fn test_premium_feature_access_restriction() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let free_user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Act - Try to access premium analytics
    let analytics_req = auth_helper::create_authenticated_request(
        "GET",
        "/analytics/advanced",
        &free_user.access_token,
        None,
    );

    let response = app.clone().oneshot(analytics_req).await.unwrap();

    // Assert - Should be forbidden for free tier
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Act - Try to bulk create tasks (premium feature)
    let bulk_data = json!({
        "tasks": [
            {"title": "Bulk Task 1", "description": "Task 1"},
            {"title": "Bulk Task 2", "description": "Task 2"},
            {"title": "Bulk Task 3", "description": "Task 3"}
        ]
    });

    let bulk_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks/batch/create",
        &free_user.access_token,
        Some(serde_json::to_string(&bulk_data).unwrap()),
    );

    let bulk_response = app.oneshot(bulk_req).await.unwrap();
    let status = bulk_response.status();

    // Assert - Bulk operations might be limited
    let body = body::to_bytes(bulk_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let bulk_result: Value = serde_json::from_slice(&body).unwrap();

    // Check if there's a limit on bulk operations
    assert_eq!(status, StatusCode::FORBIDDEN);
    let message = bulk_result["error"]["message"]
        .as_str()
        .expect("Error message should be present");
    assert!(message.to_lowercase().contains("premium feature"));
}

/// チームメンバー数制限テスト
#[tokio::test]
async fn test_team_member_limit_by_tier() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let team_owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create team
    let team_data = json!({
        "name": format!("Limited Team {}", Uuid::new_v4()),
        "description": "Team with member limit"
    });

    let create_team_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &team_owner.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_team_res = app.clone().oneshot(create_team_req).await.unwrap();
    let body = body::to_bytes(create_team_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let team: Value = serde_json::from_slice(&body).unwrap();
    let team_id = team["data"]["id"].as_str().unwrap();

    // Add members up to free tier limit (e.g., 5 members)
    for i in 1..=4 {
        let member = auth_helper::create_user_with_credentials(
            &app,
            &format!("member{}@example.com", i),
            &format!("member{}", i),
            "Complex#Pass2024",
        )
        .await
        .unwrap();

        let invite_data = json!({
            "user_id": member.id,
            "role": "Member"
        });

        let invite_req = auth_helper::create_authenticated_request(
            "POST",
            &format!("/teams/{}/members", team_id),
            &team_owner.access_token,
            Some(serde_json::to_string(&invite_data).unwrap()),
        );

        let response = app.clone().oneshot(invite_req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Act - Try to add 6th member (exceeds free tier limit of 5)
    let extra_member = auth_helper::create_user_with_credentials(
        &app,
        "extra_member@example.com",
        "extra_member",
        "Complex#Pass2024",
    )
    .await
    .unwrap();

    let invite_data = json!({
        "user_id": extra_member.id,
        "role": "Member"
    });

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &team_owner.access_token,
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    let response = app.oneshot(invite_req).await.unwrap();

    // Assert - Should be forbidden due to member limit
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error_response: Value = serde_json::from_slice(&body).unwrap();
    let message = error_response["error"]["message"]
        .as_str()
        .expect("Error message should be present");
    assert!(message.to_lowercase().contains("team_members limit"));
}
