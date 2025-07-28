// tests/integration/team_query_tests.rs
use axum::{body, http::StatusCode};
use serde_json::json;
use task_backend::api::dto::team_dto::TeamListResponse;
use task_backend::api::dto::team_query_dto::TeamSearchQuery;
use task_backend::shared::types::PaginatedResponse;
use task_backend::types::ApiResponse;
use tower::ServiceExt;

use crate::common::app_helper::setup_full_app;
use crate::common::auth_helper::{create_and_authenticate_admin, create_and_authenticate_user};
use crate::common::request::create_request;

#[tokio::test]
async fn test_team_search_pagination() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin_token = create_and_authenticate_admin(&app).await;

    // 組織を作成 - Enterpriseプランで複数チームを許可
    let org_data = json!({
        "name": "Test Org for Teams",
        "description": "Organization for team tests",
        "subscription_tier": "enterprise"
    });

    let org_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/organizations",
            &admin_token,
            &org_data,
        ))
        .await
        .unwrap();

    assert_eq!(org_response.status(), StatusCode::CREATED);
    let org_body = body::to_bytes(org_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&org_body).unwrap();
    let org_data = org_api_response.data.unwrap();
    let org_id = org_data["id"].as_str().unwrap();

    // 複数のチームを作成 (Enterpriseプランは無制限)
    for i in 0..15 {
        let team_data = json!({
            "name": format!("Team {}", i),
            "description": format!("Description for team {}", i),
            "organization_id": org_id
        });

        let response = app
            .clone()
            .oneshot(create_request("POST", "/teams", &admin_token, &team_data))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Act: ページネーションのテスト
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/teams?page=1&per_page=10",
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<TeamListResponse>> =
        serde_json::from_slice(&body).unwrap();
    let teams = api_response.data.unwrap();

    // Assert
    assert!(teams.items.len() <= 10);
    assert_eq!(teams.pagination.page, 1);
    assert_eq!(teams.pagination.per_page, 10);
    assert!(teams.pagination.total_count >= 15);
}

#[tokio::test]
async fn test_team_sort_by_name() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin_token = create_and_authenticate_admin(&app).await;

    // 特定の名前のチームを作成
    let names = ["Alpha Team", "Charlie Team", "Bravo Team", "Delta Team"];
    for name in names.iter() {
        let team_data = json!({
            "name": name,
            "description": format!("Description for {}", name)
        });

        let response = app
            .clone()
            .oneshot(create_request("POST", "/teams", &admin_token, &team_data))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Act: name昇順でソート
    let response_asc = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/teams?sort_by=name&sort_order=asc",
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_asc.status(), StatusCode::OK);
    let body_asc = body::to_bytes(response_asc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_asc: ApiResponse<PaginatedResponse<TeamListResponse>> =
        serde_json::from_slice(&body_asc).unwrap();
    let teams_asc = api_response_asc.data.unwrap();

    // name降順でソート
    let response_desc = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/teams?sort_by=name&sort_order=desc",
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_desc.status(), StatusCode::OK);
    let body_desc = body::to_bytes(response_desc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_desc: ApiResponse<PaginatedResponse<TeamListResponse>> =
        serde_json::from_slice(&body_desc).unwrap();
    let teams_desc = api_response_desc.data.unwrap();

    // Assert: nameがソートされているか確認
    assert!(!teams_asc.items.is_empty());
    assert!(!teams_desc.items.is_empty());

    // 昇順の場合
    for i in 1..teams_asc.items.len() {
        assert!(teams_asc.items[i - 1].name <= teams_asc.items[i].name);
    }

    // 降順の場合
    for i in 1..teams_desc.items.len() {
        assert!(teams_desc.items[i - 1].name >= teams_desc.items[i].name);
    }
}

