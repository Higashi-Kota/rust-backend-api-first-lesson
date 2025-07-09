// task-backend/src/utils/mod.rs

// Re-export from infrastructure
pub use crate::infrastructure::email;
pub use crate::infrastructure::jwt;
pub use crate::infrastructure::password;

pub mod error_helper;
pub mod feature_tracking;
pub mod permission;
pub mod token;
pub mod transaction;
pub mod validation;

// Re-export image_optimizer from infrastructure
#[allow(unused_imports)]
pub use crate::infrastructure::utils::image_optimizer;

// Utility modules - use specific imports instead of wildcard to avoid unused warnings
