pub mod requests;
pub mod responses;
pub mod team;
pub mod team_invitation;

// Re-export all DTOs from the new structure
// TODO: Phase 19で整理予定 - 現在は移行期間のため維持
#[allow(unused_imports)]
pub use requests::team::*;
#[allow(unused_imports)]
pub use requests::team_invitation::*;
#[allow(unused_imports)]
pub use responses::team::*;
#[allow(unused_imports)]
pub use responses::team_invitation::*;

// Note: The old team.rs and team_invitation.rs files are kept for now
// but their exports are not re-exported to avoid conflicts
