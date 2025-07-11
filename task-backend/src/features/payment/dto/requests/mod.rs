use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCheckoutRequest {
    #[validate(custom(function = "validate_tier"))]
    pub tier: String,
}

fn validate_tier(tier: &str) -> Result<(), validator::ValidationError> {
    match tier.to_lowercase().as_str() {
        "pro" | "enterprise" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_tier")),
    }
}

#[derive(Debug, Deserialize)]
pub struct PaymentHistoryQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_page() -> u64 {
    0
}

fn default_per_page() -> u64 {
    10
}
