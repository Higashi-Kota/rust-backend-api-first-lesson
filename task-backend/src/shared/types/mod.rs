// task-backend/src/shared/types/mod.rs

pub mod common;
pub mod pagination;

// Re-export commonly used types
// TODO: Phase 3完了後にapi/dto/commonから移行予定
#[allow(unused_imports)]
pub use common::{ApiResponse, OperationResult};
#[allow(unused_imports)]
pub use pagination::{PaginatedResponse, PaginationMeta, PaginationQuery};
