use serde::{Deserialize, Serialize};
use validator::Validate;

/// チームメンバー一括招待リクエスト
#[derive(Debug, Deserialize, Validate)]
pub struct BulkTeamInviteRequest {
    #[validate(length(min = 1, max = 100))]
    pub emails: Vec<String>,
    #[validate(length(min = 1, max = 50))]
    pub role: String,
    #[validate(length(max = 500))]
    pub message: Option<String>,
}

/// 招待辞退リクエスト
#[derive(Debug, Deserialize, Validate)]
pub struct DeclineInvitationRequest {
    #[validate(length(max = 500))]
    pub reason: Option<String>,
}

// ResendInvitationRequest and CreateInvitationRequest removed - unused (YAGNI)
// The actual DTOs are in dto/team_invitation.rs

/// ユーザー招待クエリ
#[derive(Debug, Deserialize, Serialize)]
pub struct UserInvitationQuery {
    pub status: Option<String>,
}

/// チーム招待クエリ
#[derive(Debug, Deserialize, Serialize)]
pub struct TeamInvitationQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

// BulkUpdateStatusRequest removed - unused (YAGNI)
// The actual DTO with correct field names is in dto/team_invitation.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulk_invite_request_validation() {
        let valid_request = BulkTeamInviteRequest {
            emails: vec![
                "test1@example.com".to_string(),
                "test2@example.com".to_string(),
            ],
            role: "member".to_string(),
            message: Some("Welcome to the team!".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = BulkTeamInviteRequest {
            emails: vec![],
            role: "member".to_string(),
            message: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_decline_invitation_request_validation() {
        let valid_request = DeclineInvitationRequest {
            reason: Some("Not interested at this time".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let no_reason_request = DeclineInvitationRequest { reason: None };
        assert!(no_reason_request.validate().is_ok());
    }

    // Tests for CreateInvitationRequest and ResendInvitationRequest removed
    // as the structs were removed (unused - YAGNI)

    #[test]
    fn test_team_invitation_query_defaults() {
        let query = TeamInvitationQuery {
            status: None,
            page: None,
            per_page: None,
        };
        assert!(query.status.is_none());
        assert!(query.page.is_none());
        assert!(query.per_page.is_none());
    }

    #[test]
    fn test_team_invitation_query_boundaries() {
        let query = TeamInvitationQuery {
            status: Some("pending".to_string()),
            page: Some(1),
            per_page: Some(100),
        };
        assert_eq!(query.status, Some("pending".to_string()));
        assert_eq!(query.page, Some(1));
        assert_eq!(query.per_page, Some(100));
    }

    // Test for BulkUpdateStatusRequest removed
    // as the struct was removed (unused - YAGNI)
}
