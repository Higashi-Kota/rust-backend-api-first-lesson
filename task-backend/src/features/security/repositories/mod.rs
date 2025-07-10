// task-backend/src/features/security/repositories/mod.rs

pub mod permission_matrix;
pub mod role;
pub mod security_incident;

// Re-export repository types
// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除
#[allow(unused_imports)]
pub use permission_matrix::PermissionMatrixRepository;
#[allow(unused_imports)]
pub use role::{CreateRoleData, RoleRepository, UpdateRoleData};
#[allow(unused_imports)]
pub use security_incident::SecurityIncidentRepository;
