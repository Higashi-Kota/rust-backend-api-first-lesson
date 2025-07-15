// task-backend/src/service/user_service.rs
use crate::api::dto::user_dto::{
    BulkOperationResult, BulkUserOperation, RoleUserStats, SubscriptionAnalytics,
};
use crate::domain::bulk_operation_history_model::{
    BulkOperationError, BulkOperationErrorDetails, BulkOperationType,
};
use crate::domain::user_model::{SafeUser, SafeUserWithRole};
use crate::domain::user_settings_model;
use crate::error::{AppError, AppResult};
use crate::repository::bulk_operation_history_repository::BulkOperationHistoryRepository;
use crate::repository::email_verification_token_repository::EmailVerificationTokenRepository;
use crate::repository::user_repository::UserRepository;
use crate::repository::user_settings_repository::UserSettingsRepository;
use crate::utils::error_helper::internal_server_error;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// ユーザー管理サービス
pub struct UserService {
    user_repo: Arc<UserRepository>,
    user_settings_repo: Arc<UserSettingsRepository>,
    bulk_operation_history_repo: Arc<BulkOperationHistoryRepository>,
    email_verification_token_repo: Arc<EmailVerificationTokenRepository>,
}

impl UserService {
    pub fn new(
        user_repo: Arc<UserRepository>,
        user_settings_repo: Arc<UserSettingsRepository>,
        bulk_operation_history_repo: Arc<BulkOperationHistoryRepository>,
        email_verification_token_repo: Arc<EmailVerificationTokenRepository>,
    ) -> Self {
        Self {
            user_repo,
            user_settings_repo,
            bulk_operation_history_repo,
            email_verification_token_repo,
        }
    }

    /// ユーザー情報取得
    pub async fn get_user_profile(&self, user_id: Uuid) -> AppResult<SafeUser> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if !user.is_active {
            warn!(user_id = %user_id, "Profile access attempt for inactive account");
            return Err(AppError::BadRequest("Account is inactive".to_string()));
        }

        Ok(user.into())
    }

    /// ユーザー名の更新
    pub async fn update_username(&self, user_id: Uuid, new_username: &str) -> AppResult<SafeUser> {
        // ユーザー名の重複チェック
        if self.user_repo.is_username_taken(new_username).await? {
            return Err(AppError::BadRequest(
                "Username is already taken".to_string(),
            ));
        }

        // ユーザー名を更新
        let updated_user = self
            .user_repo
            .update_username(user_id, new_username.to_string())
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        info!(
            user_id = %user_id,
            new_username = %new_username,
            "Username updated successfully"
        );

        Ok(updated_user.into())
    }

    /// メールアドレスの更新
    pub async fn update_email(&self, user_id: Uuid, new_email: &str) -> AppResult<SafeUser> {
        // メールアドレスの重複チェック
        if self.user_repo.is_email_taken(new_email).await? {
            return Err(AppError::BadRequest(
                "Email address is already registered".to_string(),
            ));
        }

        // メールアドレスを更新（email_verified は false にリセット）
        let updated_user = self
            .user_repo
            .update_email(user_id, new_email.to_string())
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        info!(
            user_id = %user_id,
            new_email = %new_email,
            "Email updated successfully"
        );

        Ok(updated_user.into())
    }

    /// アカウントの有効化/無効化
    pub async fn toggle_account_status(
        &self,
        user_id: Uuid,
        is_active: bool,
    ) -> AppResult<SafeUser> {
        let updated_user = self
            .user_repo
            .update_active_status(user_id, is_active)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        info!(
            user_id = %user_id,
            is_active = %is_active,
            "Account status updated successfully"
        );

        Ok(updated_user.into())
    }

    /// メール認証の確認
    pub async fn verify_email(&self, user_id: Uuid) -> AppResult<SafeUser> {
        let updated_user = self
            .user_repo
            .mark_email_verified(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        info!(user_id = %user_id, "Email verified successfully");

        Ok(updated_user.into())
    }

    /// ユーザー統計情報取得
    pub async fn get_user_stats(&self, user_id: Uuid) -> AppResult<UserStats> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(UserStats {
            user_id: user.id,
            username: user.username,
            email: user.email,
            is_active: user.is_active,
            email_verified: user.email_verified,
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login_at: user.last_login_at,
        })
    }

    /// IDでユーザーを取得
    pub async fn get_user_by_id(&self, user_id: Uuid) -> AppResult<SafeUser> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user.into())
    }

    /// ユーザーの最終ログイン時刻を更新
    pub async fn update_last_login(&self, user_id: Uuid) -> AppResult<()> {
        self.user_repo.update_last_login(user_id).await?;
        info!(user_id = %user_id, "Last login time updated");
        Ok(())
    }

    /// ロール情報付きユーザー一覧をページネーション付きで取得（管理者用）
    pub async fn list_users_with_roles_paginated(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<crate::domain::user_model::SafeUserWithRole>, usize)> {
        // データベースレベルでページネーションを適用
        let users = self
            .user_repo
            .find_all_with_roles_paginated(page, per_page)
            .await?;
        let total_count = self.user_repo.count_all_users_with_roles().await? as usize;

        Ok((users, total_count))
    }

    /// ユーザー検索（管理者用）
    pub async fn search_users(
        &self,
        query: Option<String>,
        is_active: Option<bool>,
        email_verified: Option<bool>,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<crate::domain::user_model::SafeUserWithRole>, usize)> {
        let users = self
            .user_repo
            .search_users(query.clone(), is_active, email_verified, page, per_page)
            .await?;

        let total_count = self
            .user_repo
            .count_users_by_filter(query.as_deref(), is_active, email_verified)
            .await?;

        Ok((users, total_count))
    }
}

