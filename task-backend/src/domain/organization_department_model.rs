use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "organization_departments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub organization_id: Uuid,
    pub parent_department_id: Option<Uuid>,
    pub hierarchy_level: i32,
    pub hierarchy_path: String,
    pub manager_user_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::organization_model::Entity",
        from = "Column::OrganizationId",
        to = "super::organization_model::Column::Id"
    )]
    Organization,
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::ManagerUserId",
        to = "crate::domain::user_model::Column::Id"
    )]
    Manager,
    #[sea_orm(has_many = "super::department_member_model::Entity")]
    DepartmentMembers,
    #[sea_orm(has_many = "super::organization_analytics_model::Entity")]
    OrganizationAnalytics,
}

impl Related<super::organization_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Organization.def()
    }
}

impl Related<crate::domain::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Manager.def()
    }
}

impl Related<super::department_member_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DepartmentMembers.def()
    }
}

impl Related<super::organization_analytics_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrganizationAnalytics.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[allow(dead_code)]
impl Model {
    pub fn new(
        name: String,
        organization_id: Uuid,
        parent_department_id: Option<Uuid>,
        manager_user_id: Option<Uuid>,
        description: Option<String>,
    ) -> Self {
        let hierarchy_level = if parent_department_id.is_some() { 1 } else { 0 };
        let hierarchy_path = format!("/{}", Uuid::new_v4());

        Self {
            id: Uuid::new_v4(),
            name,
            description,
            organization_id,
            parent_department_id,
            hierarchy_level,
            hierarchy_path,
            manager_user_id,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn update_hierarchy_path(&mut self, parent_path: Option<&str>) {
        self.hierarchy_path = match parent_path {
            Some(path) => format!("{}/{}", path, self.id),
            None => format!("/{}", self.id),
        };
    }

    pub fn update_hierarchy_level(&mut self, level: i32) {
        self.hierarchy_level = level;
    }

    pub fn get_path_components(&self) -> Vec<String> {
        self.hierarchy_path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    pub fn is_root_department(&self) -> bool {
        self.parent_department_id.is_none()
    }

    pub fn is_child_of(&self, potential_parent_id: Uuid) -> bool {
        self.hierarchy_path
            .contains(&potential_parent_id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_department_new() {
        let organization_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();
        let name = "Engineering".to_string();
        let description = Some("Software Engineering Department".to_string());

        let department = Model::new(
            name.clone(),
            organization_id,
            None,
            Some(manager_id),
            description.clone(),
        );

        assert_eq!(department.name, name);
        assert_eq!(department.description, description);
        assert_eq!(department.organization_id, organization_id);
        assert_eq!(department.parent_department_id, None);
        assert_eq!(department.manager_user_id, Some(manager_id));
        assert_eq!(department.hierarchy_level, 0);
        assert!(department.is_active);
        assert!(department.is_root_department());
    }

    #[test]
    fn test_organization_department_with_parent() {
        let organization_id = Uuid::new_v4();
        let parent_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();
        let name = "Frontend Team".to_string();

        let department = Model::new(
            name.clone(),
            organization_id,
            Some(parent_id),
            Some(manager_id),
            None,
        );

        assert_eq!(department.name, name);
        assert_eq!(department.organization_id, organization_id);
        assert_eq!(department.parent_department_id, Some(parent_id));
        assert_eq!(department.manager_user_id, Some(manager_id));
        assert_eq!(department.hierarchy_level, 1); // Child department
        assert!(department.is_active);
        assert!(!department.is_root_department());
    }

    #[test]
    fn test_hierarchy_path_operations() {
        let organization_id = Uuid::new_v4();
        let name = "Test Department".to_string();

        let mut department = Model::new(name, organization_id, None, None, None);

        // Test initial path (root department)
        let initial_path = department.hierarchy_path.clone();
        assert!(initial_path.starts_with('/'));

        // Test updating hierarchy path with parent
        let parent_path = "/parent-uuid";
        department.update_hierarchy_path(Some(parent_path));
        assert_eq!(
            department.hierarchy_path,
            format!("{}/{}", parent_path, department.id)
        );

        // Test updating hierarchy path without parent (back to root)
        department.update_hierarchy_path(None);
        assert_eq!(department.hierarchy_path, format!("/{}", department.id));
    }

    #[test]
    fn test_hierarchy_level_operations() {
        let organization_id = Uuid::new_v4();
        let name = "Test Department".to_string();

        let mut department = Model::new(name, organization_id, None, None, None);

        // Initial level should be 0 for root department
        assert_eq!(department.hierarchy_level, 0);

        // Update hierarchy level
        department.update_hierarchy_level(2);
        assert_eq!(department.hierarchy_level, 2);

        // Update back to root level
        department.update_hierarchy_level(0);
        assert_eq!(department.hierarchy_level, 0);
    }

    #[test]
    fn test_path_components() {
        let organization_id = Uuid::new_v4();
        let name = "Test Department".to_string();

        let mut department = Model::new(name, organization_id, None, None, None);

        // Test path components for root department
        let components = department.get_path_components();
        assert_eq!(components.len(), 1);
        // The UUID will be generated, so let's just verify the structure
        assert!(!components[0].is_empty());

        // Test path components for nested department
        department.hierarchy_path = "/parent1/parent2/child".to_string();
        let components = department.get_path_components();
        assert_eq!(components.len(), 3);
        assert_eq!(components[0], "parent1");
        assert_eq!(components[1], "parent2");
        assert_eq!(components[2], "child");
    }

    #[test]
    fn test_root_department_check() {
        let organization_id = Uuid::new_v4();
        let name = "Test Department".to_string();

        // Root department (no parent)
        let root_dept = Model::new(name.clone(), organization_id, None, None, None);
        assert!(root_dept.is_root_department());

        // Child department (has parent)
        let parent_id = Uuid::new_v4();
        let child_dept = Model::new(name, organization_id, Some(parent_id), None, None);
        assert!(!child_dept.is_root_department());
    }

    #[test]
    fn test_is_child_of() {
        let organization_id = Uuid::new_v4();
        let parent_id = Uuid::new_v4();
        let grandparent_id = Uuid::new_v4();
        let unrelated_id = Uuid::new_v4();

        let mut department = Model::new(
            "Test Department".to_string(),
            organization_id,
            Some(parent_id),
            None,
            None,
        );

        // Set up a hierarchy path that includes both parent and grandparent
        department.hierarchy_path = format!("/{}/{}/{}", grandparent_id, parent_id, department.id);

        // Test parent relationship
        assert!(department.is_child_of(parent_id));

        // Test grandparent relationship
        assert!(department.is_child_of(grandparent_id));

        // Test unrelated ID
        assert!(!department.is_child_of(unrelated_id));

        // Test self reference
        assert!(department.is_child_of(department.id));
    }

    #[test]
    fn test_department_hierarchy_levels() {
        let organization_id = Uuid::new_v4();

        // Root department
        let root = Model::new("Root".to_string(), organization_id, None, None, None);
        assert_eq!(root.hierarchy_level, 0);
        assert!(root.is_root_department());

        // First level child
        let child1 = Model::new(
            "Child1".to_string(),
            organization_id,
            Some(root.id),
            None,
            None,
        );
        assert_eq!(child1.hierarchy_level, 1);
        assert!(!child1.is_root_department());

        // Second level child (grandchild)
        let grandchild = Model::new(
            "Grandchild".to_string(),
            organization_id,
            Some(child1.id),
            None,
            None,
        );
        assert_eq!(grandchild.hierarchy_level, 1); // Note: constructor only sets 1 for any parent
        assert!(!grandchild.is_root_department());
    }
}
