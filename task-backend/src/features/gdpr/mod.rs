// task-backend/src/features/gdpr/mod.rs

pub mod dto;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod services;

// Re-export for backward compatibility
pub use handlers::gdpr as handler;
