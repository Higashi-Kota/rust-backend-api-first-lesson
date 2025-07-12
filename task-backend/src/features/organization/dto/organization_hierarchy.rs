use crate::features::organization::models::department::Model as Department;
use crate::features::organization::models::department_member::DepartmentRole;
use crate::features::organization::models::organization_analytics::{
    AnalyticsType, MetricValue, Period,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// Department Management DTOs
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateDepartmentDto {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Department name must be between 1 and 100 characters"
    ))]
    pub name: String,

    #[validate(length(max = 500, message = "Description must not exceed 500 characters"))]
    pub description: Option<String>,

    pub parent_department_id: Option<Uuid>,
    pub manager_user_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateDepartmentDto {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Department name must be between 1 and 100 characters"
    ))]
    pub name: Option<String>,

    #[validate(length(max = 500, message = "Description must not exceed 500 characters"))]
    pub description: Option<String>,

    pub manager_user_id: Option<Uuid>,
    pub new_parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentResponseDto {
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

impl From<Department> for DepartmentResponseDto {
    fn from(dept: Department) -> Self {
        Self {
            id: dept.id,
            name: dept.name,
            description: dept.description,
            organization_id: dept.organization_id,
            parent_department_id: dept.parent_department_id,
            hierarchy_level: dept.hierarchy_level,
            hierarchy_path: dept.hierarchy_path,
            manager_user_id: dept.manager_user_id,
            is_active: dept.is_active,
            created_at: dept.created_at,
            updated_at: dept.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepartmentHierarchyDto {
    pub department: DepartmentResponseDto,
    pub children: Vec<DepartmentHierarchyDto>,
    pub member_count: Option<u64>,
}

// Department Member Management DTOs
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AddDepartmentMemberDto {
    pub user_id: Uuid,
    pub role: DepartmentRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepartmentMemberResponseDto {
    pub id: Uuid,
    pub department_id: Uuid,
    pub user_id: Uuid,
    pub role: DepartmentRole,
    pub is_active: bool,
    pub joined_at: DateTime<Utc>,
    pub added_by: Uuid,
}

// Analytics DTOs
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct OrganizationAnalyticsQueryDto {
    pub period: Option<Period>,
    pub analytics_type: Option<AnalyticsType>,

    #[validate(range(min = 1, max = 1000, message = "Limit must be between 1 and 1000"))]
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAnalyticsMetricDto {
    pub department_id: Option<Uuid>,
    pub analytics_type: AnalyticsType,

    #[validate(length(
        min = 1,
        max = 100,
        message = "Metric name must be between 1 and 100 characters"
    ))]
    pub metric_name: String,

    pub metric_value: MetricValue,
    pub period: Period,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationAnalyticsResponseDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub department_id: Option<Uuid>,
    pub analytics_type: AnalyticsType,
    pub metric_name: String,
    pub metric_value: MetricValue,
    pub period: Period,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub calculated_by: Uuid,
    pub created_at: DateTime<Utc>,
}

// Permission Matrix DTOs - Removed (permission_matrix module deleted)

#[derive(Debug, Serialize, Deserialize)]
pub struct EffectivePermissionsResponseDto {
    pub organization_id: Uuid,
    pub user_id: Option<Uuid>,
    pub inheritance_chain: serde_json::Value,
    pub analyzed_at: DateTime<Utc>,
}

// Data Export DTOs
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ExportOrganizationDataDto {
    #[serde(default)]
    pub include_analytics: bool,

    #[serde(default)]
    pub include_permissions: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationDataExportResponseDto {
    pub organization_id: Uuid,
    pub departments: Vec<DepartmentResponseDto>,
    pub analytics: Option<Vec<OrganizationAnalyticsResponseDto>>,
    // pub organization_permissions: Option<PermissionMatrixResponseDto>, // Removed - permission_matrix deleted
    // pub department_permissions: Option<Vec<PermissionMatrixResponseDto>>, // Removed - permission_matrix deleted
    pub exported_at: DateTime<Utc>,
}

// Common Response DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct DepartmentOperationResponseDto {
    pub success: bool,
    pub message: String,
    pub department_id: Option<Uuid>,
    pub affected_children: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkDepartmentOperationResponseDto {
    pub processed: u32,
    pub successful: u32,
    pub failed: u32,
    pub errors: Vec<String>,
}

// Query Parameters DTOs
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct DepartmentQueryParams {
    pub include_children: Option<bool>,
    pub include_members: Option<bool>,
    pub active_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AnalyticsQueryParams {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub department_id: Option<Uuid>,
    pub metric_names: Option<Vec<String>>,
}

// Conversion implementations
// This implementation is now handled by the From<Department> implementation above
// impl From<crate::features::organization::models::organization_department::Model> for DepartmentResponseDto {
//     fn from(model: crate::features::organization::models::organization_department::Model) -> Self {
//         Self {
//             id: model.id,
//             name: model.name,
//             description: model.description,
//             organization_id: model.organization_id,
//             parent_department_id: model.parent_department_id,
//             hierarchy_level: model.hierarchy_level,
//             hierarchy_path: model.hierarchy_path,
//             manager_user_id: model.manager_user_id,
//             is_active: model.is_active,
//             created_at: model.created_at,
//             updated_at: model.updated_at,
//         }
//     }
// }

impl From<crate::features::organization::models::department_member::Model>
    for DepartmentMemberResponseDto
{
    fn from(model: crate::features::organization::models::department_member::Model) -> Self {
        // Convert old DepartmentRole to new DepartmentRole
        let role_str = model.get_role().to_string();
        let role = match role_str.as_str() {
            "Head" => DepartmentRole::Manager, // Map Head to Manager
            "Manager" => DepartmentRole::Manager,
            "Lead" => DepartmentRole::Lead,
            "Member" => DepartmentRole::Member,
            "Viewer" => DepartmentRole::Viewer,
            _ => DepartmentRole::Member, // Default fallback
        };

        Self {
            id: model.id,
            department_id: model.department_id,
            user_id: model.user_id,
            role,
            is_active: model.is_active,
            joined_at: model.joined_at,
            added_by: model.added_by,
        }
    }
}

impl From<crate::features::organization::models::organization_analytics::Model>
    for OrganizationAnalyticsResponseDto
{
    fn from(model: crate::features::organization::models::organization_analytics::Model) -> Self {
        let analytics_type = model.get_analytics_type();
        let metric_value = model.get_metric_value().unwrap_or_else(|_| MetricValue {
            value: 0.0,
            trend: None,
            benchmark: None,
            metadata: std::collections::HashMap::new(),
        });
        let period = model.get_period();

        Self {
            id: model.id,
            organization_id: model.organization_id,
            department_id: model.department_id,
            analytics_type,
            metric_name: model.metric_name,
            metric_value,
            period,
            period_start: model.period_start,
            period_end: model.period_end,
            calculated_by: model.calculated_by,
            created_at: model.created_at,
        }
    }
}

// Removed From impl for PermissionMatrixResponseDto - permission_matrix module deleted
