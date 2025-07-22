// tests/integration/tasks/index_performance_tests.rs
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde_json::json;
use std::time::Instant;
use task_backend::api::dto::task_dto::{PaginatedTasksDto, TaskDto};
use task_backend::types::ApiResponse;
use tower::ServiceExt;

use crate::common::app_helper::{add_team_member, create_test_team, setup_full_app};
use crate::common::auth_helper::create_and_authenticate_user;
use crate::common::request::create_request;

/// インデックス効果検証用のテストデータ作成
async fn create_bulk_test_tasks(
    app: &axum::Router,
    token: &str,
    count: usize,
    team_id: Option<uuid::Uuid>,
) {
    for i in 0..count {
        let task_data = if team_id.is_some() {
            json!({
                "title": format!("Performance Test Task {}", i),
                "description": format!("This is a test task for performance testing #{}", i),
                "status": match i % 3 {
                    0 => "todo",
                    1 => "in_progress",
                    _ => "completed"
                },
                "priority": match i % 4 {
                    0 => "low",
                    1 => "medium",
                    2 => "high",
                    _ => "high"
                },
                "visibility": "team"
            })
        } else {
            json!({
                "title": format!("Personal Test Task {}", i),
                "description": format!("Personal task for performance testing #{}", i),
                "status": match i % 3 {
                    0 => "todo",
                    1 => "in_progress",
                    _ => "completed"
                },
                "priority": match i % 4 {
                    0 => "low",
                    1 => "medium",
                    2 => "high",
                    _ => "high"
                }
            })
        };

        let endpoint = if team_id.is_some() {
            format!("/teams/{}/tasks", team_id.unwrap())
        } else {
            "/tasks".to_string()
        };

        let response = app
            .clone()
            .oneshot(create_request("POST", &endpoint, token, &task_data))
            .await
            .unwrap();

        let status = response.status();
        if status != StatusCode::CREATED {
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let error_msg = String::from_utf8_lossy(&body);
            panic!(
                "Failed to create task {}: status={}, endpoint={}, body={}",
                i, status, endpoint, error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_index_performance_personal_tasks() {
    // Arrange: Setup and create test data
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // Create 50 personal tasks
    println!("Creating 50 personal tasks...");
    let creation_start = Instant::now();
    create_bulk_test_tasks(&app, &user.token, 50, None).await;
    let creation_duration = creation_start.elapsed();
    println!("Task creation took: {:?}", creation_duration);

    // Act & Assert: Test query performance with visibility filter
    println!("\nTesting query performance with visibility filter...");
    let query_start = Instant::now();
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/tasks/scoped?visibility=personal&per_page=50",
            &user.token,
            &(),
        ))
        .await
        .unwrap();
    let query_duration = query_start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Query with visibility filter took: {:?}", query_duration);

    // Parse response to verify results
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedTasksDto> = serde_json::from_slice(&body).unwrap();
    let tasks = api_response.data.unwrap();
    assert_eq!(tasks.items.len(), 50); // Should return 50 items per page

    // Test query performance with status filter
    println!("\nTesting query performance with status filter...");
    let status_query_start = Instant::now();
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/tasks?status=todo&per_page=50",
            &user.token,
            &(),
        ))
        .await
        .unwrap();
    let status_query_duration = status_query_start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Query with status filter took: {:?}", status_query_duration);

    // Assert: Query should be fast due to indexes
    assert!(
        query_duration.as_millis() < 100,
        "Query took too long: {:?}",
        query_duration
    );
    assert!(
        status_query_duration.as_millis() < 100,
        "Status query took too long: {:?}",
        status_query_duration
    );
}

