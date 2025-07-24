// tests/integration/team/permission_tests.rs

use crate::common::{app_helper, auth_helper};
use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

/// 権限チェックミドルウェアを使用したチーム作成テスト（正常系）
#[tokio::test]
async fn test_create_team_with_permission_success() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // メンバーロールのユーザーを作成
    let signup_data =
        auth_helper::create_test_user_with_info("team_member@example.com", "team_member");
    let user = auth_helper::signup_test_user(&app, signup_data)
        .await
        .unwrap();

    let payload = json!({
        "name": "Test Team with Permission",
        "description": "Test team for permission check"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &user.access_token,
        Some(serde_json::to_string(&payload).unwrap()),
    );

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

/// 権限チェックミドルウェアを使用したチーム更新テスト（正常系 - オーナー）
#[tokio::test]
async fn test_update_team_as_owner_with_permission() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザーを作成
    let signup_data =
        auth_helper::create_test_user_with_info("team_owner@example.com", "team_owner");
    let user = auth_helper::signup_test_user(&app, signup_data)
        .await
        .unwrap();

    // チームを作成
    let create_payload = json!({
        "name": "Original Team Name",
        "description": "Original description"
    });

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &user.access_token,
        Some(serde_json::to_string(&create_payload).unwrap()),
    );

    let create_response = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_result: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let team_id = create_result["data"]["id"].as_str().unwrap();

    // チームを更新
    let update_payload = json!({
        "name": "Updated Team Name",
        "description": "Updated description"
    });

    let update_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/teams/{}", team_id),
        &user.access_token,
        Some(serde_json::to_string(&update_payload).unwrap()),
    );

    let response = app.oneshot(update_req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// 権限チェックミドルウェアを使用したチーム更新テスト（権限なし - 非メンバー）
#[tokio::test]
async fn test_update_team_non_member_forbidden_with_permission() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザーを作成
    let owner_data =
        auth_helper::create_test_user_with_info("team_owner2@example.com", "team_owner2");
    let owner = auth_helper::signup_test_user(&app, owner_data)
        .await
        .unwrap();

    // 別のユーザーを作成
    let other_data =
        auth_helper::create_test_user_with_info("other_user@example.com", "other_user");
    let other_user = auth_helper::signup_test_user(&app, other_data)
        .await
        .unwrap();

    // オーナーがチームを作成
    let create_payload = json!({
        "name": "Private Team",
        "description": "Should not be editable by others"
    });

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&create_payload).unwrap()),
    );

    let create_response = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_result: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let team_id = create_result["data"]["id"].as_str().unwrap();

    // 他のユーザーが更新を試みる
    let update_payload = json!({
        "name": "Should not be updated"
    });

    let update_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/teams/{}", team_id),
        &other_user.access_token,
        Some(serde_json::to_string(&update_payload).unwrap()),
    );

    let response = app.oneshot(update_req).await.unwrap();

    // 権限がない場合は403が返される
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// 権限チェックミドルウェアを使用したチーム削除テスト（正常系 - オーナー）
#[tokio::test]
async fn test_delete_team_as_owner_with_permission() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザーを作成
    let signup_data =
        auth_helper::create_test_user_with_info("team_delete_owner@example.com", "delete_owner");
    let user = auth_helper::signup_test_user(&app, signup_data)
        .await
        .unwrap();

    // チームを作成
    let create_payload = json!({
        "name": "Team to Delete",
        "description": "Will be deleted"
    });

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &user.access_token,
        Some(serde_json::to_string(&create_payload).unwrap()),
    );

    let create_response = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_result: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let team_id = create_result["data"]["id"].as_str().unwrap();

    // チームを削除
    let delete_req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/teams/{}", team_id),
        &user.access_token,
        None,
    );

    let response = app.oneshot(delete_req).await.unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

/// 権限チェックミドルウェアを使用したチーム取得テスト（正常系 - メンバー）
#[tokio::test]
async fn test_get_team_as_member_with_permission() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザーを作成
    let signup_data =
        auth_helper::create_test_user_with_info("team_viewer@example.com", "team_viewer");
    let user = auth_helper::signup_test_user(&app, signup_data)
        .await
        .unwrap();

    // チームを作成
    let create_payload = json!({
        "name": "Viewable Team",
        "description": "Can be viewed by members"
    });

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &user.access_token,
        Some(serde_json::to_string(&create_payload).unwrap()),
    );

    let create_response = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_result: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let team_id = create_result["data"]["id"].as_str().unwrap();

    // チームを取得
    let get_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/teams/{}", team_id),
        &user.access_token,
        None,
    );

    let response = app.oneshot(get_req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// 権限チェックミドルウェアを使用したメンバー招待テスト（権限あり）
#[tokio::test]
async fn test_invite_team_member_with_permission() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // チームオーナーを作成
    let owner_data =
        auth_helper::create_test_user_with_info("invite_owner@example.com", "invite_owner");
    let owner = auth_helper::signup_test_user(&app, owner_data)
        .await
        .unwrap();

    // チームを作成
    let create_payload = json!({
        "name": "Team with Members",
        "description": "Team for member management"
    });

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&create_payload).unwrap()),
    );

    let create_response = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_result: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let team_id = create_result["data"]["id"].as_str().unwrap();

    // 新しいユーザーを作成して招待
    let new_user_data =
        auth_helper::create_test_user_with_info("new_member@example.com", "new_member");
    let new_user = auth_helper::signup_test_user(&app, new_user_data)
        .await
        .unwrap();

    let invite_payload = json!({
        "user_id": new_user.user_id,
        "role": "Member"
    });

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&invite_payload).unwrap()),
    );

    let response = app.oneshot(invite_req).await.unwrap();
    let status = response.status();

    if status != StatusCode::CREATED {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_string = String::from_utf8_lossy(&body);
        println!("Error response status: {:?}", status);
        println!("Error response body: {}", body_string);
    }

    // チームオーナーはメンバーを招待できる（Update権限）
    assert_eq!(status, StatusCode::CREATED);
}
