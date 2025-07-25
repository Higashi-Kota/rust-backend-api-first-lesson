// task-backend/src/api/dto/role_dto.rs
use crate::domain::role_model::RoleWithPermissions;
use crate::service::role_service::{CreateRoleInput, UpdateRoleInput};
use crate::types::{ApiResponse, Timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// --- リクエストDTO ---

/// ロール作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateRoleRequest {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Role name must be between 1 and 50 characters"
    ))]
    pub name: String,

    #[validate(length(
        min = 1,
        max = 100,
        message = "Display name must be between 1 and 100 characters"
    ))]
    pub display_name: String,

    #[validate(length(max = 1000, message = "Description must be 1000 characters or less"))]
    pub description: Option<String>,

    pub is_active: Option<bool>,
}

impl CreateRoleRequest {
    pub fn into_service_input(self) -> CreateRoleInput {
        CreateRoleInput {
            name: self.name,
            display_name: self.display_name,
            description: self.description,
            is_active: self.is_active,
        }
    }
}

/// ロール更新リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateRoleRequest {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Role name must be between 1 and 50 characters"
    ))]
    pub name: Option<String>,

    #[validate(length(
        min = 1,
        max = 100,
        message = "Display name must be between 1 and 100 characters"
    ))]
    pub display_name: Option<String>,

    #[validate(length(max = 1000, message = "Description must be 1000 characters or less"))]
    pub description: Option<Option<String>>,

    pub is_active: Option<bool>,
}

impl UpdateRoleRequest {
    pub fn into_service_input(self) -> UpdateRoleInput {
        UpdateRoleInput {
            name: self.name,
            display_name: self.display_name,
            description: self.description,
            is_active: self.is_active,
        }
    }

    /// 更新対象があるかチェック
    pub fn has_updates(&self) -> bool {
        self.name.is_some()
            || self.display_name.is_some()
            || self.description.is_some()
            || self.is_active.is_some()
    }
}

/// ユーザーロール割り当てリクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}

// --- レスポンスDTO ---

/// ロール情報レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_system_role: bool,
    pub user_count: u64,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub permissions: RolePermissionsResponse,
}

impl From<RoleWithPermissions> for RoleResponse {
    fn from(role: RoleWithPermissions) -> Self {
        let permissions = RolePermissionsResponse::from(&role);
        let is_system_role = matches!(role.name.as_str(), "admin" | "member" | "guest");
        Self {
            id: role.id,
            name: role.name.as_str().to_string(),
            display_name: role.display_name,
            description: role.description,
            is_active: role.is_active,
            is_system_role,
            user_count: 0, // This will be populated by the handler
            created_at: Timestamp::from_datetime(role.created_at),
            updated_at: Timestamp::from_datetime(role.updated_at),
            permissions,
        }
    }
}

/// ロール権限情報レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermissionsResponse {
    pub is_admin: bool,
    pub is_member: bool,
    pub permission_level: u8,
    pub can_create_users: bool,
    pub can_create_roles: bool,
    pub can_create_tasks: bool,
    pub can_manage_all_users: bool,
    pub can_manage_all_tasks: bool,
}

impl From<&RoleWithPermissions> for RolePermissionsResponse {
    fn from(role: &RoleWithPermissions) -> Self {
        Self {
            is_admin: role.is_admin(),
            is_member: role.is_member(),
            permission_level: role.name.permission_level(),
            can_create_users: role.can_create_resource("user"),
            can_create_roles: role.can_create_resource("role"),
            can_create_tasks: role.can_create_resource("task"),
            can_manage_all_users: role.is_admin(),
            can_manage_all_tasks: role.is_admin(),
        }
    }
}

/// ロール一覧レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleListResponse {
    pub roles: Vec<RoleResponse>,
    pub total_count: usize,
}

/// ロール作成レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoleResponse {
    pub role: RoleResponse,
    pub message: String,
}

impl CreateRoleResponse {
    pub fn build(role: RoleWithPermissions) -> ApiResponse<RoleResponse> {
        ApiResponse::success(RoleResponse::from(role))
    }
}

/// ロール更新レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleResponse {
    pub role: RoleResponse,
    pub message: String,
    pub changes: Vec<String>,
}

/// ロール削除レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRoleResponse {
    pub message: String,
    pub deleted_role_id: Uuid,
}

/// ユーザーロール割り当てレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignRoleResponse {
    pub message: String,
    pub user_id: Uuid,
    pub role: RoleResponse,
}

// --- 共通レスポンス ---

// 統一レスポンス構造体を使用 (common.rs から import)

/// レスポンス構築ヘルパー
impl CreateRoleRequest {
    /// バリデーション実行
    pub fn validate_and_sanitize(&mut self) -> Result<(), validator::ValidationErrors> {
        // 名前を小文字に変換
        self.name = self.name.trim().to_lowercase();

        // 表示名をトリム
        self.display_name = self.display_name.trim().to_string();

        // 説明をトリム（存在する場合）
        if let Some(description) = &mut self.description {
            *description = description.trim().to_string();
            if description.is_empty() {
                self.description = None;
            }
        }

        // バリデーション実行
        self.validate()
    }
}

impl UpdateRoleRequest {
    /// バリデーション実行とサニタイズ
    pub fn validate_and_sanitize(&mut self) -> Result<(), validator::ValidationErrors> {
        // 名前を小文字に変換（存在する場合）
        if let Some(name) = &mut self.name {
            *name = name.trim().to_lowercase();
        }

        // 表示名をトリム（存在する場合）
        if let Some(display_name) = &mut self.display_name {
            *display_name = display_name.trim().to_string();
        }

        // 説明をトリム（存在する場合）
        if let Some(Some(description)) = &mut self.description {
            *description = description.trim().to_string();
            if description.is_empty() {
                self.description = Some(None);
            }
        }

        // バリデーション実行
        self.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_role_request_validation() {
        let mut request = CreateRoleRequest {
            name: " Test_Role ".to_string(),
            display_name: " Test Role ".to_string(),
            description: Some(" Test description ".to_string()),
            is_active: Some(true),
        };

        assert!(request.validate_and_sanitize().is_ok());
        assert_eq!(request.name, "test_role");
        assert_eq!(request.display_name, "Test Role");
        assert_eq!(request.description, Some("Test description".to_string()));
    }

    #[test]
    fn test_update_role_request_validation() {
        let mut request = UpdateRoleRequest {
            name: Some(" Updated_Role ".to_string()),
            display_name: Some(" Updated Role ".to_string()),
            description: Some(Some(" Updated description ".to_string())),
            is_active: Some(false),
        };

        assert!(request.validate_and_sanitize().is_ok());
        assert_eq!(request.name, Some("updated_role".to_string()));
        assert_eq!(request.display_name, Some("Updated Role".to_string()));
        assert_eq!(
            request.description,
            Some(Some("Updated description".to_string()))
        );
    }
}
