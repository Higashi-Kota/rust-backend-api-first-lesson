use axum::body::to_bytes;
use common::app_helper::setup_full_app;
use task_backend::features::security::dto::{AlertDetails, SecurityAlertsResponse};
use task_backend::shared::types::common::ApiResponse;
use tower::ServiceExt;

mod common {
    pub use crate::common::*;
}

#[tokio::test]
async fn test_security_alerts_success() {
    let (app, _schema, _db) = setup_full_app().await;
    let admin_user = common::auth_helper::authenticate_as_admin(&app).await;

    // セキュリティアラートを取得
    let response = app
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            "/admin/security/alerts",
            &admin_user.access_token,
            None,
        ))
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    // Try to parse as raw DTO first, then as ApiResponse
    let body = match serde_json::from_slice::<SecurityAlertsResponse>(&body_bytes) {
        Ok(dto) => ApiResponse {
            success: true,
            message: "Security alerts retrieved successfully".to_string(),
            data: Some(dto),
            metadata: None,
        },
        Err(_) => serde_json::from_slice::<ApiResponse<SecurityAlertsResponse>>(&body_bytes)
            .expect("Failed to parse response as either SecurityAlertsResponse or ApiResponse<SecurityAlertsResponse>"),
    };

    assert!(body.data.is_some());
    let alerts_response = body.data.unwrap();
    assert_eq!(
        alerts_response.message,
        "Security alerts retrieved successfully"
    );

    // アラートサマリーが含まれていることを確認
    assert_eq!(
        alerts_response.summary.total_alerts as usize,
        alerts_response.alerts.len()
    );

    // アラートの詳細を検証
    for alert in &alerts_response.alerts {
        assert!(!alert.alert_id.is_nil());
        assert!(!alert.alert_type.is_empty());
        assert!(!alert.severity.is_empty());
        assert!(!alert.title.is_empty());
        assert!(!alert.description.is_empty());

        // アラートタイプに応じた詳細を検証
        match &alert.details {
            AlertDetails::SuspiciousIp {
                ip_address,
                failed_attempts,
                last_attempt,
            } => {
                assert!(!ip_address.is_empty());
                assert!(*failed_attempts > 0);
                assert!(*last_attempt <= chrono::Utc::now());
            }
            AlertDetails::FailedLogins {
                today_count,
                week_count,
                threshold_exceeded,
            } => {
                assert!(*today_count <= *week_count);
                if *today_count > 100 || *week_count > 500 {
                    assert!(*threshold_exceeded);
                }
            }
            AlertDetails::TokenAnomaly {
                anomaly_type,
                affected_users: _,
                description,
            } => {
                assert!(!anomaly_type.is_empty());
                // affected_users is u64, always >= 0
                assert!(!description.is_empty());
            }
        }
    }
}

#[tokio::test]
async fn test_security_alerts_forbidden() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = common::auth_helper::create_and_authenticate_user(&app).await;

    // 非管理者ユーザーでアクセス
    let response = app
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            "/admin/security/alerts",
            &user.access_token,
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
        message.contains("Admin access required")
            || message.contains("admin")
            || message.contains("permission")
    );
}

#[tokio::test]
async fn test_security_alerts_unauthorized() {
    let (app, _schema, _db) = setup_full_app().await;

    // 認証なしでアクセス
    let response = app
        .oneshot(common::app_helper::create_request(
            "GET",
            "/admin/security/alerts",
            "",
            &serde_json::Value::Null,
        ))
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let body: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("Failed to parse response");

    assert_eq!(body["success"], false);
    let message = body["message"].as_str().unwrap_or("");
    assert!(
        message.contains("Authentication required")
            || message.contains("authentication")
            || message.contains("unauthorized")
            || message.contains("Unauthorized")
            || message.contains("Invalid or expired token")
    );
}