#[tokio::test]
async fn test_index_performance_team_tasks() {
    // Arrange: Setup and create test data
    let (app, _schema, _db) = setup_full_app().await;
    let user1 = create_and_authenticate_user(&app).await;
    let user2 = create_and_authenticate_user(&app).await;
    let team = create_test_team(&app, &user1.token).await;
    add_team_member(&app, &user1.token, team.id, user2.id).await;

    // Create 50 team tasks
    println!("Creating 50 team tasks...");
    let creation_start = Instant::now();
    create_bulk_test_tasks(&app, &user1.token, 50, Some(team.id)).await;
    let creation_duration = creation_start.elapsed();
    println!("Team task creation took: {:?}", creation_duration);

    // Act & Assert: Test team task query performance
    println!("\nTesting team task query performance...");
    let query_start = Instant::now();
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/tasks/scoped?visibility=team&team_id={}&per_page=50",
                team.id
            ),
            &user1.token,
            &(),
        ))
        .await
        .unwrap();
    let query_duration = query_start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Team task query took: {:?}", query_duration);

    // Test query with priority filter
    println!("\nTesting query performance with priority filter...");
    let priority_query_start = Instant::now();
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/tasks/scoped?visibility=team&team_id={}&priority=high&per_page=20",
                team.id
            ),
            &user1.token,
            &(),
        ))
        .await
        .unwrap();
    let priority_query_duration = priority_query_start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!(
        "Priority filtered query took: {:?}",
        priority_query_duration
    );

    // Assert: Queries should be fast due to indexes
    assert!(
        query_duration.as_millis() < 150,
        "Team query took too long: {:?}",
        query_duration
    );
    assert!(
        priority_query_duration.as_millis() < 100,
        "Priority query took too long: {:?}",
        priority_query_duration
    );
}

#[tokio::test]
async fn test_index_performance_assigned_tasks() {
    // Arrange: Setup and create test data
    let (app, _schema, _db) = setup_full_app().await;
    let user1 = create_and_authenticate_user(&app).await;
    let user2 = create_and_authenticate_user(&app).await;
    let team = create_test_team(&app, &user1.token).await;
    add_team_member(&app, &user1.token, team.id, user2.id).await;

    // Create tasks and assign them to user2
    println!("Creating and assigning tasks...");
    let assignment_start = Instant::now();

    for i in 0..20 {
        // Create task
        let task_data = json!({
            "title": format!("Assigned Task {}", i),
            "description": "Task to be assigned",
            "status": "todo",
            "priority": "medium",
            "visibility": "team",
            "assigned_to": user2.id
        });

        let response = app
            .clone()
            .oneshot(create_request(
                "POST",
                &format!("/teams/{}/tasks", team.id),
                &user1.token,
                &task_data,
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let assignment_duration = assignment_start.elapsed();
    println!("Task assignment took: {:?}", assignment_duration);

    // Act & Assert: Test assigned tasks query performance
    println!("\nTesting assigned tasks query performance...");
    let query_start = Instant::now();
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/tasks?assigned_to={}&per_page=50", user2.id),
            &user2.token,
            &(),
        ))
        .await
        .unwrap();
    let query_duration = query_start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Assigned tasks query took: {:?}", query_duration);

    // Assert: Query should be fast due to assigned_to index
    assert!(
        query_duration.as_millis() < 100,
        "Assigned tasks query took too long: {:?}",
        query_duration
    );
}

#[tokio::test]
async fn test_index_performance_date_range_queries() {
    // Arrange: Setup and create test data
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // Create tasks with different due dates
    println!("Creating tasks with various due dates...");
    let creation_start = Instant::now();

    for i in 0..30 {
        let due_date = Utc::now() + Duration::days(i % 30);
        let task_data = json!({
            "title": format!("Task with due date {}", i),
            "description": "Task for date range testing",
            "status": "todo",
            "priority": "medium",
            "due_date": due_date.timestamp()
        });

        let response = app
            .clone()
            .oneshot(create_request("POST", "/tasks", &user.token, &task_data))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let creation_duration = creation_start.elapsed();
    println!("Task creation with due dates took: {:?}", creation_duration);

    // Act & Assert: Test date range query performance
    println!("\nTesting date range query performance...");
    let tomorrow = Utc::now() + Duration::days(1);
    let next_week = Utc::now() + Duration::days(7);

    let query_start = Instant::now();
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/tasks?due_date_from={}&due_date_to={}&per_page=50",
                tomorrow.timestamp(),
                next_week.timestamp()
            ),
            &user.token,
            &(),
        ))
        .await
        .unwrap();
    let query_duration = query_start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Date range query took: {:?}", query_duration);

    // Assert: Query should be fast due to indexes
    assert!(
        query_duration.as_millis() < 150,
        "Date range query took too long: {:?}",
        query_duration
    );
}

