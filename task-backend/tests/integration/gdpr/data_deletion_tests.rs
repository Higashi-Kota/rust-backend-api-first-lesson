// tests/integration/gdpr/data_deletion_tests.rs

use crate::common::{app_helper, auth_helper};
use axum::http::StatusCode;
use serde_json::json;
use task_backend::features::task::repositories::task_repository::TaskRepository;
use task_backend::features::team::repositories::team::TeamRepository;
use task_backend::features::user::repositories::user::UserRepository;
use tower::ServiceExt;
use uuid::Uuid;

async fn create_comprehensive_user_data(
    app: &axum::Router,
    user: &auth_helper::TestUser,
) -> (Vec<Uuid>, Vec<Uuid>) {
    // Create multiple tasks
    let mut task_ids = Vec::new();
    for i in 0..5 {
        let task_data = json!({
            "title": format!("Task to be deleted {}", i),
            "description": format!("This task will be deleted as part of GDPR compliance {}", i),
            "status": if i % 2 == 0 { "todo" } else { "completed" }
        });

        let req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );
        let res = app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();

        if status != StatusCode::CREATED {
            let body_str = String::from_utf8_lossy(&body);
            panic!(
                "Task creation failed. Status: {:?}, Body: {}",
                status, body_str
            );
        }

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
        "name": "Team to be removed from",
        "description": "User will be removed from this team"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &user.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();

    if status != StatusCode::CREATED {
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Team creation failed. Status: {:?}, Body: {}",
            status, body_str
        );
    }

    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // Try to get ID from data field first, then directly from response
    let id_str = response["data"]["id"]
        .as_str()
        .or_else(|| response["id"].as_str())
        .unwrap_or_else(|| panic!("Team creation failed, no id in response: {:?}", response));
    team_ids.push(Uuid::parse_str(id_str).unwrap());

    // Upgrade subscription to create history
    let upgrade_request = json!({ "new_tier": "Pro" });
    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/users/{}/subscription/upgrade", user.id),
        &user.access_token,
        Some(serde_json::to_string(&upgrade_request).unwrap()),
    );
    let _ = app.clone().oneshot(req).await.unwrap();

    (task_ids, team_ids)
}

