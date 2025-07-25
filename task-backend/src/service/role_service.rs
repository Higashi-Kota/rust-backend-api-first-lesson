// task-backend/src/service/role_service.rs
use crate::domain::role_model::{RoleName, RoleWithPermissions};
use crate::domain::user_model::{SafeUserWithRole, UserClaims};
use crate::error::{AppError, AppResult};
use crate::log_with_context;
use crate::repository::role_repository::{CreateRoleData, RoleRepository, UpdateRoleData};
use crate::repository::user_repository::UserRepository;
use crate::utils::transaction::{execute_with_retry, RetryConfig};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

/// ロールサービス
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

    /// ページネーション付きですべてのロールを取得
    pub async fn list_all_roles_paginated(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<RoleWithPermissions>, usize)> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Fetching all roles with pagination",
            "page" => page,
            "per_page" => per_page
        );

        // 現在の実装では全件取得してからページネーションしているが、
        // 将来的にロール数が増えた場合はリポジトリ層でページネーションを実装する
        let all_roles = self.role_repository.find_all().await.map_err(|e| {
            log_with_context!(
                tracing::Level::ERROR,
                "Failed to fetch all roles",
                "error" => &e.to_string()
            );
            AppError::InternalServerError("Failed to fetch roles".to_string())
        })?;

        let total_count = all_roles.len();
        // ページサイズはget_pagination()で制限済み
        let page_size = per_page as usize;
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
        log_with_context!(
            tracing::Level::DEBUG,
            "Fetching active roles with pagination",
            "page" => page,
            "per_page" => per_page
        );

        let all_roles = self.role_repository.find_all_active().await.map_err(|e| {
            log_with_context!(
                tracing::Level::ERROR,
                "Failed to fetch active roles",
                "error" => &e.to_string()
            );
            AppError::InternalServerError("Failed to fetch active roles".to_string())
        })?;

        let total_count = all_roles.len();
        // ページサイズはget_pagination()で制限済み
        let page_size = per_page as usize;
        let offset = ((page - 1) * per_page) as usize;

        let paginated_roles = all_roles.into_iter().skip(offset).take(page_size).collect();

        Ok((paginated_roles, total_count))
    }

    /// IDでロールを取得
    pub async fn get_role_by_id(&self, id: Uuid) -> AppResult<RoleWithPermissions> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Fetching role by ID",
            "role_id" => id
        );

        self.role_repository
            .find_by_id(id)
            .await
            .map_err(|e| {
                log_with_context!(
                    tracing::Level::ERROR,
                    "Failed to fetch role by ID",
                    "role_id" => id,
                    "error" => &e.to_string()
                );
                AppError::InternalServerError("Failed to fetch role".to_string())
            })?
            .ok_or_else(|| {
                log_with_context!(
                    tracing::Level::WARN,
                    "Role not found",
                    "role_id" => id
                );
                AppError::NotFound("Role not found".to_string())
            })
    }

    /// サブスクリプション階層を指定してロールを取得
    pub async fn get_role_by_id_with_subscription(
        &self,
        id: Uuid,
        subscription_tier: crate::domain::subscription_tier::SubscriptionTier,
    ) -> AppResult<RoleWithPermissions> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Fetching role by ID with subscription",
            "role_id" => id,
            "subscription_tier" => &subscription_tier.to_string()
        );

        self.role_repository
            .find_by_id_with_subscription(id, subscription_tier)
            .await
            .map_err(|e| {
                log_with_context!(
                    tracing::Level::ERROR,
                    "Failed to fetch role by ID with subscription",
                    "role_id" => id,
                    "error" => &e.to_string()
                );
                AppError::InternalServerError("Failed to fetch role".to_string())
            })?
            .ok_or_else(|| {
                log_with_context!(
                    tracing::Level::WARN,
                    "Role not found",
                    "role_id" => id
                );
                AppError::NotFound("Role not found".to_string())
            })
    }

    /// 新しいロールを作成（管理者のみ）
    pub async fn create_role(
        &self,
        requesting_user: &UserClaims,
        create_data: CreateRoleInput,
    ) -> AppResult<RoleWithPermissions> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Creating new role",
            "admin_id" => requesting_user.user_id,
            "role_name" => &create_data.name
        );
        // UserClaimsのcan_create_resourceメソッドを活用
        if !requesting_user.can_create_resource("role") {
            log_with_context!(
                tracing::Level::WARN,
                "Insufficient permissions to create role",
                "user_id" => requesting_user.user_id,
                "resource" => "role"
            );
            return Err(AppError::Forbidden("Cannot create roles".to_string()));
        }

        // 入力バリデーション
        create_data.validate()?;

        // ロール名の形式チェック
        let role_name = create_data.name.to_lowercase();
        if role_name == "admin" || role_name == "member" {
            return Err(AppError::BadRequest(
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
                log_with_context!(
                    tracing::Level::ERROR,
                    "Failed to create role",
                    "role_name" => &create_data.name,
                    "error" => &e.to_string()
                );
                AppError::InternalServerError("Failed to create role".to_string())
            })?;

        log_with_context!(
            tracing::Level::INFO,
            "Role created successfully",
            "admin_id" => requesting_user.user_id,
            "role_id" => created_role.id,
            "role_name" => &created_role.name.to_string()
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
        log_with_context!(
            tracing::Level::DEBUG,
            "Updating role",
            "admin_id" => requesting_user.user_id,
            "role_id" => role_id
        );
        // 管理者権限チェック
        self.check_admin_permission(requesting_user)?;

        // 入力バリデーション
        update_data.validate()?;

        // 既存ロールの確認
        let existing_role = self.get_role_by_id(role_id).await?;

        // システムロールの保護
        if existing_role.name == RoleName::Admin || existing_role.name == RoleName::Member {
            if let Some(new_name) = &update_data.name {
                if new_name.to_lowercase() != existing_role.name.as_str() {
                    return Err(AppError::BadRequest(
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
                log_with_context!(
                    tracing::Level::ERROR,
                    "Failed to update role",
                    "role_id" => role_id,
                    "error" => &e.to_string()
                );
                AppError::InternalServerError("Failed to update role".to_string())
            })?;

        log_with_context!(
            tracing::Level::INFO,
            "Role updated successfully",
            "admin_id" => requesting_user.user_id,
            "role_id" => role_id,
            "role_name" => &updated_role.name.to_string()
        );

        Ok(updated_role)
    }

    /// ロールを削除（管理者のみ）
    pub async fn delete_role(&self, requesting_user: &UserClaims, role_id: Uuid) -> AppResult<()> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Deleting role",
            "admin_id" => requesting_user.user_id,
            "role_id" => role_id
        );
        // UserClaimsのcan_delete_resourceメソッドを活用
        if !requesting_user.can_delete_resource("role", None) {
            log_with_context!(
                tracing::Level::WARN,
                "Insufficient permissions to delete role",
                "user_id" => requesting_user.user_id,
                "resource" => "role",
                "role_id" => role_id
            );
            return Err(AppError::Forbidden("Cannot delete roles".to_string()));
        }

        // 既存ロールの確認
        let existing_role = self.get_role_by_id(role_id).await?;

        // システムロールの削除を防ぐ
        if existing_role.name == RoleName::Admin || existing_role.name == RoleName::Member {
            return Err(AppError::BadRequest(
                "Cannot delete system roles (admin/member)".to_string(),
            ));
        }

        // このロールを使用しているユーザーがいるかチェック
        let users_with_role = self
            .user_repository
            .find_by_role_id(role_id)
            .await
            .map_err(|e| {
                log_with_context!(
                    tracing::Level::ERROR,
                    "Failed to check users with role",
                    "role_id" => role_id,
                    "error" => &e.to_string()
                );
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
            log_with_context!(
                tracing::Level::ERROR,
                "Failed to delete role",
                "role_id" => role_id,
                "error" => &e.to_string()
            );
            AppError::InternalServerError("Failed to delete role".to_string())
        })?;

        log_with_context!(
            tracing::Level::INFO,
            "Role deleted successfully",
            "admin_id" => requesting_user.user_id,
            "role_id" => role_id,
            "role_name" => &existing_role.name.to_string()
        );

        Ok(())
    }

    // --- 権限チェック機能 ---

    /// 管理者権限をチェック
    pub fn check_admin_permission(&self, user: &UserClaims) -> AppResult<()> {
        // UserClaimsの動的権限チェック機能を活用
        let permission_result = user.can_perform_action("roles", "manage", None);

        match permission_result {
            crate::domain::permission::PermissionResult::Allowed { .. } => Ok(()),
            crate::domain::permission::PermissionResult::Denied { reason } => {
                log_with_context!(
                    tracing::Level::WARN,
                    "Insufficient permissions for role management",
                    "user_id" => user.user_id,
                    "role" => user.role.as_ref().map_or_else(|| "none".to_string(), |r| r.name.to_string()),
                    "reason" => &reason
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
        log_with_context!(
            tracing::Level::DEBUG,
            "Assigning role to user",
            "admin_id" => requesting_user.user_id,
            "target_user_id" => user_id,
            "role_id" => role_id
        );
        // UserClaimsのcan_update_resourceメソッドを活用 - ユーザーリソースの更新
        if !requesting_user.can_update_resource("user", Some(user_id)) {
            log_with_context!(
                tracing::Level::WARN,
                "Insufficient permissions to assign role to user",
                "user_id" => requesting_user.user_id,
                "target_user_id" => user_id,
                "resource" => "user"
            );
            return Err(AppError::Forbidden(
                "Cannot assign roles to users".to_string(),
            ));
        }

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
                            log_with_context!(
                                tracing::Level::ERROR,
                                "Failed to assign role to user",
                                "user_id" => user_id,
                                "role_id" => role_id,
                                "error" => &e.to_string()
                            );
                            AppError::InternalServerError("Failed to assign role".to_string())
                        })?
                        .ok_or_else(|| {
                            log_with_context!(
                                tracing::Level::WARN,
                                "User not found for role assignment",
                                "user_id" => user_id
                            );
                            AppError::NotFound("User not found".to_string())
                        })
                })
            },
            retry_config,
        )
        .await?;

        let user_with_role = updated_user.to_safe_user_with_role(role);

        log_with_context!(
            tracing::Level::INFO,
            "Role assigned to user successfully",
            "admin_id" => requesting_user.user_id,
            "target_user_id" => user_id,
            "role_id" => role_id,
            "role_name" => &user_with_role.role.name.to_string()
        );

        Ok(user_with_role)
    }
}

