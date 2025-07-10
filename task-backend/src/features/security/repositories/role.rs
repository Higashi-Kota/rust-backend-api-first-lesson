// task-backend/src/features/security/repositories/role.rs
use super::super::models::role::{self, Entity as Role, RoleWithPermissions};
use crate::core::subscription_tier::SubscriptionTier;
use crate::error::{AppError, AppResult};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

/// ロールリポジトリ
// TODO: Phase 19で古い参照を削除後、#[allow(dead_code)]を削除
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RoleRepository {
    db: Arc<DatabaseConnection>,
}

impl RoleRepository {
    /// 新しいロールリポジトリを作成
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// すべてのロールを取得
    pub async fn find_all(&self) -> AppResult<Vec<RoleWithPermissions>> {
        let roles = Role::find()
            .order_by_asc(role::Column::Name)
            .all(self.db.as_ref())
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to fetch all roles");
                AppError::InternalServerError("Failed to fetch roles".to_string())
            })?;

        let mut role_permissions = Vec::new();
        for role in roles {
            match RoleWithPermissions::from_model(role) {
                Ok(role_with_perms) => role_permissions.push(role_with_perms),
                Err(e) => {
                    warn!(error = %e, "Invalid role data in database");
                    continue;
                }
            }
        }

        info!(count = %role_permissions.len(), "Successfully fetched all roles");
        Ok(role_permissions)
    }

    /// アクティブなロールのみ取得
    pub async fn find_all_active(&self) -> AppResult<Vec<RoleWithPermissions>> {
        let roles = Role::find()
            .filter(role::Column::IsActive.eq(true))
            .order_by_asc(role::Column::Name)
            .all(self.db.as_ref())
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to fetch active roles");
                AppError::InternalServerError("Failed to fetch active roles".to_string())
            })?;

        let mut role_permissions = Vec::new();
        for role in roles {
            match RoleWithPermissions::from_model(role) {
                Ok(role_with_perms) => role_permissions.push(role_with_perms),
                Err(e) => {
                    warn!(error = %e, "Invalid active role data in database");
                    continue;
                }
            }
        }

        info!(count = %role_permissions.len(), "Successfully fetched active roles");
        Ok(role_permissions)
    }

    /// IDでロールを取得
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<RoleWithPermissions>> {
        let role = Role::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| {
                error!(error = %e, role_id = %id, "Failed to fetch role by ID");
                AppError::InternalServerError("Failed to fetch role".to_string())
            })?;

        match role {
            Some(role_model) => match RoleWithPermissions::from_model(role_model) {
                Ok(role_with_perms) => {
                    info!(role_id = %id, role_name = %role_with_perms.name, "Successfully fetched role by ID");
                    Ok(Some(role_with_perms))
                }
                Err(e) => {
                    error!(error = %e, role_id = %id, "Invalid role data in database");
                    Err(AppError::InternalServerError(
                        "Invalid role data".to_string(),
                    ))
                }
            },
            None => {
                info!(role_id = %id, "Role not found");
                Ok(None)
            }
        }
    }

    /// 名前でロールを取得
    pub async fn find_by_name(&self, name: &str) -> AppResult<Option<RoleWithPermissions>> {
        let role = Role::find()
            .filter(role::Column::Name.eq(name))
            .one(self.db.as_ref())
            .await
            .map_err(|e| {
                error!(error = %e, role_name = %name, "Failed to fetch role by name");
                AppError::InternalServerError("Failed to fetch role".to_string())
            })?;

        match role {
            Some(role_model) => match RoleWithPermissions::from_model(role_model) {
                Ok(role_with_perms) => {
                    info!(role_name = %name, role_id = %role_with_perms.id, "Successfully fetched role by name");
                    Ok(Some(role_with_perms))
                }
                Err(e) => {
                    error!(error = %e, role_name = %name, "Invalid role data in database");
                    Err(AppError::InternalServerError(
                        "Invalid role data".to_string(),
                    ))
                }
            },
            None => {
                info!(role_name = %name, "Role not found");
                Ok(None)
            }
        }
    }

    /// 新しいロールを作成
    pub async fn create(&self, role_data: CreateRoleData) -> AppResult<RoleWithPermissions> {
        // 名前の重複チェック
        if let Some(_existing) = self.find_by_name(&role_data.name).await? {
            return Err(AppError::Conflict(format!(
                "Role with name '{}' already exists",
                role_data.name
            )));
        }

        let role = role::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(role_data.name.clone()),
            display_name: Set(role_data.display_name.clone()),
            description: Set(role_data.description.clone()),
            is_active: Set(role_data.is_active),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };

        let created_role = role.insert(self.db.as_ref()).await.map_err(|e| {
            error!(error = %e, role_name = %role_data.name, "Failed to create role");
            AppError::InternalServerError("Failed to create role".to_string())
        })?;

        let role_with_perms = RoleWithPermissions::from_model(created_role).map_err(|e| {
            error!(error = %e, "Failed to convert created role");
            AppError::InternalServerError("Failed to process created role".to_string())
        })?;

        info!(
            role_id = %role_with_perms.id,
            role_name = %role_with_perms.name,
            "Successfully created role"
        );

        Ok(role_with_perms)
    }

    /// ロールを更新
    pub async fn update(
        &self,
        id: Uuid,
        update_data: UpdateRoleData,
    ) -> AppResult<RoleWithPermissions> {
        let role = Role::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| {
                error!(error = %e, role_id = %id, "Failed to fetch role for update");
                AppError::InternalServerError("Failed to fetch role".to_string())
            })?;

        let role = role.ok_or_else(|| {
            warn!(role_id = %id, "Role not found for update");
            AppError::NotFound("Role not found".to_string())
        })?;

        // 名前の重複チェック（自分以外）
        if let Some(new_name) = &update_data.name {
            if let Some(existing) = self.find_by_name(new_name).await? {
                if existing.id != id {
                    return Err(AppError::Conflict(format!(
                        "Role with name '{}' already exists",
                        new_name
                    )));
                }
            }
        }

        let mut active_role: role::ActiveModel = role.into();

        if let Some(name) = update_data.name {
            active_role.name = Set(name);
        }
        if let Some(display_name) = update_data.display_name {
            active_role.display_name = Set(display_name);
        }
        if let Some(description) = update_data.description {
            active_role.description = Set(description);
        }
        if let Some(is_active) = update_data.is_active {
            active_role.is_active = Set(is_active);
        }
        active_role.updated_at = Set(chrono::Utc::now());

        let updated_role = active_role.update(self.db.as_ref()).await.map_err(|e| {
            error!(error = %e, role_id = %id, "Failed to update role");
            AppError::InternalServerError("Failed to update role".to_string())
        })?;

        let role_with_perms = RoleWithPermissions::from_model(updated_role).map_err(|e| {
            error!(error = %e, "Failed to convert updated role");
            AppError::InternalServerError("Failed to process updated role".to_string())
        })?;

        info!(
            role_id = %id,
            role_name = %role_with_perms.name,
            "Successfully updated role"
        );

        Ok(role_with_perms)
    }

    /// ロールを削除
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        // ロールの存在確認
        let role = Role::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| {
                error!(error = %e, role_id = %id, "Failed to fetch role for deletion");
                AppError::InternalServerError("Failed to fetch role".to_string())
            })?;

        let role = role.ok_or_else(|| {
            warn!(role_id = %id, "Role not found for deletion");
            AppError::NotFound("Role not found".to_string())
        })?;

        // システムロール（admin/member）の削除を防ぐ
        if role.name == "admin" || role.name == "member" {
            return Err(AppError::ValidationError(
                "Cannot delete system roles (admin/member)".to_string(),
            ));
        }

        // このロールを使用しているユーザーがいるかチェック
        let users_with_role = self.count_users_with_role(id).await?;
        if users_with_role > 0 {
            warn!(
                role_id = %id,
                role_name = %role.name,
                users_count = users_with_role,
                "Cannot delete role: still in use by users"
            );
            return Err(AppError::ValidationError(format!(
                "Cannot delete role '{}': {} users are still assigned to this role",
                role.name, users_with_role
            )));
        }

        Role::delete_by_id(id)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| {
                error!(error = %e, role_id = %id, "Failed to delete role");
                AppError::InternalServerError("Failed to delete role".to_string())
            })?;

        info!(role_id = %id, role_name = %role.name, "Successfully deleted role");
        Ok(())
    }

    /// 指定したロールを使用しているユーザー数を取得
    pub async fn count_users_with_role(&self, role_id: Uuid) -> AppResult<u64> {
        use crate::domain::user_model::{Column as UserColumn, Entity as User};

        let count = User::find()
            .filter(UserColumn::RoleId.eq(role_id))
            .count(self.db.as_ref())
            .await
            .map_err(|e| {
                error!(error = %e, role_id = %role_id, "Failed to count users with role");
                AppError::InternalServerError("Failed to count users with role".to_string())
            })?;

        Ok(count)
    }

    /// サブスクリプション階層を指定してロールを取得
    pub async fn find_by_id_with_subscription(
        &self,
        id: Uuid,
        subscription_tier: SubscriptionTier,
    ) -> AppResult<Option<RoleWithPermissions>> {
        let role = Role::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| {
                error!(error = %e, role_id = %id, "Failed to fetch role by ID with subscription");
                AppError::InternalServerError("Failed to fetch role".to_string())
            })?;

        match role {
            Some(role_model) => {
                match RoleWithPermissions::from_model_with_subscription(
                    role_model,
                    subscription_tier,
                ) {
                    Ok(role_with_perms) => {
                        info!(
                            role_id = %id,
                            role_name = %role_with_perms.name,
                            subscription_tier = %subscription_tier,
                            "Successfully fetched role with subscription tier"
                        );
                        Ok(Some(role_with_perms))
                    }
                    Err(e) => {
                        error!(error = %e, role_id = %id, "Invalid role data in database");
                        Err(AppError::InternalServerError(
                            "Invalid role data".to_string(),
                        ))
                    }
                }
            }
            None => {
                info!(role_id = %id, "Role not found");
                Ok(None)
            }
        }
    }
}

/// ロール作成用データ
#[derive(Debug, Clone)]
pub struct CreateRoleData {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

/// ロール更新用データ
#[derive(Debug, Clone, Default)]
pub struct UpdateRoleData {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<Option<String>>,
    pub is_active: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_role_data() {
        let create_data = CreateRoleData {
            name: "test".to_string(),
            display_name: "Test Role".to_string(),
            description: Some("Test role description".to_string()),
            is_active: true,
        };

        assert_eq!(create_data.name, "test");
        assert_eq!(create_data.display_name, "Test Role");
        assert!(create_data.is_active);
    }

    #[test]
    fn test_update_role_data() {
        let update_data = UpdateRoleData {
            name: Some("updated_test".to_string()),
            display_name: None,
            description: Some(Some("Updated description".to_string())),
            is_active: Some(false),
        };

        assert!(update_data.name.is_some());
        assert!(update_data.display_name.is_none());
        assert!(update_data.description.is_some());
        assert_eq!(update_data.is_active, Some(false));
    }
}