#[tokio::test]
async fn test_delete_user_data_requires_confirmation() {
    // === Arrange: テスト用アプリとユーザーを設定 ===
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    let deletion_request = json!({
        "confirm_deletion": false,  // 確認フラグをfalseに設定
        "reason": "Testing deletion without confirmation"
    });

    // === Act: 確認なしで削除を試みる ===
    let req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/gdpr/users/{}/delete", user.id),
        &user.access_token,
        Some(serde_json::to_string(&deletion_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // === Assert: バリデーションエラーを確認 ===
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(response["success"], false);

    let error_message = response["error"]
        .as_str()
        .or_else(|| response["message"].as_str())
        .unwrap_or("");
    assert!(
        error_message.contains("Deletion must be confirmed"),
        "Expected error message to contain 'Deletion must be confirmed', but got: {}",
        error_message
    );
}

#[tokio::test]
async fn test_delete_user_data_complete() {
    // === Arrange: テスト用データの准備 ===
    // アプリ、スキーマ、データベースの設定
    let (app, _schema, db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let user_id = user.id;

    // ユーザーに関連する包括的なデータを作成
    let (task_ids, team_ids) = create_comprehensive_user_data(&app, &user).await;

    // 削除リクエストの構築
    let deletion_request = json!({
        "confirm_deletion": true,  // 確認フラグをtrueに設定
        "reason": "User requested complete data deletion"
    });

    // === Act: ユーザーデータの完全削除を実行 ===
    let req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/gdpr/users/{}/delete", user_id),
        &user.access_token,
        Some(serde_json::to_string(&deletion_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // === Assert: 削除結果の検証 ===
    // 1. HTTPレスポンスの検証
    assert_eq!(status, StatusCode::OK);
    assert!(
        response["success"].as_bool().unwrap(),
        "Deletion should succeed"
    );

    // 2. 削除レスポンスデータの検証
    let data = &response["data"];
    assert_eq!(data["user_id"], user_id.to_string());
    assert!(
        data["deleted_at"].is_string(),
        "deleted_at timestamp should be present"
    );

    // 3. 削除されたレコード数の検証
    let deleted_records = &data["deleted_records"];
    assert_eq!(deleted_records["user_data"], true);
    assert_eq!(deleted_records["tasks_count"], task_ids.len());
    assert_eq!(deleted_records["teams_count"], team_ids.len());
    assert!(
        deleted_records["subscription_history_count"]
            .as_u64()
            .unwrap()
            >= 1,
        "At least one subscription history record should be deleted"
    );

    // 4. データベースからの完全削除を検証
    // ユーザーが削除されたことを確認
    let user_repo = UserRepository::new(db.connection.clone());
    let deleted_user = user_repo.find_by_id(user_id).await.unwrap();
    assert!(
        deleted_user.is_none(),
        "User should be deleted from database"
    );

    // タスクが削除されたことを確認
    let task_repo = TaskRepository::new(db.connection.clone());
    for task_id in task_ids {
        let task = task_repo.find_by_id(task_id).await.unwrap();
        assert!(task.is_none(), "Task {} should be deleted", task_id);
    }

    // チームから削除されたことを確認
    let team_repo = TeamRepository::new(db.connection.clone());
    for team_id in team_ids {
        let members = team_repo.find_members_by_team_id(team_id).await.unwrap();
        assert!(
            !members.iter().any(|m| m.user_id == user_id),
            "User should be removed from team {}",
            team_id
        );
    }
}

#[tokio::test]
async fn test_user_cannot_delete_other_user_data() {
    // Arrange: Set up app and create two users
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user1 = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let user2_data = auth_helper::create_test_user_with_info("other@example.com", "OtherUser");
    let user2 = auth_helper::signup_test_user(&app, user2_data)
        .await
        .unwrap();

    let deletion_request = json!({
        "confirm_deletion": true,
        "reason": "Malicious deletion attempt"
    });

    // Act: User1 tries to delete User2's data
    let req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/gdpr/users/{}/delete", user2.id),
        &user1.access_token,
        Some(serde_json::to_string(&deletion_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should be forbidden
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_can_delete_any_user_data() {
    // Arrange: Set up app with admin and regular user
    let (app, _schema, db) = app_helper::setup_full_app().await;
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;
    let user_data = auth_helper::create_test_user_with_info("tobedeleted@example.com", "DeleteMe");
    let user = auth_helper::signup_test_user(&app, user_data)
        .await
        .unwrap();
    let user_id = user.id;

    // Create some data for the user
    let (_, _) = create_comprehensive_user_data(&app, &user).await;

    let deletion_request = json!({
        "confirm_deletion": true,
        "reason": "Admin initiated deletion"
    });

    // Act: Admin deletes user's data
    let req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/admin/gdpr/users/{}/delete", user_id),
        &admin_token,
        Some(serde_json::to_string(&deletion_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should be allowed
    assert_eq!(res.status(), StatusCode::OK);

    // Verify user is deleted
    let user_repo = UserRepository::new(db.connection.clone());
    let deleted_user = user_repo.find_by_id(user_id).await.unwrap();
    assert!(deleted_user.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_user() {
    // Arrange: Set up app and admin
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;
    let nonexistent_id = Uuid::new_v4();

    let deletion_request = json!({
        "confirm_deletion": true,
        "reason": "Testing deletion of nonexistent user"
    });

    // Act: Try to delete nonexistent user
    let req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/admin/gdpr/users/{}/delete", nonexistent_id),
        &admin_token,
        Some(serde_json::to_string(&deletion_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should return not found
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_compliance_status() {
    // Arrange: Set up app and create user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Act: Get compliance status
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/gdpr/users/{}/status", user.id),
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

    let data = &response["data"];
    assert_eq!(data["user_id"], user.id.to_string());
    assert_eq!(data["data_retention_days"], 90);
    assert_eq!(data["deletion_requested"], false);
    assert!(data["deletion_scheduled_for"].is_null());
}

#[tokio::test]
async fn test_deleted_user_cannot_authenticate() {
    // Arrange: Set up app and create user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user_data =
        auth_helper::create_test_user_with_info("willbedeleted@example.com", "DeletedUser");
    let user = auth_helper::signup_test_user(&app, user_data)
        .await
        .unwrap();

    let email = user.email.clone();
    let password = "DeletedP@ssw0rd";

    // Delete the user
    let deletion_request = json!({
        "confirm_deletion": true,
        "reason": "Testing authentication after deletion"
    });

    let req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/gdpr/users/{}/delete", user.id),
        &user.access_token,
        Some(serde_json::to_string(&deletion_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Act: Try to login with deleted user credentials
    let login_request = json!({
        "identifier": email,
        "password": password
    });

    let req = auth_helper::create_request(
        "POST",
        "/auth/signin",
        Some(serde_json::to_string(&login_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should fail authentication
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_cascade_deletion_of_related_data() {
    // Arrange: Set up app and create user with related data
    let (app, _schema, db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let user_id = user.id;

    // Create a team and invite another user
    let team_data = json!({
        "name": "Team with invitations",
        "description": "This team has pending invitations"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &user.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let team_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let team_id = team_response["data"]["id"].as_str().unwrap();

    // Create invitation
    let invitation_data = json!({
        "email": "invited@example.com",
        "message": "Join our team!"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/invitations/single", team_id),
        &user.access_token,
        Some(serde_json::to_string(&invitation_data).unwrap()),
    );
    let _ = app.clone().oneshot(req).await.unwrap();

    // Delete user
    let deletion_request = json!({
        "confirm_deletion": true,
        "reason": "Testing cascade deletion"
    });

    let req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/gdpr/users/{}/delete", user_id),
        &user.access_token,
        Some(serde_json::to_string(&deletion_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Deletion succeeds
    assert_eq!(res.status(), StatusCode::OK);

    // Verify related data is handled appropriately
    let team_repo = TeamRepository::new(db.connection.clone());
    let team_result = team_repo
        .find_by_id(Uuid::parse_str(team_id).unwrap())
        .await;

    // The team should be deleted due to CASCADE constraint
    // If we get an error about null owner_id, it means the team still exists but with null owner
    // which is incorrect with the current schema
    match team_result {
        Ok(team) => {
            // Team should be None (deleted)
            assert!(
                team.is_none(),
                "Team should be deleted due to CASCADE constraint"
            );
        }
        Err(e) => {
            // If we get an error about null owner_id, the team exists but is corrupted
            // This suggests the CASCADE constraint is not working properly
            panic!("Unexpected error when checking team after owner deletion: {:?}. This suggests the team exists with null owner_id, which violates the schema constraints.", e);
        }
    }
}

#[tokio::test]
async fn test_deletion_reason_optional() {
    // Arrange: Set up app and create user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    let deletion_request = json!({
        "confirm_deletion": true
        // No reason provided
    });

    // Act: Delete without providing reason
    let req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/gdpr/users/{}/delete", user.id),
        &user.access_token,
        Some(serde_json::to_string(&deletion_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should succeed
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_deletion_is_idempotent() {
    // Arrange: Set up app and create user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;
    let user_data =
        auth_helper::create_test_user_with_info("idempotent@example.com", "IdempotentUser");
    let user = auth_helper::signup_test_user(&app, user_data)
        .await
        .unwrap();

    let deletion_request = json!({
        "confirm_deletion": true,
        "reason": "Testing idempotent deletion"
    });

    // Act: Delete user twice
    for i in 0..2 {
        let req = auth_helper::create_authenticated_request(
            "DELETE",
            &format!("/admin/gdpr/users/{}/delete", user.id),
            &admin_token,
            Some(serde_json::to_string(&deletion_request).unwrap()),
        );
        let res = app.clone().oneshot(req).await.unwrap();

        if i == 0 {
            // First deletion should succeed
            assert_eq!(res.status(), StatusCode::OK);
        } else {
            // Second deletion should return not found
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        }
    }
}
