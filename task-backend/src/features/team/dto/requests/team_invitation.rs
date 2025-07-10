use serde::{Deserialize, Serialize};
use uuid::Uuid;
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

/// 招待再送信リクエスト
#[derive(Debug, Deserialize, Validate)]
pub struct ResendInvitationRequest {
    // TODO: Phase 19で本来の使用箇所が移行されたら#[allow(dead_code)]を削除
    #[allow(dead_code)]
    pub invitation_id: Uuid,
}

/// 単一招待作成リクエスト
#[derive(Debug, Deserialize, Validate)]
pub struct CreateInvitationRequest {
    // TODO: Phase 19で本来の使用箇所が移行されたら#[allow(dead_code)]を削除
    #[allow(dead_code)]
    pub team_id: Uuid,
    #[validate(email)]
    pub email: String,
    #[validate(length(max = 500))]
    pub message: Option<String>,
}

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

/// 一括ステータス更新リクエスト
#[derive(Debug, Deserialize, Validate)]
pub struct BulkUpdateStatusRequest {
    // TODO: Phase 19で本来の使用箇所が移行されたら#[allow(dead_code)]を削除
    #[allow(dead_code)]
    pub invitation_ids: Vec<Uuid>,
    #[allow(dead_code)]
    pub status: String,
    #[validate(length(max = 500))]
    pub reason: Option<String>,
}

/// メールアドレスのバリデーション関数
// TODO: Phase 19で本来の使用箇所が移行されたら#[allow(dead_code)]を削除
#[allow(dead_code)]
pub fn validate_emails(emails: &[String]) -> Result<(), String> {
    for email in emails {
        if email.is_empty() || !email.contains('@') || !email.contains('.') {
            return Err(format!("Invalid email address: {}", email));
        }
    }
    Ok(())
}

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

    #[test]
    fn test_create_invitation_request_validation() {
        let valid_request = CreateInvitationRequest {
            team_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            message: Some("Welcome!".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = CreateInvitationRequest {
            team_id: Uuid::new_v4(),
            email: "not-an-email".to_string(),
            message: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_resend_invitation_request_validation() {
        let request = ResendInvitationRequest {
            invitation_id: Uuid::new_v4(),
        };
        assert!(request.validate().is_ok());
    }

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

    #[test]
    fn test_bulk_update_status_request_validation() {
        let valid_request = BulkUpdateStatusRequest {
            invitation_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
            status: "cancelled".to_string(),
            reason: Some("Batch cancellation".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let empty_ids_request = BulkUpdateStatusRequest {
            invitation_ids: vec![],
            status: "cancelled".to_string(),
            reason: None,
        };
        // This would pass validation since we don't have a min length validation on invitation_ids
        assert!(empty_ids_request.validate().is_ok());
    }
}