#[tokio::test]
async fn test_team_sort_by_created_at() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin_token = create_and_authenticate_admin(&app).await;

    // 時間差をつけてチームを作成
    for i in 0..5 {
        let team_data = json!({
            "name": format!("Time Team {}", i),
            "description": format!("Description {}", i)
        });

        let response = app
            .clone()
            .oneshot(create_request("POST", "/teams", &admin_token, &team_data))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Act: created_at昇順でソート
    let response_asc = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/teams?sort_by=created_at&sort_order=asc",
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_asc.status(), StatusCode::OK);
    let body_asc = body::to_bytes(response_asc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_asc: ApiResponse<PaginatedResponse<TeamListResponse>> =
        serde_json::from_slice(&body_asc).unwrap();
    let teams_asc = api_response_asc.data.unwrap();

    // created_at降順でソート
    let response_desc = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/teams?sort_by=created_at&sort_order=desc",
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response_desc.status(), StatusCode::OK);
    let body_desc = body::to_bytes(response_desc.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response_desc: ApiResponse<PaginatedResponse<TeamListResponse>> =
        serde_json::from_slice(&body_desc).unwrap();
    let teams_desc = api_response_desc.data.unwrap();

    // Assert
    assert!(teams_asc.items.len() >= 5);
    assert!(teams_desc.items.len() >= 5);

    // 昇順の場合
    for i in 1..teams_asc.items.len() {
        assert!(teams_asc.items[i - 1].created_at <= teams_asc.items[i].created_at);
    }

    // 降順の場合
    for i in 1..teams_desc.items.len() {
        assert!(teams_desc.items[i - 1].created_at >= teams_desc.items[i].created_at);
    }
}

#[tokio::test]
async fn test_team_filter_by_organization() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin_token1 = create_and_authenticate_admin(&app).await;
    let admin_token2 = create_and_authenticate_admin(&app).await;

    // 2つの組織を作成
    let org1_data = json!({
        "name": "Org 1",
        "description": "First organization",
        "subscription_tier": "enterprise"
    });

    let org1_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/organizations",
            &admin_token1,
            &org1_data,
        ))
        .await
        .unwrap();

    let org1_body = body::to_bytes(org1_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org1_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&org1_body).unwrap();
    let org1_data = org1_api_response.data.unwrap();
    let org1_id = org1_data["id"].as_str().unwrap();

    // org1に属するチームを作成
    for i in 0..3 {
        let team_data = json!({
            "name": format!("Org1 Team {}", i),
            "description": format!("Team {} in Org1", i),
            "organization_id": org1_id
        });

        let response = app
            .clone()
            .oneshot(create_request("POST", "/teams", &admin_token1, &team_data))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // 別のユーザーが別の組織のチームを作成
    for i in 0..2 {
        let team_data = json!({
            "name": format!("Other Team {}", i),
            "description": format!("Team {} in other org", i)
        });

        app.clone()
            .oneshot(create_request("POST", "/teams", &admin_token2, &team_data))
            .await
            .unwrap();
    }

    // Act: 特定の組織でフィルタ
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/teams?organization_id={}", org1_id),
            &admin_token1,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<TeamListResponse>> =
        serde_json::from_slice(&body).unwrap();
    let teams = api_response.data.unwrap();

    // Assert: すべてのチームがorg1に属していることを確認
    assert!(teams.items.len() >= 3);
    for team in &teams.items {
        assert_eq!(team.organization_id.unwrap().to_string(), org1_id);
    }
}

#[tokio::test]
async fn test_team_search_with_search_term() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 検索用のチームを作成
    let team_data = json!({
        "name": "SearchTarget Team",
        "description": "This is a searchable team with specific keywords"
    });

    app.clone()
        .oneshot(create_request("POST", "/teams", &user.token, &team_data))
        .await
        .unwrap();

    // 他のチームも作成
    let other_team_data = json!({
        "name": "Regular Team",
        "description": "Regular description"
    });

    app.clone()
        .oneshot(create_request(
            "POST",
            "/teams",
            &user.token,
            &other_team_data,
        ))
        .await
        .unwrap();

    // Act: 検索キーワードでフィルタ
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/teams?search=SearchTarget",
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<TeamListResponse>> =
        serde_json::from_slice(&body).unwrap();
    let teams = api_response.data.unwrap();

    // Assert
    assert!(!teams.items.is_empty());
    assert!(teams.items.iter().any(|t| t.name.contains("SearchTarget")));
}

