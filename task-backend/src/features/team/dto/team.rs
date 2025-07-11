// task-backend/src/api/dto/team_dto.rs

use crate::core::subscription_tier::SubscriptionTier;
use crate::features::team::models::team::{Model as Team, TeamRole};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// チーム作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTeamRequest {
    #[validate(length(min = 1, max = 100, message = "Team name must be 1-100 characters"))]
    pub name: String,

    #[validate(length(max = 500, message = "Description cannot exceed 500 characters"))]
    pub description: Option<String>,

    pub organization_id: Option<Uuid>,
}

/// チーム更新リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateTeamRequest {
    #[validate(length(min = 1, max = 100, message = "Team name must be 1-100 characters"))]
    pub name: Option<String>,

    #[validate(length(max = 500, message = "Description cannot exceed 500 characters"))]
    pub description: Option<String>,
}

/// チームメンバー招待リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct InviteTeamMemberRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,

    pub user_id: Option<Uuid>,

    pub role: TeamRole,
}

/// チームメンバー役割更新リクエスト
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTeamMemberRoleRequest {
    pub role: TeamRole,
}

/// チーム検索クエリ（ページネーション情報を除く）
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TeamSearchQuery {
    pub name: Option<String>,
    pub organization_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub subscription_tier: Option<SubscriptionTier>,
}

/// チーム詳細レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub organization_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub subscription_tier: SubscriptionTier,
    pub max_members: i32,
    pub current_member_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub members: Vec<TeamMemberResponse>,
}

/// チームメンバーレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamMemberResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub role: TeamRole,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<Uuid>,
}

/// チームメンバー詳細レスポンス（権限情報付き）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMemberDetailResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub role: TeamRole,
    pub is_owner: bool,
    pub is_admin: bool,
    pub can_invite: bool,
    pub can_remove_members: bool,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<Uuid>,
}

/// チーム一覧レスポンス
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamListResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub organization_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub subscription_tier: SubscriptionTier,
    pub max_members: i32,
    pub current_member_count: i32,
    pub created_at: DateTime<Utc>,
}

/// チーム統計レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamStatsResponse {
    pub total_teams: i32,
    pub teams_by_tier: Vec<TeamTierStats>,
    pub total_members: i32,
    pub average_members_per_team: f64,
    pub most_active_teams: Vec<TeamActivity>,
}

/// サブスクリプション階層別チーム統計
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamTierStats {
    pub tier: SubscriptionTier,
    pub team_count: i32,
    pub member_count: i32,
}

/// チーム活動情報
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamActivity {
    pub team_id: Uuid,
    pub team_name: String,
    pub member_count: i32,
    pub recent_activity_count: i32,
}

impl From<Team> for TeamListResponse {
    fn from(team: Team) -> Self {
        let subscription_tier = team.get_subscription_tier();
        Self {
            id: team.id,
            name: team.name,
            description: team.description,
            organization_id: team.organization_id,
            owner_id: team.owner_id,
            subscription_tier,
            max_members: team.max_members,
            current_member_count: 0, // Will be populated by service
            created_at: team.created_at,
        }
    }
}

impl From<(Team, Vec<TeamMemberResponse>)> for TeamResponse {
    fn from((team, members): (Team, Vec<TeamMemberResponse>)) -> Self {
        let current_member_count = members.len() as i32;
        let subscription_tier = team.get_subscription_tier();
        Self {
            id: team.id,
            name: team.name,
            description: team.description,
            organization_id: team.organization_id,
            owner_id: team.owner_id,
            subscription_tier,
            max_members: team.max_members,
            current_member_count,
            created_at: team.created_at,
            updated_at: team.updated_at,
            members,
        }
    }
}

/// チーム一覧ページング取得クエリ
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamPaginationQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub organization_id: Option<Uuid>,
}

impl Default for TeamPaginationQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
            organization_id: None,
        }
    }
}

/// チーム一覧ページング取得レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamPaginationResponse {
    pub teams: Vec<TeamListResponse>,
    pub total_count: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

