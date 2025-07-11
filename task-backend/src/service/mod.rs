// src/service/mod.rs
pub mod feature_tracking_service;
pub mod organization_hierarchy_service;
pub mod organization_service;
pub mod permission_service;
pub mod role_service;
pub mod security_service;
pub mod team_invitation_service;
pub mod team_service;

// Re-export subscription service for backward compatibility
pub mod subscription_service {
    // TODO: Remove this module after migrating all references
    // pub use crate::features::subscription::services::subscription::*;
}
