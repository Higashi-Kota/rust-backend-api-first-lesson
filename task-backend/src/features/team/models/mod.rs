pub mod team;
pub mod team_invitation;
pub mod team_member;

// TODO: Phase 19で整理予定 - 現在は移行期間のため維持
#[allow(unused_imports)]
pub use team::{
    ActiveModel as TeamActiveModel, Column as TeamColumn, Entity as Team, Model as TeamModel,
    PrimaryKey as TeamPrimaryKey, Relation as TeamRelation, TeamRole,
};
#[allow(unused_imports)]
pub use team_invitation::{
    ActiveModel as TeamInvitationActiveModel, Column as TeamInvitationColumn,
    Entity as TeamInvitation, Model as TeamInvitationModel, PrimaryKey as TeamInvitationPrimaryKey,
    Relation as TeamInvitationRelation, TeamInvitationStatus,
};
#[allow(unused_imports)]
pub use team_member::{
    ActiveModel as TeamMemberActiveModel, Column as TeamMemberColumn, Entity as TeamMember,
    Model as TeamMemberModel, PrimaryKey as TeamMemberPrimaryKey, Relation as TeamMemberRelation,
};
