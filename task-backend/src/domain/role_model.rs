// task-backend/src/domain/role_model.rs
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ロールエンティティ
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "roles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    #[sea_orm(unique)]
    pub name: String,

    pub display_name: String,

    #[sea_orm(nullable)]
    pub description: Option<String>,

    pub is_active: bool,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_model::Entity")]
    Users,
}

impl Related<super::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// ロール名を表すenum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleName {
    Admin,
    Member,
}

impl RoleName {
    /// ロール名を文字列として取得
    pub fn as_str(&self) -> &'static str {
        match self {
            RoleName::Admin => "admin",
            RoleName::Member => "member",
        }
    }

    /// 文字列からロール名を解析
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(RoleName::Admin),
            "member" => Some(RoleName::Member),
            _ => None,
        }
    }

    /// 管理者権限があるかチェック
    pub fn is_admin(&self) -> bool {
        matches!(self, RoleName::Admin)
    }

    /// 一般ユーザー権限があるかチェック
    pub fn is_member(&self) -> bool {
        matches!(self, RoleName::Member)
    }

    /// 権限レベルを数値で取得（高いほど強い権限）
    pub fn permission_level(&self) -> u8 {
        match self {
            RoleName::Admin => 100,
            RoleName::Member => 10,
        }
    }
}

impl std::fmt::Display for RoleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for RoleName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s).ok_or_else(|| format!("Invalid role name: {}", s))
    }
}

/// ロールWithアクセス権限チェック機能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleWithPermissions {
    pub id: Uuid,
    pub name: RoleName,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RoleWithPermissions {
    /// Modelから変換
    pub fn from_model(model: Model) -> Result<Self, String> {
        let role_name = RoleName::from_str(&model.name)
            .ok_or_else(|| format!("Invalid role name in database: {}", model.name))?;

        Ok(Self {
            id: model.id,
            name: role_name,
            display_name: model.display_name,
            description: model.description,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }

    /// 管理者権限があるかチェック
    pub fn is_admin(&self) -> bool {
        self.name.is_admin() && self.is_active
    }

    /// 一般ユーザー権限があるかチェック
    pub fn is_member(&self) -> bool {
        self.name.is_member() && self.is_active
    }

    /// 指定されたユーザーIDにアクセス権限があるかチェック
    pub fn can_access_user(&self, requesting_user_id: Uuid, target_user_id: Uuid) -> bool {
        if !self.is_active {
            return false;
        }

        // 自分自身のデータには常にアクセス可能
        if requesting_user_id == target_user_id {
            return true;
        }

        // 管理者は他のユーザーのデータにもアクセス可能
        self.is_admin()
    }

    /// リソースの作成権限があるかチェック
    pub fn can_create_resource(&self, resource_type: &str) -> bool {
        if !self.is_active {
            return false;
        }

        match resource_type {
            "user" => self.is_admin(),
            "role" => self.is_admin(),
            "task" => true, // 全ロールでタスク作成可能
            _ => false,
        }
    }

    /// リソースの削除権限があるかチェック
    pub fn can_delete_resource(
        &self,
        resource_type: &str,
        owner_id: Option<Uuid>,
        requesting_user_id: Uuid,
    ) -> bool {
        if !self.is_active {
            return false;
        }

        match resource_type {
            "user" => self.is_admin(),
            "role" => self.is_admin(),
            "task" => {
                // 自分のタスクは削除可能、管理者は全タスク削除可能
                if let Some(owner) = owner_id {
                    owner == requesting_user_id || self.is_admin()
                } else {
                    self.is_admin()
                }
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_name_conversion() {
        assert_eq!(RoleName::Admin.as_str(), "admin");
        assert_eq!(RoleName::Member.as_str(), "member");

        assert_eq!(RoleName::from_str("admin"), Some(RoleName::Admin));
        assert_eq!(RoleName::from_str("ADMIN"), Some(RoleName::Admin));
        assert_eq!(RoleName::from_str("member"), Some(RoleName::Member));
        assert_eq!(RoleName::from_str("invalid"), None);
    }

    #[test]
    fn test_role_checks() {
        assert!(RoleName::Admin.is_admin());
        assert!(!RoleName::Admin.is_member());
        assert!(!RoleName::Member.is_admin());
        assert!(RoleName::Member.is_member());
    }
}
