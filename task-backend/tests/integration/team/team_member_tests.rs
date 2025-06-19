// tests/integration/team/team_member_tests.rs

use axum::{
    body::{self},
    http::StatusCode,
};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::{app_helper, auth_helper};

fn create_test_team_data(name: &str) -> Value {
    json!({
        "name": name,
        "description": "A test team for member testing",
        "organization_id": null
    })
}

fn create_invite_member_data(email: &str) -> Value {
    json!({
        "email": email,
        "role": "Member"
    })
}

fn create_invite_member_data_by_user_id(user_id: &Uuid) -> Value {
    json!({
        "user_id": user_id,
        "role": "Member"
    })
}

fn create_update_role_data(role: &str) -> Value {
    json!({
        "role": role
    })
}

#[tokio::test]
async fn test_invite_team_member_by_email() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザー登録とログイン
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // メンバーユーザー登録（emailを取得するため）
    let member = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Test Team {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response: Value = serde_json::from_slice(&create_body).unwrap();
    let team_id = create_response["data"]["id"].as_str().unwrap();

    // 実際に存在するユーザーのemailでメンバー招待
    let invite_data = create_invite_member_data(&member.email);

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    let invite_res = app.clone().oneshot(invite_req).await.unwrap();
    assert_eq!(invite_res.status(), StatusCode::CREATED);

    let invite_body = body::to_bytes(invite_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let invite_response: Value = serde_json::from_slice(&invite_body).unwrap();

    assert!(invite_response["success"].as_bool().unwrap());
    assert_eq!(
        invite_response["message"],
        "Team member invited successfully"
    );
}

#[tokio::test]
async fn test_invite_team_member_by_user_id() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザー登録とログイン
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // メンバーユーザー登録
    let member = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Test Team {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response: Value = serde_json::from_slice(&create_body).unwrap();
    let team_id = create_response["data"]["id"].as_str().unwrap();

    // ユーザーIDでメンバー招待
    let invite_data = create_invite_member_data_by_user_id(&member.id);

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    let invite_res = app.clone().oneshot(invite_req).await.unwrap();
    assert_eq!(invite_res.status(), StatusCode::CREATED);

    let invite_body = body::to_bytes(invite_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let invite_response: Value = serde_json::from_slice(&invite_body).unwrap();

    assert!(invite_response["success"].as_bool().unwrap());
    assert_eq!(
        invite_response["message"],
        "Team member invited successfully"
    );

    let member_data = &invite_response["data"];
    assert_eq!(member_data["user_id"], member.id.to_string());
    assert_eq!(member_data["role"], "Member");
}

#[tokio::test]
async fn test_update_team_member_role() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザー登録とログイン
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // メンバーユーザー登録
    let member = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Test Team {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response: Value = serde_json::from_slice(&create_body).unwrap();
    let team_id = create_response["data"]["id"].as_str().unwrap();

    // メンバー招待
    let invite_data = create_invite_member_data_by_user_id(&member.id);

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    let invite_res = app.clone().oneshot(invite_req).await.unwrap();
    assert_eq!(invite_res.status(), StatusCode::CREATED);

    let invite_body = body::to_bytes(invite_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let invite_response: Value = serde_json::from_slice(&invite_body).unwrap();
    let member_id = invite_response["data"]["id"].as_str().unwrap();

    // メンバーの役割を管理者に変更
    let update_role_data = create_update_role_data("Admin");

    let update_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/teams/{}/members/{}/role", team_id, member_id),
        &owner.access_token,
        Some(serde_json::to_string(&update_role_data).unwrap()),
    );

    let update_res = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(update_res.status(), StatusCode::OK);

    let update_body = body::to_bytes(update_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let update_response: Value = serde_json::from_slice(&update_body).unwrap();

    assert!(update_response["success"].as_bool().unwrap());
    assert_eq!(
        update_response["message"],
        "Team member role updated successfully"
    );

    let updated_member = &update_response["data"];
    assert_eq!(updated_member["role"], "Admin");
}

#[tokio::test]
async fn test_remove_team_member() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザー登録とログイン
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // メンバーユーザー登録
    let member = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Test Team {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response: Value = serde_json::from_slice(&create_body).unwrap();
    let team_id = create_response["data"]["id"].as_str().unwrap();

    // メンバー招待
    let invite_data = create_invite_member_data_by_user_id(&member.id);

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    let invite_res = app.clone().oneshot(invite_req).await.unwrap();
    assert_eq!(invite_res.status(), StatusCode::CREATED);

    let invite_body = body::to_bytes(invite_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let invite_response: Value = serde_json::from_slice(&invite_body).unwrap();
    let member_id = invite_response["data"]["id"].as_str().unwrap();

    // メンバー削除
    let remove_req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/teams/{}/members/{}", team_id, member_id),
        &owner.access_token,
        None,
    );

    let remove_res = app.clone().oneshot(remove_req).await.unwrap();
    assert_eq!(remove_res.status(), StatusCode::NO_CONTENT);

    let remove_body = body::to_bytes(remove_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let remove_response: Value = serde_json::from_slice(&remove_body).unwrap();

    assert!(remove_response["success"].as_bool().unwrap());
    assert_eq!(
        remove_response["message"],
        "Team member removed successfully"
    );
}

#[tokio::test]
async fn test_invite_member_without_team_permission() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザー登録とログイン
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 非メンバーユーザー登録とログイン
    let non_member = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Test Team {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response: Value = serde_json::from_slice(&create_body).unwrap();
    let team_id = create_response["data"]["id"].as_str().unwrap();

    // 非メンバーがメンバー招待を試行
    let invite_email = format!("member{}@example.com", Uuid::new_v4());
    let invite_data = create_invite_member_data(&invite_email);

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &non_member.access_token, // 非メンバーのトークンを使用
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    let invite_res = app.clone().oneshot(invite_req).await.unwrap();
    assert_eq!(invite_res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_invite_member_with_invalid_data() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザー登録とログイン
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Test Team {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response: Value = serde_json::from_slice(&create_body).unwrap();
    let team_id = create_response["data"]["id"].as_str().unwrap();

    // 無効なメール形式でメンバー招待
    let invalid_invite_data = json!({
        "email": "invalid-email-format",
        "role": "Member"
    });

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&invalid_invite_data).unwrap()),
    );

    let invite_res = app.clone().oneshot(invite_req).await.unwrap();
    assert_eq!(invite_res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_member_role_without_permission() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザー登録とログイン
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // メンバーユーザー登録とログイン
    let member = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Test Team {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response: Value = serde_json::from_slice(&create_body).unwrap();
    let team_id = create_response["data"]["id"].as_str().unwrap();

    // メンバー招待
    let invite_data = create_invite_member_data_by_user_id(&member.id);

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    let invite_res = app.clone().oneshot(invite_req).await.unwrap();
    assert_eq!(invite_res.status(), StatusCode::CREATED);

    let invite_body = body::to_bytes(invite_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let invite_response: Value = serde_json::from_slice(&invite_body).unwrap();
    let member_id = invite_response["data"]["id"].as_str().unwrap();

    // メンバーが自分の役割を管理者に変更しようと試行
    let update_role_data = create_update_role_data("Admin");

    let update_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/teams/{}/members/{}/role", team_id, member_id),
        &member.access_token, // メンバーのトークンを使用
        Some(serde_json::to_string(&update_role_data).unwrap()),
    );

    let update_res = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(update_res.status(), StatusCode::FORBIDDEN);
}
