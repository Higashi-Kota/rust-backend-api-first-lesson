// task-backend/src/api/dto/admin_organization_dto.rs

use crate::api::dto::common::{PaginationMeta, PaginationQuery};
use crate::api::dto::organization_dto::{OrganizationListResponse, OrganizationTierStats};
use crate::api::dto::user_dto::UserWithRoleResponse;
use crate::domain::subscription_tier::SubscriptionTier;
use serde::{Deserialize, Serialize};

/// 管理者向け組織一覧リクエスト
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminOrganizationsRequest {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub subscription_tier: Option<SubscriptionTier>,
}

/// 管理者向け組織一覧レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminOrganizationsResponse {
    pub organizations: Vec<OrganizationListResponse>,
    pub pagination: PaginationMeta,
    pub tier_summary: Vec<OrganizationTierStats>,
}

/// 管理者向けユーザー一覧リクエスト
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUsersWithRolesRequest {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub role_name: Option<String>,
}

/// 管理者向けユーザー一覧レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUsersWithRolesResponse {
    pub users: Vec<UserWithRoleResponse>,
    pub pagination: PaginationMeta,
    pub role_summary: Vec<RoleSummary>,
}

/// ロール別サマリー
#[derive(Debug, Serialize, Deserialize)]
pub struct RoleSummary {
    pub role_name: String,
    pub role_display_name: String,
    pub user_count: u64,
    pub active_users: u64,
    pub verified_users: u64,
}
