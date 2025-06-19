// src/api/dto/mod.rs
pub mod analytics_dto;
pub mod auth_dto;
pub mod common;
pub mod organization_dto;
pub mod permission_dto;
pub mod role_dto;
pub mod subscription_dto;
pub mod task_dto;
pub mod team_dto;
pub mod user_dto;

// Re-export common response types
pub use common::{ApiResponse, OperationResult, PaginatedResponse, PaginationMeta};

// Re-export ApiError for future use
#[allow(unused_imports)]
pub use common::ApiError;
