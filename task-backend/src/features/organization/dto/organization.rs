// task-backend/src/api/dto/organization_dto.rs

use crate::core::subscription_tier::SubscriptionTier;
use crate::domain::organization_model::{
    Organization, OrganizationMember, OrganizationRole, OrganizationSettings,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// 組織作成リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateOrganizationRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Organization name must be 1-100 characters"
    ))]
    pub name: String,

    #[validate(length(max = 1000, message = "Description cannot exceed 1000 characters"))]
    pub description: Option<String>,

    pub subscription_tier: SubscriptionTier,
}

/// 組織更新リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateOrganizationRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Organization name must be 1-100 characters"
    ))]
    pub name: Option<String>,

    #[validate(length(max = 1000, message = "Description cannot exceed 1000 characters"))]
    pub description: Option<String>,
}

/// 組織設定更新リクエスト
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrganizationSettingsRequest {
    pub allow_public_teams: Option<bool>,
    pub require_approval_for_new_members: Option<bool>,
    pub enable_single_sign_on: Option<bool>,
    pub default_team_subscription_tier: Option<SubscriptionTier>,
}

/// 組織サブスクリプション更新リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateOrganizationSubscriptionRequest {
    pub subscription_tier: SubscriptionTier,
}

/// 組織メンバー招待リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct InviteOrganizationMemberRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,

    pub user_id: Option<Uuid>,

    pub role: OrganizationRole,
}

/// 組織メンバー役割更新リクエスト
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrganizationMemberRoleRequest {
    pub role: OrganizationRole,
}

/// 組織検索クエリ
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationSearchQuery {
    pub name: Option<String>,
    pub owner_id: Option<Uuid>,
    pub subscription_tier: Option<SubscriptionTier>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl Default for OrganizationSearchQuery {
    fn default() -> Self {
        Self {
            name: None,
            owner_id: None,
            subscription_tier: None,
            page: Some(1),
            page_size: Some(20),
        }
    }
}

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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub members: Vec<OrganizationMemberResponse>,
}

/// 組織メンバーレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationMemberResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub role: OrganizationRole,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<Uuid>,
}

/// 組織一覧レスポンス
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
}

/// 組織統計レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationStatsResponse {
    pub total_organizations: u32,
    pub organizations_by_tier: Vec<OrganizationTierStats>,
    pub total_teams: u32,
    pub total_members: u32,
    pub average_teams_per_organization: f64,
    pub average_members_per_organization: f64,
    pub most_active_organizations: Vec<OrganizationActivity>,
}

/// サブスクリプション階層別組織統計
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationTierStats {
    pub tier: SubscriptionTier,
    pub organization_count: u32,
    pub team_count: u32,
    pub member_count: u32,
}

/// 組織活動情報
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationActivity {
    pub organization_id: Uuid,
    pub organization_name: String,
    pub team_count: u32,
    pub member_count: u32,
    pub recent_activity_count: u32,
}

/// 組織招待レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationInvitationResponse {
    pub invitation_id: Uuid,
    pub organization_id: Uuid,
    pub organization_name: String,
    pub invited_user_id: Option<Uuid>,
    pub invited_email: String,
    pub role: OrganizationRole,
    pub invited_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// 組織メンバー詳細レスポンス（権限情報付き）
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationMemberDetailResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub role: OrganizationRole,
    pub is_owner: bool,
    pub is_admin: bool,
    pub can_manage: bool,
    pub can_create_teams: bool,
    pub can_invite_members: bool,
    pub can_change_settings: bool,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<Uuid>,
}

/// 組織容量チェックレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationCapacityResponse {
    pub organization_id: Uuid,
    pub organization_name: String,
    pub subscription_tier: SubscriptionTier,
    pub max_teams: u32,
    pub current_team_count: u32,
    pub can_add_teams: bool,
    pub max_members: u32,
    pub current_member_count: u32,
    pub can_add_members: bool,
    pub utilization_percentage: f64,
}

