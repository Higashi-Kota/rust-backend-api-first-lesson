// tests/integration/tasks/permission_isolation_tests.rs

use axum::{
    body::{self},
    http::StatusCode,
};
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper};

#[tokio::test]
async fn test_user_task_isolation_strict() {
    // Setup: Test strict user isolation for tasks
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create three users to test isolation
    let user1_signup = auth_helper::create_test_user_with_info("alice@example.com", "alice");
    let user2_signup = auth_helper::create_test_user_with_info("bob@example.com", "bob");
    let user3_signup = auth_helper::create_test_user_with_info("charlie@example.com", "charlie");

    let alice = auth_helper::signup_test_user(&app, user1_signup)
        .await
        .unwrap();
    let bob = auth_helper::signup_test_user(&app, user2_signup)
        .await
        .unwrap();
    let charlie = auth_helper::signup_test_user(&app, user3_signup)
        .await
        .unwrap();

    // Each user creates a task
    let alice_task_data = json!({
        "title": "Alice's Private Task",
        "description": "This task belongs to Alice only",
        "status": "todo"
    });

    let bob_task_data = json!({
        "title": "Bob's Secret Task",
        "description": "This task belongs to Bob only",
        "status": "in_progress"
    });

    let charlie_task_data = json!({
        "title": "Charlie's Important Task",
        "description": "This task belongs to Charlie only",
        "status": "completed"
    });

    // Alice creates her task
    let alice_create_request = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &alice.access_token,
        Some(alice_task_data.to_string()),
    );

    let response = app.clone().oneshot(alice_create_request).await.unwrap();

    // Check status first
    let status = response.status();

    // If status is not CREATED, print the response body for debugging
    if status != StatusCode::CREATED {
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        eprintln!("Task creation failed with status {}: {}", status, body_str);
        if let Ok(error_response) = serde_json::from_slice::<Value>(&body) {
            eprintln!("Parsed error: {}", error_response);
        }
        panic!("Task creation failed");
    }

    assert_eq!(status, StatusCode::CREATED);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let alice_task_response: Value = serde_json::from_slice(&body).unwrap();
    let alice_task_id = alice_task_response["data"]["id"].as_str().unwrap();

    // Bob creates his task
    let bob_create_request = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &bob.access_token,
        Some(bob_task_data.to_string()),
    );

    let response = app.clone().oneshot(bob_create_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let bob_task_response: Value = serde_json::from_slice(&body).unwrap();
    let bob_task_id = bob_task_response["data"]["id"].as_str().unwrap();

    // Charlie creates his task
    let charlie_create_request = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &charlie.access_token,
        Some(charlie_task_data.to_string()),
    );

    let response = app.clone().oneshot(charlie_create_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let charlie_task_response: Value = serde_json::from_slice(&body).unwrap();
    let _charlie_task_id = charlie_task_response["data"]["id"].as_str().unwrap();

    // Test 1: Each user can only see their own tasks in list
    let alice_list_request =
        auth_helper::create_authenticated_request("GET", "/tasks", &alice.access_token, None);

    let response = app.clone().oneshot(alice_list_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();
    let alice_task_list = response["data"].as_array().unwrap();

    // Alice should see only her task
    assert_eq!(alice_task_list.len(), 1);
    assert_eq!(
        alice_task_list[0]["title"].as_str().unwrap(),
        "Alice's Private Task"
    );
    assert_eq!(
        alice_task_list[0]["user_id"].as_str().unwrap(),
        alice.id.to_string()
    );

    // Test 2: Users cannot access each other's specific tasks
    let bob_access_alice_task = auth_helper::create_authenticated_request(
        "GET",
        &format!("/tasks/{}", alice_task_id),
        &bob.access_token,
        None,
    );

    let response = app.clone().oneshot(bob_access_alice_task).await.unwrap();
    assert!(
        response.status() == StatusCode::FORBIDDEN || response.status() == StatusCode::NOT_FOUND
    );

    let charlie_access_bob_task = auth_helper::create_authenticated_request(
        "GET",
        &format!("/tasks/{}", bob_task_id),
        &charlie.access_token,
        None,
    );

    let response = app.clone().oneshot(charlie_access_bob_task).await.unwrap();
    assert!(
        response.status() == StatusCode::FORBIDDEN || response.status() == StatusCode::NOT_FOUND
    );

    // Test 3: Users cannot modify each other's tasks
    let malicious_update = json!({
        "title": "Hacked by Bob",
        "description": "Bob modified Alice's task"
    });

    let bob_modify_alice_task = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/tasks/{}", alice_task_id),
        &bob.access_token,
        Some(malicious_update.to_string()),
    );

    let response = app.clone().oneshot(bob_modify_alice_task).await.unwrap();
    assert!(
        response.status() == StatusCode::FORBIDDEN || response.status() == StatusCode::NOT_FOUND
    );

    // Test 4: Users cannot delete each other's tasks
    let charlie_delete_alice_task = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/tasks/{}", alice_task_id),
        &charlie.access_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(charlie_delete_alice_task)
        .await
        .unwrap();
    assert!(
        response.status() == StatusCode::FORBIDDEN || response.status() == StatusCode::NOT_FOUND
    );

    // Test 5: Verify tasks are still intact and owned by correct users
    let alice_verify_task = auth_helper::create_authenticated_request(
        "GET",
        &format!("/tasks/{}", alice_task_id),
        &alice.access_token,
        None,
    );

    let response = app.clone().oneshot(alice_verify_task).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        task["data"]["title"].as_str().unwrap(),
        "Alice's Private Task"
    );
    assert_eq!(
        task["data"]["user_id"].as_str().unwrap(),
        alice.id.to_string()
    );
    assert_ne!(task["data"]["title"].as_str().unwrap(), "Hacked by Bob");
}

