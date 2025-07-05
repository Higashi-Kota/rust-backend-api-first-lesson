use once_cell::sync::Lazy;
use std::env;
use stripe::Client;

pub static STRIPE_CLIENT: Lazy<Client> = Lazy::new(|| {
    let secret_key = env::var("STRIPE_SECRET_KEY").unwrap_or_else(|_| {
        tracing::warn!("STRIPE_SECRET_KEY not set, using empty key for development");
        String::new()
    });

    if secret_key.is_empty() {
        tracing::warn!("Stripe client initialized with empty key - payments will not work");
    }

    Client::new(secret_key)
});

#[derive(Clone, Debug)]
pub struct StripeConfig {
    pub secret_key: String,
    #[allow(dead_code)]
    pub publishable_key: String,
    pub pro_price_id: String,
    pub enterprise_price_id: String,
    pub webhook_secret: String,
    pub development_mode: bool,
}

impl StripeConfig {
    pub fn from_env() -> Self {
        let development_mode = env::var("PAYMENT_DEVELOPMENT_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        if development_mode {
            tracing::info!("Payment development mode enabled - using mock responses");
            return Self {
                secret_key: String::new(),
                publishable_key: String::new(),
                pro_price_id: String::new(),
                enterprise_price_id: String::new(),
                webhook_secret: String::new(),
                development_mode: true,
            };
        }

        // 本番/テストモードの設定
        let secret_key = env::var("STRIPE_SECRET_KEY")
            .expect("STRIPE_SECRET_KEY must be set when not in development mode");

        let publishable_key = env::var("STRIPE_PUBLISHABLE_KEY")
            .expect("STRIPE_PUBLISHABLE_KEY must be set when not in development mode");

        let pro_price_id = env::var("STRIPE_PRO_PRICE_ID")
            .expect("STRIPE_PRO_PRICE_ID must be set when not in development mode");

        let enterprise_price_id = env::var("STRIPE_ENTERPRISE_PRICE_ID")
            .expect("STRIPE_ENTERPRISE_PRICE_ID must be set when not in development mode");

        // 価格IDの形式を検証
        if pro_price_id.starts_with("prod_") {
            tracing::error!(
                "STRIPE_PRO_PRICE_ID is a product ID ({}), but it should be a price ID (starting with 'price_')",
                pro_price_id
            );
            panic!("Invalid STRIPE_PRO_PRICE_ID: Use price ID instead of product ID");
        }

        if enterprise_price_id.starts_with("prod_") {
            tracing::error!(
                "STRIPE_ENTERPRISE_PRICE_ID is a product ID ({}), but it should be a price ID (starting with 'price_')",
                enterprise_price_id
            );
            panic!("Invalid STRIPE_ENTERPRISE_PRICE_ID: Use price ID instead of product ID");
        }

        let webhook_secret = env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_else(|_| {
            tracing::warn!("STRIPE_WEBHOOK_SECRET not set - webhook verification will fail");
            String::new()
        });

        Self {
            secret_key,
            publishable_key,
            pro_price_id,
            enterprise_price_id,
            webhook_secret,
            development_mode: false,
        }
    }

    #[allow(dead_code)]
    pub fn is_test_mode(&self) -> bool {
        self.secret_key.starts_with("sk_test_") || self.development_mode
    }

    pub fn get_price_id(&self, tier: &str) -> Option<&str> {
        match tier.to_lowercase().as_str() {
            "pro" => Some(&self.pro_price_id),
            "enterprise" => Some(&self.enterprise_price_id),
            _ => None,
        }
    }
}