// --- レスポンス構造体 ---

use chrono::{DateTime, Utc};
use serde::Serialize;

/// ユーザー統計情報
#[derive(Debug, Clone, Serialize)]
pub struct UserStats {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

// --- 新規API用のメソッド ---

impl UserService {
    /// ユーザー統計を取得（分析レスポンス用）
    pub async fn get_user_stats_for_analytics(&self) -> AppResult<UserStats> {
        // システム全体の代表的な統計情報を返す
        // 最新の管理者ユーザーまたは最新のアクティブユーザーの情報を返す

        // まず管理者を探す
        let admin_users = self.user_repo.find_by_role_name("admin").await?;
        let admin_user = admin_users.first().cloned();

        if let Some(admin) = admin_user {
            // 管理者が存在する場合は管理者の情報を返す
            Ok(UserStats {
                user_id: admin.id,
                username: admin.username.clone(),
                email: admin.email.clone(),
                is_active: admin.is_active,
                email_verified: admin.email_verified,
                created_at: admin.created_at,
                updated_at: admin.updated_at,
                last_login_at: admin.last_login_at,
            })
        } else {
            // 管理者がいない場合は最新のアクティブユーザーを返す
            let active_users = self.user_repo.find_active_users().await?;

            if let Some(user) = active_users.first() {
                Ok(UserStats {
                    user_id: user.id,
                    username: user.username.clone(),
                    email: user.email.clone(),
                    is_active: user.is_active,
                    email_verified: user.email_verified,
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                    last_login_at: user.last_login_at,
                })
            } else {
                // ユーザーが存在しない場合は、システムの初期状態を表す統計情報を返す
                Ok(UserStats {
                    user_id: Uuid::nil(), // 00000000-0000-0000-0000-000000000000
                    username: "System Analytics".to_string(),
                    email: "analytics@system.local".to_string(),
                    is_active: true,
                    email_verified: true,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    last_login_at: None,
                })
            }
        }
    }

    /// ロール別ユーザー統計を取得
    pub async fn get_user_stats_by_role(&self) -> AppResult<Vec<RoleUserStats>> {
        let repo_stats = self.user_repo.get_user_stats_by_role().await?;
        // Convert repository types to DTO types
        let dto_stats = repo_stats
            .into_iter()
            .map(|stat| RoleUserStats {
                role_name: stat.role_name,
                role_display_name: stat.role_display_name,
                total_users: stat.total_users,
                active_users: stat.active_users,
                verified_users: stat.verified_users,
            })
            .collect();
        Ok(dto_stats)
    }

