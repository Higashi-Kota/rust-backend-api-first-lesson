use crate::common::app_helper::{
    add_team_member, create_request, create_team_task, create_team_task_assigned_to,
    create_test_task, create_test_team, create_user, parse_response_body, setup_full_app,
};
use crate::common::auth_helper::create_and_authenticate_user;
use axum::http::StatusCode;
use serde_json::json;
use task_backend::api::dto::team_task_dto::TransferTaskResponse;
use tower::ServiceExt;

#[tokio::test]
async fn test_transfer_personal_task_success() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let owner = create_and_authenticate_user(&app).await;
    let new_assignee = create_user(&app, "newassignee@example.com").await;

    // 個人タスクを作成
    let task = create_test_task(&app, &owner.access_token).await;

    // Act: タスクを引き継ぐ
    let transfer_data = json!({
        "new_assignee": new_assignee.id,
        "reason": "Going on vacation"
    });
    let response = app
        .oneshot(create_request(
            "POST",
            &format!("/tasks/{}/transfer", task.id),
            &owner.access_token,
            &transfer_data,
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body::<TransferTaskResponse>(response).await;
    assert_eq!(body.task_id, task.id);
    assert_eq!(body.previous_assignee, None);
    assert_eq!(body.new_assignee, new_assignee.id);
    assert_eq!(body.transferred_by, owner.id);
    assert_eq!(body.reason, Some("Going on vacation".to_string()));
}

#[tokio::test]
async fn test_transfer_team_task_success() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let owner = create_and_authenticate_user(&app).await;
    let member1 = create_and_authenticate_user(&app).await;
    let member2 = create_and_authenticate_user(&app).await;

    // チームを作成し、メンバーを追加
    let team = create_test_team(&app, &owner.access_token).await;
    add_team_member(&app, &owner.access_token, team.id, member1.id).await;
    add_team_member(&app, &owner.access_token, team.id, member2.id).await;

    // チームタスクを作成（member1に割り当て）
    let task = create_team_task_assigned_to(&app, &owner.access_token, team.id, member1.id).await;

    // Act: member1がタスクをmember2に引き継ぐ
    let transfer_data = json!({
        "new_assignee": member2.id,
        "reason": "Reassigning to available team member"
    });
    let response = app
        .oneshot(create_request(
            "POST",
            &format!("/tasks/{}/transfer", task.id),
            &member1.access_token,
            &transfer_data,
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_response_body::<TransferTaskResponse>(response).await;
    assert_eq!(body.task_id, task.id);
    assert_eq!(body.previous_assignee, Some(member1.id));
    assert_eq!(body.new_assignee, member2.id);
    assert_eq!(body.transferred_by, member1.id);
}

#[tokio::test]
async fn test_transfer_task_non_owner_forbidden() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let owner = create_and_authenticate_user(&app).await;
    let other_user = create_and_authenticate_user(&app).await;
    let new_assignee = create_user(&app, "newassignee@example.com").await;

    // 個人タスクを作成
    let task = create_test_task(&app, &owner.access_token).await;

    // Act: 所有者以外がタスクを引き継ごうとする
    let transfer_data = json!({
        "new_assignee": new_assignee.id,
        "reason": "Unauthorized transfer attempt"
    });
    let response = app
        .oneshot(create_request(
            "POST",
            &format!("/tasks/{}/transfer", task.id),
            &other_user.access_token,
            &transfer_data,
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_transfer_team_task_to_non_member_forbidden() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let owner = create_and_authenticate_user(&app).await;
    let member = create_and_authenticate_user(&app).await;
    let non_member = create_user(&app, "nonmember@example.com").await;

    // チームを作成し、メンバーを追加
    let team = create_test_team(&app, &owner.access_token).await;
    add_team_member(&app, &owner.access_token, team.id, member.id).await;

    // チームタスクを作成
    let task = create_team_task(&app, &owner.access_token, team.id).await;

    // Act: チームメンバーでない人に引き継ごうとする
    let transfer_data = json!({
        "new_assignee": non_member.id,
        "reason": "Invalid transfer to non-member"
    });
    let response = app
        .oneshot(create_request(
            "POST",
            &format!("/tasks/{}/transfer", task.id),
            &owner.access_token,
            &transfer_data,
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_transfer_task_invalid_assignee() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let owner = create_and_authenticate_user(&app).await;
    let task = create_test_task(&app, &owner.access_token).await;
    let invalid_user_id = uuid::Uuid::new_v4();

    // Act: 存在しないユーザーに引き継ごうとする
    let transfer_data = json!({
        "new_assignee": invalid_user_id,
        "reason": "Transfer to non-existent user"
    });
    let response = app
        .oneshot(create_request(
            "POST",
            &format!("/tasks/{}/transfer", task.id),
            &owner.access_token,
            &transfer_data,
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_transfer_task_reason_too_long() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let owner = create_and_authenticate_user(&app).await;
    let new_assignee = create_user(&app, "newassignee@example.com").await;
    let task = create_test_task(&app, &owner.access_token).await;

    let long_reason = "a".repeat(501); // 500文字を超える理由

    // Act
    let transfer_data = json!({
        "new_assignee": new_assignee.id,
        "reason": long_reason
    });
    let response = app
        .oneshot(create_request(
            "POST",
            &format!("/tasks/{}/transfer", task.id),
            &owner.access_token,
            &transfer_data,
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
