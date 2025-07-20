// tests/integration/tasks/performance_tests.rs

use crate::common::{app_helper, auth_helper};
use axum::{body, http::StatusCode};
use serde_json::{json, Value};
use std::time::Instant;
use tower::ServiceExt;
use uuid::Uuid;

fn create_test_team_data(name: &str) -> Value {
    json!({
        "name": name,
        "description": "Performance test team"
    })
}

#[tokio::test]
async fn test_large_scale_team_task_creation_performance() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create team
    let team_name = format!("Performance Test Team {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let create_team_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_team_res = app.clone().oneshot(create_team_req).await.unwrap();
    let body = body::to_bytes(create_team_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let team: Value = serde_json::from_slice(&body).unwrap();
    let team_id = team["data"]["id"].as_str().unwrap();

    // Act - Create 100 team tasks and measure time
    let start_time = Instant::now();
    let task_count = 100;

    for i in 0..task_count {
        let task_data = json!({
            "title": format!("Performance Task {}", i),
            "description": format!("Task {} for performance testing", i),
            "visibility": "team"
        });

        let create_task_req = auth_helper::create_authenticated_request(
            "POST",
            &format!("/teams/{}/tasks", team_id),
            &owner.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let response = app.clone().oneshot(create_task_req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    let elapsed = start_time.elapsed();

    // Assert - Should complete within reasonable time (10 seconds for 100 tasks)
    assert!(
        elapsed.as_secs() < 10,
        "Creating {} tasks took {:?}, which is too slow",
        task_count,
        elapsed
    );

    println!(
        "Created {} team tasks in {:?} ({:.2} tasks/sec)",
        task_count,
        elapsed,
        task_count as f64 / elapsed.as_secs_f64()
    );
}

#[tokio::test]
async fn test_large_scale_team_task_query_performance() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create team
    let team_name = format!("Query Performance Test Team {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let create_team_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_team_res = app.clone().oneshot(create_team_req).await.unwrap();
    let body = body::to_bytes(create_team_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let team: Value = serde_json::from_slice(&body).unwrap();
    let team_id = team["data"]["id"].as_str().unwrap();

    // Create 50 team tasks
    for i in 0..50 {
        let task_data = json!({
            "title": format!("Query Test Task {}", i),
            "description": format!("Task {} for query testing", i),
            "visibility": "team",
            "priority": if i % 3 == 0 { "high" } else if i % 2 == 0 { "medium" } else { "low" }
        });

        let create_task_req = auth_helper::create_authenticated_request(
            "POST",
            &format!("/teams/{}/tasks", team_id),
            &owner.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        app.clone().oneshot(create_task_req).await.unwrap();
    }

    // Act - Query team tasks multiple times
    let start_time = Instant::now();
    let query_count = 20;

    for _ in 0..query_count {
        let list_req = auth_helper::create_authenticated_request(
            "GET",
            &format!(
                "/tasks/scoped?visibility=team&team_id={}&per_page=20",
                team_id
            ),
            &owner.access_token,
            None,
        );

        let response = app.clone().oneshot(list_req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    let elapsed = start_time.elapsed();

    // Assert - Queries should be fast (under 2 seconds for 20 queries)
    assert!(
        elapsed.as_secs() < 2,
        "Performing {} queries took {:?}, which is too slow",
        query_count,
        elapsed
    );

    println!(
        "Performed {} team task queries in {:?} ({:.2} queries/sec)",
        query_count,
        elapsed,
        query_count as f64 / elapsed.as_secs_f64()
    );
}

#[tokio::test]
async fn test_concurrent_team_member_access_performance() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create team
    let team_name = format!("Concurrent Access Test Team {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let create_team_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_team_res = app.clone().oneshot(create_team_req).await.unwrap();
    let body = body::to_bytes(create_team_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let team: Value = serde_json::from_slice(&body).unwrap();
    let team_id = team["data"]["id"].as_str().unwrap();

    // Create 3 team members total (owner + 2 members) - free plan limit
    let mut members = vec![owner];
    for i in 0..2 {
        let member = auth_helper::create_user_with_credentials(
            &app,
            &format!("member{}@example.com", i),
            &format!("member{}", i),
            "Complex#Pass2024",
        )
        .await
        .unwrap();

        // Add member to team
        let invite_data = json!({
            "user_id": member.id,
            "role": "Member"
        });

        let invite_req = auth_helper::create_authenticated_request(
            "POST",
            &format!("/teams/{}/members", team_id),
            &members[0].access_token,
            Some(serde_json::to_string(&invite_data).unwrap()),
        );

        let invite_res = app.clone().oneshot(invite_req).await.unwrap();
        if invite_res.status() != StatusCode::CREATED {
            let body = body::to_bytes(invite_res.into_body(), usize::MAX)
                .await
                .unwrap();
            let error: Value = serde_json::from_slice(&body).unwrap();
            panic!(
                "Failed to add member {} to team. Status: {}, Error: {}",
                i,
                StatusCode::CREATED,
                serde_json::to_string_pretty(&error).unwrap()
            );
        }
        members.push(member);
    }

    // Create 30 team tasks
    for i in 0..30 {
        let task_data = json!({
            "title": format!("Shared Task {}", i),
            "description": format!("Task {} for concurrent access", i),
            "visibility": "team"
        });

        let create_task_req = auth_helper::create_authenticated_request(
            "POST",
            &format!("/teams/{}/tasks", team_id),
            &members[0].access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        app.clone().oneshot(create_task_req).await.unwrap();
    }

    // Act - Simulate concurrent access from all team members
    let start_time = Instant::now();
    let mut handles = vec![];

    for member in members {
        let app_clone = app.clone();
        let team_id_clone = team_id.to_string();
        let member_token = member.access_token.clone();

        let handle = tokio::spawn(async move {
            // Each member queries tasks 5 times
            for _ in 0..5 {
                let list_req = auth_helper::create_authenticated_request(
                    "GET",
                    &format!("/tasks/scoped?visibility=team&team_id={}", team_id_clone),
                    &member_token,
                    None,
                );

                let response = app_clone.clone().oneshot(list_req).await.unwrap();
                assert_eq!(response.status(), StatusCode::OK);
            }
        });

        handles.push(handle);
    }

    // Wait for all concurrent requests to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let elapsed = start_time.elapsed();

    // Assert - Concurrent access should complete within 5 seconds
    assert!(
        elapsed.as_secs() < 5,
        "Concurrent access test took {:?}, which is too slow",
        elapsed
    );

    println!(
        "Completed concurrent access test with 3 members in {:?}",
        elapsed
    );
}
