// task-backend/src/domain/organization_model.rs

use crate::domain::subscription_tier::SubscriptionTier;
use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// SeaORM Entity Model for organizations table
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "organizations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub subscription_tier: String,
    pub max_teams: i32,
    pub max_members: i32,
    pub settings_json: String,
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
    #[sea_orm(has_many = "super::organization_department_model::Entity")]
    OrganizationDepartments,
    #[sea_orm(has_many = "super::organization_analytics_model::Entity")]
    OrganizationAnalytics,
    #[sea_orm(
        has_many = "crate::domain::team_model::Entity",
        from = "Column::Id",
        to = "crate::domain::team_model::Column::OrganizationId"
    )]
    Teams,
}

impl Related<crate::domain::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Owner.def()
    }
}

impl Related<super::organization_department_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrganizationDepartments.def()
    }
}

impl Related<super::organization_analytics_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrganizationAnalytics.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// SeaORM Model implementations
#[allow(dead_code)]
impl Model {
    pub fn new(
        name: String,
        owner_id: Uuid,
        subscription_tier: SubscriptionTier,
        description: Option<String>,
        settings: Option<OrganizationSettings>,
    ) -> Self {
        let settings = settings.unwrap_or_default();
        let (max_teams, max_members) = match subscription_tier {
            SubscriptionTier::Free => (3, 10),
            SubscriptionTier::Pro => (20, 100),
            SubscriptionTier::Enterprise => (100, 1000),
        };

        Self {
            id: Uuid::new_v4(),
            name,
            description,
            owner_id,
            subscription_tier: subscription_tier.to_string(),
            max_teams,
            max_members,
            settings_json: serde_json::to_string(&settings).unwrap_or_default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn get_subscription_tier(&self) -> SubscriptionTier {
        SubscriptionTier::from_str(&self.subscription_tier).unwrap_or(SubscriptionTier::Free)
    }

    pub fn get_settings(&self) -> Result<OrganizationSettings, serde_json::Error> {
        serde_json::from_str(&self.settings_json)
    }

    pub fn update_settings(&mut self, settings: OrganizationSettings) {
        self.settings_json = serde_json::to_string(&settings).unwrap_or_default();
        self.updated_at = Utc::now();
    }

    pub fn update_subscription_tier(&mut self, new_tier: SubscriptionTier) {
        self.subscription_tier = new_tier.to_string();

        let (max_teams, max_members) = match new_tier {
            SubscriptionTier::Free => (3, 10),
            SubscriptionTier::Pro => (20, 100),
            SubscriptionTier::Enterprise => (100, 1000),
        };

        self.max_teams = max_teams;
        self.max_members = max_members;
        self.updated_at = Utc::now();
    }

    pub fn can_add_teams(&self, current_team_count: i32) -> bool {
        current_team_count < self.max_teams
    }

    pub fn can_add_members(&self, current_member_count: i32) -> bool {
        current_member_count < self.max_members
    }

    pub fn update_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    // Convert from SeaORM Model to Organization struct for backward compatibility
    pub fn to_organization(&self) -> Result<Organization, serde_json::Error> {
        let settings = self.get_settings()?;
        Ok(Organization {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            owner_id: self.owner_id,
            subscription_tier: self.get_subscription_tier(),
            max_teams: self.max_teams as u32,
            max_members: self.max_members as u32,
            settings,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }

    // Convert from Organization struct to SeaORM Model
    pub fn from_organization(org: &Organization) -> Self {
        Self {
            id: org.id,
            name: org.name.clone(),
            description: org.description.clone(),
            owner_id: org.owner_id,
            subscription_tier: org.subscription_tier.to_string(),
            max_teams: org.max_teams as i32,
            max_members: org.max_members as i32,
            settings_json: serde_json::to_string(&org.settings).unwrap_or_default(),
            created_at: org.created_at,
            updated_at: org.updated_at,
        }
    }
}

/// 組織情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub subscription_tier: SubscriptionTier,
    pub max_teams: u32,
    pub max_members: u32,
    pub settings: OrganizationSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 組織設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettings {
    pub allow_public_teams: bool,
    pub require_approval_for_new_members: bool,
    pub enable_single_sign_on: bool,
    pub default_team_subscription_tier: SubscriptionTier,
}

impl Default for OrganizationSettings {
    fn default() -> Self {
        Self {
            allow_public_teams: false,
            require_approval_for_new_members: true,
            enable_single_sign_on: false,
            default_team_subscription_tier: SubscriptionTier::Free,
        }
    }
}

/// 組織メンバー情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationMember {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub role: OrganizationRole,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<Uuid>,
}

/// 組織内の役割
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrganizationRole {
    Owner,  // 組織所有者
    Admin,  // 組織管理者
    Member, // 一般メンバー
}

#[allow(dead_code)]
impl OrganizationRole {
    /// 役割レベルを数値で取得（高いほど権限が強い）
    pub fn level(&self) -> u8 {
        match self {
            OrganizationRole::Owner => 3,
            OrganizationRole::Admin => 2,
            OrganizationRole::Member => 1,
        }
    }

    /// 指定された役割以上の権限を持つかチェック
    pub fn is_at_least(&self, other: &OrganizationRole) -> bool {
        self.level() >= other.level()
    }

    /// 管理権限を持つかチェック
    pub fn can_manage(&self) -> bool {
        matches!(self, OrganizationRole::Owner | OrganizationRole::Admin)
    }

    /// チーム作成権限を持つかチェック
    pub fn can_create_teams(&self) -> bool {
        matches!(
            self,
            OrganizationRole::Owner | OrganizationRole::Admin | OrganizationRole::Member
        )
    }

    /// メンバー招待権限を持つかチェック
    pub fn can_invite_members(&self) -> bool {
        matches!(self, OrganizationRole::Owner | OrganizationRole::Admin)
    }

    /// 設定変更権限を持つかチェック
    pub fn can_change_settings(&self) -> bool {
        matches!(self, OrganizationRole::Owner | OrganizationRole::Admin)
    }
}

impl std::fmt::Display for OrganizationRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrganizationRole::Owner => write!(f, "owner"),
            OrganizationRole::Admin => write!(f, "admin"),
            OrganizationRole::Member => write!(f, "member"),
        }
    }
}

