use serde_json::json;
use std::env;
use task_backend::domain::subscription_tier::SubscriptionTier;

/// Stripeのテストモードが有効かどうかを判定
pub fn is_stripe_test_mode() -> bool {
    // PAYMENT_DEVELOPMENT_MODE=falseかつSTRIPE_SECRET_KEYが設定されている場合
    env::var("PAYMENT_DEVELOPMENT_MODE")
        .map(|v| v.to_lowercase() == "false")
        .unwrap_or(false)
        && env::var("STRIPE_SECRET_KEY").is_ok()
}

/// テスト用のWebhookイベントペイロードを作成
pub fn create_test_webhook_payload(event_type: &str, data: serde_json::Value) -> String {
    json!({
        "id": format!("evt_test_{}", uuid::Uuid::new_v4()),
        "object": "event",
        "api_version": "2024-04-10",
        "created": chrono::Utc::now().timestamp(),
        "data": {
            "object": data
        },
        "livemode": false,
        "pending_webhooks": 1,
        "request": {
            "id": null,
            "idempotency_key": null
        },
        "type": event_type
    })
    .to_string()
}

/// チェックアウトセッション完了イベントのペイロードを作成
pub fn create_checkout_completed_payload(
    session_id: &str,
    customer_id: &str,
    subscription_id: &str,
    tier: &SubscriptionTier,
) -> String {
    let _price_id = match tier {
        SubscriptionTier::Pro => {
            env::var("STRIPE_PRO_PRICE_ID").unwrap_or_else(|_| "price_test_pro".to_string())
        }
        SubscriptionTier::Enterprise => env::var("STRIPE_ENTERPRISE_PRICE_ID")
            .unwrap_or_else(|_| "price_test_enterprise".to_string()),
        _ => "price_test_free".to_string(),
    };

    // 必要最小限のフィールドでCheckoutSessionペイロードを作成
    let timestamp = chrono::Utc::now().timestamp();
    let amount = match tier {
        SubscriptionTier::Pro => 500000,
        SubscriptionTier::Enterprise => 2000000,
        _ => 0,
    };

    let session_data = json!({
        "id": session_id,
        "object": "checkout.session",
        "amount_subtotal": amount,
        "amount_total": amount,
        "automatic_tax": {
            "enabled": false,
            "liability": null,
            "status": null
        },
        "cancel_url": "http://localhost:3001/cancel",
        "created": timestamp,
        "currency": "jpy",
        "custom_fields": [],
        "custom_text": {
            "after_submit": null,
            "shipping_address": null,
            "submit": null,
            "terms_of_service_acceptance": null
        },
        "customer": customer_id,
        "customer_details": null,
        "expires_at": timestamp + 1800,
        "livemode": false,
        "metadata": {
            "user_id": "test_user_id",
            "tier": tier.to_string().to_lowercase()
        },
        "mode": "subscription",
        "payment_method_types": ["card"],
        "payment_status": "paid",
        "phone_number_collection": {
            "enabled": false
        },
        "redirect_on_completion": "always",
        "shipping_options": [],
        "status": "complete",
        "subscription": subscription_id,
        "success_url": "http://localhost:3001/success",
        "total_details": {
            "amount_discount": 0,
            "amount_shipping": 0,
            "amount_tax": 0
        },
        "ui_mode": "hosted"
    });

    create_test_webhook_payload("checkout.session.completed", session_data)
}

/// サブスクリプション削除イベントのペイロードを作成
pub fn create_subscription_deleted_payload(subscription_id: &str, customer_id: &str) -> String {
    let timestamp = chrono::Utc::now().timestamp();

    // Stripeのサブスクリプションオブジェクトの必須フィールドのみ含める
    let subscription_data = json!({
        "id": subscription_id,
        "object": "subscription",
        "automatic_tax": {
            "enabled": false,
            "liability": null
        },
        "billing_cycle_anchor": timestamp - 86400,
        "cancel_at_period_end": false,
        "canceled_at": timestamp,
        "collection_method": "charge_automatically",
        "created": timestamp - 86400,
        "currency": "jpy",
        "current_period_end": timestamp,
        "current_period_start": timestamp - 86400,
        "customer": customer_id,
        "default_tax_rates": [],
        "ended_at": timestamp,
        "items": {
            "object": "list",
            "data": [],
            "has_more": false,
            "total_count": 0,
            "url": format!("/v1/subscriptions/{}/items", subscription_id)
        },
        "livemode": false,
        "metadata": {},
        "start_date": timestamp - 86400,
        "status": "canceled"
    });

    create_test_webhook_payload("customer.subscription.deleted", subscription_data)
}

/// 支払い失敗イベントのペイロードを作成
pub fn create_payment_failed_payload(
    invoice_id: &str,
    customer_id: &str,
    subscription_id: &str,
) -> String {
    let timestamp = chrono::Utc::now().timestamp();
    create_test_webhook_payload(
        "invoice.payment_failed",
        json!({
            "id": invoice_id,
            "object": "invoice",
            "amount_due": 500000,
            "amount_paid": 0,
            "amount_remaining": 500000,
            "attempted": true,
            "automatic_tax": {
                "enabled": false,
                "liability": null
            },
            "billing_reason": "subscription_cycle",
            "collection_method": "charge_automatically",
            "created": timestamp,
            "currency": "jpy",
            "customer": customer_id,
            "customer_email": "test@example.com",
            "customer_name": "Test User",
            "livemode": false,
            "metadata": {},
            "paid": false,
            "status": "open",
            "subscription": subscription_id,
            "total": 500000
        }),
    )
}

/// Webhook署名を生成（テスト用）
pub fn generate_test_webhook_signature(payload: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let timestamp = chrono::Utc::now().timestamp();
    let secret =
        env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_else(|_| "test_webhook_secret".to_string());

    // 署名ペイロードを作成
    let signed_payload = format!("{}.{}", timestamp, payload);

    // HMAC-SHA256で署名
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(signed_payload.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    // Stripe形式の署名ヘッダーを作成
    format!("t={},v1={}", timestamp, signature)
}

/// テスト実行時の条件付きスキップマクロ
#[macro_export]
macro_rules! skip_if_no_stripe_test_mode {
    () => {
        if !$crate::common::stripe_helper::is_stripe_test_mode() {
            eprintln!("Skipping test: Stripe test mode is not enabled");
            return;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_payload_creation() {
        let payload = create_checkout_completed_payload(
            "cs_test_123",
            "cus_test_123",
            "sub_test_123",
            &SubscriptionTier::Pro,
        );

        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["type"], "checkout.session.completed");
        assert_eq!(json["data"]["object"]["customer"], "cus_test_123");
    }

    #[test]
    fn test_webhook_signature_generation() {
        let payload = r#"{"test": "data"}"#;
        let signature = generate_test_webhook_signature(payload);

        assert!(signature.starts_with("t="));
        assert!(signature.contains(",v1="));
    }
}
