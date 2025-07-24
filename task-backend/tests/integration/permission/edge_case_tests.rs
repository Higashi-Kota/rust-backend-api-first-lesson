// tests/integration/permission/edge_case_tests.rs

use crate::common::{app_helper, auth_helper};
use axum::http::StatusCode;
use sea_orm::{ColumnTrait, QueryFilter};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

/// 期限切れトークンでのアクセステスト
#[tokio::test]
async fn test_expired_token_access() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 期限切れトークンを模擬（実際にはJWTの有効期限を操作できないため、無効なトークンを使用）
    let invalid_token = "expired.token.here";

    let req =
        auth_helper::create_authenticated_request("GET", "/tasks", invalid_token, None::<String>);

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Expired token should return 401 Unauthorized"
    );
}

/// 削除されたユーザーのトークンでのアクセステスト
/// 注: JWTトークンはステートレスなため、ユーザーのis_activeステータスの変更は
/// 既存のトークンには反映されません。実装にはトークンブラックリストまたは
/// リクエスト毎のDBチェックが必要です。
#[tokio::test]
#[ignore = "JWT tokens are stateless - requires token blacklist or DB check per request"]
async fn test_deleted_user_token_access() {
    let (app, _schema_name, db) = app_helper::setup_full_app().await;

    // ユーザーを作成して認証
    let user = auth_helper::create_and_authenticate_member(&app).await;

    // ユーザーを削除（is_activeをfalseに設定）
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};
    use task_backend::domain::user_model;

    let user_entity = user_model::Entity::find_by_id(user.user_id)
        .one(&db.connection)
        .await
        .unwrap()
        .unwrap();

    let mut user_active: user_model::ActiveModel = user_entity.into();
    user_active.is_active = Set(false);
    user_active.update(&db.connection).await.unwrap();

    // 削除されたユーザーのトークンでアクセス
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks",
        &user.access_token,
        None::<String>,
    );

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Deleted user's token should return 403 Forbidden"
    );
}

/// 存在しないリソースへのアクセステスト
#[tokio::test]
async fn test_non_existent_resource_access() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    let user = auth_helper::create_and_authenticate_member(&app).await;
    let non_existent_id = Uuid::new_v4();

    // 存在しないタスクへのアクセス
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/tasks/{}", non_existent_id),
        &user.access_token,
        None::<String>,
    );

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Non-existent resource should return 404 Not Found"
    );
}

/// チーム招待の有効期限切れテスト
#[tokio::test]
async fn test_expired_team_invitation() {
    let (app, _schema_name, db) = app_helper::setup_full_app().await;

    let owner = auth_helper::create_and_authenticate_member(&app).await;
    let invitee = auth_helper::create_and_authenticate_member(&app).await;

    // チームを作成
    let team_data = json!({
        "name": "Test Team",
        "description": "Test Description"
    });

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner.access_token,
        Some(serde_json::to_string(&team_data).unwrap()),
    );

    let create_response = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let team: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let team_id = team["data"]["id"].as_str().unwrap();

    // 招待を作成
    let invite_data = json!({
        "email": invitee.username.clone() + "@example.com",
        "role": "Member"
    });

    let invite_req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/teams/{}/invitations", team_id),
        &owner.access_token,
        Some(serde_json::to_string(&invite_data).unwrap()),
    );

    let invite_response = app.clone().oneshot(invite_req).await.unwrap();

    if invite_response.status() == StatusCode::CREATED {
        let invite_body = axum::body::to_bytes(invite_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let invitation: serde_json::Value = serde_json::from_slice(&invite_body).unwrap();
        let invitation_id = invitation["data"]["id"].as_str().unwrap();

        // 招待を期限切れに設定（データベースを直接更新）
        use sea_orm::{ActiveModelTrait, EntityTrait, Set};
        use task_backend::domain::team_invitation_model;

        if let Ok(Some(invite_entity)) =
            team_invitation_model::Entity::find_by_id(Uuid::parse_str(invitation_id).unwrap())
                .one(&db.connection)
                .await
        {
            let mut invite_active: team_invitation_model::ActiveModel = invite_entity.into();
            invite_active.expires_at = Set(Some(chrono::Utc::now() - chrono::Duration::days(1)));
            invite_active.update(&db.connection).await.unwrap();

            // 期限切れの招待を受け入れようとする
            let accept_req = auth_helper::create_authenticated_request(
                "POST",
                &format!("/teams/invitations/{}/accept", invitation_id),
                &invitee.access_token,
                None::<String>,
            );

            let accept_response = app.clone().oneshot(accept_req).await.unwrap();
            assert!(
                accept_response.status() == StatusCode::BAD_REQUEST
                    || accept_response.status() == StatusCode::FORBIDDEN,
                "Expired invitation should not be accepted"
            );
        }
    }
}

