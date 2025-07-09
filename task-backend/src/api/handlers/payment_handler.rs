use crate::api::dto::subscription_dto::{CurrentSubscriptionResponse, SubscriptionTierInfo};
use crate::api::{dto::common::ApiResponse, AppState};
use crate::core::subscription_tier::SubscriptionTier;
use crate::error::{AppError, AppResult};
use crate::features::auth::middleware::AuthenticatedUser;
use axum::{
    extract::{Json, Query, State},
    http::HeaderMap,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCheckoutResponse {
    pub checkout_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerPortalResponse {
    pub portal_url: String,
}

#[derive(Debug, Deserialize)]
pub struct PaymentHistoryQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_page() -> u64 {
    1
}

fn default_per_page() -> u64 {
    10
}

#[derive(Debug, Serialize)]
pub struct PaymentHistoryItem {
    pub id: String,
    pub amount: i32,
    pub currency: String,
    pub status: String,
    pub description: Option<String>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaymentHistoryResponse {
    pub items: Vec<PaymentHistoryItem>,
    pub total_pages: u64,
    pub current_page: u64,
    pub per_page: u64,
}

/// チェックアウトセッション作成
pub async fn create_checkout_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateCheckoutRequest>,
) -> AppResult<Json<ApiResponse<CreateCheckoutResponse>>> {
    // バリデーション
    Validate::validate(&payload).map_err(|validation_errors| {
        let errors: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();
        AppError::ValidationErrors(errors)
    })?;

    let tier = SubscriptionTier::from_str(&payload.tier)
        .ok_or_else(|| AppError::BadRequest("Invalid tier".to_string()))?;

    // 現在のサブスクリプション階層を確認
    let current_user = app_state
        .user_service
        .get_user_profile(user.claims.user_id)
        .await?;

    let current_tier = SubscriptionTier::from_str(&current_user.subscription_tier)
        .ok_or_else(|| AppError::BadRequest("Invalid current subscription tier".to_string()))?;

    // 同じか下位のプランへの変更は拒否
    if tier.level() <= current_tier.level() {
        return Err(AppError::BadRequest(
            "Cannot checkout for the same or lower tier".to_string(),
        ));
    }

    info!(
        user_id = %user.claims.user_id,
        current_tier = %current_tier.as_str(),
        target_tier = %tier.as_str(),
        "Creating checkout session"
    );

    // チェックアウトセッションを作成
    let checkout_url = app_state
        .payment_service
        .create_checkout_session(user.claims.user_id, tier)
        .await?;

    info!(
        user_id = %user.claims.user_id,
        "Checkout session created successfully"
    );

    Ok(Json(ApiResponse::success(
        "Checkout session created successfully",
        CreateCheckoutResponse { checkout_url },
    )))
}

/// カスタマーポータルセッション作成
pub async fn create_customer_portal_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<CustomerPortalResponse>>> {
    info!(
        user_id = %user.claims.user_id,
        "Creating customer portal session"
    );

    // カスタマーポータルURLを生成
    let portal_url = app_state
        .payment_service
        .create_customer_portal_url(user.claims.user_id)
        .await?;

    info!(
        user_id = %user.claims.user_id,
        "Customer portal session created successfully"
    );

    Ok(Json(ApiResponse::success(
        "Customer portal session created successfully",
        CustomerPortalResponse { portal_url },
    )))
}

/// Stripe Webhookハンドラー
pub async fn stripe_webhook_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> AppResult<()> {
    // 開発モードの場合は署名チェックをスキップ
    let development_mode = std::env::var("STRIPE_DEVELOPMENT_MODE")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";

    let stripe_signature = if development_mode {
        // 開発モードではダミーの署名を使用
        "development_mode_signature"
    } else {
        headers
            .get("stripe-signature")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                tracing::warn!("Missing stripe-signature header");
                AppError::BadRequest("Missing stripe-signature header".to_string())
            })?
    };

    app_state
        .payment_service
        .handle_webhook(&body, stripe_signature)
        .await?;

    Ok(())
}

/// 現在のサブスクリプション情報を取得
pub async fn get_current_subscription_payment_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<CurrentSubscriptionResponse>>> {
    info!(
        user_id = %user.claims.user_id,
        "Getting current subscription"
    );

    let user_profile = app_state
        .user_service
        .get_user_profile(user.claims.user_id)
        .await?;

    let subscription_info = CurrentSubscriptionResponse::new(
        user.claims.user_id,
        user_profile.subscription_tier,
        user_profile.created_at,
    );

    Ok(Json(ApiResponse::success(
        "Current subscription retrieved successfully",
        subscription_info,
    )))
}

