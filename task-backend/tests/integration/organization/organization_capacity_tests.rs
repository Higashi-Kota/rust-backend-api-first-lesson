use crate::common::{app_helper, auth_helper};
use axum::{
    body::{self},
    http::StatusCode,
};
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn test_organization_capacity_check() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー作成とログイン
    let signup_data =
        auth_helper::create_test_user_with_info("capacity_test@example.com", "capacity_test");
    let user = auth_helper::signup_test_user(&app, signup_data)
        .await
        .unwrap();

    // 組織を作成（Freeプラン: 最大3チーム、10メンバー）
    let req = auth_helper::create_authenticated_request(
        "POST",
        "/organizations",
        &user.access_token,
        Some(
            json!({
                "name": "Test Capacity Organization",
                "description": "Organization for capacity testing",
                "subscription_tier": "free"
            })
            .to_string(),
        ),
    );
    let create_org_response = app.clone().oneshot(req).await.unwrap();

    let status = create_org_response.status();
    if status != StatusCode::CREATED {
        let body = body::to_bytes(create_org_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Failed to create organization. Status: {}, Body: {}",
            status, body_str
        );
    }

    let body = body::to_bytes(create_org_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org_response: Value = serde_json::from_slice(&body).unwrap();
    let organization_id = org_response["data"]["id"].as_str().unwrap();

    // 容量チェック（初期状態）
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/organizations/{}/capacity", organization_id),
        &user.access_token,
        None,
    );
    let capacity_response = app.clone().oneshot(req).await.unwrap();

    let status = capacity_response.status();
    if status != StatusCode::OK {
        let body = body::to_bytes(capacity_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Failed to check capacity. Status: {}, Body: {}",
            status, body_str
        );
    }
    let body = body::to_bytes(capacity_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let capacity: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(capacity["data"]["can_add_teams"], true);
    assert_eq!(capacity["data"]["can_add_members"], true);
    assert_eq!(capacity["data"]["current_team_count"], 0);
    assert_eq!(capacity["data"]["max_teams"], 3);
    assert_eq!(capacity["data"]["current_member_count"], 1); // Owner
    assert_eq!(capacity["data"]["max_members"], 10);
    assert!(capacity["data"]["utilization_percentage"].as_f64().unwrap() < 20.0);
    // Low utilization
}

#[tokio::test]
async fn test_organization_capacity_with_pro_subscription() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー作成とログイン
    let signup_data =
        auth_helper::create_test_user_with_info("pro_capacity@example.com", "pro_capacity");
    let user = auth_helper::signup_test_user(&app, signup_data)
        .await
        .unwrap();

    // 組織を作成（Proプラン: 最大20チーム、100メンバー）
    let req = auth_helper::create_authenticated_request(
        "POST",
        "/organizations",
        &user.access_token,
        Some(
            json!({
                "name": "Pro Capacity Organization",
                "description": "Organization for pro capacity testing",
                "subscription_tier": "pro"
            })
            .to_string(),
        ),
    );
    let create_org_response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(create_org_response.status(), StatusCode::CREATED);
    let body = body::to_bytes(create_org_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org_response: Value = serde_json::from_slice(&body).unwrap();
    let organization_id = org_response["data"]["id"].as_str().unwrap();

    // 容量チェック
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/organizations/{}/capacity", organization_id),
        &user.access_token,
        None,
    );
    let capacity_response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(capacity_response.status(), StatusCode::OK);
    let body = body::to_bytes(capacity_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let capacity: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(capacity["data"]["max_teams"], 20);
    assert_eq!(capacity["data"]["max_members"], 100);
    assert_eq!(capacity["data"]["subscription_tier"], "pro");
}

#[tokio::test]
async fn test_organization_capacity_access_denied_for_non_member() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // オーナーユーザー作成
    let owner_signup =
        auth_helper::create_test_user_with_info("owner_capacity@example.com", "owner_capacity");
    let owner = auth_helper::signup_test_user(&app, owner_signup)
        .await
        .unwrap();

    // 非メンバーユーザー作成
    let non_member_signup =
        auth_helper::create_test_user_with_info("non_member@example.com", "non_member");
    let non_member = auth_helper::signup_test_user(&app, non_member_signup)
        .await
        .unwrap();

    // 組織を作成
    let req = auth_helper::create_authenticated_request(
        "POST",
        "/organizations",
        &owner.access_token,
        Some(
            json!({
                "name": "Private Organization",
                "description": "Organization for access test",
                "subscription_tier": "free"
            })
            .to_string(),
        ),
    );
    let create_org_response = app.clone().oneshot(req).await.unwrap();

    let status = create_org_response.status();
    if status != StatusCode::CREATED {
        let body = body::to_bytes(create_org_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Failed to create organization for non-member test. Status: {}, Body: {}",
            status, body_str
        );
    }

    let body = body::to_bytes(create_org_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org_response: Value = serde_json::from_slice(&body).unwrap();
    let organization_id = org_response["data"]["id"].as_str().unwrap();

    // 非メンバーが容量チェックを試みる
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/organizations/{}/capacity", organization_id),
        &non_member.access_token,
        None,
    );
    let capacity_response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(capacity_response.status(), StatusCode::FORBIDDEN);
}
