use axum::body::to_bytes;
use common::app_helper::setup_full_app;
use task_backend::features::security::dto::LoginAttemptsResponse;
use task_backend::shared::types::common::ApiResponse;
use tower::ServiceExt;

mod common {
    pub use crate::common::*;
}

#[tokio::test]
async fn test_login_attempts_success() {
    let (app, _schema, _db) = setup_full_app().await;
    let admin_user = common::auth_helper::authenticate_as_admin(&app).await;

    // ログイン試行を作成（テストデータ）
    let user = common::auth_helper::create_and_authenticate_user(&app).await;

    // いくつかのログイン試行を生成
    for _ in 0..3 {
        let _ = app
            .clone()
            .oneshot(common::app_helper::create_request(
                "POST",
                "/auth/signin",
                "",
                &serde_json::json!({
                    "email": user.email,
                    "password": "wrong_password"
                }),
            ))
            .await;
    }

    // ログイン試行履歴を取得
    let response = app
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            "/admin/security/login-attempts",
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
    let body = match serde_json::from_slice::<LoginAttemptsResponse>(&body_bytes) {
        Ok(dto) => ApiResponse {
            success: true,
            message: "Login attempts history retrieved successfully".to_string(),
            data: Some(dto),
            metadata: None,
        },
        Err(_) => serde_json::from_slice::<ApiResponse<LoginAttemptsResponse>>(&body_bytes)
            .expect("Failed to parse response as either LoginAttemptsResponse or ApiResponse<LoginAttemptsResponse>"),
    };

    assert!(body.data.is_some());
    let attempts_response = body.data.unwrap();
    assert_eq!(
        attempts_response.message,
        "Login attempts history retrieved successfully"
    );

    // サマリーの検証
    assert_eq!(
        attempts_response.summary.total_attempts,
        attempts_response.summary.successful_attempts + attempts_response.summary.failed_attempts
    );
    assert!(attempts_response.summary.time_range_hours == 24);

    // 試行詳細の検証
    for attempt in &attempts_response.attempts {
        assert!(!attempt.ip_address.is_empty());
        assert!(attempt.attempted_at <= chrono::Utc::now());

        if !attempt.success {
            assert!(attempt.failure_reason.is_some());
        } else {
            assert!(attempt.failure_reason.is_none());
        }
    }

    // 不審なIPアドレスの検証
    for suspicious_ip in &attempts_response.suspicious_ips {
        assert!(!suspicious_ip.ip_address.is_empty());
        assert!(suspicious_ip.failed_attempts > 0);
        assert!(suspicious_ip.last_attempt <= chrono::Utc::now());
        assert!(
            suspicious_ip.risk_level == "low"
                || suspicious_ip.risk_level == "medium"
                || suspicious_ip.risk_level == "high"
        );
    }
}

#[tokio::test]
async fn test_login_attempts_forbidden() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = common::auth_helper::create_and_authenticate_user(&app).await;

    // 非管理者ユーザーでアクセス
    let response = app
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            "/admin/security/login-attempts",
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
async fn test_login_attempts_unauthorized() {
    let (app, _schema, _db) = setup_full_app().await;

    // 認証なしでアクセス
    let response = app
        .oneshot(common::app_helper::create_request(
            "GET",
            "/admin/security/login-attempts",
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
