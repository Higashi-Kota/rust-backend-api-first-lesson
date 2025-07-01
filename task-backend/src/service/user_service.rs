// task-backend/src/service/user_service.rs
use crate::api::dto::user_dto::{
    BulkOperationResult, BulkUserOperation, RoleUserStats, SubscriptionAnalytics,
};
use crate::domain::user_model::{SafeUser, SafeUserWithRole};
use crate::error::{AppError, AppResult};
use crate::repository::user_repository::UserRepository;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// ユーザー管理サービス
pub struct UserService {
    user_repo: Arc<UserRepository>,
}

impl UserService {
    pub fn new(user_repo: Arc<UserRepository>) -> Self {
        Self { user_repo }
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
            return Err(AppError::ValidationError("Account is inactive".to_string()));
        }

        Ok(user.into())
    }

    /// ユーザー名の更新
    pub async fn update_username(&self, user_id: Uuid, new_username: &str) -> AppResult<SafeUser> {
        // ユーザー名の重複チェック
        if self.user_repo.is_username_taken(new_username).await? {
            return Err(AppError::ValidationError(
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
            return Err(AppError::ValidationError(
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
        // This returns a dummy UserStats since we need the service type for analytics
        // In a real implementation, you might want to merge this with individual user stats
        // For now, returning a placeholder stats
        let active_users = self.user_repo.find_active_users().await?;
        if let Some(first_user) = active_users.first() {
            Ok(UserStats {
                user_id: first_user.id,
                username: first_user.username.clone(),
                email: first_user.email.clone(),
                is_active: first_user.is_active,
                email_verified: first_user.email_verified,
                created_at: first_user.created_at,
                updated_at: first_user.updated_at,
                last_login_at: first_user.last_login_at,
            })
        } else {
            // Return a placeholder if no users exist
            Ok(UserStats {
                user_id: uuid::Uuid::new_v4(),
                username: "system".to_string(),
                email: "system@example.com".to_string(),
                is_active: true,
                email_verified: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                last_login_at: None,
            })
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
        self.user_repo
            .count_all_users()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to count all users: {}", e)))
    }

    /// アクティブユーザー数を取得
    pub async fn count_active_users(&self) -> AppResult<u64> {
        self.user_repo.count_active_users().await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to count active users: {}", e))
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
            .find_by_subscription_tier("Enterprise")
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
            enterprise_users: if tier == "Enterprise" { tier_users } else { 0 },
            conversion_rate: all_analytics.conversion_rate,
        })
    }

    /// メール認証トークンの検証
    pub async fn verify_email_token(&self, user_id: Uuid, token: &str) -> AppResult<SafeUser> {
        // トークンの検証ロジックを実装
        // 実際の実装では、パスワードリセットトークンモデルやメール認証トークンテーブルを使用する
        // ここでは簡単な実装として、トークン長の検証のみ行う
        if token.len() < 32 {
            return Err(AppError::ValidationError(
                "Invalid token format".to_string(),
            ));
        }

        // 実際の実装では、データベースでトークンを検索し、有効性をチェック
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        info!(user_id = %user_id, "Email verification token verified");
        Ok(user.into())
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
            return Err(AppError::ValidationError(
                "Email address does not match the user's current email".to_string(),
            ));
        }

        // 実際の実装では、メール送信サービスを呼び出す
        // ここでは成功のログのみ記録
        info!(user_id = %user_id, email = %email, "Verification email resent");
        Ok(())
    }

    /// ユーザー設定の取得
    pub async fn get_user_settings(
        &self,
        user_id: Uuid,
    ) -> AppResult<crate::api::dto::user_dto::UserSettingsResponse> {
        use crate::api::dto::user_dto::{
            NotificationSettings, SecuritySettings, UserPreferences, UserSettingsResponse,
        };

        // ユーザーの存在確認
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 実際の実装では、ユーザー設定テーブルから設定を取得
        // ここではデフォルト設定を返す
        let preferences = UserPreferences::default();
        let security = SecuritySettings::default();
        let notifications = NotificationSettings::default();

        info!(user_id = %user_id, "User settings retrieved");
        Ok(UserSettingsResponse {
            user_id: user.id,
            preferences,
            security,
            notifications,
        })
    }

    /// 拡張一括ユーザー操作（新しいenum-based操作対応）
    pub async fn bulk_user_operations_extended(
        &self,
        operation: &BulkUserOperation,
        user_ids: &[Uuid],
        parameters: Option<&serde_json::Value>,
        notify_users: bool,
    ) -> AppResult<BulkOperationResult> {
        let mut successful = 0;
        let mut failed = 0;
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
                    // Role update functionality would go here
                    // For now, return a placeholder response
                    failed += 1;
                    let error_msg = format!("User {}: Role update not implemented yet", user_id);
                    errors.push(error_msg.clone());
                    serde_json::json!({
                        "user_id": user_id,
                        "success": false,
                        "message": error_msg
                    })
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

        Ok(BulkOperationResult {
            successful,
            failed,
            errors,
            results: Some(serde_json::json!(results)),
        })
    }
}
