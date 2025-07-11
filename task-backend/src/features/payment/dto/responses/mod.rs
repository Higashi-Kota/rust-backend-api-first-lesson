use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCheckoutResponse {
    pub checkout_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerPortalResponse {
    pub portal_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentHistoryItem {
    pub id: String,
    pub amount: i32,
    pub currency: String,
    pub status: String,
    pub description: Option<String>,
    pub paid_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentHistoryResponse {
    pub items: Vec<PaymentHistoryItem>,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
