// task-backend/src/features/security/dto/role.rs
// TODO: Phase 19で古いrole_dtoからの移行後、このファイルの内容を更新

// 暫定的に旧DTOを再エクスポート
pub use crate::api::dto::role_dto::{
    AssignRoleRequest, AssignRoleResponse, CreateRoleRequest, CreateRoleResponse,
    DeleteRoleResponse, RoleListResponse, RoleResponse, UpdateRoleRequest, UpdateRoleResponse,
};