impl TeamPaginationResponse {
    pub fn new(teams: Vec<TeamListResponse>, total_count: u64, page: u64, page_size: u64) -> Self {
        let total_pages = total_count.div_ceil(page_size);
        Self {
            teams,
            total_count,
            page,
            page_size,
            total_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_create_team_request_validation() {
        // Valid request
        let valid_request = CreateTeamRequest {
            name: "Test Team".to_string(),
            description: Some("A test team".to_string()),
            organization_id: None,
        };
        assert!(valid_request.validate().is_ok());

        // Empty name
        let invalid_request = CreateTeamRequest {
            name: "".to_string(),
            description: None,
            organization_id: None,
        };
        assert!(invalid_request.validate().is_err());

        // Name too long
        let long_name_request = CreateTeamRequest {
            name: "a".repeat(101),
            description: None,
            organization_id: None,
        };
        assert!(long_name_request.validate().is_err());

        // Description too long
        let long_desc_request = CreateTeamRequest {
            name: "Test Team".to_string(),
            description: Some("a".repeat(501)),
            organization_id: None,
        };
        assert!(long_desc_request.validate().is_err());
    }

    #[test]
    fn test_invite_team_member_request_validation() {
        // Valid with email
        let valid_email_request = InviteTeamMemberRequest {
            email: Some("test@example.com".to_string()),
            user_id: None,
            role: TeamRole::Member,
        };
        assert!(valid_email_request.validate().is_ok());

        // Valid with user_id
        let valid_user_request = InviteTeamMemberRequest {
            email: None,
            user_id: Some(Uuid::new_v4()),
            role: TeamRole::Admin,
        };
        assert!(valid_user_request.validate().is_ok());

        // Invalid email format
        let invalid_email_request = InviteTeamMemberRequest {
            email: Some("invalid-email".to_string()),
            user_id: None,
            role: TeamRole::Member,
        };
        assert!(invalid_email_request.validate().is_err());
    }

    #[test]
    fn test_team_search_query_defaults() {
        let query = TeamSearchQuery::default();
        assert!(query.name.is_none());
        assert!(query.organization_id.is_none());
        assert!(query.owner_id.is_none());
        assert!(query.subscription_tier.is_none());
    }

    #[test]
    fn test_team_response_conversion() {
        let team = Team::new_team(
            "Test Team".to_string(),
            Some("Description".to_string()),
            None,
            Uuid::new_v4(),
            SubscriptionTier::Pro,
        );

        let members = vec![];
        let team_response = TeamResponse::from((team.clone(), members));

        assert_eq!(team_response.id, team.id);
        assert_eq!(team_response.name, team.name);
        assert_eq!(team_response.current_member_count, 0);
        assert!(team_response.members.is_empty());
    }

    #[test]
    fn test_team_list_response_conversion() {
        let team = Team::new_team(
            "Test Team".to_string(),
            Some("Description".to_string()),
            None,
            Uuid::new_v4(),
            SubscriptionTier::Free,
        );

        let list_response = TeamListResponse::from(team.clone());

        assert_eq!(list_response.id, team.id);
        assert_eq!(list_response.name, team.name);
        assert_eq!(list_response.subscription_tier, SubscriptionTier::Free);
        assert_eq!(list_response.max_members, 3);
    }

    #[test]
    fn test_team_stats_response() {
        let stats = TeamStatsResponse {
            total_teams: 10,
            teams_by_tier: vec![
                TeamTierStats {
                    tier: SubscriptionTier::Free,
                    team_count: 5,
                    member_count: 15,
                },
                TeamTierStats {
                    tier: SubscriptionTier::Pro,
                    team_count: 3,
                    member_count: 25,
                },
                TeamTierStats {
                    tier: SubscriptionTier::Enterprise,
                    team_count: 2,
                    member_count: 150,
                },
            ],
            total_members: 190,
            average_members_per_team: 19.0,
            most_active_teams: vec![],
        };

        assert_eq!(stats.total_teams, 10);
        assert_eq!(stats.teams_by_tier.len(), 3);
        assert_eq!(stats.total_members, 190);
        assert_eq!(stats.average_members_per_team, 19.0);
    }

    #[test]
    fn test_team_pagination_query_defaults() {
        let query = TeamPaginationQuery::default();
        assert_eq!(query.page, Some(1));
        assert_eq!(query.page_size, Some(20));
        assert!(query.organization_id.is_none());
    }

    #[test]
    fn test_team_pagination_response_creation() {
        let teams = vec![
            TeamListResponse {
                id: Uuid::new_v4(),
                name: "Team 1".to_string(),
                description: Some("Description 1".to_string()),
                organization_id: None,
                owner_id: Uuid::new_v4(),
                subscription_tier: SubscriptionTier::Free,
                max_members: 3,
                current_member_count: 2,
                created_at: Utc::now(),
            },
            TeamListResponse {
                id: Uuid::new_v4(),
                name: "Team 2".to_string(),
                description: Some("Description 2".to_string()),
                organization_id: None,
                owner_id: Uuid::new_v4(),
                subscription_tier: SubscriptionTier::Pro,
                max_members: 50,
                current_member_count: 25,
                created_at: Utc::now(),
            },
        ];

        let response = TeamPaginationResponse::new(teams.clone(), 2, 1, 20);

        assert_eq!(response.teams.len(), 2);
        assert_eq!(response.total_count, 2);
        assert_eq!(response.page, 1);
        assert_eq!(response.page_size, 20);
        assert_eq!(response.total_pages, 1);
    }

    #[test]
    fn test_team_pagination_response_multiple_pages() {
        let teams = vec![];
        let response = TeamPaginationResponse::new(teams, 45, 2, 20);

        assert_eq!(response.total_count, 45);
        assert_eq!(response.page, 2);
        assert_eq!(response.page_size, 20);
        assert_eq!(response.total_pages, 3); // 45 / 20 = 2.25 -> 3 pages
    }
}