#[tokio::test]
async fn test_concurrent_user_task_operations() {
    // Test concurrent operations by multiple users
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create two users
    let user1_signup = auth_helper::create_test_user_with_info("user1@example.com", "user1");
    let user2_signup = auth_helper::create_test_user_with_info("user2@example.com", "user2");

    let user1 = auth_helper::signup_test_user(&app, user1_signup)
        .await
        .unwrap();
    let user2 = auth_helper::signup_test_user(&app, user2_signup)
        .await
        .unwrap();

    // Both users create tasks with similar content
    let task_data = json!({
        "title": "Daily Standup",
        "description": "Prepare for daily standup meeting",
        "status": "todo"
    });

    let user1_create_request = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user1.access_token,
        Some(task_data.to_string()),
    );

    let user2_create_request = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user2.access_token,
        Some(task_data.to_string()),
    );

    // Execute requests concurrently (simulated by sequential execution)
    let response1 = app.clone().oneshot(user1_create_request).await.unwrap();
    let response2 = app.clone().oneshot(user2_create_request).await.unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);
    assert_eq!(response2.status(), StatusCode::CREATED);

    // Extract task IDs
    let body1 = body::to_bytes(response1.into_body(), usize::MAX)
        .await
        .unwrap();
    let task1_response: Value = serde_json::from_slice(&body1).unwrap();
    let task1_id = task1_response["data"]["id"].as_str().unwrap();

    let body2 = body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();
    let task2_response: Value = serde_json::from_slice(&body2).unwrap();
    let task2_id = task2_response["data"]["id"].as_str().unwrap();

    // Verify tasks are separate and belong to correct users
    assert_ne!(task1_id, task2_id, "Tasks should have different IDs");

    // Each user can only access their own task
    let user1_get_own_task = auth_helper::create_authenticated_request(
        "GET",
        &format!("/tasks/{}", task1_id),
        &user1.access_token,
        None,
    );

    let response = app.clone().oneshot(user1_get_own_task).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        task["data"]["user_id"].as_str().unwrap(),
        user1.id.to_string()
    );

    // User1 cannot access User2's task
    let user1_get_user2_task = auth_helper::create_authenticated_request(
        "GET",
        &format!("/tasks/{}", task2_id),
        &user1.access_token,
        None,
    );

    let response = app.clone().oneshot(user1_get_user2_task).await.unwrap();
    assert!(
        response.status() == StatusCode::FORBIDDEN || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_task_filtering_user_isolation() {
    // Test that task filtering respects user isolation
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create two users
    let user1_signup = auth_helper::create_test_user_with_info("dev1@example.com", "dev1");
    let user2_signup = auth_helper::create_test_user_with_info("dev2@example.com", "dev2");

    let user1 = auth_helper::signup_test_user(&app, user1_signup)
        .await
        .unwrap();
    let user2 = auth_helper::signup_test_user(&app, user2_signup)
        .await
        .unwrap();

    // User1 creates tasks with different statuses
    let user1_tasks = vec![
        ("Bug Fix #1", "Fix authentication bug", "todo"),
        ("Bug Fix #2", "Fix database connection", "in_progress"),
        ("Feature #1", "Add user management", "completed"),
    ];

    for (title, description, status) in user1_tasks {
        let task_data = json!({
            "title": title,
            "description": description,
            "status": status
        });

        let create_request = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user1.access_token,
            Some(task_data.to_string()),
        );

        let response = app.clone().oneshot(create_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // User2 creates tasks with different statuses
    let user2_tasks = vec![
        ("User2 Bug Fix", "Fix frontend bug", "todo"),
        ("User2 Feature", "Add dashboard", "completed"),
    ];

    for (title, description, status) in user2_tasks {
        let task_data = json!({
            "title": title,
            "description": description,
            "status": status
        });

        let create_request = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user2.access_token,
            Some(task_data.to_string()),
        );

        let response = app.clone().oneshot(create_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Test filtering by status for User1
    let user1_todo_filter = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/filter?status=todo",
        &user1.access_token,
        None,
    );

    let response = app.clone().oneshot(user1_todo_filter).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let filtered_response: Value = serde_json::from_slice(&body).unwrap();
    let tasks = filtered_response["data"]["items"].as_array().unwrap();

    // User1 should see only their own "todo" tasks
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"].as_str().unwrap(), "Bug Fix #1");
    assert_eq!(tasks[0]["status"].as_str().unwrap(), "todo");
    assert_eq!(tasks[0]["user_id"].as_str().unwrap(), user1.id.to_string());

    // Test filtering by status for User2
    let user2_completed_filter = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/filter?status=completed",
        &user2.access_token,
        None,
    );

    let response = app.clone().oneshot(user2_completed_filter).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let filtered_response: Value = serde_json::from_slice(&body).unwrap();
    let tasks = filtered_response["data"]["items"].as_array().unwrap();

    // User2 should see only their own "completed" tasks
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"].as_str().unwrap(), "User2 Feature");
    assert_eq!(tasks[0]["status"].as_str().unwrap(), "completed");
    assert_eq!(tasks[0]["user_id"].as_str().unwrap(), user2.id.to_string());
}

#[tokio::test]
async fn test_bulk_operations_user_isolation() {
    // Test that bulk operations respect user isolation
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create two users
    let user1_signup = auth_helper::create_test_user_with_info("bulk1@example.com", "bulk1");
    let user2_signup = auth_helper::create_test_user_with_info("bulk2@example.com", "bulk2");

    let user1 = auth_helper::signup_test_user(&app, user1_signup)
        .await
        .unwrap();
    let user2 = auth_helper::signup_test_user(&app, user2_signup)
        .await
        .unwrap();

    // Each user creates multiple tasks
    let user1_task_data = vec![
        json!({"title": "User1 Task 1", "description": "Description 1", "status": "todo"}),
        json!({"title": "User1 Task 2", "description": "Description 2", "status": "in_progress"}),
        json!({"title": "User1 Task 3", "description": "Description 3", "status": "completed"}),
    ];

    let user2_task_data = vec![
        json!({"title": "User2 Task 1", "description": "Description 1", "status": "todo"}),
        json!({"title": "User2 Task 2", "description": "Description 2", "status": "todo"}),
    ];

    // User1 creates their tasks
    for task_data in user1_task_data {
        let create_request = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user1.access_token,
            Some(task_data.to_string()),
        );

        let response = app.clone().oneshot(create_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // User2 creates their tasks
    for task_data in user2_task_data {
        let create_request = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user2.access_token,
            Some(task_data.to_string()),
        );

        let response = app.clone().oneshot(create_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Verify each user sees only their own tasks
    let user1_list_request =
        auth_helper::create_authenticated_request("GET", "/tasks", &user1.access_token, None);

    let response = app.clone().oneshot(user1_list_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();
    let user1_task_list = response["data"].as_array().unwrap();

    assert_eq!(user1_task_list.len(), 3);
    for task in user1_task_list {
        assert_eq!(task["user_id"].as_str().unwrap(), user1.id.to_string());
        assert!(task["title"].as_str().unwrap().starts_with("User1"));
    }

    let user2_list_request =
        auth_helper::create_authenticated_request("GET", "/tasks", &user2.access_token, None);

    let response = app.clone().oneshot(user2_list_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();
    let user2_task_list = response["data"].as_array().unwrap();

    assert_eq!(user2_task_list.len(), 2);
    for task in user2_task_list {
        assert_eq!(task["user_id"].as_str().unwrap(), user2.id.to_string());
        assert!(task["title"].as_str().unwrap().starts_with("User2"));
    }
}
