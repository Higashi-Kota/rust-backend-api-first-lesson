// task-backend/src/api/handlers/auth_handler.rs
use crate::api::dto::auth_dto::*;
use crate::api::{AppState, CookieConfig, HasJwtManager, SecurityHeaders};
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthenticatedUser;
use crate::types::ApiResponse;
use crate::utils::error_helper::convert_validation_errors;
use axum::{
    extract::{FromRequestParts, Json, State},
    http::{header, request::Parts, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use tracing::{info, warn};
use validator::Validate;

/// 認証済みユーザー情報抽出器（FromRequestParts実装）
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: HasJwtManager + Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Authorization ヘッダーからトークンを取得
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .and_then(|auth_str| auth_str.strip_prefix("Bearer "));

        // Cookieからトークンを取得（フォールバック）
        let cookie_token = parts
            .headers
            .get(header::COOKIE)
            .and_then(|header| header.to_str().ok())
            .and_then(|cookie_str| {
                Cookie::parse(cookie_str)
                    .ok()
                    .filter(|cookie| cookie.name() == state.cookie_config().access_token_name)
                    .map(|cookie| cookie.value().to_string())
            });

        let token = auth_header.or(cookie_token.as_deref()).ok_or_else(|| {
            warn!("Authentication attempt without token");
            AppError::Unauthorized("Missing authentication token".to_string())
        })?;

        // JWT検証
        let access_claims = state
            .jwt_manager()
            .verify_access_token(token)
            .map_err(|e| {
                warn!(error = %e, "JWT verification failed");
                match e {
                    crate::utils::jwt::JwtError::TokenExpired => {
                        AppError::Unauthorized("Access token has expired".to_string())
                    }
                    crate::utils::jwt::JwtError::InvalidToken => {
                        AppError::Unauthorized("Invalid access token".to_string())
                    }
                    _ => AppError::Unauthorized("Authentication failed".to_string()),
                }
            })?;

        info!(
            user_id = %access_claims.user.user_id,
            username = %access_claims.user.username,
            "User authenticated successfully"
        );

        Ok(AuthenticatedUser::new(
            access_claims.user,
            token.to_string(),
        ))
    }
}

// --- 認証ハンドラー ---

/// ユーザー登録
pub async fn signup_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> AppResult<impl IntoResponse> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "auth_handler::signup"))?;

    info!(
        email = %payload.email,
        username = %payload.username,
        "User signup attempt"
    );

    // ユーザー登録
    let auth_response = app_state.auth_service.signup(payload).await?;

    // レスポンスクッキーを設定
    let api_response = ApiResponse::success(auth_response.clone());
    let mut response = api_response.into_response();
    response.extensions_mut().insert(StatusCode::CREATED);

    let cookie_jar = create_auth_cookies(&auth_response.tokens, &app_state.cookie_config);

    // セキュリティヘッダーを追加
    add_security_headers(response.headers_mut(), &app_state.security_headers);

    // Cookieを追加
    add_cookies_to_response(&mut response, cookie_jar);

    info!(
        user_id = %auth_response.user.id,
        "User registered successfully"
    );

    Ok((StatusCode::CREATED, response))
}

/// ログイン
pub async fn signin_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SigninRequest>,
) -> AppResult<impl IntoResponse> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "auth_handler::signin"))?;

    info!(identifier = %payload.identifier, "User signin attempt");

    // IPアドレスとUser-Agentを抽出
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // ログイン
    let auth_response = app_state
        .auth_service
        .signin(payload, ip_address, user_agent)
        .await?;

    // レスポンスクッキーを設定
    let api_response = ApiResponse::success(auth_response.clone());
    let mut response = api_response.into_response();
    let cookie_jar = create_auth_cookies(&auth_response.tokens, &app_state.cookie_config);

    // セキュリティヘッダーを追加
    add_security_headers(response.headers_mut(), &app_state.security_headers);

    // Cookieを追加
    add_cookies_to_response(&mut response, cookie_jar);

    info!(
        user_id = %auth_response.user.id,
        "User signed in successfully"
    );

    Ok(response)
}

