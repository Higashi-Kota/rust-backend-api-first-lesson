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
