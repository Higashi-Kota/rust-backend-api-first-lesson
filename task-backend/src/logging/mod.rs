// src/logging/mod.rs

use axum::{body::Body, http::Request, middleware::Next, response::Response};
use std::time::Instant;
use uuid::Uuid;

#[macro_export]
macro_rules! log_with_context {
    ($level:expr, $msg:expr $(, $($key:expr => $value:expr),* $(,)?)?) => {
        match $level {
            tracing::Level::ERROR => {
                tracing::error!(
                    message = $msg
                    $(, $($key = ?$value,)*)?
                );
            }
            tracing::Level::WARN => {
                tracing::warn!(
                    message = $msg
                    $(, $($key = ?$value,)*)?
                );
            }
            tracing::Level::INFO => {
                tracing::info!(
                    message = $msg
                    $(, $($key = ?$value,)*)?
                );
            }
            tracing::Level::DEBUG => {
                tracing::debug!(
                    message = $msg
                    $(, $($key = ?$value,)*)?
                );
            }
            _ => {}
        }
    };
}

// リクエストコンテキスト
#[derive(Clone, Debug)]
pub struct RequestContext {
    pub request_id: String,
    pub user_id: Option<Uuid>,
    pub path: String,
    pub method: String,
}

// ロギングミドルウェア
pub async fn logging_middleware(req: Request<Body>, next: Next) -> Response {
    let start = Instant::now();

    // RequestContextを取得
    let context = req.extensions().get::<RequestContext>().cloned();

    if let Some(context) = &context {
        log_with_context!(
            tracing::Level::INFO,
            "Request started",
            "request_id" => &context.request_id,
            "method" => &context.method,
            "path" => &context.path,
            "user_id" => context.user_id,
        );
    }

    let response = next.run(req).await;
    let duration = start.elapsed();
    let status = response.status().as_u16();

    if let Some(context) = &context {
        log_with_context!(
            if status >= 500 { tracing::Level::ERROR }
            else if status >= 400 { tracing::Level::WARN }
            else { tracing::Level::INFO },
            "Request completed",
            "request_id" => &context.request_id,
            "method" => &context.method,
            "path" => &context.path,
            "status" => status,
            "duration_ms" => duration.as_millis(),
            "user_id" => context.user_id,
        );
    }

    response
}

// RequestContextを生成するミドルウェア
pub async fn inject_request_context(mut req: Request<Body>, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    let path = req.uri().path().to_string();
    let method = req.method().to_string();

    // 後でユーザーIDを設定できるようにNoneで初期化
    let context = RequestContext {
        request_id,
        user_id: None,
        path,
        method,
    };

    req.extensions_mut().insert(context);
    next.run(req).await
}
