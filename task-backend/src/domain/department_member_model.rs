use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "department_members")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub department_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub is_active: bool,
    pub joined_at: DateTime<Utc>,
    pub added_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::organization_department_model::Entity",
        from = "Column::DepartmentId",
        to = "super::organization_department_model::Column::Id"
    )]
    Department,
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::UserId",
        to = "crate::domain::user_model::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::AddedBy",
        to = "crate::domain::user_model::Column::Id"
    )]
    AddedByUser,
}

impl Related<super::organization_department_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Department.def()
    }
}

impl Related<crate::domain::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DepartmentRole {
    Manager,
    Lead,
    Member,
    Viewer,
}

impl std::fmt::Display for DepartmentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DepartmentRole::Manager => write!(f, "manager"),
            DepartmentRole::Lead => write!(f, "lead"),
            DepartmentRole::Member => write!(f, "member"),
            DepartmentRole::Viewer => write!(f, "viewer"),
        }
    }
}

impl From<String> for DepartmentRole {
    fn from(value: String) -> Self {
        match value.as_str() {
            "manager" => DepartmentRole::Manager,
            "lead" => DepartmentRole::Lead,
            "member" => DepartmentRole::Member,
            "viewer" => DepartmentRole::Viewer,
            _ => DepartmentRole::Member,
        }
    }
}

impl Model {
    pub fn get_role(&self) -> DepartmentRole {
        DepartmentRole::from(self.role.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_department_role_display() {
        assert_eq!(DepartmentRole::Manager.to_string(), "manager");
        assert_eq!(DepartmentRole::Lead.to_string(), "lead");
        assert_eq!(DepartmentRole::Member.to_string(), "member");
        assert_eq!(DepartmentRole::Viewer.to_string(), "viewer");
    }

    #[test]
    fn test_department_role_from_string() {
        assert_eq!(
            DepartmentRole::from("manager".to_string()),
            DepartmentRole::Manager
        );
        assert_eq!(
            DepartmentRole::from("lead".to_string()),
            DepartmentRole::Lead
        );
        assert_eq!(
            DepartmentRole::from("member".to_string()),
            DepartmentRole::Member
        );
        assert_eq!(
            DepartmentRole::from("viewer".to_string()),
            DepartmentRole::Viewer
        );
        assert_eq!(
            DepartmentRole::from("invalid".to_string()),
            DepartmentRole::Member
        );
    }
}
