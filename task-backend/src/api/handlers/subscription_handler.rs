// Temporary re-export for backward compatibility
// Note: The original API expected app_state as a parameter, but the new implementation
// returns Router<AppState> directly. We need to adapt the interface.

pub use crate::features::subscription::handlers::subscription::subscription_router_with_state;
pub use crate::features::subscription::handlers::subscription::subscription_router;