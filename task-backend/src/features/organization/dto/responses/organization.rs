// TODO: Phase 20で統一後は直接modelsからインポート
use crate::core::subscription_tier::SubscriptionTier;
use crate::domain::organization_model::{
    Organization, OrganizationMember, OrganizationRole, OrganizationSettings,
};
use crate::domain::user_model::Model as User;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 組織詳細レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub subscription_tier: SubscriptionTier,
    pub max_teams: u32,
    pub max_members: u32,
    pub current_team_count: u32,
    pub current_member_count: u32,
    pub settings: OrganizationSettings,
    pub members: Vec<OrganizationMemberResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<(Organization, Vec<OrganizationMemberResponse>, u32)> for OrganizationResponse {
    fn from(
        (org, members, team_count): (Organization, Vec<OrganizationMemberResponse>, u32),
    ) -> Self {
        Self {
            id: org.id,
            name: org.name,
            description: org.description,
            owner_id: org.owner_id,
            subscription_tier: org.subscription_tier,
            max_teams: org.max_teams,
            max_members: org.max_members,
            current_team_count: team_count,
            current_member_count: members.len() as u32,
            settings: org.settings,
            members,
            created_at: org.created_at,
            updated_at: org.updated_at,
        }
    }
}

/// 組織一覧アイテムレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationListResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub subscription_tier: SubscriptionTier,
    pub max_teams: u32,
    pub max_members: u32,
    pub current_team_count: u32,
    pub current_member_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Organization> for OrganizationListResponse {
    fn from(org: Organization) -> Self {
        Self {
            id: org.id,
            name: org.name,
            description: org.description,
            owner_id: org.owner_id,
            subscription_tier: org.subscription_tier,
            max_teams: org.max_teams,
            max_members: org.max_members,
            current_team_count: 0,   // サービス層で設定される
            current_member_count: 0, // サービス層で設定される
            created_at: org.created_at,
            updated_at: org.updated_at,
        }
    }
}

/// 組織メンバーレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationMemberResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub role: OrganizationRole,
    pub joined_at: DateTime<Utc>,
}

impl From<(OrganizationMember, User)> for OrganizationMemberResponse {
    fn from((member, user): (OrganizationMember, User)) -> Self {
        Self {
            id: member.id,
            user_id: member.user_id,
            name: user.username, // Use username as display name
            email: user.email,
            role: member.role,
            joined_at: member.joined_at,
        }
    }
}

/// 組織メンバー詳細レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationMemberDetailResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub role: OrganizationRole,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<Uuid>,
}

impl From<(OrganizationMember, User)> for OrganizationMemberDetailResponse {
    fn from((member, user): (OrganizationMember, User)) -> Self {
        Self {
            id: member.id,
            organization_id: member.organization_id,
            user_id: member.user_id,
            name: user.username,
            email: user.email,
            role: member.role,
            joined_at: member.joined_at,
            invited_by: member.invited_by,
        }
    }
}

/// 組織容量レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationCapacityResponse {
    pub organization_id: Uuid,
    pub max_teams: u32,
    pub current_team_count: u32,
    pub max_members: u32,
    pub current_member_count: u32,
    pub can_add_team: bool,
    pub can_add_member: bool,
}

/// 組織統計レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationStatsResponse {
    pub organization_id: Uuid,
    pub total_members: u32,
    pub total_teams: u32,
    pub owner_count: u32,
    pub admin_count: u32,
    pub member_count: u32,
    pub tier_info: OrganizationUsageInfo,
    pub recent_activity: Option<OrganizationActivity>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 組織使用状況情報
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationUsageInfo {
    pub current_tier: SubscriptionTier,
    pub max_teams_allowed: u32,
    pub max_members_allowed: u32,
    pub teams_usage_percentage: f32,
    pub members_usage_percentage: f32,
}

/// 組織階層統計
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationTierStats {
    pub tier: SubscriptionTier,
    pub organization_count: u32,
    pub team_count: u32,
    pub member_count: u32,
}

/// 組織アクティビティ
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationActivity {
    pub activity_type: String,
    pub description: String,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}
