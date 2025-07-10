// task-backend/src/features/security/services/role.rs
use super::super::models::role::{RoleName, RoleWithPermissions};
use super::super::repositories::role::{CreateRoleData, RoleRepository, UpdateRoleData};
use crate::domain::user_model::{SafeUserWithRole, UserClaims};
use crate::error::{AppError, AppResult};
use crate::features::auth::repository::user_repository::UserRepository;
use crate::shared::dto::role_types::{CreateRoleInput, UpdateRoleInput};
use crate::utils::transaction::{execute_with_retry, RetryConfig};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

/// ロールサービス
// TODO: Phase 19で古い参照を削除後、#[allow(dead_code)]を削除
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RoleService {
    db: Arc<DatabaseConnection>,
    role_repository: Arc<RoleRepository>,
    user_repository: Arc<UserRepository>,
}

impl RoleService {
    /// 新しいロールサービスを作成
    pub fn new(
        db: Arc<DatabaseConnection>,
        role_repository: Arc<RoleRepository>,
        user_repository: Arc<UserRepository>,
    ) -> Self {
        Self {
            db,
            role_repository,
            user_repository,
        }
    }

    // --- 基本CRUD操作 ---

    /// すべてのロールを取得
    pub async fn list_all_roles(&self) -> AppResult<Vec<RoleWithPermissions>> {
        info!("Fetching all roles");

        self.role_repository.find_all().await.map_err(|e| {
            error!(error = %e, "Failed to fetch all roles");
            AppError::InternalServerError("Failed to fetch roles".to_string())
        })
    }

    /// アクティブなロールのみ取得
    pub async fn list_active_roles(&self) -> AppResult<Vec<RoleWithPermissions>> {
        info!("Fetching active roles");

        self.role_repository.find_all_active().await.map_err(|e| {
            error!(error = %e, "Failed to fetch active roles");
            AppError::InternalServerError("Failed to fetch active roles".to_string())
        })
    }

    /// ページネーション付きですべてのロールを取得
    pub async fn list_all_roles_paginated(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<RoleWithPermissions>, usize)> {
        info!(page = %page, per_page = %per_page, "Fetching all roles with pagination");

        // 現在の実装では全件取得してからページネーションしているが、
        // 将来的にロール数が増えた場合はリポジトリ層でページネーションを実装する
        let all_roles = self.role_repository.find_all().await.map_err(|e| {
            error!(error = %e, "Failed to fetch all roles");
            AppError::InternalServerError("Failed to fetch roles".to_string())
        })?;

        let total_count = all_roles.len();
        let page_size = std::cmp::min(per_page as usize, 100); // 最大100件に制限
        let offset = ((page - 1) * per_page) as usize;

        let paginated_roles = all_roles.into_iter().skip(offset).take(page_size).collect();

        Ok((paginated_roles, total_count))
    }

    /// ページネーション付きでアクティブなロールを取得
    pub async fn list_active_roles_paginated(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<RoleWithPermissions>, usize)> {
        info!(page = %page, per_page = %per_page, "Fetching active roles with pagination");

        let all_roles = self.role_repository.find_all_active().await.map_err(|e| {
            error!(error = %e, "Failed to fetch active roles");
            AppError::InternalServerError("Failed to fetch active roles".to_string())
        })?;

        let total_count = all_roles.len();
        let page_size = std::cmp::min(per_page as usize, 100); // 最大100件に制限
        let offset = ((page - 1) * per_page) as usize;

        let paginated_roles = all_roles.into_iter().skip(offset).take(page_size).collect();

        Ok((paginated_roles, total_count))
    }

    /// IDでロールを取得
    pub async fn get_role_by_id(&self, id: Uuid) -> AppResult<RoleWithPermissions> {
        info!(role_id = %id, "Fetching role by ID");

        self.role_repository
            .find_by_id(id)
            .await
            .map_err(|e| {
                error!(error = %e, role_id = %id, "Failed to fetch role by ID");
                AppError::InternalServerError("Failed to fetch role".to_string())
            })?
            .ok_or_else(|| {
                warn!(role_id = %id, "Role not found");
                AppError::NotFound("Role not found".to_string())
            })
    }

    /// サブスクリプション階層を指定してロールを取得
    pub async fn get_role_by_id_with_subscription(
        &self,
        id: Uuid,
        subscription_tier: crate::core::subscription_tier::SubscriptionTier,
    ) -> AppResult<RoleWithPermissions> {
        info!(role_id = %id, subscription_tier = %subscription_tier, "Fetching role by ID with subscription");

        self.role_repository
            .find_by_id_with_subscription(id, subscription_tier)
            .await
            .map_err(|e| {
                error!(error = %e, role_id = %id, "Failed to fetch role by ID with subscription");
                AppError::InternalServerError("Failed to fetch role".to_string())
            })?
            .ok_or_else(|| {
                warn!(role_id = %id, "Role not found");
                AppError::NotFound("Role not found".to_string())
            })
    }

