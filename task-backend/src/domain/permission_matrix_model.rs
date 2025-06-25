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

#[allow(dead_code)]
impl EntityType {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "organization" => Ok(EntityType::Organization),
            "department" => Ok(EntityType::Department),
            "team" => Ok(EntityType::Team),
            "user" => Ok(EntityType::User),
            _ => Err(format!("Invalid entity type: {}", s)),
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

#[allow(dead_code)]
impl Model {
    pub fn new(
        entity_type: EntityType,
        entity_id: Uuid,
        matrix_data: PermissionMatrix,
        updated_by: Uuid,
        inheritance_settings: Option<InheritanceSettings>,
        compliance_settings: Option<ComplianceSettings>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type: entity_type.to_string(),
            entity_id,
            matrix_version: "v1.0".to_string(),
            matrix_data: serde_json::to_value(matrix_data).unwrap_or_default(),
            inheritance_settings: inheritance_settings
                .map(|s| serde_json::to_value(s).unwrap_or_default()),
            compliance_settings: compliance_settings
                .map(|s| serde_json::to_value(s).unwrap_or_default()),
            updated_by,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn get_entity_type(&self) -> EntityType {
        EntityType::from(self.entity_type.clone())
    }

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

    pub fn update_matrix_data(&mut self, matrix_data: PermissionMatrix) {
        self.matrix_data = serde_json::to_value(matrix_data).unwrap_or_default();
        self.updated_at = Utc::now();
    }

    pub fn update_inheritance_settings(&mut self, settings: InheritanceSettings) {
        self.inheritance_settings = Some(serde_json::to_value(settings).unwrap_or_default());
        self.updated_at = Utc::now();
    }

    pub fn increment_version(&mut self) {
        let current_version = self
            .matrix_version
            .replace("v", "")
            .parse::<f32>()
            .unwrap_or(1.0);
        self.matrix_version = format!("v{:.1}", current_version + 0.1);
        self.updated_at = Utc::now();
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
    fn test_entity_type_from_str() {
        assert_eq!(
            EntityType::from_str("organization").unwrap(),
            EntityType::Organization
        );
        assert_eq!(
            EntityType::from_str("department").unwrap(),
            EntityType::Department
        );
        assert_eq!(EntityType::from_str("team").unwrap(), EntityType::Team);
        assert_eq!(EntityType::from_str("user").unwrap(), EntityType::User);
        assert!(EntityType::from_str("invalid").is_err());
    }

    #[test]
    fn test_permission_matrix_model_new() {
        let entity_id = Uuid::new_v4();
        let updated_by = Uuid::new_v4();

        let mut tasks_permissions = HashMap::new();
        tasks_permissions.insert("read".to_string(), true);
        tasks_permissions.insert("write".to_string(), true);
        tasks_permissions.insert("delete".to_string(), false);

        let mut analytics_permissions = HashMap::new();
        analytics_permissions.insert("view".to_string(), true);
        analytics_permissions.insert("export".to_string(), false);

        let mut admin_permissions = HashMap::new();
        admin_permissions.insert("manage_users".to_string(), false);
        admin_permissions.insert("system_config".to_string(), false);

        let matrix_data = PermissionMatrix {
            tasks: tasks_permissions,
            analytics: analytics_permissions,
            administration: admin_permissions,
        };

        let inheritance_settings = InheritanceSettings {
            inherit_from_parent: true,
            allow_override: false,
            inheritance_priority: 1,
        };

        let compliance_settings = ComplianceSettings {
            audit_required: true,
            approval_workflow: false,
            retention_period_days: 90,
            compliance_level: "medium".to_string(),
        };

        let model = Model::new(
            EntityType::Department,
            entity_id,
            matrix_data,
            updated_by,
            Some(inheritance_settings),
            Some(compliance_settings),
        );

        assert_eq!(model.entity_id, entity_id);
        assert_eq!(model.get_entity_type(), EntityType::Department);
        assert_eq!(model.updated_by, updated_by);
        assert_eq!(model.matrix_version, "v1.0");
        assert!(model.is_active);
    }

    #[test]
    fn test_permission_matrix_getters() {
        let entity_id = Uuid::new_v4();
        let updated_by = Uuid::new_v4();

        let mut tasks_permissions = HashMap::new();
        tasks_permissions.insert("read".to_string(), true);
        tasks_permissions.insert("write".to_string(), false);

        let matrix_data = PermissionMatrix {
            tasks: tasks_permissions.clone(),
            analytics: HashMap::new(),
            administration: HashMap::new(),
        };

        let model = Model::new(
            EntityType::User,
            entity_id,
            matrix_data,
            updated_by,
            None,
            None,
        );

        assert_eq!(model.get_entity_type(), EntityType::User);

        let retrieved_matrix = model.get_permission_matrix().unwrap();
        assert_eq!(retrieved_matrix.tasks.len(), tasks_permissions.len());
        assert_eq!(retrieved_matrix.tasks.get("read"), Some(&true));
        assert_eq!(retrieved_matrix.tasks.get("write"), Some(&false));
    }

    #[test]
    fn test_inheritance_settings() {
        let entity_id = Uuid::new_v4();
        let updated_by = Uuid::new_v4();

        let matrix_data = PermissionMatrix {
            tasks: HashMap::new(),
            analytics: HashMap::new(),
            administration: HashMap::new(),
        };

        let inheritance_settings = InheritanceSettings {
            inherit_from_parent: false,
            allow_override: true,
            inheritance_priority: 3,
        };

        let model = Model::new(
            EntityType::Organization,
            entity_id,
            matrix_data,
            updated_by,
            Some(inheritance_settings.clone()),
            None,
        );

        let retrieved_settings = model.get_inheritance_settings().unwrap().unwrap();
        assert_eq!(
            retrieved_settings.inherit_from_parent,
            inheritance_settings.inherit_from_parent
        );
        assert_eq!(
            retrieved_settings.allow_override,
            inheritance_settings.allow_override
        );
        assert_eq!(
            retrieved_settings.inheritance_priority,
            inheritance_settings.inheritance_priority
        );

        // Test model without inheritance settings
        let model_without_inheritance = Model::new(
            EntityType::Team,
            entity_id,
            PermissionMatrix {
                tasks: HashMap::new(),
                analytics: HashMap::new(),
                administration: HashMap::new(),
            },
            updated_by,
            None,
            None,
        );

        assert!(model_without_inheritance
            .get_inheritance_settings()
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_compliance_settings() {
        let entity_id = Uuid::new_v4();
        let updated_by = Uuid::new_v4();

        let matrix_data = PermissionMatrix {
            tasks: HashMap::new(),
            analytics: HashMap::new(),
            administration: HashMap::new(),
        };

        let compliance_settings = ComplianceSettings {
            audit_required: true,
            approval_workflow: true,
            retention_period_days: 365,
            compliance_level: "high".to_string(),
        };

        let model = Model::new(
            EntityType::Organization,
            entity_id,
            matrix_data,
            updated_by,
            None,
            Some(compliance_settings.clone()),
        );

        let retrieved_settings = model.get_compliance_settings().unwrap().unwrap();
        assert_eq!(
            retrieved_settings.audit_required,
            compliance_settings.audit_required
        );
        assert_eq!(
            retrieved_settings.approval_workflow,
            compliance_settings.approval_workflow
        );
        assert_eq!(
            retrieved_settings.retention_period_days,
            compliance_settings.retention_period_days
        );
        assert_eq!(
            retrieved_settings.compliance_level,
            compliance_settings.compliance_level
        );

        // Test model without compliance settings
        let model_without_compliance = Model::new(
            EntityType::User,
            entity_id,
            PermissionMatrix {
                tasks: HashMap::new(),
                analytics: HashMap::new(),
                administration: HashMap::new(),
            },
            updated_by,
            None,
            None,
        );

        assert!(model_without_compliance
            .get_compliance_settings()
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_update_matrix_data() {
        let entity_id = Uuid::new_v4();
        let updated_by = Uuid::new_v4();

        let initial_matrix = PermissionMatrix {
            tasks: HashMap::new(),
            analytics: HashMap::new(),
            administration: HashMap::new(),
        };

        let mut model = Model::new(
            EntityType::Team,
            entity_id,
            initial_matrix,
            updated_by,
            None,
            None,
        );

        // Update with new matrix data
        let mut new_tasks = HashMap::new();
        new_tasks.insert("read".to_string(), true);
        new_tasks.insert("write".to_string(), true);

        let new_matrix = PermissionMatrix {
            tasks: new_tasks.clone(),
            analytics: HashMap::new(),
            administration: HashMap::new(),
        };

        model.update_matrix_data(new_matrix);

        let retrieved_matrix = model.get_permission_matrix().unwrap();
        assert_eq!(retrieved_matrix.tasks.len(), new_tasks.len());
        assert_eq!(retrieved_matrix.tasks.get("read"), Some(&true));
        assert_eq!(retrieved_matrix.tasks.get("write"), Some(&true));
    }

    #[test]
    fn test_update_inheritance_settings() {
        let entity_id = Uuid::new_v4();
        let updated_by = Uuid::new_v4();

        let matrix_data = PermissionMatrix {
            tasks: HashMap::new(),
            analytics: HashMap::new(),
            administration: HashMap::new(),
        };

        let mut model = Model::new(
            EntityType::Department,
            entity_id,
            matrix_data,
            updated_by,
            None,
            None,
        );

        // Update inheritance settings
        let new_inheritance = InheritanceSettings {
            inherit_from_parent: true,
            allow_override: true,
            inheritance_priority: 2,
        };

        model.update_inheritance_settings(new_inheritance.clone());

        let retrieved_settings = model.get_inheritance_settings().unwrap().unwrap();
        assert_eq!(
            retrieved_settings.inherit_from_parent,
            new_inheritance.inherit_from_parent
        );
        assert_eq!(
            retrieved_settings.allow_override,
            new_inheritance.allow_override
        );
        assert_eq!(
            retrieved_settings.inheritance_priority,
            new_inheritance.inheritance_priority
        );
    }

    #[test]
    fn test_increment_version() {
        let entity_id = Uuid::new_v4();
        let updated_by = Uuid::new_v4();

        let matrix_data = PermissionMatrix {
            tasks: HashMap::new(),
            analytics: HashMap::new(),
            administration: HashMap::new(),
        };

        let mut model = Model::new(
            EntityType::User,
            entity_id,
            matrix_data,
            updated_by,
            None,
            None,
        );

        // Initial version should be v1.0
        assert_eq!(model.matrix_version, "v1.0");

        // Increment version
        model.increment_version();
        assert_eq!(model.matrix_version, "v1.1");

        // Increment again
        model.increment_version();
        assert_eq!(model.matrix_version, "v1.2");

        // Test with custom version format
        model.matrix_version = "v2.5".to_string();
        model.increment_version();
        assert_eq!(model.matrix_version, "v2.6");
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
