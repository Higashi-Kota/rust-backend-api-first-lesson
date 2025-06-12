// task-backend/src/service/auth_service.rs
use crate::api::dto::auth_dto::*;
use crate::domain::password_reset_token_model::TokenValidationError;
use crate::domain::refresh_token_model::CreateRefreshToken;
use crate::domain::user_model::{SafeUser, UserClaims};
use crate::error::{AppError, AppResult};
use crate::repository::password_reset_token_repository::PasswordResetTokenRepository;
use crate::repository::refresh_token_repository::RefreshTokenRepository;
use crate::repository::user_repository::{CreateUser, UserRepository};
use crate::utils::jwt::{JwtManager, TokenPair};
use crate::utils::password::{PasswordChangeInput, PasswordManager};
use chrono::{Duration, Utc};
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;
use validator::Validate;

/// 認証サービス
pub struct AuthService {
    user_repo: Arc<UserRepository>,
    refresh_token_repo: Arc<RefreshTokenRepository>,
    password_reset_token_repo: Arc<PasswordResetTokenRepository>,
    password_manager: Arc<PasswordManager>,
    jwt_manager: Arc<JwtManager>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<UserRepository>,
        refresh_token_repo: Arc<RefreshTokenRepository>,
        password_reset_token_repo: Arc<PasswordResetTokenRepository>,
        password_manager: Arc<PasswordManager>,
        jwt_manager: Arc<JwtManager>,
    ) -> Self {
        Self {
            user_repo,
            refresh_token_repo,
            password_reset_token_repo,
            password_manager,
            jwt_manager,
        }
    }

    // --- ユーザー登録・ログイン ---

    /// ユーザー登録
    pub async fn signup(&self, signup_data: SignupRequest) -> AppResult<AuthResponse> {
        // バリデーション
        signup_data
            .validate()
            .map_err(|e| AppError::ValidationError(format!("Validation failed: {}", e)))?;

        // メールアドレスとユーザー名の重複チェック
        if self.user_repo.is_email_taken(&signup_data.email).await? {
            return Err(AppError::Conflict(
                "email address is already registered".to_string(),
            ));
        }

        if self
            .user_repo
            .is_username_taken(&signup_data.username)
            .await?
        {
            return Err(AppError::Conflict("username is already taken".to_string()));
        }

        // パスワード強度チェック
        self.password_manager
            .validate_password_strength(&signup_data.password)
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // パスワードハッシュ化
        let password_hash = self
            .password_manager
            .hash_password(&signup_data.password)
            .map_err(|e| {
                AppError::InternalServerError(format!("Password hashing failed: {}", e))
            })?;

        // ユーザー作成
        let create_user = CreateUser {
            email: signup_data.email.clone(),
            username: signup_data.username.clone(),
            password_hash,
            is_active: Some(true),
            email_verified: Some(false), // メール認証は別途実装
        };

        let user = self.user_repo.create(create_user).await?;

        info!(
            user_id = %user.id,
            email = %user.email,
            username = %user.username,
            "User registered successfully"
        );

        // JWT トークン生成
        let user_claims = UserClaims::from(user.clone());
        let token_pair = self.create_token_pair(&user_claims).await?;

        Ok(AuthResponse {
            user: user.into(),
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

        info!(
            user_id = %user.id,
            email = %user.email,
            "User signed in successfully"
        );

        // JWT トークン生成
        let user_claims = UserClaims::from(user.clone());
        let token_pair = self.create_token_pair(&user_claims).await?;

        Ok(AuthResponse {
            user: user.into(),
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

        // ユーザー情報取得
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // ユーザーがアクティブかチェック
        if !user.can_authenticate() {
            warn!(user_id = %user_id, "Token refresh for inactive user");
            return Err(AppError::Unauthorized("Account is inactive".to_string()));
        }

        // 新しいトークンペアを生成（リフレッシュトークンローテーション）
        let user_claims = UserClaims::from(user.clone());

        // 新しいリフレッシュトークンを生成
        let new_refresh_token = self
            .jwt_manager
            .generate_refresh_token(user.id, token_claims.ver + 1)
            .map_err(|e| {
                AppError::InternalServerError(format!("Token generation failed: {}", e))
            })?;

        let new_refresh_token_hash = self.hash_token(&new_refresh_token);
        let refresh_expires_at = Utc::now() + Duration::days(7);

        let create_refresh_token = CreateRefreshToken {
            user_id: user.id,
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

        let token_pair = TokenPair::new(
            access_token.clone(),
            new_refresh_token.clone(),
            15, // 15分
            7,  // 7日
        );

        Ok(TokenRefreshResponse {
            user: user.into(),
            tokens: token_pair,
            access_token,
            refresh_token: new_refresh_token,
            access_token_expires_in: 15 * 60,           // 15分（秒）
            refresh_token_expires_in: 7 * 24 * 60 * 60, // 7日（秒）
            token_type: "Bearer".to_string(),
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
    pub async fn get_current_user(&self, user_id: Uuid) -> AppResult<SafeUser> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user.into())
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

        Ok(TokenPair::new(
            access_token,
            refresh_token,
            15, // 15分
            7,  // 7日
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
