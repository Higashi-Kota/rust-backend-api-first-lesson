use axum::body::to_bytes;
use common::app_helper::setup_full_app;
use task_backend::features::organization::dto::OrganizationDepartmentsResponse;
use task_backend::shared::types::common::ApiResponse;
use tower::ServiceExt;

mod common {
    pub use crate::common::*;
}

#[tokio::test]
async fn test_organization_departments_success() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = common::auth_helper::create_and_authenticate_user(&app).await;

    // 組織を作成
    let create_response = app
        .clone()
        .oneshot(common::app_helper::create_request(
            "POST",
            "/organizations",
            &user.access_token,
            &serde_json::json!({
                "name": "Test Departments Organization",
                "description": "Organization for departments testing",
                "subscription_tier": "free"
            }),
        ))
        .await
        .expect("Failed to send request");

    assert_eq!(create_response.status(), 201);
    let body_bytes = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let org_body: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&body_bytes).expect("Failed to parse response");
    let organization_id = org_body.data.unwrap()["id"]
        .as_str()
        .unwrap()
        .parse::<uuid::Uuid>()
        .unwrap();

    // 組織部門一覧を取得
    let response = app
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            &format!("/organizations/{}/departments", organization_id),
            &user.access_token,
            None,
        ))
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let body: ApiResponse<OrganizationDepartmentsResponse> =
        serde_json::from_slice(&body_bytes).expect("Failed to parse response");

    assert!(body.data.is_some());
    let departments_response = body.data.unwrap();
    assert_eq!(departments_response.organization_id, organization_id);
    assert_eq!(
        departments_response.organization_name,
        "Test Departments Organization"
    );
    assert_eq!(
        departments_response.message,
        "Organization departments retrieved successfully"
    );

    // 部門情報の検証
    for dept in &departments_response.departments {
        assert!(!dept.id.is_nil());
        assert!(!dept.name.is_empty());
        assert!(dept.hierarchy_level >= 0);
        assert!(!dept.hierarchy_path.is_empty());
        assert!(dept.is_active);
        assert!(dept.created_at <= chrono::Utc::now());
        assert!(dept.updated_at <= chrono::Utc::now());

        // 子部門の検証（再帰的）
        validate_department_children(&dept.children);
    }
}

fn validate_department_children(
    children: &[task_backend::features::organization::dto::DepartmentInfo],
) {
    for child in children {
        assert!(!child.id.is_nil());
        assert!(!child.name.is_empty());
        assert!(child.hierarchy_level > 0);
        assert!(child.parent_department_id.is_some());
        validate_department_children(&child.children);
    }
}

#[tokio::test]
async fn test_organization_departments_forbidden() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = common::auth_helper::create_and_authenticate_user(&app).await;
    let other_user = common::auth_helper::create_and_authenticate_user(&app).await;

    // 組織を作成
    let create_response = app
        .clone()
        .oneshot(common::app_helper::create_request(
            "POST",
            "/organizations",
            &user.access_token,
            &serde_json::json!({
                "name": "Private Organization",
                "description": "Organization for forbidden test",
                "subscription_tier": "free"
            }),
        ))
        .await
        .expect("Failed to send request");

    assert_eq!(create_response.status(), 201);
    let body_bytes = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let org_body: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&body_bytes).expect("Failed to parse response");
    let organization_id = org_body.data.unwrap()["id"]
        .as_str()
        .unwrap()
        .parse::<uuid::Uuid>()
        .unwrap();

    // 別のユーザーでアクセス
    let response = app
        .clone()
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            &format!("/organizations/{}/departments", organization_id),
            &other_user.access_token,
            None,
        ))
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 403);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let body: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("Failed to parse response");

    assert_eq!(body["success"], false);
    let message = body["message"].as_str().unwrap_or("");
    assert!(
        message.contains("member")
            || message.contains("permission")
            || message.contains("forbidden")
            || message.contains("Forbidden")
            || message.contains("access")
            || message.contains("No access to this organization")
    );
}

#[tokio::test]
async fn test_organization_departments_not_found() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = common::auth_helper::create_and_authenticate_user(&app).await;

    let non_existent_id = uuid::Uuid::new_v4();

    // 存在しない組織の部門一覧にアクセス
    let response = app
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            &format!("/organizations/{}/departments", non_existent_id),
            &user.access_token,
            None,
        ))
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let body: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("Failed to parse response");

    assert_eq!(body["success"], false);
    let message = body["message"].as_str().unwrap_or("");
    assert!(message.contains("not found") || message.contains("Organization"));
}
