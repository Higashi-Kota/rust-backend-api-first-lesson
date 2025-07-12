pub mod requests;
pub mod responses;
pub mod team;
pub mod team_invitation;

// Re-export all DTOs from the new structure

// Note: The old team.rs and team_invitation.rs files are kept for now
// but their exports are not re-exported to avoid conflicts
