pub mod dto;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod services;
pub mod usecases;

// Re-export commonly used types
// pub use models::{
//     permission_matrix::{Model as PermissionMatrix},
//     role::{Model as Role, RoleName, RoleWithPermissions},
//     security_incident::{Model as SecurityIncident},
// };

// Re-export services
// pub use services::{permission::PermissionService, role::RoleService, security::SecurityService};

// Re-export usecases
// pub use usecases::{
//     permission_checker::PermissionCheckerUseCase, role_hierarchy::RoleHierarchyUseCase,
// };

// Re-export router function
// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除
// Removed: security_router_with_state was deleted as unused