/// ログアウト
pub async fn signout_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    cookie_jar: CookieJar,
) -> AppResult<impl IntoResponse> {
    // リフレッシュトークンを取得
    let refresh_token = cookie_jar
        .get(&app_state.cookie_config.refresh_token_name)
        .map(|cookie| cookie.value().to_string())
        .unwrap_or_default();

    // ログアウト処理
    let _logout_response = app_state.auth_service.signout(&refresh_token).await?;

    // 成功レスポンスを作成
    let success_response = LogoutResponse {
        message: "Successfully signed out".to_string(),
    };
    let api_response = ApiResponse::success(success_response);
    let mut response = api_response.into_response();

    // Cookieを削除
    let expired_cookies = create_expired_auth_cookies(&app_state.cookie_config);
    add_cookies_to_response(&mut response, expired_cookies);

    // セキュリティヘッダーを追加
    add_security_headers(response.headers_mut(), &app_state.security_headers);

    info!(user_id = %user.claims.user_id, "User signed out successfully");

    Ok(response)
}

/// 全デバイスからログアウト
pub async fn signout_all_devices_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<ApiResponse<LogoutResponse>> {
    let logout_response = app_state
        .auth_service
        .signout_all_devices(user.claims.user_id)
        .await?;

    info!(user_id = %user.claims.user_id, "User signed out from all devices");

    Ok(ApiResponse::success(logout_response))
}

/// トークンリフレッシュ
pub async fn refresh_token_handler(
    State(app_state): State<AppState>,
    _cookie_jar: CookieJar,
    Json(payload): Json<RefreshTokenRequest>,
) -> AppResult<impl IntoResponse> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "auth_handler::refresh_token"))?;

    let refresh_token = payload.refresh_token;

    info!("Token refresh attempt");

    // トークンリフレッシュ
    let refresh_response = app_state
        .auth_service
        .refresh_access_token(&refresh_token)
        .await?;

    // レスポンスクッキーを設定
    let api_response = ApiResponse::success(refresh_response.clone());
    let mut response = api_response.into_response();
    let cookie_jar = create_auth_cookies(&refresh_response.tokens, &app_state.cookie_config);

    // セキュリティヘッダーを追加
    add_security_headers(response.headers_mut(), &app_state.security_headers);

    // Cookieを追加
    add_cookies_to_response(&mut response, cookie_jar);

    info!("Token refreshed successfully");

    Ok(response)
}

/// パスワードリセット要求
pub async fn forgot_password_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<PasswordResetRequestRequest>,
) -> AppResult<ApiResponse<PasswordResetRequestResponse>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "auth_handler::password_reset_request"))?;

    info!(email = %payload.email, "Password reset requested");

    let response = app_state
        .auth_service
        .request_password_reset(&payload.email)
        .await?;

    Ok(ApiResponse::success(response))
}

/// パスワードリセット実行
pub async fn reset_password_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<PasswordResetRequest>,
) -> AppResult<ApiResponse<PasswordResetResponse>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "auth_handler::password_reset"))?;

    info!("Password reset execution attempt");

    let response = app_state.auth_service.reset_password(payload).await?;

    info!("Password reset completed successfully");

    Ok(ApiResponse::success(response))
}

/// パスワード変更
pub async fn change_password_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<PasswordChangeRequest>,
) -> AppResult<ApiResponse<PasswordChangeResponse>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "auth_handler::password_change"))?;

    // カスタムバリデーション
    payload.validate_password_change().map_err(|e| {
        warn!("Password change custom validation failed: {}", e);
        AppError::BadRequest(e)
    })?;

    info!(user_id = %user.claims.user_id, "Password change attempt");

    // パスワード変更用の構造体に変換
    let change_input = crate::utils::password::PasswordChangeInput {
        current_password: payload.current_password,
        new_password: payload.new_password.clone(),
        new_password_confirmation: payload.new_password_confirmation,
    };

    let response = app_state
        .auth_service
        .change_password(user.claims.user_id, change_input)
        .await?;

    info!(user_id = %user.claims.user_id, "Password changed successfully");

    Ok(ApiResponse::success(response))
}