/// 組織使用状況情報（Phase 19互換性のため追加）
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationUsageInfo {
    pub current_tier: SubscriptionTier,
    pub max_teams_allowed: u32,
    pub max_members_allowed: u32,
    pub teams_usage_percentage: f32,
    pub members_usage_percentage: f32,
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
            current_team_count: 0,   // Will be populated by service
            current_member_count: 0, // Will be populated by service
            created_at: org.created_at,
        }
    }
}

impl From<(Organization, Vec<OrganizationMemberResponse>, u32)> for OrganizationResponse {
    fn from(
        (org, members, team_count): (Organization, Vec<OrganizationMemberResponse>, u32),
    ) -> Self {
        let current_member_count = members.len() as u32;
        Self {
            id: org.id,
            name: org.name,
            description: org.description,
            owner_id: org.owner_id,
            subscription_tier: org.subscription_tier,
            max_teams: org.max_teams,
            max_members: org.max_members,
            current_team_count: team_count,
            current_member_count,
            settings: org.settings,
            created_at: org.created_at,
            updated_at: org.updated_at,
            members,
        }
    }
}

// Phase 19互換性のために追加
impl From<(OrganizationMember, crate::domain::user_model::Model)> for OrganizationMemberResponse {
    fn from((member, user): (OrganizationMember, crate::domain::user_model::Model)) -> Self {
        Self {
            id: member.id,
            user_id: member.user_id,
            username: user.username,
            email: user.email,
            role: member.role,
            joined_at: member.joined_at,
            invited_by: member.invited_by,
        }
    }
}

