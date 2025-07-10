// task-backend/src/features/security/models/mod.rs

pub mod permission_matrix;
pub mod role;
pub mod security_incident;

// Re-export main types
// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除
#[allow(unused_imports)]
pub use permission_matrix::{
    ActiveModel as PermissionMatrixActiveModel,
    ActiveModelTrait as PermissionMatrixActiveModelTrait, Column as PermissionMatrixColumn,
    ComplianceSettings, DepartmentOverride, Entity as PermissionMatrixEntity, EntityType,
    InheritanceSettings, Model as PermissionMatrixModel, PermissionMatrix, PermissionRule,
    PrimaryKey as PermissionMatrixPrimaryKey, Relation as PermissionMatrixRelation,
};

#[allow(unused_imports)]
pub use role::{
    ActiveModel as RoleActiveModel, ActiveModelTrait as RoleActiveModelTrait, Column as RoleColumn,
    Entity as RoleEntity, Model as RoleModel, PrimaryKey as RolePrimaryKey,
    Relation as RoleRelation, RoleName, RoleWithPermissions,
};

#[allow(unused_imports)]
pub use security_incident::{
    ActiveModel as SecurityIncidentActiveModel,
    ActiveModelTrait as SecurityIncidentActiveModelTrait, Column as SecurityIncidentColumn,
    Entity as SecurityIncidentEntity, IncidentSeverity, IncidentStatus,
    Model as SecurityIncidentModel, PrimaryKey as SecurityIncidentPrimaryKey,
    Relation as SecurityIncidentRelation,
};