/// 利用可能なサブスクリプションティアを取得
pub async fn get_subscription_tiers_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<Vec<SubscriptionTierInfo>>>> {
    info!(
        user_id = %user.claims.user_id,
        "Getting available subscription tiers"
    );

    let tiers = SubscriptionTier::all();
    let tier_infos: Vec<SubscriptionTierInfo> = tiers
        .into_iter()
        .map(|tier| CurrentSubscriptionResponse::get_tier_info(tier.as_str()))
        .collect();

    Ok(Json(ApiResponse::success(
        "Available subscription tiers retrieved successfully",
        tier_infos,
    )))
}

/// アップグレード可能なオプションを取得
pub async fn get_upgrade_options_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<Vec<SubscriptionTierInfo>>>> {
    info!(
        user_id = %user.claims.user_id,
        "Getting upgrade options"
    );

    // 現在のサブスクリプション情報を取得
    let current_user = app_state
        .user_service
        .get_user_profile(user.claims.user_id)
        .await?;

    let current_tier = SubscriptionTier::from_str(&current_user.subscription_tier)
        .ok_or_else(|| AppError::BadRequest("Invalid current subscription tier".to_string()))?;

    // アップグレード可能なティアのみをフィルタリング
    let all_tiers = SubscriptionTier::all();
    let upgrade_options: Vec<SubscriptionTierInfo> = all_tiers
        .into_iter()
        .filter(|tier| tier.level() > current_tier.level())
        .map(|tier| CurrentSubscriptionResponse::get_tier_info(tier.as_str()))
        .collect();

    Ok(Json(ApiResponse::success(
        "Upgrade options retrieved successfully",
        upgrade_options,
    )))
}

/// 支払い履歴を取得
pub async fn get_payment_history_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<PaymentHistoryQuery>,
) -> AppResult<Json<ApiResponse<PaymentHistoryResponse>>> {
    info!(
        user_id = %user.claims.user_id,
        page = %query.page,
        per_page = %query.per_page,
        "Getting payment history"
    );

    // ページネーションの検証
    if query.page == 0 {
        return Err(AppError::BadRequest(
            "Page number must be 1 or greater".to_string(),
        ));
    }
    if query.per_page == 0 || query.per_page > 100 {
        return Err(AppError::BadRequest(
            "Per page must be between 1 and 100".to_string(),
        ));
    }

    // 支払い履歴を取得（0-indexed）
    let (history_items, total_pages) = app_state
        .payment_service
        .get_payment_history(user.claims.user_id, query.page - 1, query.per_page)
        .await?;

    // DTOに変換
    let items: Vec<PaymentHistoryItem> = history_items
        .into_iter()
        .map(|item| PaymentHistoryItem {
            id: item.id.to_string(),
            amount: item.amount,
            currency: item.currency,
            status: item.status,
            description: item.description,
            paid_at: item.paid_at,
            created_at: item.created_at,
        })
        .collect();

    let response = PaymentHistoryResponse {
        items,
        total_pages,
        current_page: query.page,
        per_page: query.per_page,
    };

    Ok(Json(ApiResponse::success(
        "Payment history retrieved successfully",
        response,
    )))
}

/// 決済関連のルーター
pub fn payment_router(app_state: AppState) -> Router {
    Router::new()
        // 認証が必要なエンドポイント
        .route("/payments/checkout", post(create_checkout_handler))
        .route("/payments/portal", post(create_customer_portal_handler))
        .route("/payments/history", get(get_payment_history_handler))
        // サブスクリプション情報関連のエンドポイント
        .route(
            "/payments/subscription",
            get(get_current_subscription_payment_handler),
        )
        .route(
            "/payments/subscription/tiers",
            get(get_subscription_tiers_handler),
        )
        .route(
            "/payments/subscription/upgrade-options",
            get(get_upgrade_options_handler),
        )
        // Stripe設定エンドポイント（管理者専用）
        .route("/admin/payments/config", get(get_stripe_config_handler))
        // Webhookエンドポイント（認証不要）
        .route("/webhooks/stripe", post(stripe_webhook_handler))
        .with_state(app_state)
}

/// 決済ルーターをAppStateから作成
pub fn payment_router_with_state(app_state: AppState) -> Router {
    payment_router(app_state)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StripeConfigResponse {
    pub publishable_key: String,
    pub is_test_mode: bool,
}

/// Stripe設定情報を取得
pub async fn get_stripe_config_handler(
    State(_app_state): State<AppState>,
) -> AppResult<Json<ApiResponse<StripeConfigResponse>>> {
    let config = std::sync::Arc::new(crate::config::stripe::StripeConfig::from_env());

    Ok(Json(ApiResponse::success(
        "Stripe configuration retrieved",
        StripeConfigResponse {
            publishable_key: config.publishable_key.clone(),
            is_test_mode: config.is_test_mode(),
        },
    )))
}