impl std::str::FromStr for OrganizationRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(OrganizationRole::Owner),
            "admin" => Ok(OrganizationRole::Admin),
            "member" => Ok(OrganizationRole::Member),
            _ => Err(format!("Invalid organization role: {}", s)),
        }
    }
}

#[allow(dead_code)]
impl Organization {
    /// 新しい組織を作成
    pub fn new(
        name: String,
        description: Option<String>,
        owner_id: Uuid,
        subscription_tier: SubscriptionTier,
    ) -> Self {
        let (max_teams, max_members) = match subscription_tier {
            SubscriptionTier::Free => (3, 10),
            SubscriptionTier::Pro => (20, 100),
            SubscriptionTier::Enterprise => (100, 1000),
        };

        let settings = OrganizationSettings {
            default_team_subscription_tier: subscription_tier,
            ..Default::default()
        };

        Self {
            id: Uuid::new_v4(),
            name,
            description,
            owner_id,
            subscription_tier,
            max_teams,
            max_members,
            settings,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 組織名を更新
    pub fn update_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    /// 説明を更新
    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    /// サブスクリプション階層を更新
    pub fn update_subscription_tier(&mut self, tier: SubscriptionTier) {
        self.subscription_tier = tier;
        let (max_teams, max_members) = match tier {
            SubscriptionTier::Free => (3, 10),
            SubscriptionTier::Pro => (20, 100),
            SubscriptionTier::Enterprise => (100, 1000),
        };
        self.max_teams = max_teams;
        self.max_members = max_members;
        self.updated_at = Utc::now();
    }

    /// 設定を更新
    pub fn update_settings(&mut self, settings: OrganizationSettings) {
        self.settings = settings;
        self.updated_at = Utc::now();
    }

    /// チーム数制限をチェック
    pub fn can_add_team(&self, current_team_count: u32) -> bool {
        current_team_count < self.max_teams
    }

    /// メンバー数制限をチェック
    pub fn can_add_member(&self, current_member_count: u32) -> bool {
        current_member_count < self.max_members
    }
}

#[allow(dead_code)]
impl OrganizationMember {
    /// 新しい組織メンバーを作成
    pub fn new(
        organization_id: Uuid,
        user_id: Uuid,
        role: OrganizationRole,
        invited_by: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            organization_id,
            user_id,
            role,
            joined_at: Utc::now(),
            invited_by,
        }
    }

    /// 役割を更新
    pub fn update_role(&mut self, role: OrganizationRole) {
        self.role = role;
    }

    /// オーナーかチェック
    pub fn is_owner(&self) -> bool {
        self.role == OrganizationRole::Owner
    }

    /// 管理者かチェック
    pub fn is_admin(&self) -> bool {
        self.role == OrganizationRole::Admin
    }

    /// 管理権限を持つかチェック
    pub fn can_manage(&self) -> bool {
        self.role.can_manage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_role_levels() {
        assert_eq!(OrganizationRole::Owner.level(), 3);
        assert_eq!(OrganizationRole::Admin.level(), 2);
        assert_eq!(OrganizationRole::Member.level(), 1);
    }

    #[test]
    fn test_organization_role_permissions() {
        let owner = OrganizationRole::Owner;
        let admin = OrganizationRole::Admin;
        let member = OrganizationRole::Member;

        // Owner permissions
        assert!(owner.can_manage());
        assert!(owner.can_create_teams());
        assert!(owner.can_invite_members());
        assert!(owner.can_change_settings());

        // Admin permissions
        assert!(admin.can_manage());
        assert!(admin.can_create_teams());
        assert!(admin.can_invite_members());
        assert!(admin.can_change_settings());

        // Member permissions
        assert!(!member.can_manage());
        assert!(member.can_create_teams());
        assert!(!member.can_invite_members());
        assert!(!member.can_change_settings());
    }

    #[test]
    fn test_organization_creation() {
        let owner_id = Uuid::new_v4();
        let org = Organization::new(
            "Test Org".to_string(),
            Some("A test organization".to_string()),
            owner_id,
            SubscriptionTier::Pro,
        );

        assert_eq!(org.name, "Test Org");
        assert_eq!(org.description, Some("A test organization".to_string()));
        assert_eq!(org.owner_id, owner_id);
        assert_eq!(org.subscription_tier, SubscriptionTier::Pro);
        assert_eq!(org.max_teams, 20);
        assert_eq!(org.max_members, 100);
    }

    #[test]
    fn test_organization_limits() {
        let owner_id = Uuid::new_v4();
        let free_org = Organization::new(
            "Free Org".to_string(),
            None,
            owner_id,
            SubscriptionTier::Free,
        );
        let pro_org =
            Organization::new("Pro Org".to_string(), None, owner_id, SubscriptionTier::Pro);
        let enterprise_org = Organization::new(
            "Enterprise Org".to_string(),
            None,
            owner_id,
            SubscriptionTier::Enterprise,
        );

        assert_eq!((free_org.max_teams, free_org.max_members), (3, 10));
        assert_eq!((pro_org.max_teams, pro_org.max_members), (20, 100));
        assert_eq!(
            (enterprise_org.max_teams, enterprise_org.max_members),
            (100, 1000)
        );

        assert!(free_org.can_add_team(2));
        assert!(!free_org.can_add_team(3));
        assert!(free_org.can_add_member(9));
        assert!(!free_org.can_add_member(10));
    }

    #[test]
    fn test_organization_member_creation() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let invited_by = Uuid::new_v4();

        let member =
            OrganizationMember::new(org_id, user_id, OrganizationRole::Member, Some(invited_by));

        assert_eq!(member.organization_id, org_id);
        assert_eq!(member.user_id, user_id);
        assert_eq!(member.role, OrganizationRole::Member);
        assert_eq!(member.invited_by, Some(invited_by));
        assert!(!member.is_owner());
        assert!(!member.is_admin());
        assert!(!member.can_manage());
    }

    #[test]
    fn test_organization_settings() {
        let settings = OrganizationSettings {
            allow_public_teams: true,
            require_approval_for_new_members: false,
            enable_single_sign_on: true,
            default_team_subscription_tier: SubscriptionTier::Pro,
        };

        assert!(settings.allow_public_teams);
        assert!(!settings.require_approval_for_new_members);
        assert!(settings.enable_single_sign_on);
        assert_eq!(
            settings.default_team_subscription_tier,
            SubscriptionTier::Pro
        );
    }

    #[test]
    fn test_organization_role_string_conversion() {
        assert_eq!(OrganizationRole::Owner.to_string(), "owner");
        assert_eq!(OrganizationRole::Admin.to_string(), "admin");
        assert_eq!(OrganizationRole::Member.to_string(), "member");

        assert_eq!(
            "owner".parse::<OrganizationRole>().unwrap(),
            OrganizationRole::Owner
        );
        assert_eq!(
            "admin".parse::<OrganizationRole>().unwrap(),
            OrganizationRole::Admin
        );
        assert_eq!(
            "member".parse::<OrganizationRole>().unwrap(),
            OrganizationRole::Member
        );

        assert!("invalid".parse::<OrganizationRole>().is_err());
    }

    #[test]
    fn test_organization_model_new() {
        let owner_id = Uuid::new_v4();
        let name = "Test Organization".to_string();
        let description = Some("A test organization".to_string());
        let expected_settings = OrganizationSettings {
            allow_public_teams: true,
            require_approval_for_new_members: false,
            enable_single_sign_on: true,
            default_team_subscription_tier: SubscriptionTier::Pro,
        };

        let model = Model::new(
            name.clone(),
            owner_id,
            SubscriptionTier::Pro,
            description.clone(),
            Some(expected_settings.clone()),
        );

        assert_eq!(model.name, name);
        assert_eq!(model.description, description);
        assert_eq!(model.owner_id, owner_id);
        assert_eq!(model.get_subscription_tier(), SubscriptionTier::Pro);
        assert_eq!(model.max_teams, 20);
        assert_eq!(model.max_members, 100);

        let retrieved_settings = model.get_settings().unwrap();
        assert_eq!(
            retrieved_settings.allow_public_teams,
            expected_settings.allow_public_teams
        );
        assert_eq!(
            retrieved_settings.require_approval_for_new_members,
            expected_settings.require_approval_for_new_members
        );
        assert_eq!(
            retrieved_settings.enable_single_sign_on,
            expected_settings.enable_single_sign_on
        );
        assert_eq!(
            retrieved_settings.default_team_subscription_tier,
            expected_settings.default_team_subscription_tier
        );
    }

    #[test]
    fn test_organization_model_subscription_tier_operations() {
        let owner_id = Uuid::new_v4();
        let mut model = Model::new(
            "Test Org".to_string(),
            owner_id,
            SubscriptionTier::Free,
            None,
            None,
        );

        // Initial free tier
        assert_eq!(model.get_subscription_tier(), SubscriptionTier::Free);
        assert_eq!(model.max_teams, 3);
        assert_eq!(model.max_members, 10);

        // Update to Pro
        model.update_subscription_tier(SubscriptionTier::Pro);
        assert_eq!(model.get_subscription_tier(), SubscriptionTier::Pro);
        assert_eq!(model.max_teams, 20);
        assert_eq!(model.max_members, 100);

        // Update to Enterprise
        model.update_subscription_tier(SubscriptionTier::Enterprise);
        assert_eq!(model.get_subscription_tier(), SubscriptionTier::Enterprise);
        assert_eq!(model.max_teams, 100);
        assert_eq!(model.max_members, 1000);
    }

    #[test]
    fn test_organization_model_settings_operations() {
        let owner_id = Uuid::new_v4();
        let mut model = Model::new(
            "Test Org".to_string(),
            owner_id,
            SubscriptionTier::Pro,
            None,
            None,
        );

        // Test default settings
        let initial_settings = model.get_settings().unwrap();
        assert!(!initial_settings.allow_public_teams);
        assert!(initial_settings.require_approval_for_new_members);
        assert!(!initial_settings.enable_single_sign_on);

        // Update settings
        let new_settings = OrganizationSettings {
            allow_public_teams: true,
            require_approval_for_new_members: false,
            enable_single_sign_on: true,
            default_team_subscription_tier: SubscriptionTier::Enterprise,
        };

        model.update_settings(new_settings.clone());

        let updated_settings = model.get_settings().unwrap();
        assert_eq!(
            updated_settings.allow_public_teams,
            new_settings.allow_public_teams
        );
        assert_eq!(
            updated_settings.require_approval_for_new_members,
            new_settings.require_approval_for_new_members
        );
        assert_eq!(
            updated_settings.enable_single_sign_on,
            new_settings.enable_single_sign_on
        );
        assert_eq!(
            updated_settings.default_team_subscription_tier,
            new_settings.default_team_subscription_tier
        );
    }

    #[test]
    fn test_organization_model_capacity_checks() {
        let owner_id = Uuid::new_v4();
        let model = Model::new(
            "Test Org".to_string(),
            owner_id,
            SubscriptionTier::Free, // 3 teams, 10 members max
            None,
            None,
        );

        // Team capacity checks
        assert!(model.can_add_teams(2)); // 2 < 3, should be true
        assert!(!model.can_add_teams(3)); // 3 >= 3, should be false
        assert!(!model.can_add_teams(5)); // 5 > 3, should be false

        // Member capacity checks
        assert!(model.can_add_members(9)); // 9 < 10, should be true
        assert!(!model.can_add_members(10)); // 10 >= 10, should be false
        assert!(!model.can_add_members(15)); // 15 > 10, should be false
    }

    #[test]
    fn test_organization_model_name_and_description_updates() {
        let owner_id = Uuid::new_v4();
        let mut model = Model::new(
            "Original Name".to_string(),
            owner_id,
            SubscriptionTier::Pro,
            Some("Original Description".to_string()),
            None,
        );

        // Update name
        let new_name = "Updated Organization Name".to_string();
        model.update_name(new_name.clone());
        assert_eq!(model.name, new_name);

        // Update description
        let new_description = Some("Updated organization description".to_string());
        model.update_description(new_description.clone());
        assert_eq!(model.description, new_description);

        // Clear description
        model.update_description(None);
        assert_eq!(model.description, None);
    }

    #[test]
    fn test_organization_model_conversion() {
        let owner_id = Uuid::new_v4();
        let model = Model::new(
            "Test Org".to_string(),
            owner_id,
            SubscriptionTier::Pro,
            Some("Test Description".to_string()),
            None,
        );

        // Convert to Organization struct
        let organization = model.to_organization().unwrap();
        assert_eq!(organization.id, model.id);
        assert_eq!(organization.name, model.name);
        assert_eq!(organization.description, model.description);
        assert_eq!(organization.owner_id, model.owner_id);
        assert_eq!(organization.subscription_tier, SubscriptionTier::Pro);
        assert_eq!(organization.max_teams, 20);
        assert_eq!(organization.max_members, 100);

        // Convert back to Model
        let converted_model = Model::from_organization(&organization);
        assert_eq!(converted_model.id, organization.id);
        assert_eq!(converted_model.name, organization.name);
        assert_eq!(converted_model.description, organization.description);
        assert_eq!(converted_model.owner_id, organization.owner_id);
        assert_eq!(
            converted_model.subscription_tier,
            organization.subscription_tier.to_string()
        );
        assert_eq!(converted_model.max_teams, organization.max_teams as i32);
        assert_eq!(converted_model.max_members, organization.max_members as i32);
    }
}
