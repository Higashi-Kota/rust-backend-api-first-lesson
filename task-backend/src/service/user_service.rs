// task-backend/src/service/user_service.rs
use crate::domain::user_model::SafeUser;
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
        // 簡単な実装として、全ユーザーを取得して手動でページネーション
        let all_users = self.user_repo.find_all_with_roles().await?;
        let total_count = all_users.len();

        let offset = ((page - 1) * per_page) as usize;
        let limit = per_page as usize;

        let paginated_users = all_users.into_iter().skip(offset).take(limit).collect();

        Ok((paginated_users, total_count))
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

#[cfg(test)]
mod tests {
    // テストは実際の実装では適切なモックライブラリを使用する
    #[tokio::test]
    async fn test_user_service_creation() {
        // UserServiceの作成テスト
        // 実際のテストでは mock を使用
    }
}
