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

#[allow(dead_code)]
impl DepartmentRole {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "manager" => Ok(DepartmentRole::Manager),
            "lead" => Ok(DepartmentRole::Lead),
            "member" => Ok(DepartmentRole::Member),
            "viewer" => Ok(DepartmentRole::Viewer),
            _ => Err(format!("Invalid department role: {}", s)),
        }
    }

    pub fn has_management_permissions(&self) -> bool {
        matches!(self, DepartmentRole::Manager | DepartmentRole::Lead)
    }

    pub fn can_modify_members(&self) -> bool {
        matches!(self, DepartmentRole::Manager)
    }

    pub fn can_view_analytics(&self) -> bool {
        !matches!(self, DepartmentRole::Viewer)
    }

    pub fn get_permission_level(&self) -> u8 {
        match self {
            DepartmentRole::Manager => 4,
            DepartmentRole::Lead => 3,
            DepartmentRole::Member => 2,
            DepartmentRole::Viewer => 1,
        }
    }
}

#[allow(dead_code)]
impl Model {
    pub fn new(department_id: Uuid, user_id: Uuid, role: DepartmentRole, added_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            department_id,
            user_id,
            role: role.to_string(),
            is_active: true,
            joined_at: Utc::now(),
            added_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn get_role(&self) -> DepartmentRole {
        DepartmentRole::from(self.role.clone())
    }

    pub fn update_role(&mut self, new_role: DepartmentRole) {
        self.role = new_role.to_string();
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn has_management_permissions(&self) -> bool {
        self.get_role().has_management_permissions()
    }

    pub fn can_modify_members(&self) -> bool {
        self.get_role().can_modify_members()
    }

    pub fn can_view_analytics(&self) -> bool {
        self.get_role().can_view_analytics()
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

    #[test]
    fn test_department_role_from_str() {
        assert_eq!(
            DepartmentRole::from_str("manager").unwrap(),
            DepartmentRole::Manager
        );
        assert_eq!(
            DepartmentRole::from_str("lead").unwrap(),
            DepartmentRole::Lead
        );
        assert_eq!(
            DepartmentRole::from_str("member").unwrap(),
            DepartmentRole::Member
        );
        assert_eq!(
            DepartmentRole::from_str("viewer").unwrap(),
            DepartmentRole::Viewer
        );
        assert!(DepartmentRole::from_str("invalid").is_err());
    }

    #[test]
    fn test_department_role_permissions() {
        let manager = DepartmentRole::Manager;
        let lead = DepartmentRole::Lead;
        let member = DepartmentRole::Member;
        let viewer = DepartmentRole::Viewer;

        // Management permissions
        assert!(manager.has_management_permissions());
        assert!(lead.has_management_permissions());
        assert!(!member.has_management_permissions());
        assert!(!viewer.has_management_permissions());

        // Member modification permissions
        assert!(manager.can_modify_members());
        assert!(!lead.can_modify_members());
        assert!(!member.can_modify_members());
        assert!(!viewer.can_modify_members());

        // Analytics viewing permissions
        assert!(manager.can_view_analytics());
        assert!(lead.can_view_analytics());
        assert!(member.can_view_analytics());
        assert!(!viewer.can_view_analytics());
    }

    #[test]
    fn test_department_role_permission_levels() {
        assert_eq!(DepartmentRole::Manager.get_permission_level(), 4);
        assert_eq!(DepartmentRole::Lead.get_permission_level(), 3);
        assert_eq!(DepartmentRole::Member.get_permission_level(), 2);
        assert_eq!(DepartmentRole::Viewer.get_permission_level(), 1);
    }

    #[test]
    fn test_department_member_model_new() {
        let department_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let added_by = Uuid::new_v4();
        let role = DepartmentRole::Manager;

        let member = Model::new(department_id, user_id, role.clone(), added_by);

        assert_eq!(member.department_id, department_id);
        assert_eq!(member.user_id, user_id);
        assert_eq!(member.role, role.to_string());
        assert_eq!(member.added_by, added_by);
        assert!(member.is_active);
        assert_eq!(member.get_role(), role);
    }

    #[test]
    fn test_department_member_model_role_operations() {
        let department_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let added_by = Uuid::new_v4();

        let mut member = Model::new(department_id, user_id, DepartmentRole::Member, added_by);

        // Test getting role
        assert_eq!(member.get_role(), DepartmentRole::Member);

        // Test updating role
        member.update_role(DepartmentRole::Lead);
        assert_eq!(member.get_role(), DepartmentRole::Lead);
        assert_eq!(member.role, "lead");
    }

    #[test]
    fn test_department_member_model_activation() {
        let department_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let added_by = Uuid::new_v4();

        let mut member = Model::new(department_id, user_id, DepartmentRole::Member, added_by);

        // Test deactivation
        member.deactivate();
        assert!(!member.is_active);

        // Test activation
        member.activate();
        assert!(member.is_active);
    }

    #[test]
    fn test_department_member_model_permissions() {
        let department_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let added_by = Uuid::new_v4();

        let manager = Model::new(department_id, user_id, DepartmentRole::Manager, added_by);
        let viewer = Model::new(department_id, user_id, DepartmentRole::Viewer, added_by);

        // Manager permissions
        assert!(manager.has_management_permissions());
        assert!(manager.can_modify_members());
        assert!(manager.can_view_analytics());

        // Viewer permissions
        assert!(!viewer.has_management_permissions());
        assert!(!viewer.can_modify_members());
        assert!(!viewer.can_view_analytics());
    }
}