    /// 新しいロールを作成（管理者のみ）
    pub async fn create_role(
        &self,
        requesting_user: &UserClaims,
        create_data: CreateRoleInput,
    ) -> AppResult<RoleWithPermissions> {
        // UserClaimsのcan_create_resourceメソッドを活用
        if !requesting_user.can_create_resource("role") {
            warn!(
                user_id = %requesting_user.user_id,
                resource = "role",
                "Insufficient permissions to create role"
            );
            return Err(AppError::Forbidden("Cannot create roles".to_string()));
        }

        // 入力バリデーション
        create_data.validate()?;

        info!(
            admin_id = %requesting_user.user_id,
            role_name = %create_data.name,
            "Creating new role"
        );

        // ロール名の形式チェック
        let role_name = create_data.name.to_lowercase();
        if role_name == "admin" || role_name == "member" {
            return Err(AppError::ValidationError(
                "Cannot create system roles (admin/member)".to_string(),
            ));
        }

        let repo_create_data = CreateRoleData {
            name: role_name,
            display_name: create_data.display_name,
            description: create_data.description,
            is_active: create_data.is_active.unwrap_or(true),
        };

        let created_role = self
            .role_repository
            .create(repo_create_data)
            .await
            .map_err(|e| {
                error!(error = %e, role_name = %create_data.name, "Failed to create role");
                AppError::InternalServerError("Failed to create role".to_string())
            })?;

        info!(
            admin_id = %requesting_user.user_id,
            role_id = %created_role.id,
            role_name = %created_role.name,
            "Role created successfully"
        );

        Ok(created_role)
    }

    /// ロールを更新（管理者のみ）
    pub async fn update_role(
        &self,
        requesting_user: &UserClaims,
        role_id: Uuid,
        update_data: UpdateRoleInput,
    ) -> AppResult<RoleWithPermissions> {
        // 管理者権限チェック
        self.check_admin_permission(requesting_user)?;

        // 入力バリデーション
        update_data.validate()?;

        info!(
            admin_id = %requesting_user.user_id,
            role_id = %role_id,
            "Updating role"
        );

        // 既存ロールの確認
        let existing_role = self.get_role_by_id(role_id).await?;

        // システムロールの保護
        if existing_role.name == RoleName::Admin || existing_role.name == RoleName::Member {
            if let Some(new_name) = &update_data.name {
                if new_name.to_lowercase() != existing_role.name.as_str() {
                    return Err(AppError::ValidationError(
                        "Cannot change system role names".to_string(),
                    ));
                }
            }
        }

        let repo_update_data = UpdateRoleData {
            name: update_data.name.map(|n| n.to_lowercase()),
            display_name: update_data.display_name,
            description: update_data.description,
            is_active: update_data.is_active,
        };

        let updated_role = self
            .role_repository
            .update(role_id, repo_update_data)
            .await
            .map_err(|e| {
                error!(error = %e, role_id = %role_id, "Failed to update role");
                AppError::InternalServerError("Failed to update role".to_string())
            })?;

        info!(
            admin_id = %requesting_user.user_id,
            role_id = %role_id,
            role_name = %updated_role.name,
            "Role updated successfully"
        );

        Ok(updated_role)
    }

    /// ロールを削除（管理者のみ）
    pub async fn delete_role(&self, requesting_user: &UserClaims, role_id: Uuid) -> AppResult<()> {
        // UserClaimsのcan_delete_resourceメソッドを活用
        if !requesting_user.can_delete_resource("role", None) {
            warn!(
                user_id = %requesting_user.user_id,
                resource = "role",
                role_id = %role_id,
                "Insufficient permissions to delete role"
            );
            return Err(AppError::Forbidden("Cannot delete roles".to_string()));
        }

        info!(
            admin_id = %requesting_user.user_id,
            role_id = %role_id,
            "Deleting role"
        );

        // 既存ロールの確認
        let existing_role = self.get_role_by_id(role_id).await?;

        // システムロールの削除を防ぐ
        if existing_role.name == RoleName::Admin || existing_role.name == RoleName::Member {
            return Err(AppError::ValidationError(
                "Cannot delete system roles (admin/member)".to_string(),
            ));
        }

        // このロールを使用しているユーザーがいるかチェック
        let users_with_role = self
            .user_repository
            .find_by_role_id(role_id)
            .await
            .map_err(|e| {
                error!(error = %e, role_id = %role_id, "Failed to check users with role");
                AppError::InternalServerError("Failed to check role usage".to_string())
            })?;

        if !users_with_role.is_empty() {
            return Err(AppError::Conflict(format!(
                "Cannot delete role '{}' as it is assigned to {} user(s)",
                existing_role.name,
                users_with_role.len()
            )));
        }

        self.role_repository.delete(role_id).await.map_err(|e| {
            error!(error = %e, role_id = %role_id, "Failed to delete role");
            AppError::InternalServerError("Failed to delete role".to_string())
        })?;

        info!(
            admin_id = %requesting_user.user_id,
            role_id = %role_id,
            role_name = %existing_role.name,
            "Role deleted successfully"
        );

        Ok(())
    }