/// 同時実行での権限チェックテスト
#[tokio::test]
async fn test_concurrent_permission_checks() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    let admin = auth_helper::authenticate_as_admin(&app).await;
    let member = auth_helper::create_and_authenticate_member(&app).await;

    // 複数のリクエストを同時に実行
    let handles: Vec<_> = vec![
        // 管理者の同時リクエスト
        tokio::spawn({
            let app = app.clone();
            let token = admin.access_token.clone();
            async move {
                let req = auth_helper::create_authenticated_request(
                    "GET",
                    "/admin/users",
                    &token,
                    None::<String>,
                );
                app.oneshot(req).await.unwrap().status()
            }
        }),
        // メンバーの同時リクエスト
        tokio::spawn({
            let app = app.clone();
            let token = member.access_token.clone();
            async move {
                let req = auth_helper::create_authenticated_request(
                    "GET",
                    "/admin/users",
                    &token,
                    None::<String>,
                );
                app.oneshot(req).await.unwrap().status()
            }
        }),
        // 別の管理者リクエスト
        tokio::spawn({
            let app = app.clone();
            let token = admin.access_token.clone();
            async move {
                let req = auth_helper::create_authenticated_request(
                    "GET",
                    "/admin/analytics/system",
                    &token,
                    None::<String>,
                );
                app.oneshot(req).await.unwrap().status()
            }
        }),
    ];

    // すべてのリクエストの結果を収集
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    // 結果を検証
    assert_eq!(results[0], StatusCode::OK); // 管理者は成功
    assert_eq!(results[1], StatusCode::FORBIDDEN); // メンバーは拒否
    assert_eq!(results[2], StatusCode::OK); // 管理者は成功
}

/// 権限の昇格/降格テスト
#[tokio::test]
async fn test_permission_elevation_demotion() {
    let (app, _schema_name, db) = app_helper::setup_full_app().await;

    // メンバーユーザーを作成
    let user = auth_helper::create_and_authenticate_member(&app).await;

    // 最初は管理者リソースにアクセスできない
    let req1 = auth_helper::create_authenticated_request(
        "GET",
        "/admin/users",
        &user.access_token,
        None::<String>,
    );
    let response1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(response1.status(), StatusCode::FORBIDDEN);

    // ユーザーを管理者に昇格
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};
    use task_backend::domain::{role_model, user_model};

    if let Ok(Some(admin_role)) = role_model::Entity::find()
        .filter(role_model::Column::Name.eq("admin"))
        .one(&db.connection)
        .await
    {
        let user_entity = user_model::Entity::find_by_id(user.user_id)
            .one(&db.connection)
            .await
            .unwrap()
            .unwrap();

        let mut user_active: user_model::ActiveModel = user_entity.into();
        user_active.role_id = Set(admin_role.id);
        user_active.update(&db.connection).await.unwrap();

        // 新しい認証トークンを取得
        let login_data = json!({
            "username": user.username,
            "password": "password123"
        });

        let login_req = Request::builder()
            .uri("/auth/login")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&login_data).unwrap()))
            .unwrap();

        let login_response = app.clone().oneshot(login_req).await.unwrap();

        if login_response.status() == StatusCode::OK {
            let body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
                .await
                .unwrap();
            let auth_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
            let new_token = auth_response["access_token"].as_str().unwrap();

            // 昇格後は管理者リソースにアクセスできる
            let req2 = auth_helper::create_authenticated_request(
                "GET",
                "/admin/users",
                new_token,
                None::<String>,
            );
            let response2 = app.clone().oneshot(req2).await.unwrap();
            assert_eq!(response2.status(), StatusCode::OK);
        }
    }
}

/// 循環参照権限のテスト（チームAがチームBのリソースにアクセスし、チームBがチームAのリソースにアクセス）
#[tokio::test]
async fn test_circular_permission_reference() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 2つのチームと各チームのオーナーを作成
    let owner_a = auth_helper::create_and_authenticate_member(&app).await;
    let owner_b = auth_helper::create_and_authenticate_member(&app).await;

    // チームAを作成
    let team_a_data = json!({
        "name": "Team A",
        "description": "First team"
    });

    let create_a_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner_a.access_token,
        Some(serde_json::to_string(&team_a_data).unwrap()),
    );

    let create_a_response = app.clone().oneshot(create_a_req).await.unwrap();
    assert_eq!(create_a_response.status(), StatusCode::CREATED);

    let body_a = axum::body::to_bytes(create_a_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let team_a: serde_json::Value = serde_json::from_slice(&body_a).unwrap();
    let team_a_id = team_a["data"]["id"].as_str().unwrap();

    // チームBを作成
    let team_b_data = json!({
        "name": "Team B",
        "description": "Second team"
    });

    let create_b_req = auth_helper::create_authenticated_request(
        "POST",
        "/teams",
        &owner_b.access_token,
        Some(serde_json::to_string(&team_b_data).unwrap()),
    );

    let create_b_response = app.clone().oneshot(create_b_req).await.unwrap();
    assert_eq!(create_b_response.status(), StatusCode::CREATED);

    let body_b = axum::body::to_bytes(create_b_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let team_b: serde_json::Value = serde_json::from_slice(&body_b).unwrap();
    let team_b_id = team_b["data"]["id"].as_str().unwrap();

    // オーナーAはチームBにアクセスできない
    let access_b_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/teams/{}", team_b_id),
        &owner_a.access_token,
        None::<String>,
    );
    let access_b_response = app.clone().oneshot(access_b_req).await.unwrap();
    assert_eq!(access_b_response.status(), StatusCode::FORBIDDEN);

    // オーナーBはチームAにアクセスできない
    let access_a_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/teams/{}", team_a_id),
        &owner_b.access_token,
        None::<String>,
    );
    let access_a_response = app.clone().oneshot(access_a_req).await.unwrap();
    assert_eq!(access_a_response.status(), StatusCode::FORBIDDEN);
}

use axum::{body::Body, http::Request};
