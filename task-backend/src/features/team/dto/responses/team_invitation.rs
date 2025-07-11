use crate::features::team::models::team_invitation::Model as TeamInvitation;
use crate::features::user::models::user::SafeUser;
use crate::shared::types::pagination::PaginationMeta;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// チーム招待レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInvitationResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub team_name: String,
    pub invited_email: String,
    pub invited_user_id: Option<Uuid>,
    pub invited_by: SafeUser,
    pub status: String,
    pub message: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub declined_at: Option<DateTime<Utc>>,
    pub decline_reason: Option<String>,
}

/// 一括招待レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct BulkInviteResponse {
    pub successful: Vec<TeamInvitationResponse>,
    pub failed: Vec<String>,
    pub total_sent: usize,
}

/// チーム招待リストレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamInvitationsListResponse {
    pub invitations: Vec<TeamInvitationResponse>,
    pub total: i64,
    pub status_counts: TeamInvitationStatusCounts,
}

/// 招待ステータス別カウント
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamInvitationStatusCounts {
    pub pending: i64,
    pub accepted: i64,
    pub declined: i64,
    pub expired: i64,
    pub cancelled: i64,
}

/// 招待統計レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct InvitationStatisticsResponse {
    pub total_sent: i64,
    pub pending: i64,
    pub accepted: i64,
    pub declined: i64,
    pub expired: i64,
    pub acceptance_rate: f64,
}

/// 招待存在確認レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckInvitationResponse {
    pub exists: bool,
    pub invitation_id: Option<Uuid>,
}

/// 招待ページネーションレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct InvitationPaginationResponse {
    pub invitations: Vec<TeamInvitationResponse>,
    pub meta: PaginationMeta,
}

impl InvitationPaginationResponse {
    // TODO: Phase 19で本来の使用箇所が移行されたら#[allow(dead_code)]を削除
    #[allow(dead_code)]
    pub fn new(invitations: Vec<TeamInvitationResponse>, meta: PaginationMeta) -> Self {
        Self { invitations, meta }
    }
}

/// ユーザー招待統計レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInvitationStatsResponse {
    pub pending_invitations: i64,
    pub teams_joined: i64,
    pub invitations_sent: i64,
    pub recent_invitations: Vec<TeamInvitationResponse>,
}

/// 一括ステータス更新レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct BulkUpdateStatusResponse {
    pub updated_count: usize,
    pub failed_count: usize,
    pub failed_ids: Vec<Uuid>,
}

/// ドメインモデルからの変換実装
impl From<TeamInvitation> for TeamInvitationResponse {
    fn from(invitation: TeamInvitation) -> Self {
        Self {
            id: invitation.id,
            team_id: invitation.team_id,
            team_name: String::new(), // This should be populated from the team relation
            invited_email: invitation.invited_email,
            invited_user_id: invitation.invited_user_id,
            invited_by: SafeUser {
                id: invitation.invited_by_user_id,
                username: String::new(),
                email: String::new(),
                is_active: true,
                email_verified: true,
                role_id: Uuid::new_v4(),
                subscription_tier: String::new(),
                last_login_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }, // This should be populated from the user relation
            status: invitation.status,
            message: invitation.message,
            expires_at: invitation
                .expires_at
                .unwrap_or_else(|| Utc::now() + chrono::Duration::days(7)),
            created_at: invitation.created_at,
            accepted_at: invitation.accepted_at,
            declined_at: invitation.declined_at,
            decline_reason: invitation.decline_reason,
        }
    }
}

