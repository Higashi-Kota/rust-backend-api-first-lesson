// task-backend/src/middleware/rate_limit.rs

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
#[cfg(test)]
use axum::{http::StatusCode, response::IntoResponse};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    domain::subscription_tier::SubscriptionTier, error::AppError,
    middleware::auth::AuthenticatedUser,
};

/// レート制限の設定
#[derive(Clone)]
#[allow(dead_code)] // レート制限ミドルウェアが一時的に無効化されているため
pub struct RateLimitConfig {
    #[allow(dead_code)] // レート制限ミドルウェアが一時的に無効化されているため
    pub window_duration: Duration,
    #[allow(dead_code)] // レート制限ミドルウェアが一時的に無効化されているため
    pub limits_by_tier: HashMap<SubscriptionTier, usize>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut limits = HashMap::new();
        limits.insert(SubscriptionTier::Free, 100); // 100 requests per window
        limits.insert(SubscriptionTier::Pro, 1000); // 1000 requests per window
        limits.insert(SubscriptionTier::Enterprise, usize::MAX); // Unlimited

        Self {
            window_duration: Duration::from_secs(3600), // 1 hour window
            limits_by_tier: limits,
        }
    }
}

/// ユーザーごとのレート制限状態
#[derive(Clone)]
#[allow(dead_code)] // レート制限ミドルウェアが一時的に無効化されているため
struct UserRateLimit {
    count: usize,
    window_start: Instant,
}

/// レート制限のストレージ
#[derive(Clone)]
#[allow(dead_code)] // レート制限ミドルウェアが一時的に無効化されているため
pub struct RateLimitStorage {
    limits: Arc<Mutex<HashMap<Uuid, UserRateLimit>>>,
    config: RateLimitConfig,
}

impl RateLimitStorage {
    #[allow(dead_code)] // レート制限ミドルウェアが一時的に無効化されているため
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limits: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }
}

/// レート制限ミドルウェア
#[allow(dead_code)] // レート制限ミドルウェアが一時的に無効化されているため
pub async fn rate_limit_middleware(
    State(storage): State<RateLimitStorage>,
    user: Option<AuthenticatedUser>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // 認証されていないリクエストはレート制限しない
    let user = match user {
        Some(u) => u,
        None => return Ok(next.run(request).await),
    };

    // 管理者は制限なし
    if user.is_admin() {
        return Ok(next.run(request).await);
    }

    let user_id = user.user_id();
    let user_tier =
        SubscriptionTier::from_str(&user.subscription_tier()).unwrap_or(SubscriptionTier::Free);

    let limit = storage
        .config
        .limits_by_tier
        .get(&user_tier)
        .copied()
        .unwrap_or(100);

    let mut limits = storage.limits.lock().await;
    let now = Instant::now();

    let user_limit = limits.entry(user_id).or_insert_with(|| UserRateLimit {
        count: 0,
        window_start: now,
    });

    // ウィンドウが終了している場合はリセット
    if now.duration_since(user_limit.window_start) > storage.config.window_duration {
        user_limit.count = 0;
        user_limit.window_start = now;
    }

    // レート制限チェック
    if user_limit.count >= limit {
        return Err(AppError::TooManyRequests(
            "Rate limit exceeded. Please try again later.".to_string(),
        ));
    }

    // カウントを増やす
    user_limit.count += 1;

    // リクエストを処理
    Ok(next.run(request).await)
}

/// テスト用のレート制限ミドルウェア（すぐに制限に達する）
#[cfg(test)]
#[allow(dead_code)] // レート制限ミドルウェアが一時的に無効化されているため
pub async fn test_rate_limit_middleware(
    user: Option<AuthenticatedUser>,
    State(storage): State<RateLimitStorage>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // 認証されていないリクエストはレート制限しない
    let user = match user {
        Some(u) => u,
        None => return Ok(next.run(request).await),
    };

    let user_id = user.user_id();

    // テスト用：3回目のリクエストから制限
    let mut limits = storage.limits.lock().await;
    let user_limit = limits.entry(user_id).or_insert_with(|| UserRateLimit {
        count: 0,
        window_start: Instant::now(),
    });

    user_limit.count += 1;

    if user_limit.count > 3 {
        return Ok((StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response());
    }

    Ok(next.run(request).await)
}