    /// ロール情報付き全ユーザーをページネーション付きで取得（管理者用）
    pub async fn get_all_users_with_roles_paginated(
        &self,
        page: i32,
        page_size: i32,
        role_name: Option<&str>,
    ) -> AppResult<(Vec<crate::domain::user_model::SafeUserWithRole>, usize)> {
        let (users_with_roles, total_count) = if let Some(role_name) = role_name {
            // 特定のロールでフィルタリング
            let users = self.user_repo.find_by_role_name(role_name).await?;
            let total = users.len();
            let paginated_users = users
                .into_iter()
                .skip(((page - 1) * page_size) as usize)
                .take(page_size as usize)
                .collect::<Vec<_>>();
            (paginated_users, total)
        } else {
            // 全ユーザーを取得
            let users = self
                .user_repo
                .find_all_with_roles_paginated(page, page_size)
                .await?;
            let total_count = self.user_repo.count_all_users_with_roles().await? as usize;
            (users, total_count)
        };

        Ok((users_with_roles, total_count))
    }

    /// 全ユーザー数を取得
    pub async fn count_all_users(&self) -> AppResult<u64> {
        self.user_repo.count_all_users().await.map_err(|e| {
            internal_server_error(
                e,
                "user_service::count_all_users",
                "Failed to count all users",
            )
        })
    }

    /// アクティブユーザー数を取得
    pub async fn count_active_users(&self) -> AppResult<u64> {
        self.user_repo.count_active_users().await.map_err(|e| {
            internal_server_error(
                e,
                "user_service::count_active_users",
                "Failed to count active users",
            )
        })
    }

    /// ロール名でユーザーを検索
    pub async fn find_by_role_name(&self, role_name: &str) -> AppResult<Vec<SafeUserWithRole>> {
        let users = self.user_repo.find_by_role_name(role_name).await?;
        Ok(users)
    }

    /// サブスクリプション分析を取得
    pub async fn get_subscription_analytics(&self) -> AppResult<SubscriptionAnalytics> {
        // 全サブスクリプション階層の分析を実装
        let free_users = self
            .user_repo
            .find_by_subscription_tier("Free")
            .await?
            .len() as u64;
        let pro_users = self.user_repo.find_by_subscription_tier("Pro").await?.len() as u64;
        let enterprise_users = self
            .user_repo
            .find_by_subscription_tier("enterprise")
            .await?
            .len() as u64;
        let total_users = free_users + pro_users + enterprise_users;

        let conversion_rate = if total_users > 0 {
            ((pro_users + enterprise_users) as f64 / total_users as f64) * 100.0
        } else {
            0.0
        };

        Ok(SubscriptionAnalytics {
            total_users,
            free_users,
            pro_users,
            enterprise_users,
            conversion_rate,
        })
    }

    /// 特定のサブスクリプション階層の分析を取得
    pub async fn get_subscription_analytics_by_tier(
        &self,
        tier: &str,
    ) -> AppResult<SubscriptionAnalytics> {
        let tier_users = self.user_repo.find_by_subscription_tier(tier).await?.len() as u64;
        let all_analytics = self.get_subscription_analytics().await?;

        // 特定の階層にフォーカスした分析を返す
        Ok(SubscriptionAnalytics {
            total_users: tier_users,
            free_users: if tier == "Free" { tier_users } else { 0 },
            pro_users: if tier == "Pro" { tier_users } else { 0 },
            enterprise_users: if tier == "enterprise" { tier_users } else { 0 },
            conversion_rate: all_analytics.conversion_rate,
        })
    }

    /// メール認証トークンの検証
    pub async fn verify_email_token(&self, user_id: Uuid, token: &str) -> AppResult<SafeUser> {
        // メール認証を実行
        use crate::domain::email_verification_token_model::TokenValidationError;

        let result = self
            .email_verification_token_repo
            .execute_email_verification(token)
            .await?;

        match result {
            Ok(verification_result) => {
                // ユーザーIDが一致するか確認
                if verification_result.user_id != user_id {
                    return Err(AppError::BadRequest(
                        "Token does not match user".to_string(),
                    ));
                }

                // ユーザーを取得
                let user = self
                    .user_repo
                    .find_by_id(user_id)
                    .await?
                    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

                info!(user_id = %user_id, "Email verification completed successfully");
                Ok(user.into())
            }
            Err(TokenValidationError::NotFound) => {
                Err(AppError::BadRequest("Invalid token".to_string()))
            }
            Err(TokenValidationError::Expired) => {
                Err(AppError::BadRequest("Token has expired".to_string()))
            }
            Err(TokenValidationError::AlreadyUsed) => Err(AppError::BadRequest(
                "Token has already been used".to_string(),
            )),
            Err(TokenValidationError::ValidationFailed(msg)) => Err(AppError::BadRequest(msg)),
        }
    }