/// 現在のユーザー情報取得
pub async fn me_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<ApiResponse<CurrentUserResponse>> {
    let current_user_response = app_state
        .auth_service
        .get_current_user(user.claims.user_id)
        .await?;

    Ok(ApiResponse::success(current_user_response))
}

/// アカウント削除
pub async fn delete_account_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<DeleteAccountRequest>,
) -> AppResult<impl IntoResponse> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "auth_handler::delete_account"))?;

    // カスタムバリデーション
    payload.validate_deletion().map_err(|e| {
        warn!("Account deletion custom validation failed: {}", e);
        AppError::BadRequest(e)
    })?;

    warn!(user_id = %user.claims.user_id, "Account deletion attempt");

    let response = app_state
        .auth_service
        .delete_account(user.claims.user_id, &payload.password)
        .await?;

    // レスポンスを作成
    let api_response = ApiResponse::success(response);
    let mut response = api_response.into_response();

    // Cookieを削除
    let expired_cookies = create_expired_auth_cookies(&app_state.cookie_config);
    add_cookies_to_response(&mut response, expired_cookies);

    // セキュリティヘッダーを追加
    add_security_headers(response.headers_mut(), &app_state.security_headers);

    info!(user_id = %user.claims.user_id, "Account deleted successfully");

    Ok(response)
}

/// 認証ステータス確認
pub async fn auth_status_handler() -> ApiResponse<AuthStatusResponse> {
    // このエンドポイントは認証が不要なので、常に未認証として返す
    // 実際の認証状態を確認する場合は、/auth/me エンドポイントを使用
    ApiResponse::success(AuthStatusResponse {
        authenticated: false,
        user: None,
        access_token_expires_in: None,
    })
}

/// メール認証実行
pub async fn verify_email_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<EmailVerificationRequest>,
) -> AppResult<ApiResponse<EmailVerificationResponse>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "auth_handler::verify_email"))?;

    info!("Email verification attempt");

    let response = app_state.auth_service.verify_email(&payload.token).await?;

    info!("Email verification completed successfully");

    Ok(ApiResponse::success(response))
}

/// メール認証再送
pub async fn resend_verification_email_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<ResendVerificationEmailRequest>,
) -> AppResult<ApiResponse<ResendVerificationEmailResponse>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "auth_handler::resend_verification_email"))?;

    info!(email = %payload.email, "Resend verification email requested");

    let response = app_state
        .auth_service
        .resend_verification_email(&payload.email)
        .await?;

    Ok(ApiResponse::success(response))
}

// --- ヘルパー関数 ---

/// 認証用Cookieを作成
fn create_auth_cookies(tokens: &crate::utils::jwt::TokenPair, config: &CookieConfig) -> CookieJar {
    let mut jar = CookieJar::new();

    // アクセストークンクッキー
    let access_cookie = Cookie::build((
        config.access_token_name.clone(),
        tokens.access_token.clone(),
    ))
    .path(config.path.clone())
    .secure(config.secure)
    .http_only(config.http_only)
    .same_site(axum_extra::extract::cookie::SameSite::Strict)
    .max_age(time::Duration::seconds(tokens.access_token_expires_in))
    .build();

    // リフレッシュトークンクッキー
    let refresh_cookie = Cookie::build((
        config.refresh_token_name.clone(),
        tokens.refresh_token.clone(),
    ))
    .path(config.path.clone())
    .secure(config.secure)
    .http_only(config.http_only)
    .same_site(axum_extra::extract::cookie::SameSite::Strict)
    .max_age(time::Duration::seconds(tokens.refresh_token_expires_in))
    .build();

    jar = jar.add(access_cookie);
    jar = jar.add(refresh_cookie);

    jar
}

