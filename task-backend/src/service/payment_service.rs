use crate::config::stripe::{StripeConfig, STRIPE_CLIENT};
use crate::core::subscription_tier::SubscriptionTier;
use crate::db::DbPool;
use crate::domain::stripe_payment_history_model::PaymentStatus;
use crate::error::{AppError, AppResult};
use crate::features::auth::repository::user_repository::UserRepository;
use crate::features::subscription::repositories::stripe_subscription::{
    CreateStripeSubscription, StripeSubscriptionRepository, UpdateStripeSubscription,
};
use crate::features::subscription::services::subscription::SubscriptionService;
use crate::repository::stripe_payment_history_repository::{
    CreatePaymentHistory, StripePaymentHistoryRepository,
};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use stripe::{
    BillingPortalSession, CheckoutSession, CheckoutSessionMode, CreateBillingPortalSession,
    CreateCheckoutSession, CreateCheckoutSessionLineItems, CreateCustomer, Customer, EventObject,
    EventType, Invoice, Subscription, Webhook,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct PaymentService {
    user_repo: Arc<UserRepository>,
    payment_history_repo: Arc<StripePaymentHistoryRepository>,
    subscription_repo: Arc<StripeSubscriptionRepository>,
    subscription_service: Arc<SubscriptionService>,
    stripe_config: StripeConfig,
}

impl PaymentService {
    pub fn new(db: DbPool, subscription_service: Arc<SubscriptionService>) -> Self {
        let user_repo = Arc::new(UserRepository::new(db.clone()));
        let payment_history_repo = Arc::new(StripePaymentHistoryRepository::new(db.clone()));
        let subscription_repo = Arc::new(StripeSubscriptionRepository::new(db));
        let stripe_config = StripeConfig::from_env();

        Self {
            user_repo,
            payment_history_repo,
            subscription_repo,
            subscription_service,
            stripe_config,
        }
    }

    /// Stripeチェックアウトセッションを作成
    pub async fn create_checkout_session(
        &self,
        user_id: Uuid,
        tier: SubscriptionTier,
    ) -> AppResult<String> {
        // 開発モードの場合はモックURLを返す
        if self.stripe_config.development_mode {
            tracing::info!("Development mode: returning mock checkout URL");
            return Ok(format!(
                "http://localhost:3000/mock-checkout?user_id={}&tier={}",
                user_id,
                tier.as_str()
            ));
        }

        // ユーザー情報を取得
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Stripe顧客IDを取得または作成
        let stripe_customer_id = match &user.stripe_customer_id {
            Some(id) => id.clone(),
            None => {
                let customer_params = CreateCustomer {
                    email: Some(&user.email),
                    name: Some(&user.username),
                    metadata: Some(
                        [("user_id".to_string(), user_id.to_string())]
                            .into_iter()
                            .collect(),
                    ),
                    ..Default::default()
                };

                let customer = Customer::create(&STRIPE_CLIENT, customer_params)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to create Stripe customer: {}", e);
                        AppError::ExternalServiceError(format!("Stripe error: {}", e))
                    })?;

                // Stripe顧客IDを保存
                self.user_repo
                    .update_stripe_customer_id(user_id, customer.id.as_str())
                    .await?;

                customer.id.to_string()
            }
        };

        // 価格IDを選択
        let price_id = self
            .stripe_config
            .get_price_id(tier.as_str())
            .ok_or_else(|| {
                AppError::BadRequest(format!("Invalid subscription tier: {}", tier.as_str()))
            })?;

        // チェックアウトセッションを作成
        let frontend_url =
            std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        let cancel_url = format!("{}/subscription/cancel", frontend_url);
        let success_url = format!(
            "{}/subscription/success?session_id={{CHECKOUT_SESSION_ID}}",
            frontend_url
        );

        let checkout_params = CreateCheckoutSession {
            cancel_url: Some(&cancel_url),
            success_url: Some(&success_url),
            customer: Some(stripe_customer_id.parse().map_err(|_| {
                AppError::InternalServerError("Invalid customer ID format".to_string())
            })?),
            line_items: Some(vec![CreateCheckoutSessionLineItems {
                price: Some(price_id.to_string()),
                quantity: Some(1),
                ..Default::default()
            }]),
            mode: Some(CheckoutSessionMode::Subscription),
            metadata: Some(
                [
                    ("user_id".to_string(), user_id.to_string()),
                    ("tier".to_string(), tier.as_str().to_string()),
                ]
                .into_iter()
                .collect(),
            ),
            ..Default::default()
        };

        let checkout_session = CheckoutSession::create(&STRIPE_CLIENT, checkout_params)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create checkout session: {}", e);

                // 価格IDエラーの場合、より具体的なメッセージを提供
                let error_message = if e.to_string().contains("No such price") {
                    format!(
                        "Stripe error: {}. Please check that price IDs (not product IDs) are configured correctly in environment variables. Price IDs start with 'price_', not 'prod_'",
                        e
                    )
                } else {
                    format!("Stripe error: {}", e)
                };

                AppError::ExternalServiceError(error_message)
            })?;

        checkout_session.url.ok_or_else(|| {
            AppError::InternalServerError("No checkout URL returned from Stripe".to_string())
        })
    }

    /// Webhookイベントを処理
    pub async fn handle_webhook(&self, payload: &str, stripe_signature: &str) -> AppResult<()> {
        // 開発モードの場合は、ペイロードを直接処理
        let event = if self.stripe_config.development_mode {
            tracing::info!("Development mode: processing webhook without signature verification");
            serde_json::from_str::<stripe::Event>(payload).map_err(|e| {
                tracing::error!("Failed to parse webhook payload: {}", e);
                AppError::BadRequest(format!("Invalid webhook payload: {}", e))
            })?
        } else if self.stripe_config.webhook_secret.is_empty() {
            // Webhook secretが設定されていない場合は、署名検証をスキップしてペイロードを直接パース
            tracing::warn!("STRIPE_WEBHOOK_SECRET not set - skipping signature verification (NOT SAFE FOR PRODUCTION)");
            serde_json::from_str::<stripe::Event>(payload).map_err(|e| {
                tracing::error!("Failed to parse webhook payload: {}", e);
                AppError::BadRequest(format!("Invalid webhook payload: {}", e))
            })?
        } else {
            // 署名を検証
            Webhook::construct_event(
                payload,
                stripe_signature,
                &self.stripe_config.webhook_secret,
            )
            .map_err(|e| {
                tracing::warn!("Invalid webhook signature: {}", e);
                AppError::BadRequest(format!("Invalid webhook: {}", e))
            })?
        };

        tracing::info!("Processing webhook event: {:?}", event.type_);

        // イベントタイプに応じて処理
        match event.type_ {
            EventType::CheckoutSessionCompleted => {
                if let EventObject::CheckoutSession(session) = event.data.object {
                    self.handle_checkout_completed(session).await?;
                }
            }
            EventType::CustomerSubscriptionDeleted => {
                if let EventObject::Subscription(subscription) = event.data.object {
                    self.handle_subscription_deleted(subscription).await?;
                }
            }
            EventType::CustomerSubscriptionUpdated => {
                if let EventObject::Subscription(subscription) = event.data.object {
                    self.handle_subscription_updated(subscription).await?;
                }
            }
            EventType::InvoicePaymentFailed => {
                if let EventObject::Invoice(invoice) = event.data.object {
                    self.handle_payment_failed(invoice).await?;
                }
            }
            _ => {
                tracing::debug!("Unhandled webhook event type: {:?}", event.type_);
            }
        }

        Ok(())
    }

    /// チェックアウト完了処理
    async fn handle_checkout_completed(&self, session: CheckoutSession) -> AppResult<()> {
        let user_id = session
            .metadata
            .as_ref()
            .and_then(|m| m.get("user_id"))
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| {
                tracing::error!("Invalid or missing user_id in checkout session metadata");
                AppError::BadRequest("Invalid user_id in metadata".to_string())
            })?;

        let tier = session
            .metadata
            .as_ref()
            .and_then(|m| m.get("tier"))
            .ok_or_else(|| {
                tracing::error!("Missing tier in checkout session metadata");
                AppError::BadRequest("Missing tier in metadata".to_string())
            })?;

        // 支払い履歴を記録
        let payment_history = CreatePaymentHistory {
            user_id,
            stripe_payment_intent_id: session
                .payment_intent
                .as_ref()
                .map(|pi| pi.id().to_string()),
            stripe_invoice_id: session.invoice.as_ref().map(|inv| inv.id().to_string()),
            amount: session.amount_total.unwrap_or(0) as i32,
            currency: session
                .currency
                .map_or_else(|| "jpy".to_string(), |c| c.to_string()),
            status: PaymentStatus::Succeeded.as_str().to_string(),
            description: Some(format!("Subscription upgrade to {} tier", tier)),
            paid_at: Some(Utc::now()),
        };

        self.payment_history_repo
            .create(payment_history)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create payment history: {}", e);
                AppError::InternalServerError(format!("Database error: {}", e))
            })?;

        // サブスクリプション階層を更新
        self.subscription_service
            .change_subscription_tier(
                user_id,
                tier.to_string(),
                None, // システムによる自動変更
                Some("Stripe payment completed".to_string()),
            )
            .await?;

        tracing::info!(
            "Subscription activated for user {} with tier {}",
            user_id,
            tier
        );

        Ok(())
    }

    /// サブスクリプション削除処理
    async fn handle_subscription_deleted(&self, subscription: Subscription) -> AppResult<()> {
        let customer_id = subscription.customer.id();

        // 顧客IDからユーザーを検索
        let user = self
            .user_repo
            .find_by_stripe_customer_id(&customer_id)
            .await?
            .ok_or_else(|| {
                tracing::warn!("User not found for Stripe customer: {}", customer_id);
                AppError::NotFound("User not found".to_string())
            })?;

        // サブスクリプションの詳細情報を保存/更新
        if let Some(existing_sub) = self
            .subscription_repo
            .find_by_stripe_subscription_id(subscription.id.as_ref())
            .await?
        {
            // 既存のサブスクリプション情報を更新
            let update_sub = UpdateStripeSubscription {
                status: Some("canceled".to_string()),
                canceled_at: Some(Utc::now()),
                ..Default::default()
            };

            self.subscription_repo
                .update(existing_sub.id, update_sub)
                .await?;
        }

        // キャンセルタイミングに応じて処理を分岐
        match subscription.cancel_at_period_end {
            true => {
                // 請求期間終了時にキャンセル（猶予期間あり）
                tracing::info!(
                    "Subscription set to cancel at period end for user {}, will remain active until {}",
                    user.id,
                    DateTime::<Utc>::from_timestamp(subscription.current_period_end, 0)
                        .map_or_else(|| "unknown".to_string(), |dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                );

                // 現在のティアは維持（請求期間終了まで機能を利用可能）
                // ユーザーへの通知などを行う場合はここに追加
            }
            false => {
                // 即座にキャンセル
                self.subscription_service
                    .change_subscription_tier(
                        user.id,
                        SubscriptionTier::Free.as_str().to_string(),
                        None,
                        Some("Stripe subscription cancelled immediately".to_string()),
                    )
                    .await?;

                tracing::info!(
                    "Subscription cancelled immediately for user {}, reverted to Free tier",
                    user.id
                );
            }
        }

        Ok(())
    }

    /// サブスクリプション更新処理
    async fn handle_subscription_updated(&self, subscription: Subscription) -> AppResult<()> {
        let customer_id = subscription.customer.id();

        // 顧客IDからユーザーを検索
        let user = self
            .user_repo
            .find_by_stripe_customer_id(&customer_id)
            .await?
            .ok_or_else(|| {
                tracing::warn!("User not found for Stripe customer: {}", customer_id);
                AppError::NotFound("User not found".to_string())
            })?;

        // 現在のサブスクリプション情報を更新または作成
        let sub_id = subscription.id.to_string();
        let existing_sub = self
            .subscription_repo
            .find_by_stripe_subscription_id(&sub_id)
            .await?;

        match existing_sub {
            Some(existing) => {
                // 既存のサブスクリプション情報を更新
                let update_sub = UpdateStripeSubscription {
                    status: Some(subscription.status.to_string()),
                    stripe_price_id: subscription
                        .items
                        .data
                        .first()
                        .and_then(|item| item.price.as_ref())
                        .map(|price| price.id.to_string()),
                    current_period_start: Some(
                        DateTime::<Utc>::from_timestamp(subscription.current_period_start, 0)
                            .expect("Invalid timestamp"),
                    ),
                    current_period_end: Some(
                        DateTime::<Utc>::from_timestamp(subscription.current_period_end, 0)
                            .expect("Invalid timestamp"),
                    ),
                    cancel_at: subscription
                        .cancel_at
                        .map(|ts| DateTime::<Utc>::from_timestamp(ts, 0).unwrap()),
                    canceled_at: subscription
                        .canceled_at
                        .map(|ts| DateTime::<Utc>::from_timestamp(ts, 0).unwrap()),
                };

                self.subscription_repo
                    .update(existing.id, update_sub)
                    .await?;
            }
            None => {
                // 新規サブスクリプション情報を作成
                let create_sub = CreateStripeSubscription {
                    user_id: user.id,
                    stripe_subscription_id: sub_id,
                    stripe_price_id: subscription
                        .items
                        .data
                        .first()
                        .and_then(|item| item.price.as_ref())
                        .map(|price| price.id.to_string())
                        .unwrap_or_default(),
                    status: subscription.status.to_string(),
                    current_period_start: Some(
                        DateTime::<Utc>::from_timestamp(subscription.current_period_start, 0)
                            .expect("Invalid timestamp"),
                    ),
                    current_period_end: Some(
                        DateTime::<Utc>::from_timestamp(subscription.current_period_end, 0)
                            .expect("Invalid timestamp"),
                    ),
                    cancel_at: subscription
                        .cancel_at
                        .map(|ts| DateTime::<Utc>::from_timestamp(ts, 0).unwrap()),
                    canceled_at: subscription
                        .canceled_at
                        .map(|ts| DateTime::<Utc>::from_timestamp(ts, 0).unwrap()),
                };

                self.subscription_repo.create(create_sub).await?;
            }
        }

        // キャンセル期間終了の場合、Freeプランに戻す
        if subscription.status.as_str() == "canceled" && !subscription.cancel_at_period_end {
            self.subscription_service
                .change_subscription_tier(
                    user.id,
                    SubscriptionTier::Free.as_str().to_string(),
                    None,
                    Some("Subscription period ended".to_string()),
                )
                .await?;

            tracing::info!(
                "Subscription period ended for user {}, reverted to Free tier",
                user.id
            );
        }

        Ok(())
    }

    /// 支払い失敗処理
    async fn handle_payment_failed(&self, invoice: Invoice) -> AppResult<()> {
        let customer_id = invoice.customer.as_ref().map(|c| c.id()).ok_or_else(|| {
            tracing::error!("No customer ID in failed invoice");
            AppError::BadRequest("No customer ID in invoice".to_string())
        })?;

        // 顧客IDからユーザーを検索
        let user = self
            .user_repo
            .find_by_stripe_customer_id(&customer_id)
            .await?
            .ok_or_else(|| {
                tracing::warn!("User not found for Stripe customer: {}", customer_id);
                AppError::NotFound("User not found".to_string())
            })?;

        // 支払い失敗履歴を記録
        let payment_history = CreatePaymentHistory {
            user_id: user.id,
            stripe_payment_intent_id: invoice
                .payment_intent
                .as_ref()
                .map(|pi| pi.id().to_string()),
            stripe_invoice_id: Some(invoice.id.to_string()),
            amount: invoice.amount_due.unwrap_or(0) as i32,
            currency: invoice
                .currency
                .map_or_else(|| "jpy".to_string(), |c| c.to_string()),
            status: PaymentStatus::Failed.as_str().to_string(),
            description: Some("Payment failed for subscription".to_string()),
            paid_at: None,
        };

        self.payment_history_repo
            .create(payment_history)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create payment history: {}", e);
                AppError::InternalServerError(format!("Database error: {}", e))
            })?;

        // TODO: メール通知を送信
        tracing::warn!(
            "Payment failed for user {} (customer: {})",
            user.id,
            customer_id
        );

        Ok(())
    }

    /// カスタマーポータルのURLを生成
    pub async fn create_customer_portal_url(&self, user_id: Uuid) -> AppResult<String> {
        // 開発モードの場合はモックURLを返す
        if self.stripe_config.development_mode {
            return Ok(format!(
                "http://localhost:3000/mock-portal?user_id={}",
                user_id
            ));
        }

        // ユーザー情報を取得
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let stripe_customer_id = user
            .stripe_customer_id
            .ok_or_else(|| AppError::BadRequest("No Stripe customer ID found".to_string()))?;

        // カスタマーポータルセッションを作成
        let frontend_url =
            std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        let return_url = format!("{}/subscription", frontend_url);

        let customer_id = stripe_customer_id
            .parse()
            .map_err(|_| AppError::InternalServerError("Invalid customer ID format".to_string()))?;

        let params = CreateBillingPortalSession {
            customer: customer_id,
            return_url: Some(&return_url),
            configuration: None,
            expand: &[],
            flow_data: None,
            locale: None,
            on_behalf_of: None,
        };

        let session = BillingPortalSession::create(&STRIPE_CLIENT, params)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create customer portal session: {}", e);
                AppError::ExternalServiceError(format!("Stripe error: {}", e))
            })?;

        Ok(session.url)
    }

    /// ユーザーの支払い履歴を取得
    pub async fn get_payment_history(
        &self,
        user_id: Uuid,
        page: u64,
        per_page: u64,
    ) -> AppResult<(Vec<crate::domain::stripe_payment_history_model::Model>, u64)> {
        self.payment_history_repo
            .find_by_user_id_paginated(user_id, page, per_page)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get payment history: {}", e);
                AppError::InternalServerError(format!("Database error: {}", e))
            })
    }
}
