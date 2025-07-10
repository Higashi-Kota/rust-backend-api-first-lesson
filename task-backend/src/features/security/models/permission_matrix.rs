use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "permission_matrices")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub matrix_version: String,
    pub matrix_data: JsonValue,
    pub inheritance_settings: Option<JsonValue>,
    pub compliance_settings: Option<JsonValue>,
    pub updated_by: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // TODO: Phase 19でUserモデルがfeatures/authに移行後に更新
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::UpdatedBy",
        to = "crate::domain::user_model::Column::Id"
    )]
    UpdatedByUser,
}

impl Related<crate::domain::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UpdatedByUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    Organization,
    Department,
    Team,
    User,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Organization => write!(f, "organization"),
            EntityType::Department => write!(f, "department"),
            EntityType::Team => write!(f, "team"),
            EntityType::User => write!(f, "user"),
        }
    }
}

impl From<String> for EntityType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "organization" => EntityType::Organization,
            "department" => EntityType::Department,
            "team" => EntityType::Team,
            "user" => EntityType::User,
            _ => EntityType::User,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    pub scope: String,
    pub conditions: Vec<String>,
    pub quota: Option<HashMap<String, i32>>,
    pub inheritance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceSettings {
    pub inherit_from_parent: bool,
    pub allow_override: bool,
    pub inheritance_priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSettings {
    pub audit_required: bool,
    pub approval_workflow: bool,
    pub retention_period_days: u32,
    pub compliance_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentOverride {
    pub department_id: String,
    pub resource: String,
    pub action: String,
    pub override_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionMatrix {
    pub tasks: HashMap<String, bool>,
    pub analytics: HashMap<String, bool>,
    pub administration: HashMap<String, bool>,
}

impl Model {
    pub fn get_permission_matrix(&self) -> Result<PermissionMatrix, serde_json::Error> {
        serde_json::from_value(self.matrix_data.clone())
    }

    pub fn get_inheritance_settings(
        &self,
    ) -> Result<Option<InheritanceSettings>, serde_json::Error> {
        match &self.inheritance_settings {
            Some(settings) => Ok(Some(serde_json::from_value(settings.clone())?)),
            None => Ok(None),
        }
    }

    pub fn get_compliance_settings(&self) -> Result<Option<ComplianceSettings>, serde_json::Error> {
        match &self.compliance_settings {
            Some(settings) => Ok(Some(serde_json::from_value(settings.clone())?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_display() {
        assert_eq!(EntityType::Organization.to_string(), "organization");
        assert_eq!(EntityType::Department.to_string(), "department");
        assert_eq!(EntityType::Team.to_string(), "team");
        assert_eq!(EntityType::User.to_string(), "user");
    }

    #[test]
    fn test_entity_type_from_string() {
        assert_eq!(
            EntityType::from("organization".to_string()),
            EntityType::Organization
        );
        assert_eq!(
            EntityType::from("department".to_string()),
            EntityType::Department
        );
        assert_eq!(EntityType::from("team".to_string()), EntityType::Team);
        assert_eq!(EntityType::from("user".to_string()), EntityType::User);
        assert_eq!(EntityType::from("invalid".to_string()), EntityType::User);
    }

    #[test]
    fn test_permission_matrix_serialization() {
        let mut tasks_permissions = HashMap::new();
        tasks_permissions.insert("create".to_string(), true);
        tasks_permissions.insert("read".to_string(), true);
        tasks_permissions.insert("update".to_string(), false);
        tasks_permissions.insert("delete".to_string(), false);

        let mut analytics_permissions = HashMap::new();
        analytics_permissions.insert("view_reports".to_string(), true);
        analytics_permissions.insert("export_data".to_string(), false);

        let mut admin_permissions = HashMap::new();
        admin_permissions.insert("user_management".to_string(), false);

        let matrix = PermissionMatrix {
            tasks: tasks_permissions,
            analytics: analytics_permissions,
            administration: admin_permissions,
        };

        // Test serialization and deserialization
        let json_value = serde_json::to_value(&matrix).unwrap();
        let deserialized: PermissionMatrix = serde_json::from_value(json_value).unwrap();

        assert_eq!(deserialized.tasks.len(), matrix.tasks.len());
        assert_eq!(deserialized.analytics.len(), matrix.analytics.len());
        assert_eq!(
            deserialized.administration.len(),
            matrix.administration.len()
        );

        assert_eq!(deserialized.tasks.get("create"), Some(&true));
        assert_eq!(deserialized.tasks.get("update"), Some(&false));
        assert_eq!(deserialized.analytics.get("view_reports"), Some(&true));
        assert_eq!(deserialized.analytics.get("export_data"), Some(&false));
        assert_eq!(
            deserialized.administration.get("user_management"),
            Some(&false)
        );
    }
}
