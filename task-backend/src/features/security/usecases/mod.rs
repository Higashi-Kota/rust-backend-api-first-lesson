// task-backend/src/features/security/usecases/mod.rs

pub mod permission_checker;
pub mod role_hierarchy;

// Re-export main types
pub use permission_checker::{
    BatchPermissionResult, ComplexPermissionContext, HierarchicalPermissionResult, HierarchyLevel,
    PermissionCheckerUseCase, PermissionDecision, PermissionHierarchy, PermissionRequest,
    TimeRestriction,
};
pub use role_hierarchy::{
    InheritedFrom, InheritedPermissions, RiskLevel, RoleHierarchyTree, RoleHierarchyUseCase,
    RoleMigrationImpact, RoleNode, RoleStatistics,
};
