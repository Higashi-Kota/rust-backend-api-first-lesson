// task-backend/src/service/permission_service.rs

use crate::domain::role_model::RoleWithPermissions;
use crate::domain::subscription_tier::SubscriptionTier;
use crate::error::AppResult;
// use crate::repository::permission_repository::PermissionRepository; // TODO: Implement when PermissionRepository is created
use crate::error::AppError;
use crate::repository::role_repository::RoleRepository;
use crate::repository::user_repository::UserRepository;
use crate::utils::permission::{PermissionChecker, PermissionType, ResourceContext};
use std::sync::Arc;
use uuid::Uuid;

/// 権限管理の統合サービス
pub struct PermissionService {
    // permission_repository: Arc<PermissionRepository>, // TODO: Implement when PermissionRepository is created
    role_repository: Arc<RoleRepository>,
    user_repository: Arc<UserRepository>,
}

impl PermissionService {
    pub fn new(
        // permission_repository: Arc<PermissionRepository>, // TODO: Implement when PermissionRepository is created
        role_repository: Arc<RoleRepository>,
        user_repository: Arc<UserRepository>,
    ) -> Self {
        Self {
            // permission_repository,
            role_repository,
            user_repository,
        }
    }

    /// ユーザーがリソースへのアクセス権限を持っているか確認
    pub async fn check_resource_access(
        &self,
        user_id: Uuid,
        resource_type: &str,
        resource_id: Option<Uuid>,
        action: &str,
    ) -> AppResult<()> {
        let role = self.get_user_role(user_id).await?;

        let has_permission = match action {
            "view" => {
                PermissionChecker::can_view_resource(&role, resource_type, resource_id, user_id)
            }
            "create" => PermissionChecker::can_create_resource(&role, resource_type),
            "update" => {
                PermissionChecker::can_update_resource(&role, resource_type, resource_id, user_id)
            }
            "delete" => {
                PermissionChecker::can_delete_resource(&role, resource_type, resource_id, user_id)
            }
            _ => false,
        };

        if !has_permission {
            return Err(AppError::Forbidden(format!(
                "Permission denied for {} action on {} resource",
                action, resource_type
            )));
        }

        Ok(())
    }

    /// ユーザーが他のユーザーのデータにアクセスできるか確認
    pub async fn check_user_access(
        &self,
        requesting_user_id: Uuid,
        target_user_id: Uuid,
    ) -> AppResult<()> {
        let role = self.get_user_role(requesting_user_id).await?;

        if !PermissionChecker::can_access_user(&role, requesting_user_id, target_user_id) {
            return Err(AppError::Forbidden(
                "Cannot access other user's data".to_string(),
            ));
        }

        Ok(())
    }

    /// 権限タイプに基づいた権限チェック
    pub async fn check_permission_type(
        &self,
        user_id: Uuid,
        permission_type: PermissionType,
        context: Option<ResourceContext>,
    ) -> AppResult<()> {
        let role = self.get_user_role(user_id).await?;

        let has_permission = match permission_type {
            PermissionType::IsAdmin => PermissionChecker::is_admin(&role),
            PermissionType::IsMember => PermissionChecker::is_member(&role),
            PermissionType::CanAccessUser => {
                if let Some(ctx) = context {
                    if let Some(target_id) = ctx.target_user_id {
                        PermissionChecker::can_access_user(&role, user_id, target_id)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            PermissionType::CanCreateResource => {
                if let Some(ctx) = context {
                    PermissionChecker::can_create_resource(&role, &ctx.resource_type)
                } else {
                    false
                }
            }
            PermissionType::CanDeleteResource => {
                if let Some(ctx) = context {
                    PermissionChecker::can_delete_resource(
                        &role,
                        &ctx.resource_type,
                        ctx.owner_id,
                        user_id,
                    )
                } else {
                    false
                }
            }
        };

        if !has_permission {
            return Err(AppError::Forbidden("Permission denied".to_string()));
        }

        Ok(())
    }

    /// ユーザーの管理機能へのアクセス権限を確認
    pub async fn check_admin_features_access(&self, user_id: Uuid) -> AppResult<()> {
        let role = self.get_user_role(user_id).await?;

        if !PermissionChecker::can_access_admin_features(&role) {
            return Err(AppError::Forbidden(
                "Admin features access denied".to_string(),
            ));
        }

        Ok(())
    }

    /// ユーザーのロール情報を取得（内部ヘルパー）
    async fn get_user_role(&self, user_id: Uuid) -> AppResult<RoleWithPermissions> {
        // ユーザーを取得
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // ロールを取得（ユーザーのサブスクリプション階層も考慮）
        let subscription_tier = user
            .subscription_tier
            .parse()
            .unwrap_or(SubscriptionTier::Free);
        let role = self
            .role_repository
            .find_by_id_with_subscription(user.role_id, subscription_tier)
            .await?
            .ok_or_else(|| AppError::NotFound("Role not found".to_string()))?;

        Ok(role)
    }

    /// 複数の権限を一度にチェック
    pub async fn check_multiple_permissions(
        &self,
        user_id: Uuid,
        checks: Vec<(PermissionType, Option<ResourceContext>)>,
    ) -> AppResult<Vec<bool>> {
        let role = self.get_user_role(user_id).await?;

        let results = checks
            .into_iter()
            .map(|(permission_type, context)| match permission_type {
                PermissionType::IsAdmin => PermissionChecker::is_admin(&role),
                PermissionType::IsMember => PermissionChecker::is_member(&role),
                PermissionType::CanAccessUser => {
                    if let Some(ctx) = context {
                        if let Some(target_id) = ctx.target_user_id {
                            PermissionChecker::can_access_user(&role, user_id, target_id)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                PermissionType::CanCreateResource => {
                    if let Some(ctx) = context {
                        PermissionChecker::can_create_resource(&role, &ctx.resource_type)
                    } else {
                        false
                    }
                }
                PermissionType::CanDeleteResource => {
                    if let Some(ctx) = context {
                        PermissionChecker::can_delete_resource(
                            &role,
                            &ctx.resource_type,
                            ctx.owner_id,
                            user_id,
                        )
                    } else {
                        false
                    }
                }
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::domain::role_model::RoleName;
    // use crate::domain::subscription_tier::SubscriptionTier;
    // use chrono::Utc;
    // use mockall::predicate::*;
    // use mockall::mock;

    // モックリポジトリの定義は省略（実際のテストでは必要）

    #[tokio::test]
    async fn test_check_admin_permission() {
        // テスト実装
    }

    #[tokio::test]
    async fn test_check_resource_access() {
        // テスト実装
    }

    #[tokio::test]
    async fn test_check_user_access() {
        // テスト実装
    }
}
