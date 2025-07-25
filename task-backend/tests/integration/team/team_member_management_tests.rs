// tests/integration/team_member_management_test.rs

use axum::{body, http::StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper};

#[tokio::test]
async fn test_team_member_invitation_flow() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザーを作成
    let owner = auth_helper::create_and_authenticate_member(&app).await;

    // チームを作成
    let create_team_data = json!({
        "name": "Test Team",
        "description": "A test team for member management"
    });

    let create_team_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&create_team_data).unwrap()),
    );

    let create_team_res = app.clone().oneshot(create_team_req).await.unwrap();
    assert_eq!(create_team_res.status(), StatusCode::CREATED);

    let body = body::to_bytes(create_team_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();
    let team_id = response["data"]["id"].as_str().unwrap();

    // 新しいメンバーを作成
    let new_member = auth_helper::create_and_authenticate_member(&app).await;

    // メンバーを招待（emailで招待）
    let invite_data = json!({
        "email": new_member.email,
        "role": "Member"
    });

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    let invite_res = app.clone().oneshot(invite_req).await.unwrap();
    let status = invite_res.status();
    let body = body::to_bytes(invite_res.into_body(), usize::MAX)
        .await
        .unwrap();

    if status != StatusCode::CREATED {
        let body_str = String::from_utf8_lossy(&body);
        panic!("Invitation failed with status {}: {}", status, body_str);
    }
    assert_eq!(status, StatusCode::CREATED);

    let response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["data"]["user_id"], new_member.id.to_string());
    assert_eq!(response["data"]["role"], "Member");
}

#[tokio::test]
async fn test_update_team_member_role() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザーを作成
    let owner = auth_helper::create_and_authenticate_member(&app).await;

    // チームを作成
    let create_team_data = json!({
        "name": "Role Update Team"
    });

    let create_team_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&create_team_data).unwrap()),
    );

    let create_team_res = app.clone().oneshot(create_team_req).await.unwrap();
    let body = body::to_bytes(create_team_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();
    let team_id = response["data"]["id"].as_str().unwrap();

    // 新しいメンバーを作成して招待
    let new_member = auth_helper::create_and_authenticate_member(&app).await;

    let invite_data = json!({
        "email": new_member.email,
        "role": "Member"
    });

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    let invite_res = app.clone().oneshot(invite_req).await.unwrap();
    let body = body::to_bytes(invite_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();
    let member_id = response["data"]["id"].as_str().unwrap();

    // メンバーの役割をモデレーターに更新
    let update_role_data = json!({
        "role": "Admin"
    });

    let update_role_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/teams/{}/members/{}/role", team_id, member_id),
        &owner.access_token,
        Some(serde_json::to_string(&update_role_data).unwrap()),
    );

    let update_role_res = app.clone().oneshot(update_role_req).await.unwrap();
    assert_eq!(update_role_res.status(), StatusCode::OK);

    let body = body::to_bytes(update_role_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["data"]["role"], "Admin");
}

#[tokio::test]
async fn test_remove_team_member() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザーを作成
    let owner = auth_helper::create_and_authenticate_member(&app).await;

    // チームを作成
    let create_team_data = json!({
        "name": "Remove Member Team"
    });

    let create_team_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&create_team_data).unwrap()),
    );

    let create_team_res = app.clone().oneshot(create_team_req).await.unwrap();
    let body = body::to_bytes(create_team_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();
    let team_id = response["data"]["id"].as_str().unwrap();

    // 新しいメンバーを作成して招待
    let new_member = auth_helper::create_and_authenticate_member(&app).await;

    let invite_data = json!({
        "email": new_member.email,
        "role": "Member"
    });

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    let invite_res = app.clone().oneshot(invite_req).await.unwrap();
    let body = body::to_bytes(invite_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();
    let member_id = response["data"]["id"].as_str().unwrap();

    // メンバーを削除
    let remove_req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/teams/{}/members/{}", team_id, member_id),
        &owner.access_token,
        None,
    );

    let remove_res = app.clone().oneshot(remove_req).await.unwrap();
    assert_eq!(remove_res.status(), StatusCode::NO_CONTENT);

    // チーム詳細を確認してメンバーが削除されたことを確認
    let get_team_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/teams/{}", team_id),
        &owner.access_token,
        None,
    );

    let get_team_res = app.clone().oneshot(get_team_req).await.unwrap();
    let body = body::to_bytes(get_team_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // メンバー数が1（オーナーのみ）になっていることを確認
    assert_eq!(response["data"]["current_member_count"], 1);
}

#[tokio::test]
async fn test_member_permissions() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザーを作成
    let owner = auth_helper::create_and_authenticate_member(&app).await;

    // チームを作成
    let create_team_data = json!({
        "name": "Permission Test Team"
    });

    let create_team_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&create_team_data).unwrap()),
    );

    let create_team_res = app.clone().oneshot(create_team_req).await.unwrap();
    let body = body::to_bytes(create_team_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();
    let team_id = response["data"]["id"].as_str().unwrap();

    // 2人のメンバーを作成
    let member1 = auth_helper::create_and_authenticate_member(&app).await;
    let member2 = auth_helper::create_and_authenticate_member(&app).await;

    // member1を招待
    let invite_data = json!({
        "email": member1.email,
        "role": "Member"
    });

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    app.clone().oneshot(invite_req).await.unwrap();

    // member1がmember2を招待しようとする（失敗するはず）
    let invite_data2 = json!({
        "email": member2.email,
        "role": "Member"
    });

    let invite_req2 = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/members", team_id),
        &member1.access_token,
        Some(serde_json::to_string(&invite_data2).unwrap()),
    );

    let invite_res2 = app.clone().oneshot(invite_req2).await.unwrap();
    assert_eq!(invite_res2.status(), StatusCode::FORBIDDEN);
}
