use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// チーム作成リクエスト
#[derive(Debug, Deserialize, Validate)]
pub struct CreateTeamRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    // TODO: Phase 19で本来の使用箇所が移行されたら#[allow(dead_code)]を削除
    #[allow(dead_code)]
    pub organization_id: Option<Uuid>,
}

/// チーム更新リクエスト
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTeamRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
}

/// チームメンバー招待リクエスト
#[derive(Debug, Deserialize, Validate)]
pub struct InviteTeamMemberRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 50))]
    pub role: String,
    #[validate(length(max = 500))]
    pub message: Option<String>,
}

/// チームメンバーのロール更新リクエスト
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTeamMemberRoleRequest {
    // TODO: Phase 19で本来の使用箇所が移行されたら#[allow(dead_code)]を削除
    #[allow(dead_code)]
    pub role: String,
}

/// チーム検索クエリパラメータ
#[derive(Debug, Deserialize, Serialize)]
pub struct TeamSearchQuery {
    pub name: Option<String>,
    pub organization_id: Option<Uuid>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// チームページネーションクエリ
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TeamPaginationQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub name: Option<String>,
    pub organization_id: Option<Uuid>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

impl Default for TeamPaginationQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(10),
            name: None,
            organization_id: None,
            sort_by: Some("created_at".to_string()),
            sort_order: Some("desc".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_team_request_validation() {
        let valid_request = CreateTeamRequest {
            name: "Valid Team".to_string(),
            description: Some("Valid description".to_string()),
            organization_id: None,
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = CreateTeamRequest {
            name: "".to_string(),
            description: None,
            organization_id: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_invite_team_member_request_validation() {
        let valid_request = InviteTeamMemberRequest {
            email: "test@example.com".to_string(),
            role: "member".to_string(),
            message: Some("Welcome!".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = InviteTeamMemberRequest {
            email: "invalid-email".to_string(),
            role: "member".to_string(),
            message: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_team_pagination_query_defaults() {
        let query = TeamPaginationQuery::default();
        assert_eq!(query.page, Some(1));
        assert_eq!(query.per_page, Some(10));
        assert_eq!(query.sort_by, Some("created_at".to_string()));
        assert_eq!(query.sort_order, Some("desc".to_string()));
    }

    #[test]
    fn test_team_search_query_defaults() {
        let query = TeamSearchQuery {
            name: None,
            organization_id: None,
            page: None,
            per_page: None,
        };
        assert!(query.name.is_none());
        assert!(query.organization_id.is_none());
        assert!(query.page.is_none());
        assert!(query.per_page.is_none());
    }
}