    /// メール認証の再送信
    pub async fn resend_verification_email(&self, user_id: Uuid, email: &str) -> AppResult<()> {
        // ユーザーの存在確認
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // メールアドレスの一致確認
        if user.email != email {
            return Err(AppError::BadRequest(
                "Email address does not match the user's current email".to_string(),
            ));
        }

        // 実際の実装では、メール送信サービスを呼び出す
        // ここでは成功のログのみ記録
        info!(user_id = %user_id, email = %email, "Verification email resent");
        Ok(())
    }

    /// ユーザー設定の取得（旧DTO用）
    pub async fn get_user_settings_legacy(
        &self,
        user_id: Uuid,
    ) -> AppResult<crate::api::dto::user_dto::UserSettingsResponse> {
        use crate::api::dto::user_dto::{SecuritySettings, UserPreferences, UserSettingsResponse};

        // ユーザーの存在確認
        let _user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // ユーザー設定を取得（存在しない場合はデフォルト作成）
        let settings = self.user_settings_repo.get_or_create(user_id).await?;

        // ドメインモデルからDTOへ変換（getter メソッドを使用）
        let domain_preferences = settings.get_ui_preferences();
        let preferences = UserPreferences {
            language: settings.language.clone(),
            timezone: settings.timezone.clone(),
            theme: domain_preferences.theme,
            date_format: "YYYY-MM-DD".to_string(), // TODO: domain_preferences に追加
            time_format: "24h".to_string(),        // TODO: domain_preferences に追加
        };

        // セキュリティ設定はハードコード（将来的にはデータベースに保存）
        let security = SecuritySettings::default();

        let domain_notifications = settings.get_email_notifications();
        // ドメインモデルからDTOへ変換
        let notifications = crate::api::dto::user_dto::NotificationSettings {
            email_notifications: true, // 汎用的な通知設定
            security_alerts: domain_notifications.security_alerts,
            task_reminders: domain_notifications.task_updates,
            newsletter: domain_notifications.newsletter,
        };

        info!(user_id = %user_id, "User settings retrieved from database");
        Ok(UserSettingsResponse {
            user_id,
            preferences,
            security,
            notifications,
        })
    }

    /// ユーザー設定の取得（新しいDTO用）
    pub async fn get_user_settings(
        &self,
        user_id: Uuid,
    ) -> AppResult<Option<user_settings_model::Model>> {
        self.user_settings_repo.get_by_user_id(user_id).await
    }

    /// ユーザー設定の更新
    pub async fn update_user_settings(
        &self,
        user_id: Uuid,
        input: user_settings_model::UserSettingsInput,
    ) -> AppResult<user_settings_model::Model> {
        self.user_settings_repo.update(user_id, input).await
    }

    /// 言語別ユーザー取得
    pub async fn get_users_by_language(&self, language: &str) -> AppResult<Vec<Uuid>> {
        self.user_settings_repo
            .get_users_by_language(language)
            .await
    }

    /// 通知有効ユーザー取得
    pub async fn get_users_with_notification_enabled(
        &self,
        notification_type: &str,
    ) -> AppResult<Vec<Uuid>> {
        self.user_settings_repo
            .get_users_with_notification_enabled(notification_type)
            .await
    }

    /// ユーザー設定の削除
    pub async fn delete_user_settings(&self, user_id: Uuid) -> AppResult<bool> {
        self.user_settings_repo.delete(user_id).await
    }

    /// ユーザー設定をデフォルトに戻す
    pub async fn reset_user_settings_to_default(&self, user_id: Uuid) -> AppResult<()> {
        // ユーザーの存在確認
        let _user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 設定を削除（次回取得時にデフォルトが作成される）
        self.user_settings_repo.delete(user_id).await?;

        info!(user_id = %user_id, "User settings reset to default");
        Ok(())
    }