/// TeamInvitationStatisticsからの変換実装
impl From<crate::features::team::services::team_invitation::TeamInvitationStatistics>
    for InvitationStatisticsResponse
{
    fn from(
        stats: crate::features::team::services::team_invitation::TeamInvitationStatistics,
    ) -> Self {
        let acceptance_rate = if stats.total > 0 {
            (stats.accepted as f64 / stats.total as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total_sent: stats.total as i64,
            pending: stats.pending as i64,
            accepted: stats.accepted as i64,
            declined: stats.declined as i64,
            expired: stats.expired as i64,
            acceptance_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_invitation() -> TeamInvitation {
        TeamInvitation {
            id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            invited_email: "test@example.com".to_string(),
            invited_user_id: None,
            invited_by_user_id: Uuid::new_v4(),
            status: crate::features::team::models::team_invitation::TeamInvitationStatus::Pending
                .to_string(),
            message: Some("Welcome to the team!".to_string()),
            expires_at: Some(Utc::now() + chrono::Duration::days(7)),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            accepted_at: None,
            declined_at: None,
            decline_reason: None,
        }
    }

    #[test]
    fn test_team_invitation_response_from_model() {
        let invitation = create_test_invitation();
        let response = TeamInvitationResponse::from(invitation.clone());

        assert_eq!(response.id, invitation.id);
        assert_eq!(response.team_id, invitation.team_id);
        assert_eq!(response.invited_email, invitation.invited_email);
        assert_eq!(response.status, invitation.status);
        assert_eq!(response.message, invitation.message);
    }

    #[test]
    fn test_bulk_invite_response() {
        let successful_invitation = TeamInvitationResponse::from(create_test_invitation());
        let response = BulkInviteResponse {
            successful: vec![successful_invitation],
            failed: vec!["invalid@".to_string()],
            total_sent: 2,
        };

        assert_eq!(response.successful.len(), 1);
        assert_eq!(response.failed.len(), 1);
        assert_eq!(response.total_sent, 2);
    }

    #[test]
    fn test_invitation_statistics_response_conversion() {
        let stats = InvitationStatisticsResponse {
            total_sent: 100,
            pending: 20,
            accepted: 70,
            declined: 5,
            expired: 5,
            acceptance_rate: 70.0,
        };

        assert_eq!(stats.acceptance_rate, 70.0);
        assert_eq!(
            stats.total_sent,
            stats.pending + stats.accepted + stats.declined + stats.expired
        );
    }

    #[test]
    fn test_check_invitation_response() {
        let exists_response = CheckInvitationResponse {
            exists: true,
            invitation_id: Some(Uuid::new_v4()),
        };
        assert!(exists_response.exists);
        assert!(exists_response.invitation_id.is_some());

        let not_exists_response = CheckInvitationResponse {
            exists: false,
            invitation_id: None,
        };
        assert!(!not_exists_response.exists);
        assert!(not_exists_response.invitation_id.is_none());
    }

    #[test]
    fn test_invitation_pagination_response_creation() {
        let invitations = vec![
            TeamInvitationResponse::from(create_test_invitation()),
            TeamInvitationResponse::from(create_test_invitation()),
        ];

        let meta = PaginationMeta {
            page: 1,
            per_page: 10,
            total_pages: 1,
            total_count: 2,
            has_next: false,
            has_prev: false,
        };

        let response = InvitationPaginationResponse::new(invitations.clone(), meta.clone());
        assert_eq!(response.invitations.len(), 2);
        assert_eq!(response.meta.total_count, 2);
    }

    #[test]
    fn test_user_invitation_stats_response() {
        let stats = UserInvitationStatsResponse {
            pending_invitations: 5,
            teams_joined: 3,
            invitations_sent: 10,
            recent_invitations: vec![],
        };

        assert_eq!(stats.pending_invitations, 5);
        assert_eq!(stats.teams_joined, 3);
        assert_eq!(stats.invitations_sent, 10);
    }

    #[test]
    fn test_bulk_update_status_response() {
        let response = BulkUpdateStatusResponse {
            updated_count: 8,
            failed_count: 2,
            failed_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
        };

        assert_eq!(response.updated_count, 8);
        assert_eq!(response.failed_count, 2);
        assert_eq!(response.failed_ids.len(), 2);
    }

    #[test]
    fn test_invitation_statistics_response_creation() {
        let stats = InvitationStatisticsResponse {
            total_sent: 150,
            pending: 30,
            accepted: 100,
            declined: 10,
            expired: 10,
            acceptance_rate: 66.67,
        };

        assert_eq!(stats.total_sent, 150);
        assert!(stats.acceptance_rate > 66.0 && stats.acceptance_rate < 67.0);
    }
}
