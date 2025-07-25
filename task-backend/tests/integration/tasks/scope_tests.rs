use crate::common::{
    app_helper::{
        add_team_member, create_request, create_team_task, create_test_task, create_test_team,
        create_user, setup_full_app,
    },
    auth_helper::create_and_authenticate_user,
};
use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_task_list_with_personal_scope() {
    let (app, _schema, _db) = setup_full_app().await;

    // ユーザーを作成して認証
    let user = create_and_authenticate_user(&app).await;
    let token = &user.token;

    // 個人タスクを作成
    let personal_task = create_test_task(&app, token).await;

    // チームを作成してチームタスクを作成
    let team = create_test_team(&app, token).await;
    let _team_task = create_team_task(&app, token, team.id).await;

    // 個人スコープでタスクを取得
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/tasks/search?visibility=personal",
            token,
            &json!({}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // レスポンス構造を確認
    let tasks = if let Some(tasks_array) = json["data"]["tasks"].as_array() {
        tasks_array
    } else if let Some(items_array) = json["data"]["items"].as_array() {
        items_array
    } else if let Some(data_array) = json["data"].as_array() {
        data_array
    } else {
        // デバッグ用にレスポンス全体を出力
        eprintln!("Unexpected response structure: {}", json);
        panic!("Cannot find tasks array in response");
    };

    // 個人タスクのみが返されることを確認
    assert_eq!(
        tasks.len(),
        1,
        "Expected 1 personal task, got {}",
        tasks.len()
    );
    assert_eq!(tasks[0]["id"], personal_task.id.to_string());
}

#[tokio::test]
async fn test_task_list_with_team_scope() {
    let (app, _schema, _db) = setup_full_app().await;

    // チームオーナーとメンバーを作成
    let owner = create_and_authenticate_user(&app).await;
    let member = create_user(&app, "member@example.com").await;

    // チームを作成
    let team = create_test_team(&app, &owner.token).await;

    // メンバーを追加
    add_team_member(&app, &owner.token, team.id, member.id).await;

    // チームタスクを2つ作成
    let team_task1 = create_team_task(&app, &owner.token, team.id).await;
    let team_task2 = create_team_task(&app, &owner.token, team.id).await;

    // 個人タスクも作成
    let _personal_task = create_test_task(&app, &owner.token).await;

    // チームスコープでタスクを取得
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/tasks/search?visibility=team&team_id={}", team.id),
            &owner.token,
            &json!({}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // レスポンス構造を確認
    let tasks = if let Some(tasks_array) = json["data"]["tasks"].as_array() {
        tasks_array
    } else if let Some(items_array) = json["data"]["items"].as_array() {
        items_array
    } else if let Some(data_array) = json["data"].as_array() {
        data_array
    } else {
        eprintln!("Unexpected response structure: {}", json);
        panic!("Cannot find tasks array in response");
    };

    // チームタスクのみが返されることを確認
    assert_eq!(tasks.len(), 2, "Expected 2 team tasks, got {}", tasks.len());
    let task_ids: Vec<String> = tasks
        .iter()
        .map(|t| t["id"].as_str().unwrap().to_string())
        .collect();
    assert!(task_ids.contains(&team_task1.id.to_string()));
    assert!(task_ids.contains(&team_task2.id.to_string()));
}

#[tokio::test]
async fn test_task_list_with_include_assigned() {
    let (app, _schema, _db) = setup_full_app().await;

    // ユーザーを2人作成
    let user1 = create_and_authenticate_user(&app).await;
    let user2 = create_user(&app, "user2@example.com").await;

    // チームを作成してuser2を追加
    let team = create_test_team(&app, &user1.token).await;
    add_team_member(&app, &user1.token, team.id, user2.id).await;

    // user1の個人タスク
    let _personal_task = create_test_task(&app, &user1.token).await;

    // user2に割り当てられたチームタスク
    let assigned_task = crate::common::app_helper::create_team_task_assigned_to(
        &app,
        &user1.token,
        team.id,
        user2.id,
    )
    .await;

    // user2として、include_assigned=trueでタスクを取得
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/tasks/search?visibility=personal&include_assigned=true",
            &user2.token,
            &json!({}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // レスポンス構造を確認
    let tasks = if let Some(tasks_array) = json["data"]["tasks"].as_array() {
        tasks_array
    } else if let Some(items_array) = json["data"]["items"].as_array() {
        items_array
    } else if let Some(data_array) = json["data"].as_array() {
        data_array
    } else {
        eprintln!("Unexpected response structure: {}", json);
        panic!("Cannot find tasks array in response");
    };

    // デバッグ：レスポンスを確認
    eprintln!("User2 tasks response: {}", json);
    eprintln!(
        "Assigned task: id={}, assigned_to={}",
        assigned_task.id,
        assigned_task.assigned_to.unwrap_or_default()
    );

    // 割り当てられたタスクが含まれることを確認
    assert_eq!(
        tasks.len(),
        1,
        "Expected 1 assigned task, got {}. Response: {:?}",
        tasks.len(),
        tasks
    );
    assert_eq!(tasks[0]["id"], assigned_task.id.to_string());
}

#[tokio::test]
async fn test_audit_log_for_team_role_change() {
    let (app, _schema, db) = setup_full_app().await;

    // チームオーナーとメンバーを作成
    let owner = create_and_authenticate_user(&app).await;
    let member = create_user(&app, "member@example.com").await;

    // チームを作成してメンバーを追加
    let team = create_test_team(&app, &owner.token).await;
    add_team_member(&app, &owner.token, team.id, member.id).await;

    // チーム詳細を取得してメンバー情報を取得
    let team_response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/teams/{}", team.id),
            &owner.token,
            &json!({}),
        ))
        .await
        .unwrap();

    assert_eq!(team_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(team_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let members = json["data"]["members"].as_array().unwrap();
    let member_info = members
        .iter()
        .find(|m| m["user_id"] == member.id.to_string())
        .unwrap();
    let member_id = member_info["id"].as_str().unwrap();

    // ロールを変更
    let response = app
        .clone()
        .oneshot(create_request(
            "PATCH",
            &format!("/teams/{}/members/{}/role", team.id, member_id),
            &owner.token,
            &json!({ "role": "Admin" }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 監査ログを確認
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use task_backend::domain::audit_log_model;

    let audit_logs = audit_log_model::Entity::find()
        .filter(audit_log_model::Column::Action.eq("team_role_changed"))
        .filter(audit_log_model::Column::TeamId.eq(team.id))
        .all(&db.connection)
        .await
        .unwrap();

    assert_eq!(audit_logs.len(), 1);
    let log = &audit_logs[0];
    assert_eq!(log.user_id, owner.id);
    assert_eq!(log.resource_type, "team_member");

    let details = log.details.as_ref().unwrap();
    assert_eq!(details["old_role"], "member");
    assert_eq!(details["new_role"], "admin");
}