// --- 入力データ構造体 ---

/// ロール作成用入力データ
#[derive(Debug, Clone)]
pub struct CreateRoleInput {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

impl CreateRoleInput {
    pub fn validate(&self) -> AppResult<()> {
        let mut errors = Vec::new();

        // 名前バリデーション
        if self.name.trim().is_empty() {
            errors.push("Role name cannot be empty".to_string());
        } else if self.name.len() > 50 {
            errors.push("Role name must be 50 characters or less".to_string());
        } else if !self
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            errors.push(
                "Role name can only contain alphanumeric characters, underscores, and hyphens"
                    .to_string(),
            );
        }

        // 表示名バリデーション
        if self.display_name.trim().is_empty() {
            errors.push("Display name cannot be empty".to_string());
        } else if self.display_name.len() > 100 {
            errors.push("Display name must be 100 characters or less".to_string());
        }

        // 説明バリデーション
        if let Some(description) = &self.description {
            if description.len() > 1000 {
                errors.push("Description must be 1000 characters or less".to_string());
            }
        }

        if !errors.is_empty() {
            return Err(AppError::BadRequest(errors.join(", ")));
        }

        Ok(())
    }
}

/// ロール更新用入力データ
#[derive(Debug, Clone, Default)]
pub struct UpdateRoleInput {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<Option<String>>,
    pub is_active: Option<bool>,
}

impl UpdateRoleInput {
    pub fn validate(&self) -> AppResult<()> {
        let mut errors = Vec::new();

        // 名前バリデーション
        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                errors.push("Role name cannot be empty".to_string());
            } else if name.len() > 50 {
                errors.push("Role name must be 50 characters or less".to_string());
            } else if !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                errors.push(
                    "Role name can only contain alphanumeric characters, underscores, and hyphens"
                        .to_string(),
                );
            }
        }

        // 表示名バリデーション
        if let Some(display_name) = &self.display_name {
            if display_name.trim().is_empty() {
                errors.push("Display name cannot be empty".to_string());
            } else if display_name.len() > 100 {
                errors.push("Display name must be 100 characters or less".to_string());
            }
        }

        // 説明バリデーション
        if let Some(Some(description)) = &self.description {
            if description.len() > 1000 {
                errors.push("Description must be 1000 characters or less".to_string());
            }
        }

        if !errors.is_empty() {
            return Err(AppError::BadRequest(errors.join(", ")));
        }

        Ok(())
    }
}

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
