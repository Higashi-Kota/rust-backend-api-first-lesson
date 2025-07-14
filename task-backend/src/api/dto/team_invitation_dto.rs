// task-backend/src/api/dto/team_invitation_dto.rs

use crate::api::dto::common::{PaginatedResponse, PaginationQuery};
use crate::domain::team_invitation_model::{Model as TeamInvitationModel, TeamInvitationStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BulkTeamInviteRequest {
    #[validate(length(min = 1, max = 50, message = "Must provide 1-50 email addresses"))]
    #[validate(custom(function = "validate_emails"))]
    pub emails: Vec<String>,

    #[validate(length(max = 500, message = "Message cannot exceed 500 characters"))]
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamInvitationResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub invited_email: String,
    pub invited_user_id: Option<Uuid>,
    pub invited_by_user_id: Uuid,
    pub status: TeamInvitationStatus,
    pub message: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub declined_at: Option<DateTime<Utc>>,
    pub decline_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkInviteResponse {
    pub success_count: usize,
    pub invitations: Vec<TeamInvitationResponse>,
    pub failed_emails: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct DeclineInvitationRequest {
    #[validate(length(max = 500, message = "Decline reason cannot exceed 500 characters"))]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamInvitationsListResponse {
    pub invitations: Vec<TeamInvitationResponse>,
    pub total_count: u64,
    pub status_counts: TeamInvitationStatusCounts,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamInvitationStatusCounts {
    pub pending: u64,
    pub accepted: u64,
    pub declined: u64,
    pub expired: u64,
    pub cancelled: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamInvitationQuery {
    pub status: Option<TeamInvitationStatus>,
    #[serde(flatten)]
    pub pagination: PaginationQuery,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ResendInvitationRequest {
    #[validate(length(max = 500, message = "Message cannot exceed 500 characters"))]
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvitationStatisticsResponse {
    pub total: u64,
    pub pending: u64,
    pub accepted: u64,
    pub declined: u64,
    pub expired: u64,
    pub cancelled: u64,
}

fn validate_emails(emails: &[String]) -> Result<(), ValidationError> {
    use regex::Regex;
    let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();

    for email in emails {
        if !email_regex.is_match(email) {
            return Err(ValidationError::new("Invalid email format"));
        }
    }
    Ok(())
}

impl From<TeamInvitationModel> for TeamInvitationResponse {
    fn from(model: TeamInvitationModel) -> Self {
        let status = model.get_status();
        Self {
            id: model.id,
            team_id: model.team_id,
            invited_email: model.invited_email,
            invited_user_id: model.invited_user_id,
            invited_by_user_id: model.invited_by_user_id,
            status,
            message: model.message,
            expires_at: model.expires_at,
            accepted_at: model.accepted_at,
            declined_at: model.declined_at,
            decline_reason: model.decline_reason,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl From<&TeamInvitationModel> for TeamInvitationResponse {
    fn from(model: &TeamInvitationModel) -> Self {
        Self {
            id: model.id,
            team_id: model.team_id,
            invited_email: model.invited_email.clone(),
            invited_user_id: model.invited_user_id,
            invited_by_user_id: model.invited_by_user_id,
            status: model.get_status(),
            message: model.message.clone(),
            expires_at: model.expires_at,
            accepted_at: model.accepted_at,
            declined_at: model.declined_at,
            decline_reason: model.decline_reason.clone(),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl From<crate::service::team_invitation_service::TeamInvitationStatistics>
    for InvitationStatisticsResponse
{
    fn from(stats: crate::service::team_invitation_service::TeamInvitationStatistics) -> Self {
        Self {
            total: stats.total,
            pending: stats.pending,
            accepted: stats.accepted,
            declined: stats.declined,
            expired: stats.expired,
            cancelled: stats.cancelled,
        }
    }
}

impl Default for TeamInvitationQuery {
    fn default() -> Self {
        Self {
            status: None,
            pagination: PaginationQuery {
                page: Some(1),
                per_page: Some(20),
            },
        }
    }
}

/// 単一招待作成リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateInvitationRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(max = 500, message = "Message cannot exceed 500 characters"))]
    pub message: Option<String>,
}

/// ユーザー招待一覧取得クエリ
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInvitationQuery {
    pub email: String,
}

/// チーム招待確認レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckInvitationResponse {
    pub exists: bool,
    pub invitation: Option<TeamInvitationResponse>,
}

/// 招待ページング取得レスポンス
pub type InvitationPaginationResponse = PaginatedResponse<TeamInvitationResponse>;

/// ユーザー招待統計レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInvitationStatsResponse {
    pub total_invitations: u64,
    pub pending_invitations: u64,
    pub accepted_invitations: u64,
    pub declined_invitations: u64,
}

/// 一括ステータス更新リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BulkUpdateStatusRequest {
    #[validate(length(min = 1, max = 100, message = "Must provide 1-100 invitation IDs"))]
    pub invitation_ids: Vec<Uuid>,
    pub new_status: TeamInvitationStatus,
}

/// 一括ステータス更新レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct BulkUpdateStatusResponse {
    pub updated_count: u64,
    pub failed_ids: Vec<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulk_invite_request_validation() {
        let valid_request = BulkTeamInviteRequest {
            emails: vec![
                "user1@example.com".to_string(),
                "user2@example.com".to_string(),
            ],
            message: Some("Welcome to our team!".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_empty_emails = BulkTeamInviteRequest {
            emails: vec![],
            message: None,
        };
        assert!(invalid_empty_emails.validate().is_err());

        let invalid_too_many_emails = BulkTeamInviteRequest {
            emails: (0..51).map(|i| format!("user{}@example.com", i)).collect(),
            message: None,
        };
        assert!(invalid_too_many_emails.validate().is_err());

        let invalid_email_format = BulkTeamInviteRequest {
            emails: vec!["invalid-email".to_string()],
            message: None,
        };
        assert!(invalid_email_format.validate().is_err());
    }

    #[test]
    fn test_decline_invitation_request_validation() {
        let valid_request = DeclineInvitationRequest {
            reason: Some("Not interested at this time".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let valid_no_reason = DeclineInvitationRequest { reason: None };
        assert!(valid_no_reason.validate().is_ok());

        let invalid_long_reason = DeclineInvitationRequest {
            reason: Some("a".repeat(501)),
        };
        assert!(invalid_long_reason.validate().is_err());
    }

    #[test]
    fn test_team_invitation_query_defaults() {
        let query = TeamInvitationQuery::default();
        let (page, per_page) = query.pagination.get_pagination();
        assert_eq!(page, 1);
        assert_eq!(per_page, 20);
        assert!(query.status.is_none());
    }

    #[test]
    fn test_team_invitation_query_boundaries() {
        let query = TeamInvitationQuery {
            status: None,
            pagination: PaginationQuery {
                page: Some(0),       // Should be clamped to 1
                per_page: Some(200), // Should be clamped to 100
            },
        };
        let (page, per_page) = query.pagination.get_pagination();
        assert_eq!(page, 1);
        assert_eq!(per_page, 100);

        let query2 = TeamInvitationQuery {
            status: None,
            pagination: PaginationQuery {
                page: Some(5),
                per_page: Some(0), // Should be clamped to 1
            },
        };
        let (page2, per_page2) = query2.pagination.get_pagination();
        assert_eq!(page2, 5);
        assert_eq!(per_page2, 1);
    }

    #[test]
    fn test_team_invitation_response_from_model() {
        let model = TeamInvitationModel::new(
            Uuid::new_v4(),
            "test@example.com".to_string(),
            Uuid::new_v4(),
            Some("Welcome!".to_string()),
            None,
        );

        let response = TeamInvitationResponse::from(&model);
        assert_eq!(response.invited_email, model.invited_email);
        assert_eq!(response.message, model.message);
        assert_eq!(response.status, TeamInvitationStatus::Pending);
    }

    #[test]
    fn test_invitation_statistics_response_conversion() {
        let stats = crate::service::team_invitation_service::TeamInvitationStatistics {
            total: 100,
            pending: 20,
            accepted: 65,
            declined: 10,
            expired: 5,
            cancelled: 0,
        };

        let response = InvitationStatisticsResponse::from(stats);
        assert_eq!(response.total, 100);
        assert_eq!(response.pending, 20);
        assert_eq!(response.accepted, 65);
        assert_eq!(response.declined, 10);
        assert_eq!(response.expired, 5);
        assert_eq!(response.cancelled, 0);
    }

    #[test]
    fn test_resend_invitation_request_validation() {
        let valid_request = ResendInvitationRequest {
            message: Some("Updated message".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let valid_no_message = ResendInvitationRequest { message: None };
        assert!(valid_no_message.validate().is_ok());

        let invalid_long_message = ResendInvitationRequest {
            message: Some("a".repeat(501)),
        };
        assert!(invalid_long_message.validate().is_err());
    }

    #[test]
    fn test_create_invitation_request_validation() {
        let valid_request = CreateInvitationRequest {
            email: "test@example.com".to_string(),
            message: Some("Welcome!".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let valid_no_message = CreateInvitationRequest {
            email: "user@domain.org".to_string(),
            message: None,
        };
        assert!(valid_no_message.validate().is_ok());

        let invalid_email = CreateInvitationRequest {
            email: "invalid-email".to_string(),
            message: None,
        };
        assert!(invalid_email.validate().is_err());

        let invalid_long_message = CreateInvitationRequest {
            email: "test@example.com".to_string(),
            message: Some("a".repeat(501)),
        };
        assert!(invalid_long_message.validate().is_err());
    }

    #[test]
    fn test_check_invitation_response() {
        let invitation_model = TeamInvitationModel::new(
            Uuid::new_v4(),
            "test@example.com".to_string(),
            Uuid::new_v4(),
            Some("Test message".to_string()),
            None,
        );

        let response_with_invitation = CheckInvitationResponse {
            exists: true,
            invitation: Some(TeamInvitationResponse::from(invitation_model)),
        };

        assert!(response_with_invitation.exists);
        assert!(response_with_invitation.invitation.is_some());

        let response_without_invitation = CheckInvitationResponse {
            exists: false,
            invitation: None,
        };

        assert!(!response_without_invitation.exists);
        assert!(response_without_invitation.invitation.is_none());
    }

    #[test]
    fn test_invitation_pagination_response_creation() {
        let invitations = vec![
            TeamInvitationResponse {
                id: Uuid::new_v4(),
                team_id: Uuid::new_v4(),
                invited_email: "user1@example.com".to_string(),
                invited_user_id: None,
                invited_by_user_id: Uuid::new_v4(),
                status: TeamInvitationStatus::Pending,
                message: Some("Welcome!".to_string()),
                expires_at: Some(Utc::now() + chrono::Duration::days(7)),
                accepted_at: None,
                declined_at: None,
                decline_reason: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            TeamInvitationResponse {
                id: Uuid::new_v4(),
                team_id: Uuid::new_v4(),
                invited_email: "user2@example.com".to_string(),
                invited_user_id: Some(Uuid::new_v4()),
                invited_by_user_id: Uuid::new_v4(),
                status: TeamInvitationStatus::Accepted,
                message: None,
                expires_at: None,
                accepted_at: Some(Utc::now()),
                declined_at: None,
                decline_reason: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        let response = InvitationPaginationResponse::new(invitations.clone(), 1, 10, 2);

        assert_eq!(response.items.len(), 2);
        assert_eq!(response.pagination.total_count, 2);
        assert_eq!(response.pagination.page, 1);
        assert_eq!(response.pagination.per_page, 10);
        assert_eq!(response.pagination.total_pages, 1);
    }

    #[test]
    fn test_user_invitation_stats_response() {
        let stats = UserInvitationStatsResponse {
            total_invitations: 10,
            pending_invitations: 3,
            accepted_invitations: 5,
            declined_invitations: 2,
        };

        assert_eq!(stats.total_invitations, 10);
        assert_eq!(stats.pending_invitations, 3);
        assert_eq!(stats.accepted_invitations, 5);
        assert_eq!(stats.declined_invitations, 2);

        // Test logical consistency
        assert_eq!(
            stats.pending_invitations + stats.accepted_invitations + stats.declined_invitations,
            10
        );
    }

    #[test]
    fn test_bulk_update_status_request_validation() {
        let valid_request = BulkUpdateStatusRequest {
            invitation_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
            new_status: TeamInvitationStatus::Cancelled,
        };
        assert!(valid_request.validate().is_ok());

        let empty_ids_request = BulkUpdateStatusRequest {
            invitation_ids: vec![],
            new_status: TeamInvitationStatus::Cancelled,
        };
        assert!(empty_ids_request.validate().is_err());

        let too_many_ids_request = BulkUpdateStatusRequest {
            invitation_ids: (0..101).map(|_| Uuid::new_v4()).collect(),
            new_status: TeamInvitationStatus::Cancelled,
        };
        assert!(too_many_ids_request.validate().is_err());
    }

    #[test]
    fn test_bulk_update_status_response() {
        let response = BulkUpdateStatusResponse {
            updated_count: 5,
            failed_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
        };

        assert_eq!(response.updated_count, 5);
        assert_eq!(response.failed_ids.len(), 2);
    }
}
