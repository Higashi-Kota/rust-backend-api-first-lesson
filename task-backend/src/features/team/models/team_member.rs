// task-backend/src/features/team/models/team_member.rs

use super::team::TeamRole;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

/// チームメンバーエンティティ
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "team_members")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    #[sea_orm(nullable)]
    pub invited_by: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::team::Entity",
        from = "Column::TeamId",
        to = "super::team::Column::Id"
    )]
    Team,
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::UserId",
        to = "crate::domain::user_model::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::InvitedBy",
        to = "crate::domain::user_model::Column::Id"
    )]
    Inviter,
}

impl Related<super::team::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Team.def()
    }
}

impl Related<crate::domain::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            joined_at: Set(Utc::now()),
            ..ActiveModelTrait::default()
        }
    }
}

impl Model {
    /// 新しいチームメンバーを作成
    pub fn new_member(
        team_id: Uuid,
        user_id: Uuid,
        role: TeamRole,
        invited_by: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            team_id,
            user_id,
            role: role.to_string(),
            joined_at: Utc::now(),
            invited_by,
        }
    }

    /// 役割を取得
    pub fn get_role(&self) -> TeamRole {
        self.role.parse().unwrap_or(TeamRole::Viewer)
    }

    /// オーナーかチェック
    pub fn is_owner(&self) -> bool {
        self.get_role() == TeamRole::Owner
    }

    /// 管理者かチェック
    pub fn is_admin(&self) -> bool {
        self.get_role() == TeamRole::Admin
    }

    /// 管理権限を持つかチェック
    pub fn can_manage(&self) -> bool {
        self.get_role().can_manage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_member_creation() {
        let team_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let invited_by = Uuid::new_v4();

        let member = Model::new_member(team_id, user_id, TeamRole::Member, Some(invited_by));

        assert_eq!(member.team_id, team_id);
        assert_eq!(member.user_id, user_id);
        assert_eq!(member.role, "member");
        assert_eq!(member.invited_by, Some(invited_by));
        assert!(!member.is_owner());
        assert!(!member.is_admin());
        assert!(!member.can_manage());
    }

    #[test]
    fn test_team_member_role_parsing() {
        let owner = Model::new_member(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Owner, None);
        let admin = Model::new_member(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Admin, None);
        let member = Model::new_member(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Member, None);

        assert_eq!(owner.get_role(), TeamRole::Owner);
        assert_eq!(admin.get_role(), TeamRole::Admin);
        assert_eq!(member.get_role(), TeamRole::Member);

        assert!(owner.is_owner());
        assert!(admin.is_admin());
        assert!(owner.can_manage());
        assert!(admin.can_manage());
        assert!(!member.can_manage());
    }
}