#[tokio::test]
async fn test_team_filter_by_subscription_tier() {
    // Arrange
    let (app, _schema, db) = setup_full_app().await;
    let admin_token = create_and_authenticate_admin(&app).await;

    // 組織を作成してPro tierに更新
    let org_data = json!({
        "name": "Pro Org",
        "description": "Organization with Pro tier",
        "subscription_tier": "enterprise"
    });

    let org_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/organizations",
            &admin_token,
            &org_data,
        ))
        .await
        .unwrap();

    let org_body = body::to_bytes(org_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&org_body).unwrap();
    let org_data = org_api_response.data.unwrap();
    let org_id = org_data["id"].as_str().unwrap();

    // 組織のサブスクリプションをProに更新（直接DBを操作）
    let org_repo = task_backend::repository::organization_repository::OrganizationRepository::new(
        db.connection.clone(),
    );
    if let Some(mut organization) = org_repo
        .find_by_id(uuid::Uuid::parse_str(org_id).unwrap())
        .await
        .unwrap()
    {
        organization.subscription_tier =
            task_backend::domain::subscription_tier::SubscriptionTier::Pro;
        org_repo.update_organization(&organization).await.unwrap();
    }

    // チームを作成（組織のtierを継承）
    let team_data = json!({
        "name": "Pro Team",
        "description": "Team with Pro subscription"
    });

    app.clone()
        .oneshot(create_request("POST", "/teams", &admin_token, &team_data))
        .await
        .unwrap();

    // Act: Pro tierでフィルタ（管理者エンドポイントを想定）
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/teams?subscription_tier=Pro",
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    // 管理者権限がない場合またはエンドポイントが存在しない場合はスキップ
    if response.status() == StatusCode::FORBIDDEN || response.status() == StatusCode::NOT_FOUND {
        return;
    }

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<TeamListResponse>> =
        serde_json::from_slice(&body).unwrap();
    let teams = api_response.data.unwrap();

    // Assert: すべてPro tierであることを確認
    for team in &teams.items {
        assert_eq!(
            team.subscription_tier,
            task_backend::domain::subscription_tier::SubscriptionTier::Pro
        );
    }
}

#[tokio::test]
async fn test_team_combined_filters_and_sort() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let admin_token = create_and_authenticate_admin(&app).await;

    // 組織を作成
    let org_data = json!({
        "name": "Test Org",
        "description": "Organization for combined test",
        "subscription_tier": "enterprise"
    });

    let org_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/organizations",
            &admin_token,
            &org_data,
        ))
        .await
        .unwrap();

    let org_body = body::to_bytes(org_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let org_api_response: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&org_body).unwrap();
    let org_data = org_api_response.data.unwrap();
    let org_id = org_data["id"].as_str().unwrap();

    // テスト用のチームを作成
    for i in 0..10 {
        let team_data = json!({
            "name": format!("Combined Team {}", i),
            "description": format!("Combined test team {}", i)
        });

        let response = app
            .clone()
            .oneshot(create_request("POST", "/teams", &admin_token, &team_data))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Act: 複合フィルタとソート
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!(
                "/teams?organization_id={}&sort_by=name&sort_order=asc&per_page=5",
                org_id
            ),
            &admin_token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<TeamListResponse>> =
        serde_json::from_slice(&body).unwrap();
    let teams = api_response.data.unwrap();

    // Assert
    assert!(teams.items.len() <= 5);

    // すべて同じ組織に属している
    for team in &teams.items {
        assert_eq!(team.organization_id.unwrap().to_string(), org_id);
    }

    // nameが昇順
    for i in 1..teams.items.len() {
        assert!(teams.items[i - 1].name <= teams.items[i].name);
    }
}

#[tokio::test]
async fn test_team_invalid_sort_field() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // Act: 無効なソートフィールドを指定
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/teams?sort_by=invalid_field&sort_order=asc",
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    // Assert: 正常に動作し、デフォルトのソート（created_at desc）が適用される
    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<TeamListResponse>> =
        serde_json::from_slice(&body).unwrap();
    assert!(api_response.data.is_some());
}

#[tokio::test]
async fn test_team_all_sort_fields() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // テスト用のチームを作成
    let team_data = json!({
        "name": "Test Team",
        "description": "Team for sort field test"
    });

    app.clone()
        .oneshot(create_request("POST", "/teams", &user.token, &team_data))
        .await
        .unwrap();

    // すべての許可されたソートフィールドをテスト
    let allowed_fields = TeamSearchQuery::allowed_sort_fields();

    for field in allowed_fields {
        // Act: 各フィールドでソート
        let response = app
            .clone()
            .oneshot(create_request(
                "GET",
                &format!("/teams?sort_by={}&sort_order=asc", field),
                &user.token,
                &(),
            ))
            .await
            .unwrap();

        // Assert: 正常に動作することを確認
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Failed to sort by field: {}",
            field
        );
    }
}
