// tests/integration/teams/viewer_role_tests.rs
//
// Viewerロールの包括的なアクセス制御テスト
// 読み取り専用権限の検証と、更新・削除操作の拒否テスト

use crate::common::{app_helper, auth_helper};
use axum::{body, http::StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

fn create_test_team_data(name: &str) -> Value {
    json!({
        "name": name,
        "description": "Test team for viewer role testing"
    })
}

/// Viewerロールでチーム情報を閲覧できることを確認
#[tokio::test]
async fn test_viewer_can_view_team() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create owner
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create viewer
    let viewer = auth_helper::create_user_with_credentials(
        &app,
        "viewer@example.com",
        "viewer",
        "Complex#Pass2024",
    )
    .await
    .unwrap();

    // Create team
    let team_name = format!("Viewer Test Team {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let create_team_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_team_res = app.clone().oneshot(create_team_req).await.unwrap();
    assert_eq!(create_team_res.status(), StatusCode::CREATED);

    let body = body::to_bytes(create_team_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let team: Value = serde_json::from_slice(&body).unwrap();
    let team_id = team["data"]["id"].as_str().unwrap();

    // Add viewer to team with Viewer role
    let add_member_data = json!({
        "user_id": viewer.id,
        "role": "Viewer"
    });

    let add_member_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&add_member_data).unwrap()),
    );

    let add_member_res = app.clone().oneshot(add_member_req).await.unwrap();
    assert_eq!(add_member_res.status(), StatusCode::CREATED);

    // Act - Viewer tries to view team
    let view_team_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/teams/{}", team_id),
        &viewer.access_token,
        None,
    );

    let response = app.oneshot(view_team_req).await.unwrap();

    // Assert - Viewer can view team
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let team_response: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(team_response["data"]["name"], team_name);
}

/// Viewerロールでチームメンバー一覧を閲覧できることを確認
#[tokio::test]
async fn test_viewer_can_view_team_members() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let viewer = auth_helper::create_user_with_credentials(
        &app,
        "viewer2@example.com",
        "viewer2",
        "Complex#Pass2024",
    )
    .await
    .unwrap();

    // Create team
    let team_name = format!("Viewer Test Team {}", Uuid::new_v4());
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

    // Add viewer to team
    let add_member_data = json!({
        "user_id": viewer.id,
        "role": "Viewer"
    });

    let add_member_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&add_member_data).unwrap()),
    );

    let add_member_res = app.clone().oneshot(add_member_req).await.unwrap();
    assert_eq!(add_member_res.status(), StatusCode::CREATED);

    // Act - Viewer tries to view team (which should include members info)
    let view_team_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/teams/{}", team_id),
        &viewer.access_token,
        None,
    );

    let response = app.oneshot(view_team_req).await.unwrap();

    // Assert - Viewer can view team
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let team_response: Value = serde_json::from_slice(&body).unwrap();
    // Team info should be accessible to viewer
    assert_eq!(team_response["data"]["name"], team_name);
}

/// Viewerロールでチームタスクを閲覧できることを確認
#[tokio::test]
async fn test_viewer_can_view_team_tasks() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let viewer = auth_helper::create_user_with_credentials(
        &app,
        "viewer3@example.com",
        "viewer3",
        "Complex#Pass2024",
    )
    .await
    .unwrap();

    // Create team
    let team_name = format!("Viewer Test Team {}", Uuid::new_v4());
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

    // Add viewer to team
    let add_member_data = json!({
        "user_id": viewer.id,
        "role": "Viewer"
    });

    let add_member_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&add_member_data).unwrap()),
    );

    let add_member_res = app.clone().oneshot(add_member_req).await.unwrap();
    assert_eq!(add_member_res.status(), StatusCode::CREATED);

    // Create team task
    let task_data = json!({
        "title": "Team Task for Viewer",
        "description": "This task should be visible to viewers",
        "visibility": "team"
    });

    let create_task_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/tasks", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    app.clone().oneshot(create_task_req).await.unwrap();

    // Act - Viewer tries to view team tasks
    let view_tasks_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/tasks/scoped?visibility=team&team_id={}", team_id),
        &viewer.access_token,
        None,
    );

    let response = app.oneshot(view_tasks_req).await.unwrap();

    // Assert - Viewer can view tasks
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let tasks_response: Value = serde_json::from_slice(&body).unwrap();
    let tasks = tasks_response["data"]["items"].as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"], "Team Task for Viewer");
}

