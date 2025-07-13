use super::super::super::models::OrganizationRole;
use crate::core::subscription_tier::SubscriptionTier;
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

    pub user_id: Uuid, // Option<Uuid>から変更（サービス層で必須になっているため）

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
