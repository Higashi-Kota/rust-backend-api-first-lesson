// task-backend/src/service/auth_service.rs
use crate::api::dto::auth_dto::*;
use crate::domain::password_reset_token_model::TokenValidationError;
use crate::domain::refresh_token_model::CreateRefreshToken;
use crate::domain::role_model::RoleName;
use crate::domain::user_model::UserClaims;
use crate::error::{AppError, AppResult};
use crate::repository::password_reset_token_repository::PasswordResetTokenRepository;
use crate::repository::refresh_token_repository::RefreshTokenRepository;
use crate::repository::role_repository::RoleRepository;
use crate::repository::user_repository::{CreateUser, UserRepository};
use crate::utils::error_helper::{
    conflict_error, convert_validation_errors, internal_server_error, not_found_error,
};
use crate::utils::jwt::{JwtManager, TokenPair};
use crate::utils::password::{PasswordChangeInput, PasswordManager};
use crate::utils::transaction::ServiceTransactionManager;
use chrono::{Duration, Utc};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;
use validator::Validate;

/// 認証サービス
pub struct AuthService {
    user_repo: Arc<UserRepository>,
    role_repo: Arc<RoleRepository>,
    refresh_token_repo: Arc<RefreshTokenRepository>,
    password_reset_token_repo: Arc<PasswordResetTokenRepository>,
    password_manager: Arc<PasswordManager>,
    jwt_manager: Arc<JwtManager>,
    db: Arc<DatabaseConnection>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<UserRepository>,
        role_repo: Arc<RoleRepository>,
        refresh_token_repo: Arc<RefreshTokenRepository>,
        password_reset_token_repo: Arc<PasswordResetTokenRepository>,
        password_manager: Arc<PasswordManager>,
        jwt_manager: Arc<JwtManager>,
        db: Arc<DatabaseConnection>,
    ) -> Self {
        Self {
            user_repo,
            role_repo,
            refresh_token_repo,
            password_reset_token_repo,
            password_manager,
            jwt_manager,
            db,
        }
    }

    // --- ユーザー登録・ログイン ---

    /// ユーザー登録（統一化されたエラーハンドリングとトランザクション管理）
    #[instrument(skip(self, signup_data), fields(email = %signup_data.email, username = %signup_data.username))]
    pub async fn signup(&self, signup_data: SignupRequest) -> AppResult<AuthResponse> {
        // バリデーション（統一化されたエラー処理）
        signup_data
            .validate()
            .map_err(|e| convert_validation_errors(e, "user signup"))?;

        // パスワード強度チェック
        self.password_manager
            .validate_password_strength(&signup_data.password)
            .map_err(|_| {
                AppError::ValidationError(
                    "password: Password does not meet strength requirements".to_string(),
                )
            })?;

        // パスワードハッシュ化
        let password_hash = self
            .password_manager
            .hash_password(&signup_data.password)
            .map_err(|e| {
                internal_server_error(e, "auth_service::signup", "Failed to process password")
            })?;

        // トランザクション内でユーザー登録処理を実行
        let user_repo = Arc::clone(&self.user_repo);
        let role_repo = Arc::clone(&self.role_repo);
        let signup_data_cloned = signup_data.clone();
        let password_hash_cloned = password_hash.clone();

        let auth_response = self
            .db
            .execute_service_transaction(move |_txn| {
                Box::pin(async move {
                    // メールアドレスとユーザー名の重複チェック
                    if user_repo.is_email_taken(&signup_data_cloned.email).await? {
                        return Err(conflict_error(
                            "Email address is already registered",
                            "auth_service::signup::email_check",
                        ));
                    }

                    if user_repo
                        .is_username_taken(&signup_data_cloned.username)
                        .await?
                    {
                        return Err(conflict_error(
                            "Username is already taken",
                            "auth_service::signup::username_check",
                        ));
                    }

                    // デフォルトのメンバーロールを取得
                    let member_role = role_repo
                        .find_by_name(RoleName::Member.as_str())
                        .await?
                        .ok_or_else(|| {
                            not_found_error(
                                "Role",
                                RoleName::Member.as_str(),
                                "auth_service::signup::role_lookup",
                            )
                        })?;

                    // ユーザー作成
                    let create_user = CreateUser {
                        email: signup_data_cloned.email.clone(),
                        username: signup_data_cloned.username.clone(),
                        password_hash: password_hash_cloned,
                        role_id: member_role.id,
                        is_active: Some(true),
                        email_verified: Some(false), // メール認証は別途実装
                    };

                    let user = user_repo.create(create_user).await?;

                    info!(
                        user_id = %user.id,
                        email = %user.email,
                        username = %user.username,
                        "User registered successfully"
                    );

                    // ロール情報付きユーザーを取得
                    let user_with_role = user_repo
                        .find_by_id_with_role(user.id)
                        .await?
                        .ok_or_else(|| {
                            internal_server_error(
                                "User with role lookup failed after creation",
                                "auth_service::signup::user_with_role_lookup",
                                "Registration failed",
                            )
                        })?;

                    Ok(user_with_role)
                })
            })
            .await?;

        // JWT トークン生成（トランザクション外で実行）
        let user_claims = auth_response.to_simple_claims();
        let token_pair = self.create_token_pair(&user_claims).await?;

        Ok(AuthResponse {
            user: auth_response.into(),
            tokens: token_pair,
            message: "Registration successful".to_string(),
        })
    }

    /// ログイン
    pub async fn signin(&self, signin_data: SigninRequest) -> AppResult<AuthResponse> {
        // バリデーション
        signin_data
            .validate()
            .map_err(|e| AppError::ValidationError(format!("Validation failed: {}", e)))?;

        // ユーザー検索（メールアドレスまたはユーザー名）
        let user = self
            .user_repo
            .find_by_email_or_username(&signin_data.identifier)
            .await?
            .ok_or_else(|| {
                warn!(
                    identifier = %signin_data.identifier,
                    "Login attempt with invalid credentials"
                );
                AppError::Unauthorized("Invalid credentials".to_string())
            })?;

        // アカウント状態チェック
        if !user.can_authenticate() {
            warn!(
                user_id = %user.id,
                is_active = %user.is_active,
                "Login attempt for inactive account"
            );
            return Err(AppError::Unauthorized("Account is inactive".to_string()));
        }

        // パスワード検証
        let is_valid = self
            .password_manager
            .verify_password(&signin_data.password, &user.password_hash)
            .map_err(|e| {
                error!(
                    user_id = %user.id,
                    error = %e,
                    "Password verification failed"
                );
                AppError::InternalServerError("Authentication failed".to_string())
            })?;

        if !is_valid {
            warn!(
                user_id = %user.id,
                identifier = %signin_data.identifier,
                "Login attempt with incorrect password"
            );
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }

        // パスワード再ハッシュが必要かチェック
        if self
            .password_manager
            .needs_rehash(&user.password_hash)
            .unwrap_or(false)
        {
            if let Ok(new_hash) = self.password_manager.hash_password(&signin_data.password) {
                let _ = self.user_repo.update_password_hash(user.id, new_hash).await;
                info!(user_id = %user.id, "Password rehashed with updated parameters");
            }
        }

        // ロール情報付きユーザーを取得
        let user_with_role = self
            .user_repo
            .find_by_id_with_role(user.id)
            .await?
            .ok_or_else(|| {
                AppError::InternalServerError(
                    "User with role not found after authentication".to_string(),
                )
            })?;

        info!(
            user_id = %user_with_role.id,
            email = %user_with_role.email,
            "User signed in successfully"
        );

        // JWT トークン生成
        let user_claims = user_with_role.to_simple_claims();
        let token_pair = self.create_token_pair(&user_claims).await?;

        Ok(AuthResponse {
            user: user_with_role.into(),
            tokens: token_pair,
            message: "Login successful".to_string(),
        })
    }

    /// ログアウト
    pub async fn signout(&self, refresh_token: &str) -> AppResult<LogoutResponse> {
        // リフレッシュトークンをハッシュ化してから無効化
        let token_hash = self.hash_token(refresh_token);
        let revoked = self
            .refresh_token_repo
            .revoke_by_token_hash(&token_hash)
            .await?;

        if revoked {
            info!("User signed out successfully");
            Ok(LogoutResponse {
                message: "Logout successful".to_string(),
            })
        } else {
            warn!("Logout attempt with invalid refresh token");
            Ok(LogoutResponse {
                message: "Already logged out".to_string(),
            })
        }
    }

    /// 全デバイスからログアウト
    pub async fn signout_all_devices(&self, user_id: Uuid) -> AppResult<LogoutResponse> {
        let revoked_count = self
            .refresh_token_repo
            .revoke_all_user_tokens(user_id)
            .await?;

        info!(
            user_id = %user_id,
            revoked_count = %revoked_count,
            "User signed out from all devices"
        );

        Ok(LogoutResponse {
            message: format!("Logged out from {} devices", revoked_count),
        })
    }

    // --- トークン管理 ---

    /// アクセストークンをリフレッシュ
    pub async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> AppResult<TokenRefreshResponse> {
        // リフレッシュトークンの検証
        let token_claims = self
            .jwt_manager
            .verify_refresh_token(refresh_token)
            .map_err(|e| {
                warn!(error = %e, "Invalid refresh token");
                AppError::Unauthorized("Invalid refresh token".to_string())
            })?;

        let user_id = Uuid::parse_str(&token_claims.sub)
            .map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))?;

        // データベースでトークンの有効性を確認
        let token_hash = self.hash_token(refresh_token);
        let _db_token = self
            .refresh_token_repo
            .find_valid_by_token_hash(&token_hash)
            .await?
            .ok_or_else(|| {
                warn!(user_id = %user_id, "Refresh token not found in database");
                AppError::Unauthorized("Invalid refresh token".to_string())
            })?;

        // ユーザー情報取得（ロール情報付き）
        let user_with_role = self
            .user_repo
            .find_by_id_with_role(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // ユーザーがアクティブかチェック
        if !user_with_role.can_authenticate() {
            warn!(user_id = %user_id, "Token refresh for inactive user");
            return Err(AppError::Unauthorized("Account is inactive".to_string()));
        }

        // 新しいトークンペアを生成（リフレッシュトークンローテーション）
        let user_claims = user_with_role.to_simple_claims();

        // 新しいリフレッシュトークンを生成
        let new_refresh_token = self
            .jwt_manager
            .generate_refresh_token(user_with_role.id, token_claims.ver + 1)
            .map_err(|e| {
                AppError::InternalServerError(format!("Token generation failed: {}", e))
            })?;

        let new_refresh_token_hash = self.hash_token(&new_refresh_token);
        let refresh_expires_at = Utc::now() + Duration::days(7);

        let create_refresh_token = CreateRefreshToken {
            user_id: user_with_role.id,
            token_hash: new_refresh_token_hash.clone(),
            expires_at: refresh_expires_at,
        };

        // トークンローテーション（古いトークンを無効化し、新しいトークンを作成）
        let rotation_result = self
            .refresh_token_repo
            .rotate_token(&token_hash, create_refresh_token)
            .await?;

        if rotation_result.is_none() {
            return Err(AppError::InternalServerError(
                "Token rotation failed".to_string(),
            ));
        }

        // 新しいアクセストークンを生成
        let access_token = self
            .jwt_manager
            .generate_access_token(user_claims)
            .map_err(|e| {
                AppError::InternalServerError(format!("Token generation failed: {}", e))
            })?;

        info!(user_id = %user_id, "Access token refreshed successfully");

        let token_pair = TokenPair::create_with_jwt_manager(
            access_token.clone(),
            new_refresh_token.clone(),
            15, // 15分
            7,  // 7日
            &self.jwt_manager,
        );

        Ok(TokenRefreshResponse {
            user: user_with_role.into(),
            tokens: token_pair,
        })
    }

    // --- パスワードリセット ---

    /// パスワードリセット要求
    pub async fn request_password_reset(
        &self,
        email: &str,
    ) -> AppResult<PasswordResetRequestResponse> {
        // ユーザー検索
        let user = self.user_repo.find_by_email(email).await?;

        // セキュリティ上、ユーザーが存在しなくても成功レスポンスを返す
        if user.is_none() {
            info!(email = %email, "Password reset requested for non-existent email");
            return Ok(PasswordResetRequestResponse {
                message: "If the email address exists, a password reset link has been sent"
                    .to_string(),
            });
        }

        let user = user.unwrap();

        // アカウントがアクティブかチェック
        if !user.is_active {
            warn!(user_id = %user.id, "Password reset requested for inactive account");
            return Ok(PasswordResetRequestResponse {
                message: "If the email address exists, a password reset link has been sent"
                    .to_string(),
            });
        }

        // パスワードリセットトークンを生成
        let token_hash = self.generate_token_hash();
        let expires_at = Utc::now() + Duration::hours(1); // 1時間有効

        let result = self
            .password_reset_token_repo
            .create_reset_request(user.id, token_hash.clone(), expires_at)
            .await?;

        info!(
            user_id = %user.id,
            token_id = %result.token_id,
            old_tokens_invalidated = %result.old_tokens_invalidated,
            "Password reset token created"
        );

        // TODO: メール送信の実装
        // email_service.send_password_reset_email(&user.email, &token_hash).await?;

        Ok(PasswordResetRequestResponse {
            message: "If the email address exists, a password reset link has been sent".to_string(),
        })
    }

    /// パスワードリセット実行
    pub async fn reset_password(
        &self,
        reset_data: PasswordResetRequest,
    ) -> AppResult<PasswordResetResponse> {
        // バリデーション
        reset_data
            .validate()
            .map_err(|e| AppError::ValidationError(format!("Validation failed: {}", e)))?;

        // パスワード強度チェック
        self.password_manager
            .validate_password_strength(&reset_data.new_password)
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // パスワードリセットトークンの検証と実行
        let reset_result = self
            .password_reset_token_repo
            .execute_password_reset(&reset_data.token)
            .await?;

        let reset_result = match reset_result {
            Ok(result) => result,
            Err(TokenValidationError::NotFound) => {
                warn!("Password reset with invalid token");
                return Err(AppError::ValidationError(
                    "Invalid or expired reset token".to_string(),
                ));
            }
            Err(TokenValidationError::Expired) => {
                warn!("Password reset with expired token");
                return Err(AppError::ValidationError(
                    "Reset token has expired".to_string(),
                ));
            }
            Err(TokenValidationError::AlreadyUsed) => {
                warn!("Password reset with already used token");
                return Err(AppError::ValidationError(
                    "Reset token has already been used".to_string(),
                ));
            }
            Err(e) => {
                error!(error = %e, "Password reset token validation failed");
                return Err(AppError::ValidationError("Invalid reset token".to_string()));
            }
        };

        // 新しいパスワードをハッシュ化
        let new_password_hash = self
            .password_manager
            .hash_password(&reset_data.new_password)
            .map_err(|e| {
                AppError::InternalServerError(format!("Password hashing failed: {}", e))
            })?;

        // パスワードを更新
        self.user_repo
            .update_password_hash(reset_result.user_id, new_password_hash)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 全リフレッシュトークンを無効化（セキュリティ上）
        let revoked_count = self
            .refresh_token_repo
            .revoke_all_user_tokens(reset_result.user_id)
            .await?;

        info!(
            user_id = %reset_result.user_id,
            token_id = %reset_result.token_id,
            revoked_tokens = %revoked_count,
            "Password reset completed successfully"
        );

        Ok(PasswordResetResponse {
            message: "Password has been reset successfully. Please log in with your new password"
                .to_string(),
        })
    }

    // --- パスワード変更 ---

    /// パスワード変更
    pub async fn change_password(
        &self,
        user_id: Uuid,
        change_data: PasswordChangeInput,
    ) -> AppResult<PasswordChangeResponse> {
        // バリデーション
        change_data
            .validate()
            .map_err(|e| AppError::ValidationError(format!("Validation failed: {}", e)))?;

        // ユーザー取得
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 現在のパスワードを検証
        let is_current_valid = self
            .password_manager
            .verify_password(&change_data.current_password, &user.password_hash)
            .map_err(|e| {
                AppError::InternalServerError(format!("Password verification failed: {}", e))
            })?;

        if !is_current_valid {
            warn!(user_id = %user_id, "Password change with incorrect current password");
            return Err(AppError::Unauthorized(
                "Current password is incorrect".to_string(),
            ));
        }

        // 新しいパスワードの強度チェック
        self.password_manager
            .validate_password_strength(&change_data.new_password)
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // 現在のパスワードと同じでないかチェック
        if change_data.current_password == change_data.new_password {
            return Err(AppError::ValidationError(
                "New password must be different from current password".to_string(),
            ));
        }

        // 新しいパスワードをハッシュ化
        let new_password_hash = self
            .password_manager
            .hash_password(&change_data.new_password)
            .map_err(|e| {
                AppError::InternalServerError(format!("Password hashing failed: {}", e))
            })?;

        // パスワードを更新
        self.user_repo
            .update_password_hash(user_id, new_password_hash)
            .await?;

        info!(user_id = %user_id, "Password changed successfully");

        Ok(PasswordChangeResponse {
            message: "Password has been changed successfully".to_string(),
        })
    }

    // --- アカウント管理 ---

    /// 現在のユーザー情報取得
    pub async fn get_current_user(&self, user_id: Uuid) -> AppResult<CurrentUserResponse> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(CurrentUserResponse { user: user.into() })
    }

    /// アカウント削除
    pub async fn delete_account(
        &self,
        user_id: Uuid,
        password: &str,
    ) -> AppResult<AccountDeletionResponse> {
        // ユーザー取得
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // パスワード検証
        let is_valid = self
            .password_manager
            .verify_password(password, &user.password_hash)
            .map_err(|e| {
                AppError::InternalServerError(format!("Password verification failed: {}", e))
            })?;

        if !is_valid {
            warn!(user_id = %user_id, "Account deletion with incorrect password");
            return Err(AppError::Unauthorized("Password is incorrect".to_string()));
        }

        // 全リフレッシュトークンを削除
        let refresh_cleanup = self.refresh_token_repo.cleanup_revoked_tokens().await?;

        // パスワードリセットトークンを削除
        let password_reset_cleanup = self
            .password_reset_token_repo
            .cleanup_all(0) // 全て削除
            .await?;

        // ユーザーを削除（CASCADE により関連データも削除される）
        let delete_result = self.user_repo.delete(user_id).await?;

        if delete_result.rows_affected == 0 {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        info!(
            user_id = %user_id,
            email = %user.email,
            refresh_tokens_deleted = %refresh_cleanup.deleted_count,
            password_reset_tokens_deleted = %password_reset_cleanup.total_deleted,
            "Account deleted successfully"
        );

        Ok(AccountDeletionResponse {
            message: "Account has been permanently deleted".to_string(),
        })
    }

    // --- ヘルパーメソッド ---

    /// トークンペアを作成
    async fn create_token_pair(&self, user_claims: &UserClaims) -> AppResult<TokenPair> {
        // アクセストークン生成
        let access_token = self
            .jwt_manager
            .generate_access_token(user_claims.clone())
            .map_err(|e| {
                AppError::InternalServerError(format!("Access token generation failed: {}", e))
            })?;

        // リフレッシュトークン生成
        let refresh_token = self
            .jwt_manager
            .generate_refresh_token(user_claims.user_id, 1)
            .map_err(|e| {
                AppError::InternalServerError(format!("Refresh token generation failed: {}", e))
            })?;

        // リフレッシュトークンをデータベースに保存
        let refresh_token_hash = self.hash_token(&refresh_token);
        let refresh_expires_at = Utc::now() + Duration::days(7);

        let create_refresh_token = CreateRefreshToken {
            user_id: user_claims.user_id,
            token_hash: refresh_token_hash,
            expires_at: refresh_expires_at,
        };

        self.refresh_token_repo.create(create_refresh_token).await?;

        Ok(TokenPair::create_with_jwt_manager(
            access_token,
            refresh_token,
            15, // 15分
            7,  // 7日
            &self.jwt_manager,
        ))
    }

    /// セキュアなトークンハッシュを生成
    fn generate_token_hash(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        hex::encode(bytes)
    }

    /// トークンをハッシュ化
    fn hash_token(&self, token: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }
}
