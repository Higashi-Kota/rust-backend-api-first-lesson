use crate::features::team::models::team::Model as Team;
use crate::features::user::models::user::SafeUser;
use crate::shared::types::pagination::PaginationMeta;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// チームレスポンス（詳細）
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub organization_id: Option<Uuid>,
    pub owner: SafeUser,
    pub subscription_tier: String,
    pub max_members: i32,
    pub current_members: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub members: Vec<TeamMemberResponse>,
}

/// チームメンバーレスポンス（基本情報）
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamMemberResponse {
    pub user_id: Uuid,
    pub user: SafeUser,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<SafeUser>,
}

/// チームメンバー詳細レスポンス（権限情報付き）
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamMemberDetailResponse {
    pub user_id: Uuid,
    pub user: SafeUser,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<SafeUser>,
    pub permissions: Vec<String>,
    pub can_manage_team: bool,
    pub can_invite_members: bool,
    pub can_remove_members: bool,
}

/// チームリストレスポンス（一覧表示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamListResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub organization_id: Option<Uuid>,
    pub organization_name: Option<String>,
    pub subscription_tier: String,
    pub member_count: i32,
    pub max_members: i32,
    pub created_at: DateTime<Utc>,
    pub is_owner: bool,
}

/// チーム統計レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamStatsResponse {
    pub total_teams: i64,
    pub total_members: i64,
    pub teams_by_tier: Vec<TeamTierStats>,
    pub recent_activities: Vec<TeamActivity>,
}

/// チームの階層別統計
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamTierStats {
    pub tier: String,
    pub team_count: i64,
    pub average_members: f64,
}

/// チームアクティビティ
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamActivity {
    pub team_id: Uuid,
    pub team_name: String,
    pub activity_type: String,
    pub timestamp: DateTime<Utc>,
}

/// チームページネーションレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamPaginationResponse {
    pub teams: Vec<TeamListResponse>,
    pub meta: PaginationMeta,
}

impl TeamPaginationResponse {
    // TODO: Phase 19で本来の使用箇所が移行されたら#[allow(dead_code)]を削除
    #[allow(dead_code)]
    pub fn new(teams: Vec<TeamListResponse>, meta: PaginationMeta) -> Self {
        Self { teams, meta }
    }
}

/// ドメインモデルからの変換実装
impl From<Team> for TeamListResponse {
    fn from(team: Team) -> Self {
        Self {
            id: team.id,
            name: team.name,
            description: team.description,
            organization_id: team.organization_id,
            organization_name: None,
            subscription_tier: team.subscription_tier,
            member_count: 0,
            max_members: team.max_members,
            created_at: team.created_at,
            is_owner: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_response_conversion() {
        let team = Team {
            id: Uuid::new_v4(),
            name: "Test Team".to_string(),
            description: Some("Test Description".to_string()),
            organization_id: None,
            owner_id: Uuid::new_v4(),
            subscription_tier: "free".to_string(),
            max_members: 5,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = TeamListResponse::from(team.clone());
        assert_eq!(response.id, team.id);
        assert_eq!(response.name, team.name);
        assert_eq!(response.description, team.description);
        assert_eq!(response.subscription_tier, team.subscription_tier);
    }

    #[test]
    fn test_team_list_response_conversion() {
        let team = Team {
            id: Uuid::new_v4(),
            name: "Test Team".to_string(),
            description: None,
            organization_id: Some(Uuid::new_v4()),
            owner_id: Uuid::new_v4(),
            subscription_tier: "pro".to_string(),
            max_members: 20,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = TeamListResponse::from(team.clone());
        assert_eq!(response.organization_id, team.organization_id);
        assert_eq!(response.max_members, team.max_members);
        assert_eq!(response.member_count, 0);
        assert!(!response.is_owner);
    }

    #[test]
    fn test_team_stats_response() {
        let stats = TeamStatsResponse {
            total_teams: 100,
            total_members: 500,
            teams_by_tier: vec![
                TeamTierStats {
                    tier: "free".to_string(),
                    team_count: 60,
                    average_members: 3.5,
                },
                TeamTierStats {
                    tier: "pro".to_string(),
                    team_count: 30,
                    average_members: 8.2,
                },
                TeamTierStats {
                    tier: "enterprise".to_string(),
                    team_count: 10,
                    average_members: 15.5,
                },
            ],
            recent_activities: vec![],
        };

        assert_eq!(stats.total_teams, 100);
        assert_eq!(stats.total_members, 500);
        assert_eq!(stats.teams_by_tier.len(), 3);
    }

    #[test]
    fn test_team_pagination_response_creation() {
        let teams = vec![
            TeamListResponse {
                id: Uuid::new_v4(),
                name: "Team 1".to_string(),
                description: None,
                organization_id: None,
                organization_name: None,
                subscription_tier: "free".to_string(),
                member_count: 3,
                max_members: 5,
                created_at: Utc::now(),
                is_owner: true,
            },
            TeamListResponse {
                id: Uuid::new_v4(),
                name: "Team 2".to_string(),
                description: None,
                organization_id: None,
                organization_name: None,
                subscription_tier: "pro".to_string(),
                member_count: 10,
                max_members: 20,
                created_at: Utc::now(),
                is_owner: false,
            },
        ];

        let meta = PaginationMeta {
            page: 1,
            per_page: 10,
            total_pages: 1,
            total_count: 2,
            has_next: false,
            has_prev: false,
        };

        let response = TeamPaginationResponse::new(teams.clone(), meta.clone());
        assert_eq!(response.teams.len(), 2);
        assert_eq!(response.meta.total_count, 2);
    }

    #[test]
    fn test_team_pagination_response_multiple_pages() {
        let meta = PaginationMeta {
            page: 2,
            per_page: 10,
            total_pages: 5,
            total_count: 45,
            has_next: true,
            has_prev: true,
        };

        let response = TeamPaginationResponse::new(vec![], meta.clone());
        assert_eq!(response.meta.page, 2);
        assert_eq!(response.meta.total_pages, 5);
        assert_eq!(response.meta.total_count, 45);
    }
}
