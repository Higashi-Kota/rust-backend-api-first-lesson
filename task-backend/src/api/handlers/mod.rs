// src/api/handlers/mod.rs
pub mod analytics_handler;
pub mod organization_hierarchy_handler;
pub mod payment_handler;
pub mod permission_handler;
pub mod role_handler;
pub mod system_handler;
pub mod user_handler;

pub mod organization_handler {
    // Re-exported from features module
}
pub mod security_handler {
    // Re-exported from features module
}
pub mod subscription_handler {
    // Re-exported from features module
}

// Handler modules - use specific imports instead of wildcard to avoid unused warnings