    // --- 権限チェック機能 ---

    /// 管理者権限をチェック
    pub fn check_admin_permission(&self, user: &UserClaims) -> AppResult<()> {
        // UserClaimsの動的権限チェック機能を活用
        let permission_result = user.can_perform_action("roles", "manage", None);

        match permission_result {
            crate::core::permission::PermissionResult::Allowed { .. } => Ok(()),
            crate::core::permission::PermissionResult::Denied { reason } => {
                warn!(
                    user_id = %user.user_id,
                    role = ?user.role.as_ref().map(|r| &r.name),
                    reason = %reason,
                    "Insufficient permissions for role management"
                );
                Err(AppError::Forbidden(reason))
            }
        }
    }

    // --- ユーザーロール管理 ---

    /// ユーザーにロールを割り当て（管理者のみ）
    pub async fn assign_role_to_user(
        &self,
        requesting_user: &UserClaims,
        user_id: Uuid,
        role_id: Uuid,
    ) -> AppResult<SafeUserWithRole> {
        // UserClaimsのcan_update_resourceメソッドを活用 - ユーザーリソースの更新
        if !requesting_user.can_update_resource("user", Some(user_id)) {
            warn!(
                user_id = %requesting_user.user_id,
                target_user_id = %user_id,
                resource = "user",
                "Insufficient permissions to assign role to user"
            );
            return Err(AppError::Forbidden(
                "Cannot assign roles to users".to_string(),
            ));
        }

        info!(
            admin_id = %requesting_user.user_id,
            target_user_id = %user_id,
            role_id = %role_id,
            "Assigning role to user with retry capability"
        );

        // ロールの存在確認
        let role = self.get_role_by_id(role_id).await?;

        // execute_with_retryを使用してロール割り当てを実行
        let retry_config = RetryConfig {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 1000,
        };

        let user_repo = self.user_repository.clone();
        let updated_user = execute_with_retry(
            &self.db,
            move |_txn| {
                let user_repo = user_repo.clone();
                Box::pin(async move {
                    user_repo
                        .update_user_role(user_id, role_id)
                        .await
                        .map_err(|e| {
                            error!(error = %e, user_id = %user_id, role_id = %role_id, "Failed to assign role to user");
                            AppError::InternalServerError("Failed to assign role".to_string())
                        })?
                        .ok_or_else(|| {
                            warn!(user_id = %user_id, "User not found for role assignment");
                            AppError::NotFound("User not found".to_string())
                        })
                })
            },
            retry_config,
        )
        .await?;

        let user_with_role = updated_user.to_safe_user_with_role(role);

        info!(
            admin_id = %requesting_user.user_id,
            target_user_id = %user_id,
            role_id = %role_id,
            role_name = %user_with_role.role.name,
            "Role assigned to user successfully"
        );

        Ok(user_with_role)
    }
}

// --- 入力データ構造体 ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_role_input_validation() {
        // 正常なケース
        let valid_input = CreateRoleInput {
            name: "test_role".to_string(),
            display_name: "Test Role".to_string(),
            description: Some("Test description".to_string()),
            is_active: Some(true),
        };
        assert!(valid_input.validate().is_ok());

        // 空の名前
        let invalid_input = CreateRoleInput {
            name: "".to_string(),
            display_name: "Test Role".to_string(),
            description: None,
            is_active: None,
        };
        assert!(invalid_input.validate().is_err());

        // 長すぎる名前
        let invalid_input = CreateRoleInput {
            name: "a".repeat(51),
            display_name: "Test Role".to_string(),
            description: None,
            is_active: None,
        };
        assert!(invalid_input.validate().is_err());
    }

    #[test]
    fn test_update_role_input_validation() {
        // 正常なケース
        let valid_input = UpdateRoleInput {
            name: Some("updated_role".to_string()),
            display_name: Some("Updated Role".to_string()),
            description: Some(Some("Updated description".to_string())),
            is_active: Some(false),
        };
        assert!(valid_input.validate().is_ok());

        // 空のアップデート
        let valid_input = UpdateRoleInput::default();
        assert!(valid_input.validate().is_ok());
    }
}
