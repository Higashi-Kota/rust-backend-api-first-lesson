// task-backend/src/shared/types/mod.rs

pub mod common;
pub mod pagination;

// Re-export commonly used types
pub use common::{ApiResponse, OperationResult};
#[allow(unused_imports)]
pub use pagination::{PaginatedResponse, PaginationMeta, PaginationQuery};