impl From<(OrganizationMember, crate::domain::user_model::Model)>
    for OrganizationMemberDetailResponse
{
    fn from((member, user): (OrganizationMember, crate::domain::user_model::Model)) -> Self {
        let role = member.role.clone();
        Self {
            id: member.id,
            user_id: member.user_id,
            username: user.username,
            email: user.email,
            role: role.clone(),
            is_owner: matches!(role, OrganizationRole::Owner),
            is_admin: matches!(role, OrganizationRole::Admin),
            can_manage: matches!(role, OrganizationRole::Owner | OrganizationRole::Admin),
            can_create_teams: matches!(role, OrganizationRole::Owner | OrganizationRole::Admin),
            can_invite_members: matches!(role, OrganizationRole::Owner | OrganizationRole::Admin),
            can_change_settings: matches!(role, OrganizationRole::Owner),
            joined_at: member.joined_at,
            invited_by: member.invited_by,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_create_organization_request_validation() {
        // Valid request
        let valid_request = CreateOrganizationRequest {
            name: "Test Organization".to_string(),
            description: Some("A test organization".to_string()),
            subscription_tier: SubscriptionTier::Pro,
        };
        assert!(valid_request.validate().is_ok());

        // Empty name
        let invalid_request = CreateOrganizationRequest {
            name: "".to_string(),
            description: None,
            subscription_tier: SubscriptionTier::Free,
        };
        assert!(invalid_request.validate().is_err());

        // Name too long
        let long_name_request = CreateOrganizationRequest {
            name: "a".repeat(101),
            description: None,
            subscription_tier: SubscriptionTier::Free,
        };
        assert!(long_name_request.validate().is_err());

        // Description too long
        let long_desc_request = CreateOrganizationRequest {
            name: "Test Org".to_string(),
            description: Some("a".repeat(1001)),
            subscription_tier: SubscriptionTier::Free,
        };
        assert!(long_desc_request.validate().is_err());
    }

    #[test]
    fn test_invite_organization_member_request_validation() {
        // Valid with email
        let valid_email_request = InviteOrganizationMemberRequest {
            email: Some("test@example.com".to_string()),
            user_id: None,
            role: OrganizationRole::Member,
        };
        assert!(valid_email_request.validate().is_ok());

        // Valid with user_id
        let valid_user_request = InviteOrganizationMemberRequest {
            email: None,
            user_id: Some(Uuid::new_v4()),
            role: OrganizationRole::Admin,
        };
        assert!(valid_user_request.validate().is_ok());

        // Invalid email format
        let invalid_email_request = InviteOrganizationMemberRequest {
            email: Some("invalid-email".to_string()),
            user_id: None,
            role: OrganizationRole::Member,
        };
        assert!(invalid_email_request.validate().is_err());
    }

    #[test]
    fn test_organization_search_query_defaults() {
        let query = OrganizationSearchQuery::default();
        assert_eq!(query.page, Some(1));
        assert_eq!(query.page_size, Some(20));
        assert!(query.name.is_none());
        assert!(query.owner_id.is_none());
        assert!(query.subscription_tier.is_none());
    }

    #[test]
    fn test_organization_response_conversion() {
        let org = Organization::new(
            "Test Organization".to_string(),
            Some("Description".to_string()),
            Uuid::new_v4(),
            SubscriptionTier::Enterprise,
        );

        let members = vec![];
        let team_count = 5;
        let org_response = OrganizationResponse::from((org.clone(), members, team_count));

        assert_eq!(org_response.id, org.id);
        assert_eq!(org_response.name, org.name);
        assert_eq!(org_response.current_member_count, 0);
        assert_eq!(org_response.current_team_count, 5);
        assert!(org_response.members.is_empty());
    }

    #[test]
    fn test_organization_list_response_conversion() {
        let org = Organization::new(
            "Test Organization".to_string(),
            Some("Description".to_string()),
            Uuid::new_v4(),
            SubscriptionTier::Pro,
        );

        let list_response = OrganizationListResponse::from(org.clone());

        assert_eq!(list_response.id, org.id);
        assert_eq!(list_response.name, org.name);
        assert_eq!(list_response.subscription_tier, SubscriptionTier::Pro);
        assert_eq!(list_response.max_teams, 20);
        assert_eq!(list_response.max_members, 100);
    }

    #[test]
    fn test_organization_stats_response() {
        let stats = OrganizationStatsResponse {
            total_organizations: 5,
            organizations_by_tier: vec![
                OrganizationTierStats {
                    tier: SubscriptionTier::Free,
                    organization_count: 2,
                    team_count: 6,
                    member_count: 20,
                },
                OrganizationTierStats {
                    tier: SubscriptionTier::Pro,
                    organization_count: 2,
                    team_count: 30,
                    member_count: 150,
                },
                OrganizationTierStats {
                    tier: SubscriptionTier::Enterprise,
                    organization_count: 1,
                    team_count: 50,
                    member_count: 500,
                },
            ],
            total_teams: 86,
            total_members: 670,
            average_teams_per_organization: 17.2,
            average_members_per_organization: 134.0,
            most_active_organizations: vec![],
        };

        assert_eq!(stats.total_organizations, 5);
        assert_eq!(stats.organizations_by_tier.len(), 3);
        assert_eq!(stats.total_teams, 86);
        assert_eq!(stats.total_members, 670);
        assert_eq!(stats.average_teams_per_organization, 17.2);
        assert_eq!(stats.average_members_per_organization, 134.0);
    }

    #[test]
    fn test_update_organization_settings_request() {
        let settings_request = UpdateOrganizationSettingsRequest {
            allow_public_teams: Some(true),
            require_approval_for_new_members: Some(false),
            enable_single_sign_on: Some(true),
            default_team_subscription_tier: Some(SubscriptionTier::Pro),
        };

        assert_eq!(settings_request.allow_public_teams, Some(true));
        assert_eq!(
            settings_request.require_approval_for_new_members,
            Some(false)
        );
        assert_eq!(settings_request.enable_single_sign_on, Some(true));
        assert_eq!(
            settings_request.default_team_subscription_tier,
            Some(SubscriptionTier::Pro)
        );
    }
}