/// 期限切れ認証Cookieを作成（削除用）
fn create_expired_auth_cookies(config: &CookieConfig) -> CookieJar {
    let mut jar = CookieJar::new();

    // 期限切れアクセストークンクッキー
    let expired_access_cookie = Cookie::build((config.access_token_name.clone(), ""))
        .path(config.path.clone())
        .secure(config.secure)
        .http_only(config.http_only)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .max_age(time::Duration::seconds(0))
        .build();

    // 期限切れリフレッシュトークンクッキー
    let expired_refresh_cookie = Cookie::build((config.refresh_token_name.clone(), ""))
        .path(config.path.clone())
        .secure(config.secure)
        .http_only(config.http_only)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .max_age(time::Duration::seconds(0))
        .build();

    jar = jar.add(expired_access_cookie);
    jar = jar.add(expired_refresh_cookie);

    jar
}

/// レスポンスにCookieを追加
fn add_cookies_to_response(response: &mut Response, cookie_jar: CookieJar) {
    let headers = response.headers_mut();
    for cookie in cookie_jar.iter() {
        if let Ok(header_value) = cookie.to_string().parse() {
            headers.append(header::SET_COOKIE, header_value);
        }
    }
}

/// セキュリティヘッダーを追加
fn add_security_headers(headers: &mut HeaderMap, security: &SecurityHeaders) {
    headers.insert(
        "Content-Security-Policy",
        security.content_security_policy.parse().unwrap(),
    );
    headers.insert("X-Frame-Options", security.x_frame_options.parse().unwrap());
    headers.insert(
        "X-Content-Type-Options",
        security.x_content_type_options.parse().unwrap(),
    );
    headers.insert("Referrer-Policy", security.referrer_policy.parse().unwrap());
    headers.insert(
        "Permissions-Policy",
        security.permissions_policy.parse().unwrap(),
    );
}

/// 保留中のメール認証状態確認
pub async fn check_pending_verification_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<ApiResponse<crate::api::dto::user_dto::PendingEmailVerificationResponse>> {
    let response = app_state
        .auth_service
        .check_pending_email_verification(user.claims.user_id)
        .await?;

    info!(
        user_id = %user.claims.user_id,
        has_pending = response.has_pending_verification,
        "Checked pending email verification status"
    );

    Ok(ApiResponse::success(response))
}

/// トークン状態確認リクエスト
#[derive(Debug, serde::Deserialize)]
pub struct TokenStatusRequest {
    pub token: String,
}

/// トークン状態確認（管理者用）
pub async fn check_token_status_handler(
    State(app_state): State<AppState>,
    _admin: AuthenticatedUser,
    Json(payload): Json<TokenStatusRequest>,
) -> AppResult<ApiResponse<crate::api::dto::user_dto::TokenStatusResponse>> {
    let response = app_state
        .auth_service
        .check_token_status(&payload.token)
        .await?;

    info!(
        exists = response.exists,
        is_valid = response.is_valid,
        "Checked token status"
    );

    Ok(ApiResponse::success(response))
}

// --- ルーター ---

/// 認証ルーターを作成
pub fn auth_router(app_state: AppState) -> Router {
    use crate::middleware::auth::rate_limit_middleware;

    Router::new()
        .route("/auth/signup", post(signup_handler))
        .route("/auth/signin", post(signin_handler))
        .route("/auth/signout", post(signout_handler))
        .route("/auth/signout-all", post(signout_all_devices_handler))
        .route("/auth/refresh", post(refresh_token_handler))
        .route("/auth/forgot-password", post(forgot_password_handler))
        .route("/auth/reset-password", post(reset_password_handler))
        .route("/auth/change-password", put(change_password_handler))
        .route("/auth/verify-email", post(verify_email_handler))
        .route(
            "/auth/resend-verification",
            post(resend_verification_email_handler),
        )
        .route(
            "/auth/pending-verification",
            get(check_pending_verification_handler),
        )
        .route("/auth/token-status", post(check_token_status_handler))
        .route("/auth/me", get(me_handler))
        .route("/auth/account", delete(delete_account_handler))
        .route("/auth/status", get(auth_status_handler))
        // rate_limit_middlewareを適用（認証系エンドポイントへのレート制限）
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .with_state(app_state)
}

/// 認証ルーターをAppStateから作成
pub fn auth_router_with_state(app_state: AppState) -> Router {
    auth_router(app_state)
}
