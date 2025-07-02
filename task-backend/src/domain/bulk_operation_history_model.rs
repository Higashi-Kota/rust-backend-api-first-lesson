use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bulk_operation_histories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub operation_type: String,
    pub performed_by: Uuid,
    pub affected_count: i32,
    pub status: String,
    pub error_details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user_model::Entity",
        from = "Column::PerformedBy",
        to = "super::user_model::Column::Id"
    )]
    User,
}

impl Related<super::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BulkOperationType {
    UpdateRole,
    DeleteUsers,
    ActivateUsers,
    DeactivateUsers,
    UpdateOrganization,
    UpdateTeam,
}

impl std::fmt::Display for BulkOperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BulkOperationType::UpdateRole => "update_role",
            BulkOperationType::DeleteUsers => "delete_users",
            BulkOperationType::ActivateUsers => "activate_users",
            BulkOperationType::DeactivateUsers => "deactivate_users",
            BulkOperationType::UpdateOrganization => "update_organization",
            BulkOperationType::UpdateTeam => "update_team",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BulkOperationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    PartiallyCompleted,
}

impl std::fmt::Display for BulkOperationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BulkOperationStatus::Pending => "pending",
            BulkOperationStatus::InProgress => "in_progress",
            BulkOperationStatus::Completed => "completed",
            BulkOperationStatus::Failed => "failed",
            BulkOperationStatus::PartiallyCompleted => "partially_completed",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationError {
    pub entity_id: String,
    pub error_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationErrorDetails {
    pub errors: Vec<BulkOperationError>,
    pub total_errors: usize,
}

impl Model {
    pub fn new(operation_type: BulkOperationType, performed_by: Uuid, affected_count: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            operation_type: operation_type.to_string(),
            performed_by,
            affected_count,
            status: BulkOperationStatus::Pending.to_string(),
            error_details: None,
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn start(&mut self) {
        self.status = BulkOperationStatus::InProgress.to_string();
    }

    pub fn complete(&mut self) {
        self.status = BulkOperationStatus::Completed.to_string();
        self.completed_at = Some(Utc::now());
    }

    pub fn fail(&mut self, error_details: Option<BulkOperationErrorDetails>) {
        self.status = BulkOperationStatus::Failed.to_string();
        self.completed_at = Some(Utc::now());
        if let Some(details) = error_details {
            self.error_details = serde_json::to_value(details).ok();
        }
    }

    pub fn partially_complete(&mut self, error_details: BulkOperationErrorDetails) {
        self.status = BulkOperationStatus::PartiallyCompleted.to_string();
        self.completed_at = Some(Utc::now());
        self.error_details = serde_json::to_value(error_details).ok();
    }
}
