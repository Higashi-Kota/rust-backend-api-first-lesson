// src/domain/audit_log_model.rs
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "audit_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub details: Option<Json>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub result: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user_model::Entity",
        from = "Column::UserId",
        to = "super::user_model::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::team_model::Entity",
        from = "Column::TeamId",
        to = "super::team_model::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Team,
    #[sea_orm(
        belongs_to = "super::organization_model::Entity",
        from = "Column::OrganizationId",
        to = "super::organization_model::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Organization,
}

impl Related<super::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::team_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Team.def()
    }
}

impl Related<super::organization_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Organization.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// 監査アクションの定義
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditAction {
    // タスク関連
    TaskCreated,
    TaskUpdated,
    TaskDeleted,
    TaskAssigned,
    TaskTransferred,
    TaskVisibilityChanged,

    // チーム関連
    TeamCreated,
    TeamUpdated,
    TeamDeleted,
    TeamMemberAdded,
    TeamMemberRemoved,
    TeamRoleChanged,

    // 組織関連
    OrganizationCreated,
    OrganizationUpdated,
    OrganizationDeleted,

    // 権限関連
    PermissionGranted,
    PermissionRevoked,
    RoleAssigned,
    RoleRemoved,

    // 認証関連
    UserLogin,
    UserLogout,
    PasswordChanged,
    EmailVerified,

    // その他
    Custom(String),
}

impl AuditAction {
    pub fn as_str(&self) -> &str {
        match self {
            AuditAction::TaskCreated => "task_created",
            AuditAction::TaskUpdated => "task_updated",
            AuditAction::TaskDeleted => "task_deleted",
            AuditAction::TaskAssigned => "task_assigned",
            AuditAction::TaskTransferred => "task_transferred",
            AuditAction::TaskVisibilityChanged => "task_visibility_changed",
            AuditAction::TeamCreated => "team_created",
            AuditAction::TeamUpdated => "team_updated",
            AuditAction::TeamDeleted => "team_deleted",
            AuditAction::TeamMemberAdded => "team_member_added",
            AuditAction::TeamMemberRemoved => "team_member_removed",
            AuditAction::TeamRoleChanged => "team_role_changed",
            AuditAction::OrganizationCreated => "organization_created",
            AuditAction::OrganizationUpdated => "organization_updated",
            AuditAction::OrganizationDeleted => "organization_deleted",
            AuditAction::PermissionGranted => "permission_granted",
            AuditAction::PermissionRevoked => "permission_revoked",
            AuditAction::RoleAssigned => "role_assigned",
            AuditAction::RoleRemoved => "role_removed",
            AuditAction::UserLogin => "user_login",
            AuditAction::UserLogout => "user_logout",
            AuditAction::PasswordChanged => "password_changed",
            AuditAction::EmailVerified => "email_verified",
            AuditAction::Custom(action) => action,
        }
    }
}

// 監査結果の定義
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditResult {
    Success,
    Failure,
    PartialSuccess,
}

impl AuditResult {
    pub fn as_str(&self) -> &str {
        match self {
            AuditResult::Success => "success",
            AuditResult::Failure => "failure",
            AuditResult::PartialSuccess => "partial_success",
        }
    }
}

// 監査ログエントリービルダー
pub struct AuditLogBuilder {
    user_id: Uuid,
    action: AuditAction,
    resource_type: String,
    resource_id: Option<Uuid>,
    team_id: Option<Uuid>,
    organization_id: Option<Uuid>,
    details: Option<serde_json::Value>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    result: AuditResult,
}

impl AuditLogBuilder {
    pub fn new(user_id: Uuid, action: AuditAction, resource_type: impl Into<String>) -> Self {
        Self {
            user_id,
            action,
            resource_type: resource_type.into(),
            resource_id: None,
            team_id: None,
            organization_id: None,
            details: None,
            ip_address: None,
            user_agent: None,
            result: AuditResult::Success,
        }
    }

    pub fn resource_id(mut self, id: Uuid) -> Self {
        self.resource_id = Some(id);
        self
    }

    pub fn team_id(mut self, id: Uuid) -> Self {
        self.team_id = Some(id);
        self
    }

    pub fn organization_id(mut self, id: Uuid) -> Self {
        self.organization_id = Some(id);
        self
    }

    pub fn details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = Some(agent.into());
        self
    }

    pub fn result(mut self, result: AuditResult) -> Self {
        self.result = result;
        self
    }

    pub fn build(self) -> ActiveModel {
        ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(self.user_id),
            action: Set(self.action.as_str().to_string()),
            resource_type: Set(self.resource_type),
            resource_id: Set(self.resource_id),
            team_id: Set(self.team_id),
            organization_id: Set(self.organization_id),
            details: Set(self.details),
            ip_address: Set(self.ip_address),
            user_agent: Set(self.user_agent),
            result: Set(self.result.as_str().to_string()),
            created_at: Set(Utc::now()),
        }
    }
}