/// Viewerロールでチーム更新が拒否されることを確認
#[tokio::test]
async fn test_viewer_cannot_update_team() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let viewer = auth_helper::create_user_with_credentials(
        &app,
        "viewer4@example.com",
        "viewer4",
        "Complex#Pass2024",
    )
    .await
    .unwrap();

    // Create team
    let team_name = format!("Viewer Test Team {}", Uuid::new_v4());
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

    // Add viewer to team
    let add_member_data = json!({
        "user_id": viewer.id,
        "role": "Viewer"
    });

    let add_member_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&add_member_data).unwrap()),
    );

    let add_member_res = app.clone().oneshot(add_member_req).await.unwrap();
    assert_eq!(add_member_res.status(), StatusCode::CREATED);

    // Act - Viewer tries to update team
    let update_data = json!({
        "name": "Hacked Team Name",
        "description": "This should not be allowed"
    });

    let update_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/teams/{}", team_id),
        &viewer.access_token,
        Some(serde_json::to_string(&update_data).unwrap()),
    );

    let response = app.oneshot(update_req).await.unwrap();

    // Assert - Update is forbidden
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// Viewerロールでチームタスク作成が拒否されることを確認
#[tokio::test]
async fn test_viewer_cannot_create_team_task() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let viewer = auth_helper::create_user_with_credentials(
        &app,
        "viewer5@example.com",
        "viewer5",
        "Complex#Pass2024",
    )
    .await
    .unwrap();

    // Create team
    let team_name = format!("Viewer Test Team {}", Uuid::new_v4());
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

    // Add viewer to team
    let add_member_data = json!({
        "user_id": viewer.id,
        "role": "Viewer"
    });

    let add_member_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&add_member_data).unwrap()),
    );

    let add_member_res = app.clone().oneshot(add_member_req).await.unwrap();
    assert_eq!(add_member_res.status(), StatusCode::CREATED);

    // Act - Viewer tries to create team task
    let task_data = json!({
        "title": "Unauthorized Task",
        "description": "This should not be allowed",
        "visibility": "team"
    });

    let create_task_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/tasks", team_id),
        &viewer.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let response = app.oneshot(create_task_req).await.unwrap();

    // Assert - Create is forbidden
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// Viewerロールでチームタスク更新が拒否されることを確認
#[tokio::test]
async fn test_viewer_cannot_update_team_task() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let viewer = auth_helper::create_user_with_credentials(
        &app,
        "viewer6@example.com",
        "viewer6",
        "Complex#Pass2024",
    )
    .await
    .unwrap();

    // Create team
    let team_name = format!("Viewer Test Team {}", Uuid::new_v4());
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

    // Add viewer to team
    let add_member_data = json!({
        "user_id": viewer.id,
        "role": "Viewer"
    });

    let add_member_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&add_member_data).unwrap()),
    );

    let add_member_res = app.clone().oneshot(add_member_req).await.unwrap();
    assert_eq!(add_member_res.status(), StatusCode::CREATED);

    // Create team task as owner
    let task_data = json!({
        "title": "Original Task",
        "description": "Task to be updated",
        "visibility": "team"
    });

    let create_task_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/tasks", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_task_res = app.clone().oneshot(create_task_req).await.unwrap();
    let body = body::to_bytes(create_task_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let task_response: Value = serde_json::from_slice(&body).unwrap();
    let task_id = task_response["data"]["id"].as_str().unwrap();

    // Act - Viewer tries to update task
    let update_data = json!({
        "title": "Hacked Task Title"
    });

    let update_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/tasks/{}", task_id),
        &viewer.access_token,
        Some(serde_json::to_string(&update_data).unwrap()),
    );

    let response = app.oneshot(update_req).await.unwrap();

    // Assert - Update is forbidden
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// Viewerロールでチームタスク削除が拒否されることを確認
#[tokio::test]
async fn test_viewer_cannot_delete_team_task() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let viewer = auth_helper::create_user_with_credentials(
        &app,
        "viewer7@example.com",
        "viewer7",
        "Complex#Pass2024",
    )
    .await
    .unwrap();

    // Create team
    let team_name = format!("Viewer Test Team {}", Uuid::new_v4());
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

    // Add viewer to team
    let add_member_data = json!({
        "user_id": viewer.id,
        "role": "Viewer"
    });

    let add_member_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&add_member_data).unwrap()),
    );

    let add_member_res = app.clone().oneshot(add_member_req).await.unwrap();
    assert_eq!(add_member_res.status(), StatusCode::CREATED);

    // Create team task as owner
    let task_data = json!({
        "title": "Task to Delete",
        "description": "This task should not be deletable by viewer",
        "visibility": "team"
    });

    let create_task_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/tasks", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_task_res = app.clone().oneshot(create_task_req).await.unwrap();
    let body = body::to_bytes(create_task_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let task_response: Value = serde_json::from_slice(&body).unwrap();
    let task_id = task_response["data"]["id"].as_str().unwrap();

    // Act - Viewer tries to delete task
    let delete_req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/tasks/{}", task_id),
        &viewer.access_token,
        None,
    );

    let response = app.oneshot(delete_req).await.unwrap();

    // Assert - Delete is forbidden
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// Viewerロールでメンバー追加が拒否されることを確認
#[tokio::test]
async fn test_viewer_cannot_add_team_members() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let viewer = auth_helper::create_user_with_credentials(
        &app,
        "viewer8@example.com",
        "viewer8",
        "Complex#Pass2024",
    )
    .await
    .unwrap();
    let new_user = auth_helper::create_user_with_credentials(
        &app,
        "newuser@example.com",
        "newuser",
        "Complex#Pass2024",
    )
    .await
    .unwrap();

    // Create team
    let team_name = format!("Viewer Test Team {}", Uuid::new_v4());
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

    // Add viewer to team
    let add_viewer_data = json!({
        "user_id": viewer.id,
        "role": "Viewer"
    });

    let add_viewer_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&add_viewer_data).unwrap()),
    );

    let add_viewer_res = app.clone().oneshot(add_viewer_req).await.unwrap();
    assert_eq!(add_viewer_res.status(), StatusCode::CREATED);

    // Act - Viewer tries to add new member
    let add_member_data = json!({
        "user_id": new_user.id,
        "role": "Member"
    });

    let add_member_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &viewer.access_token,
        Some(serde_json::to_string(&add_member_data).unwrap()),
    );

    let response = app.oneshot(add_member_req).await.unwrap();

    // Assert - Add member is forbidden
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// Viewerロールでメンバーロール変更が拒否されることを確認
#[tokio::test]
async fn test_viewer_cannot_update_member_role() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let viewer = auth_helper::create_user_with_credentials(
        &app,
        "viewer9@example.com",
        "viewer9",
        "Complex#Pass2024",
    )
    .await
    .unwrap();
    let member = auth_helper::create_user_with_credentials(
        &app,
        "member@example.com",
        "member",
        "Complex#Pass2024",
    )
    .await
    .unwrap();

    // Create team
    let team_name = format!("Viewer Test Team {}", Uuid::new_v4());
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

    // Add viewer to team
    let add_viewer_data = json!({
        "user_id": viewer.id,
        "role": "Viewer"
    });

    let add_viewer_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&add_viewer_data).unwrap()),
    );

    let add_viewer_res = app.clone().oneshot(add_viewer_req).await.unwrap();
    assert_eq!(add_viewer_res.status(), StatusCode::CREATED);

    // Add member to team
    let add_member_data = json!({
        "user_id": member.id,
        "role": "Member"
    });

    let add_member_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&add_member_data).unwrap()),
    );

    let add_member_res = app.clone().oneshot(add_member_req).await.unwrap();
    assert_eq!(add_member_res.status(), StatusCode::CREATED);

    // Act - Viewer tries to update member's role
    let update_role_data = json!({
        "role": "Admin"
    });

    let update_role_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/teams/{}/members/{}/role", team_id, member.id),
        &viewer.access_token,
        Some(serde_json::to_string(&update_role_data).unwrap()),
    );

    let response = app.oneshot(update_role_req).await.unwrap();

    // Assert - Update role is forbidden
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
