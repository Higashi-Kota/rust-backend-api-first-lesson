pub mod dto;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod services;

// Re-export for backward compatibility
pub use handlers::auth as handler;
pub use handlers::middleware;
pub use services::auth as service;
