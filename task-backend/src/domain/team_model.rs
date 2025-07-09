// task-backend/src/domain/team_model.rs

use crate::core::subscription_tier::SubscriptionTier;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait, DbErr, Set};
use serde::{Deserialize, Serialize};

/// チームエンティティ
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "teams")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    #[sea_orm(nullable)]
    pub organization_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub subscription_tier: String,
    pub max_members: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::OwnerId",
        to = "crate::domain::user_model::Column::Id"
    )]
    Owner,
    #[sea_orm(
        belongs_to = "super::organization_model::Entity",
        from = "Column::OrganizationId",
        to = "super::organization_model::Column::Id"
    )]
    Organization,
    #[sea_orm(has_many = "super::team_member_model::Entity")]
    TeamMembers,
    #[sea_orm(has_many = "super::team_invitation_model::Entity")]
    TeamInvitations,
}

impl Related<crate::domain::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Owner.def()
    }
}

impl Related<super::organization_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Organization.def()
    }
}

impl Related<super::team_invitation_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TeamInvitations.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..ActiveModelTrait::default()
        }
    }

    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert {
            self.updated_at = Set(Utc::now());
        }
        Ok(self)
    }
}

/// チーム内の役割
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeamRole {
    Owner,  // チーム所有者
    Admin,  // チーム管理者
    Member, // 一般メンバー
    Viewer, // 閲覧のみ
}

impl std::fmt::Display for TeamRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamRole::Owner => write!(f, "owner"),
            TeamRole::Admin => write!(f, "admin"),
            TeamRole::Member => write!(f, "member"),
            TeamRole::Viewer => write!(f, "viewer"),
        }
    }
}

impl std::str::FromStr for TeamRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(TeamRole::Owner),
            "admin" => Ok(TeamRole::Admin),
            "member" => Ok(TeamRole::Member),
            "viewer" => Ok(TeamRole::Viewer),
            _ => Err(format!("Invalid team role: {}", s)),
        }
    }
}

impl TeamRole {
    /// Check if role can manage team settings
    pub fn can_manage(&self) -> bool {
        matches!(self, TeamRole::Owner | TeamRole::Admin)
    }

    /// Check if role can invite members
    pub fn can_invite(&self) -> bool {
        matches!(self, TeamRole::Owner | TeamRole::Admin)
    }
}

impl Model {
    /// 新しいチームを作成
    pub fn new_team(
        name: String,
        description: Option<String>,
        organization_id: Option<Uuid>,
        owner_id: Uuid,
        subscription_tier: SubscriptionTier,
    ) -> Self {
        let max_members = match subscription_tier {
            SubscriptionTier::Free => 3,
            SubscriptionTier::Pro => 10,
            SubscriptionTier::Enterprise => 100,
        };

        Self {
            id: Uuid::new_v4(),
            name,
            description,
            organization_id,
            owner_id,
            subscription_tier: subscription_tier.to_string(),
            max_members,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// サブスクリプション階層を取得
    pub fn get_subscription_tier(&self) -> SubscriptionTier {
        self.subscription_tier
            .parse()
            .unwrap_or(SubscriptionTier::Free)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_role_permissions() {
        let owner = TeamRole::Owner;
        let admin = TeamRole::Admin;
        let member = TeamRole::Member;
        let viewer = TeamRole::Viewer;

        // Owner permissions
        assert!(owner.can_manage());
        assert!(owner.can_invite());

        // Admin permissions
        assert!(admin.can_manage());
        assert!(admin.can_invite());

        // Member permissions
        assert!(!member.can_manage());
        assert!(!member.can_invite());

        // Viewer permissions
        assert!(!viewer.can_manage());
        assert!(!viewer.can_invite());
    }

    #[test]
    fn test_team_creation() {
        let owner_id = Uuid::new_v4();
        let team = Model::new_team(
            "Test Team".to_string(),
            Some("A test team".to_string()),
            None,
            owner_id,
            SubscriptionTier::Pro,
        );

        assert_eq!(team.name, "Test Team");
        assert_eq!(team.description, Some("A test team".to_string()));
        assert_eq!(team.owner_id, owner_id);
        assert_eq!(team.subscription_tier, "pro");
        assert_eq!(team.max_members, 10);
    }

    #[test]
    fn test_team_member_limits() {
        let owner_id = Uuid::new_v4();
        let free_team = Model::new_team(
            "Free Team".to_string(),
            None,
            None,
            owner_id,
            SubscriptionTier::Free,
        );
        let pro_team = Model::new_team(
            "Pro Team".to_string(),
            None,
            None,
            owner_id,
            SubscriptionTier::Pro,
        );
        let enterprise_team = Model::new_team(
            "Enterprise Team".to_string(),
            None,
            None,
            owner_id,
            SubscriptionTier::Enterprise,
        );

        assert_eq!(free_team.max_members, 3);
        assert_eq!(pro_team.max_members, 10);
        assert_eq!(enterprise_team.max_members, 100);

        // Check member limits are set correctly
        assert_eq!(free_team.max_members, 3);
        assert!(2 < free_team.max_members);
        assert!(3 >= free_team.max_members);
    }

    #[test]
    fn test_team_role_string_conversion() {
        assert_eq!(TeamRole::Owner.to_string(), "owner");
        assert_eq!(TeamRole::Admin.to_string(), "admin");
        assert_eq!(TeamRole::Member.to_string(), "member");
        assert_eq!(TeamRole::Viewer.to_string(), "viewer");

        assert_eq!("owner".parse::<TeamRole>().unwrap(), TeamRole::Owner);
        assert_eq!("admin".parse::<TeamRole>().unwrap(), TeamRole::Admin);
        assert_eq!("member".parse::<TeamRole>().unwrap(), TeamRole::Member);
        assert_eq!("viewer".parse::<TeamRole>().unwrap(), TeamRole::Viewer);

        assert!("invalid".parse::<TeamRole>().is_err());
    }
}
