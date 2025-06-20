// tests/integration/team/team_permission_tests.rs

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
        "description": "A test team for permission testing",
        "organization_id": null
    })
}

fn create_invite_member_data_by_user_id(user_id: &Uuid, role: &str) -> Value {
    json!({
        "user_id": user_id,
        "role": role
    })
}

#[tokio::test]
async fn test_team_owner_permissions() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザー登録とログイン
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Owner Test Team {}", Uuid::new_v4());
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

    // オーナーはチーム詳細を取得できる
    let get_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/teams/{}", team_id),
        &owner.access_token,
        None,
    );

    let get_res = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);

    // オーナーはチームを更新できる
    let update_data = json!({
        "name": "Updated Team Name",
        "description": "Updated description"
    });

    let update_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/teams/{}", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&update_data).unwrap()),
    );

    let update_res = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(update_res.status(), StatusCode::OK);

    // オーナーはチームを削除できる
    let delete_req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/teams/{}", team_id),
        &owner.access_token,
        None,
    );

    let delete_res = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(delete_res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_team_admin_permissions() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザー登録とログイン
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 管理者ユーザー登録とログイン
    let admin = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Admin Test Team {}", Uuid::new_v4());
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

    // 管理者として招待
    let invite_data = create_invite_member_data_by_user_id(&admin.id, "Admin");

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    let invite_res = app.clone().oneshot(invite_req).await.unwrap();
    assert_eq!(invite_res.status(), StatusCode::CREATED);

    // 管理者はチーム詳細を取得できる
    let get_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/teams/{}", team_id),
        &admin.access_token,
        None,
    );

    let get_res = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);

    // 管理者はチームを更新できる
    let update_data = json!({
        "name": "Updated by Admin",
        "description": "Updated by admin user"
    });

    let update_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/teams/{}", team_id),
        &admin.access_token,
        Some(serde_json::to_string(&update_data).unwrap()),
    );

    let update_res = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(update_res.status(), StatusCode::OK);

    // 管理者はメンバーを招待できる
    let new_member = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let invite_member_data = create_invite_member_data_by_user_id(&new_member.id, "Member");

    let invite_member_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &admin.access_token,
        Some(serde_json::to_string(&invite_member_data).unwrap()),
    );

    let invite_member_res = app.clone().oneshot(invite_member_req).await.unwrap();
    assert_eq!(invite_member_res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_team_member_permissions() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザー登録とログイン
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // メンバーユーザー登録とログイン
    let member = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Member Test Team {}", Uuid::new_v4());
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

    // 通常メンバーとして招待
    let invite_data = create_invite_member_data_by_user_id(&member.id, "Member");

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    let invite_res = app.clone().oneshot(invite_req).await.unwrap();
    assert_eq!(invite_res.status(), StatusCode::CREATED);

    // メンバーはチーム詳細を取得できる
    let get_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/teams/{}", team_id),
        &member.access_token,
        None,
    );

    let get_res = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);

    // メンバーはチームを更新できない
    let update_data = json!({
        "name": "Unauthorized Update",
        "description": "This should fail"
    });

    let update_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/teams/{}", team_id),
        &member.access_token,
        Some(serde_json::to_string(&update_data).unwrap()),
    );

    let update_res = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(update_res.status(), StatusCode::FORBIDDEN);

    // メンバーは他のメンバーを招待できない
    let new_user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let invite_member_data = create_invite_member_data_by_user_id(&new_user.id, "Member");

    let invite_member_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &member.access_token,
        Some(serde_json::to_string(&invite_member_data).unwrap()),
    );

    let invite_member_res = app.clone().oneshot(invite_member_req).await.unwrap();
    assert_eq!(invite_member_res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_non_member_permissions() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザー登録とログイン
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 非メンバーユーザー登録とログイン
    let non_member = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Non-Member Test Team {}", Uuid::new_v4());
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

    // 非メンバーはチーム詳細を取得できない
    let get_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/teams/{}", team_id),
        &non_member.access_token,
        None,
    );

    let get_res = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::FORBIDDEN);

    // 非メンバーはチームを更新できない
    let update_data = json!({
        "name": "Unauthorized Update",
        "description": "This should fail"
    });

    let update_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/teams/{}", team_id),
        &non_member.access_token,
        Some(serde_json::to_string(&update_data).unwrap()),
    );

    let update_res = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(update_res.status(), StatusCode::FORBIDDEN);

    // 非メンバーはチームを削除できない
    let delete_req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/teams/{}", team_id),
        &non_member.access_token,
        None,
    );

    let delete_res = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(delete_res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_team_visibility_in_list() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 3人のユーザーを作成
    let user1 = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let user2 = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let user3 = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // User1がチーム作成
    let team1_name = format!("User1 Team {}", Uuid::new_v4());
    let team1_data = create_test_team_data(&team1_name);

    let create_req1 = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &user1.access_token,
        Some(serde_json::to_string(&team1_data).unwrap()),
    );

    let create_res1 = app.clone().oneshot(create_req1).await.unwrap();
    assert_eq!(create_res1.status(), StatusCode::CREATED);

    let create_body1 = body::to_bytes(create_res1.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response1: Value = serde_json::from_slice(&create_body1).unwrap();
    let team1_id = create_response1["data"]["id"].as_str().unwrap();

    // User2がチーム作成
    let team2_name = format!("User2 Team {}", Uuid::new_v4());
    let team2_data = create_test_team_data(&team2_name);

    let create_req2 = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &user2.access_token,
        Some(serde_json::to_string(&team2_data).unwrap()),
    );

    let create_res2 = app.clone().oneshot(create_req2).await.unwrap();
    assert_eq!(create_res2.status(), StatusCode::CREATED);

    // User2をUser1のチームに招待
    let invite_data = create_invite_member_data_by_user_id(&user2.id, "Member");

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team1_id),
        &user1.access_token,
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    let invite_res = app.clone().oneshot(invite_req).await.unwrap();
    assert_eq!(invite_res.status(), StatusCode::CREATED);

    // User1は自分のチームと参加しているチームを見ることができる
    let list_req1 =
        auth_helper::create_authenticated_request("GET", "/teams", &user1.access_token, None);

    let list_res1 = app.clone().oneshot(list_req1).await.unwrap();
    assert_eq!(list_res1.status(), StatusCode::OK);

    let list_body1 = body::to_bytes(list_res1.into_body(), usize::MAX)
        .await
        .unwrap();
    let list_response1: Value = serde_json::from_slice(&list_body1).unwrap();
    let teams1 = list_response1["data"].as_array().unwrap();

    // User1は少なくとも自分のチームを見ることができる
    assert!(teams1.iter().any(|team| team["name"] == team1_name));

    // User2は自分のチームとUser1のチーム（メンバーなので）を見ることができる
    let list_req2 =
        auth_helper::create_authenticated_request("GET", "/teams", &user2.access_token, None);

    let list_res2 = app.clone().oneshot(list_req2).await.unwrap();
    assert_eq!(list_res2.status(), StatusCode::OK);

    let list_body2 = body::to_bytes(list_res2.into_body(), usize::MAX)
        .await
        .unwrap();
    let list_response2: Value = serde_json::from_slice(&list_body2).unwrap();
    let teams2 = list_response2["data"].as_array().unwrap();

    // User2は自分のチームとUser1のチームを見ることができる
    assert!(teams2.iter().any(|team| team["name"] == team2_name));
    assert!(teams2.iter().any(|team| team["name"] == team1_name));

    // User3は他人のチームを見ることができない
    let list_req3 =
        auth_helper::create_authenticated_request("GET", "/teams", &user3.access_token, None);

    let list_res3 = app.clone().oneshot(list_req3).await.unwrap();
    assert_eq!(list_res3.status(), StatusCode::OK);

    let list_body3 = body::to_bytes(list_res3.into_body(), usize::MAX)
        .await
        .unwrap();
    let list_response3: Value = serde_json::from_slice(&list_body3).unwrap();
    let teams3 = list_response3["data"].as_array().unwrap();

    // User3は他人のチームを見ることができない
    assert!(!teams3.iter().any(|team| team["name"] == team1_name));
    assert!(!teams3.iter().any(|team| team["name"] == team2_name));
}
