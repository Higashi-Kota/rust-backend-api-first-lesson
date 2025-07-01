// tests/integration/team/team_crud_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::{app_helper, auth_helper};

fn create_test_team_data(name: &str) -> Value {
    json!({
        "name": name,
        "description": "A test team for integration testing",
        "organization_id": null
    })
}

fn create_test_team_update_data() -> Value {
    json!({
        "name": "Updated Team Name",
        "description": "Updated team description"
    })
}

#[tokio::test]
async fn test_create_team_with_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Test Team {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &user.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert!(response["success"].as_bool().unwrap());
    assert_eq!(response["message"], "Team created successfully");

    let team_data = &response["data"];
    assert_eq!(team_data["name"], team_name);
    assert_eq!(
        team_data["description"],
        "A test team for integration testing"
    );
    assert!(team_data["id"].is_string());
    assert_eq!(team_data["owner_id"], user.id.to_string());
    assert_eq!(team_data["subscription_tier"], "free");
    assert!(team_data["created_at"].is_string());
}

#[tokio::test]
async fn test_create_team_without_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 認証なしでチーム作成試行
    let team_data = create_test_team_data("Unauthorized Team");

    let req = Request::builder()
        .uri("/teams")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&team_data).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_team_details() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Test Team {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &user.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response: Value = serde_json::from_slice(&create_body).unwrap();
    let team_id = create_response["data"]["id"].as_str().unwrap();

    // チーム詳細取得
    let get_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/teams/{}", team_id),
        &user.access_token,
        None,
    );

    let get_res = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);

    let get_body = body::to_bytes(get_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let get_response: Value = serde_json::from_slice(&get_body).unwrap();

    assert!(get_response["success"].as_bool().unwrap());
    assert_eq!(get_response["message"], "Team retrieved successfully");

    let team_data = &get_response["data"];
    assert_eq!(team_data["id"], team_id);
    assert_eq!(team_data["name"], team_name);
    assert_eq!(team_data["owner_id"], user.id.to_string());
}

#[tokio::test]
async fn test_list_teams() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 複数のチームを作成
    for i in 1..=3 {
        let team_name = format!("Test Team {} - {}", i, Uuid::new_v4());
        let team_data = create_test_team_data(&team_name);

        let req = auth_helper::create_authenticated_request(
            "POST",
            "/teams",
            &user.access_token,
            Some(serde_json::to_string(&team_data).unwrap()),
        );

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    // チーム一覧取得
    let list_req =
        auth_helper::create_authenticated_request("GET", "/teams", &user.access_token, None);

    let list_res = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_res.status(), StatusCode::OK);

    let list_body = body::to_bytes(list_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let list_response: Value = serde_json::from_slice(&list_body).unwrap();

    assert!(list_response["success"].as_bool().unwrap());
    assert_eq!(list_response["message"], "Teams retrieved successfully");

    let teams = list_response["data"].as_array().unwrap();
    assert!(teams.len() >= 3); // 少なくとも3つのチームが作成されている
}

#[tokio::test]
async fn test_update_team() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Original Team {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &user.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response: Value = serde_json::from_slice(&create_body).unwrap();
    let team_id = create_response["data"]["id"].as_str().unwrap();

    // チーム更新
    let update_data = create_test_team_update_data();

    let update_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/teams/{}", team_id),
        &user.access_token,
        Some(serde_json::to_string(&update_data).unwrap()),
    );

    let update_res = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(update_res.status(), StatusCode::OK);

    let update_body = body::to_bytes(update_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let update_response: Value = serde_json::from_slice(&update_body).unwrap();

    assert!(update_response["success"].as_bool().unwrap());
    assert_eq!(update_response["message"], "Team updated successfully");

    let updated_team = &update_response["data"];
    assert_eq!(updated_team["name"], "Updated Team Name");
    assert_eq!(updated_team["description"], "Updated team description");
}

#[tokio::test]
async fn test_delete_team() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // チーム作成
    let team_name = format!("Team to Delete {}", Uuid::new_v4());
    let team_data = create_test_team_data(&team_name);

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &user.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_response: Value = serde_json::from_slice(&create_body).unwrap();
    let team_id = create_response["data"]["id"].as_str().unwrap();

    // チーム削除
    let delete_req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/teams/{}", team_id),
        &user.access_token,
        None,
    );

    let delete_res = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(delete_res.status(), StatusCode::NO_CONTENT);

    let delete_body = body::to_bytes(delete_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let delete_response: Value = serde_json::from_slice(&delete_body).unwrap();

    assert!(delete_response["success"].as_bool().unwrap());
    assert_eq!(delete_response["message"], "Team deleted successfully");

    // 削除されたチームにアクセスしようとすると404が返される
    let get_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/teams/{}", team_id),
        &user.access_token,
        None,
    );

    let get_res = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_team_with_invalid_data() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 無効なデータでチーム作成（空の名前）
    let invalid_team_data = json!({
        "name": "",
        "description": "This team has an empty name"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &user.access_token,
        Some(serde_json::to_string(&invalid_team_data).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_team_stats() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // いくつかのチームを作成
    for i in 1..=2 {
        let team_name = format!("Stats Test Team {} - {}", i, Uuid::new_v4());
        let team_data = create_test_team_data(&team_name);

        let req = auth_helper::create_authenticated_request(
            "POST",
            "/teams",
            &user.access_token,
            Some(serde_json::to_string(&team_data).unwrap()),
        );

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    // チーム統計取得
    let stats_req =
        auth_helper::create_authenticated_request("GET", "/teams/stats", &user.access_token, None);

    let stats_res = app.clone().oneshot(stats_req).await.unwrap();
    assert_eq!(stats_res.status(), StatusCode::OK);

    let stats_body = body::to_bytes(stats_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let stats_response: Value = serde_json::from_slice(&stats_body).unwrap();

    assert!(stats_response["success"].as_bool().unwrap());
    assert_eq!(
        stats_response["message"],
        "Team stats retrieved successfully"
    );

    let stats = &stats_response["data"];
    assert!(stats["total_teams"].as_i64().unwrap() >= 2);
    assert!(stats["total_members"].as_i64().unwrap() >= 0);
}
