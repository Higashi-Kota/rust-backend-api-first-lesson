// task-backend/src/api/dto/admin_role_dto.rs
use crate::features::security::dto::legacy::role_dto::RoleResponse;
use crate::shared::types::pagination::PaginationMeta;
use serde::{Deserialize, Serialize};

/// 管理者向けロール一覧クエリパラメータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminRoleListQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub active_only: Option<bool>,
}

impl Default for AdminRoleListQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
            active_only: None,
        }
    }
}

/// 管理者向けロール一覧レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminRoleListResponse {
    pub roles: Vec<RoleResponse>,
    pub pagination: PaginationMeta,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_role_list_query_default() {
        let query = AdminRoleListQuery::default();
        assert_eq!(query.page, Some(1));
        assert_eq!(query.page_size, Some(20));
        assert_eq!(query.active_only, None);
    }
}
