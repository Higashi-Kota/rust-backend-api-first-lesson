// task-backend/src/features/team/services/mod.rs

pub mod team;
pub mod team_invitation;

// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除予定
#[allow(unused_imports)]
pub use team::TeamService;
#[allow(unused_imports)]
pub use team_invitation::TeamInvitationService;