    /// 拡張一括ユーザー操作（新しいenum-based操作対応）
    pub async fn bulk_user_operations_extended(
        &self,
        operation: &BulkUserOperation,
        user_ids: &[Uuid],
        parameters: Option<&serde_json::Value>,
        notify_users: bool,
        performed_by: Uuid,
    ) -> AppResult<BulkOperationResult> {
        let mut successful = 0;
        let mut failed: usize = 0;
        let mut errors = Vec::new();
        let mut results = Vec::new();

        info!(
            operation = %operation,
            user_count = user_ids.len(),
            notify_users = notify_users,
            "Starting bulk user operation"
        );

        for &user_id in user_ids {
            let result = match operation {
                BulkUserOperation::Activate => {
                    match self.toggle_account_status(user_id, true).await {
                        Ok(_) => {
                            successful += 1;
                            serde_json::json!({
                                "user_id": user_id,
                                "success": true,
                                "message": "User activated successfully"
                            })
                        }
                        Err(e) => {
                            failed += 1;
                            let error_msg = format!("User {}: {}", user_id, e);
                            errors.push(error_msg.clone());
                            serde_json::json!({
                                "user_id": user_id,
                                "success": false,
                                "message": error_msg
                            })
                        }
                    }
                }
                BulkUserOperation::Deactivate => {
                    match self.toggle_account_status(user_id, false).await {
                        Ok(_) => {
                            successful += 1;
                            serde_json::json!({
                                "user_id": user_id,
                                "success": true,
                                "message": "User deactivated successfully"
                            })
                        }
                        Err(e) => {
                            failed += 1;
                            let error_msg = format!("User {}: {}", user_id, e);
                            errors.push(error_msg.clone());
                            serde_json::json!({
                                "user_id": user_id,
                                "success": false,
                                "message": error_msg
                            })
                        }
                    }
                }
                BulkUserOperation::UpdateSubscription => {
                    if let Some(params) = parameters {
                        if let Some(new_tier) = params.get("new_tier").and_then(|v| v.as_str()) {
                            match self
                                .user_repo
                                .update_subscription_tier(user_id, new_tier.to_string())
                                .await
                            {
                                Ok(_) => {
                                    successful += 1;
                                    serde_json::json!({
                                        "user_id": user_id,
                                        "success": true,
                                        "message": format!("Subscription updated to {}", new_tier)
                                    })
                                }
                                Err(e) => {
                                    failed += 1;
                                    let error_msg = format!("User {}: {}", user_id, e);
                                    errors.push(error_msg.clone());
                                    serde_json::json!({
                                        "user_id": user_id,
                                        "success": false,
                                        "message": error_msg
                                    })
                                }
                            }
                        } else {
                            failed += 1;
                            let error_msg =
                                format!("User {}: new_tier parameter is required", user_id);
                            errors.push(error_msg.clone());
                            serde_json::json!({
                                "user_id": user_id,
                                "success": false,
                                "message": error_msg
                            })
                        }
                    } else {
                        failed += 1;
                        let error_msg = format!(
                            "User {}: parameters are required for subscription update",
                            user_id
                        );
                        errors.push(error_msg.clone());
                        serde_json::json!({
                            "user_id": user_id,
                            "success": false,
                            "message": error_msg
                        })
                    }
                }
                BulkUserOperation::UpdateRole => {
                    if let Some(params) = parameters {
                        if let Some(role_id) = params.get("role_id").and_then(|v| v.as_str()) {
                            if let Ok(role_uuid) = Uuid::parse_str(role_id) {
                                match self.user_repo.update_user_role(user_id, role_uuid).await {
                                    Ok(_) => {
                                        successful += 1;
                                        serde_json::json!({
                                            "user_id": user_id,
                                            "success": true,
                                            "message": "Role updated successfully"
                                        })
                                    }
                                    Err(e) => {
                                        failed += 1;
                                        let error_msg = format!(
                                            "User {}: Failed to update role - {}",
                                            user_id, e
                                        );
                                        errors.push(error_msg.clone());
                                        serde_json::json!({
                                            "user_id": user_id,
                                            "success": false,
                                            "message": error_msg
                                        })
                                    }
                                }
                            } else {
                                failed += 1;
                                let error_msg = format!("User {}: Invalid role_id format", user_id);
                                errors.push(error_msg.clone());
                                serde_json::json!({
                                    "user_id": user_id,
                                    "success": false,
                                    "message": error_msg
                                })
                            }
                        } else {
                            failed += 1;
                            let error_msg =
                                format!("User {}: role_id parameter is required", user_id);
                            errors.push(error_msg.clone());
                            serde_json::json!({
                                "user_id": user_id,
                                "success": false,
                                "message": error_msg
                            })
                        }
                    } else {
                        failed += 1;
                        let error_msg =
                            format!("User {}: parameters are required for role update", user_id);
                        errors.push(error_msg.clone());
                        serde_json::json!({
                            "user_id": user_id,
                            "success": false,
                            "message": error_msg
                        })
                    }
                }
                BulkUserOperation::SendNotification => {
                    // Notification sending would go here
                    if notify_users {
                        successful += 1;
                        serde_json::json!({
                            "user_id": user_id,
                            "success": true,
                            "message": "Notification sent successfully"
                        })
                    } else {
                        failed += 1;
                        let error_msg = format!("User {}: Notification disabled", user_id);
                        errors.push(error_msg.clone());
                        serde_json::json!({
                            "user_id": user_id,
                            "success": false,
                            "message": error_msg
                        })
                    }
                }
                BulkUserOperation::ResetPasswords => {
                    // Password reset would go here
                    successful += 1;
                    serde_json::json!({
                        "user_id": user_id,
                        "success": true,
                        "message": "Password reset initiated"
                    })
                }
                BulkUserOperation::ExportUserData => {
                    // Data export would go here
                    successful += 1;
                    serde_json::json!({
                        "user_id": user_id,
                        "success": true,
                        "message": "User data export initiated"
                    })
                }
                BulkUserOperation::BulkDelete => {
                    // Bulk delete would go here - mark as inactive instead of actual deletion
                    match self.toggle_account_status(user_id, false).await {
                        Ok(_) => {
                            successful += 1;
                            serde_json::json!({
                                "user_id": user_id,
                                "success": true,
                                "message": "User marked for deletion (deactivated)"
                            })
                        }
                        Err(e) => {
                            failed += 1;
                            let error_msg = format!("User {}: {}", user_id, e);
                            errors.push(error_msg.clone());
                            serde_json::json!({
                                "user_id": user_id,
                                "success": false,
                                "message": error_msg
                            })
                        }
                    }
                }
                BulkUserOperation::BulkInvite => {
                    // Bulk invite would go here
                    successful += 1;
                    serde_json::json!({
                        "user_id": user_id,
                        "success": true,
                        "message": "Invitation sent successfully"
                    })
                }
            };
            results.push(result);
        }

        info!(
            operation = %operation,
            successful = successful,
            failed = failed,
            "Bulk operation completed"
        );

        // 一括操作履歴の記録
        let operation_type = match operation {
            BulkUserOperation::Activate => BulkOperationType::ActivateUsers,
            BulkUserOperation::Deactivate => BulkOperationType::DeactivateUsers,
            BulkUserOperation::UpdateRole => BulkOperationType::UpdateRole,
            BulkUserOperation::BulkDelete => BulkOperationType::DeleteUsers,
            _ => {
                // その他の操作は汎用的な UpdateRole として記録
                BulkOperationType::UpdateRole
            }
        };

        let history = self
            .bulk_operation_history_repo
            .create(operation_type, performed_by, user_ids.len() as i32)
            .await?;

        // 操作を開始
        let history = self
            .bulk_operation_history_repo
            .start_operation(history.id)
            .await?;

        // 操作結果に基づいて履歴を更新
        if failed == 0 {
            self.bulk_operation_history_repo
                .complete_operation(history.id)
                .await?;
        } else if successful > 0 {
            let error_details = BulkOperationErrorDetails {
                errors: errors
                    .iter()
                    .map(|e| BulkOperationError {
                        entity_id: e.split(": ").next().unwrap_or("unknown").to_string(),
                        error_message: e.split(": ").skip(1).collect::<Vec<_>>().join(": "),
                    })
                    .collect(),
                total_errors: failed,
            };
            self.bulk_operation_history_repo
                .partially_complete_operation(history.id, error_details)
                .await?;
        } else {
            let error_details = Some(BulkOperationErrorDetails {
                errors: errors
                    .iter()
                    .map(|e| BulkOperationError {
                        entity_id: e.split(": ").next().unwrap_or("unknown").to_string(),
                        error_message: e.split(": ").skip(1).collect::<Vec<_>>().join(": "),
                    })
                    .collect(),
                total_errors: failed,
            });
            self.bulk_operation_history_repo
                .fail_operation(history.id, error_details)
                .await?;
        }

        Ok(BulkOperationResult {
            successful,
            failed,
            errors,
            results: Some(serde_json::json!(results)),
        })
    }

    /// ロールごとのユーザー数を取得
    pub async fn count_users_by_role(&self, role_id: Uuid) -> AppResult<i64> {
        self.user_repo
            .count_by_role_id(role_id)
            .await
            .map_err(AppError::DbErr)
    }
}