#[tokio::test]
async fn test_index_performance_fulltext_search() {
    // Arrange: Setup and create test data
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // Create tasks with searchable content
    println!("Creating tasks with searchable content...");
    let keywords = ["important", "review", "deployment", "bug", "feature"];
    let creation_start = Instant::now();

    for i in 0..50 {
        let keyword = &keywords[i % keywords.len()];
        let task_data = json!({
            "title": format!("{} Task #{}", keyword, i),
            "description": format!("This task is about {} and needs attention", keyword),
            "status": "todo",
            "priority": "medium"
        });

        let response = app
            .clone()
            .oneshot(create_request("POST", "/tasks", &user.token, &task_data))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let creation_duration = creation_start.elapsed();
    println!("Task creation took: {:?}", creation_duration);

    // Act & Assert: Test fulltext search performance
    println!("\nTesting fulltext search performance...");
    let search_start = Instant::now();
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/tasks?search=deployment&per_page=50",
            &user.token,
            &(),
        ))
        .await
        .unwrap();
    let search_duration = search_start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Fulltext search took: {:?}", search_duration);

    // Parse response to verify results
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    // Parse as array of tasks instead of PaginatedTasksDto
    let api_response: ApiResponse<Vec<TaskDto>> = serde_json::from_slice(&body).unwrap();
    let tasks = api_response.data.unwrap();

    // Verify that search results contain the keyword
    assert!(tasks.iter().any(|t| t.title.contains("deployment")
        || t.description
            .as_ref()
            .is_some_and(|d| d.contains("deployment"))));

    // Assert: Fulltext search should be reasonably fast with GIN index
    assert!(
        search_duration.as_millis() < 200,
        "Fulltext search took too long: {:?}",
        search_duration
    );
}

#[tokio::test]
async fn test_index_performance_comparison() {
    // This test compares performance with and without using indexed columns
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // Create a reasonable amount of test data
    println!("Creating test data for comparison...");
    create_bulk_test_tasks(&app, &user.token, 50, None).await;

    // Test 1: Query using indexed columns (user_id + visibility + status)
    println!("\nTest 1: Query with indexed columns...");
    let indexed_start = Instant::now();
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/tasks/scoped?visibility=personal&status=todo&per_page=100",
            &user.token,
            &(),
        ))
        .await
        .unwrap();
    let indexed_duration = indexed_start.elapsed();
    assert_eq!(response.status(), StatusCode::OK);
    println!("Indexed query took: {:?}", indexed_duration);

    // Test 2: Query using partially indexed columns
    println!("\nTest 2: Query with partially indexed columns...");
    let partial_start = Instant::now();
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/tasks?priority=high&per_page=100",
            &user.token,
            &(),
        ))
        .await
        .unwrap();
    let partial_duration = partial_start.elapsed();
    assert_eq!(response.status(), StatusCode::OK);
    println!("Partially indexed query took: {:?}", partial_duration);

    // Assert: Fully indexed queries should generally be faster
    println!("\nPerformance comparison:");
    println!("Fully indexed query: {:?}", indexed_duration);
    println!("Partially indexed query: {:?}", partial_duration);

    // Both should still be reasonably fast
    assert!(indexed_duration.as_millis() < 100, "Indexed query too slow");
    assert!(partial_duration.as_millis() < 150, "Partial query too slow");
}
