// tests/integration/security/security_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper};

/// Phase 1.2 統合テスト: 新規実装の3エンドポイント

#[tokio::test]
async fn test_revoke_all_tokens_integration() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 既存の管理者でログイン
    let admin_token = auth_helper::create_and_authenticate_admin(&app).await;

    // 全トークン無効化リクエスト
    let revoke_request = json!({
        "user_id": null,
        "reason": "Security maintenance test",
        "exclude_current_user": true
    });

    let request = Request::builder()
        .method("POST")
        .uri("/admin/security/revoke-all-tokens")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", admin_token))
        .body(Body::from(revoke_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(response["success"].as_bool().unwrap());

    let response_data = &response["data"];
    assert!(response_data["message"]
        .as_str()
        .unwrap()
        .contains("successfully"));
    assert_eq!(
        response_data["result"]["revocation_reason"],
        "Security maintenance test"
    );
}

#[tokio::test]
async fn test_session_analytics_integration() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 既存の管理者でログイン
    let admin_token = auth_helper::create_and_authenticate_admin(&app).await;

    let request = Request::builder()
        .method("GET")
        .uri("/admin/security/session-analytics")
        .header("authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(response["success"].as_bool().unwrap());

    let response_data = &response["data"];
    assert!(response_data["message"]
        .as_str()
        .unwrap()
        .contains("successfully"));

    let analytics = &response_data["analytics"];
    assert!(
        analytics["active_sessions"].as_u64().unwrap()
            <= analytics["total_sessions"].as_u64().unwrap()
    );
    assert!(
        analytics["unique_users_today"].as_u64().unwrap()
            <= analytics["unique_users_this_week"].as_u64().unwrap()
    );
}

#[tokio::test]
async fn test_audit_report_integration() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 既存の管理者でログイン
    let admin_token = auth_helper::create_and_authenticate_admin(&app).await;

    let audit_request = json!({
        "report_type": "security",
        "date_range": null,
        "include_details": false
    });

    let request = Request::builder()
        .method("POST")
        .uri("/admin/security/audit-report")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", admin_token))
        .body(Body::from(audit_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(response["success"].as_bool().unwrap());

    let response_data = &response["data"];
    assert!(response_data["message"]
        .as_str()
        .unwrap()
        .contains("successfully"));

    let report = &response_data["report"];
    assert_eq!(report["report_type"], "security");
    assert!(!report["recommendations"].as_array().unwrap().is_empty());
}
