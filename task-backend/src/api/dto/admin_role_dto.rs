// task-backend/src/api/dto/admin_role_dto.rs
use crate::api::dto::common::{PaginationMeta, PaginationQuery};
use crate::api::dto::role_dto::RoleResponse;
use crate::domain::role_model::RoleWithPermissions;
use crate::domain::subscription_tier::SubscriptionTier;
use crate::types::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

/// 管理者向けロール一覧クエリパラメータ
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminRoleListQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub active_only: Option<bool>,
}

/// 管理者向けロール一覧レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminRoleListResponse {
    pub roles: Vec<RoleResponse>,
    pub pagination: PaginationMeta,
}

/// サブスクリプション情報付きロール詳細
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleWithSubscriptionResponse {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub permissions: RolePermissionsWithSubscription,
    pub subscription_info: SubscriptionInfo,
}

/// サブスクリプション階層ごとの権限情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermissionsWithSubscription {
    pub base_permissions: BasePermissions,
    pub subscription_based_permissions: Vec<TierPermissions>,
}

/// 基本権限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasePermissions {
    pub tasks: TaskPermissions,
    pub teams: TeamPermissions,
    pub users: UserPermissions,
    pub admin: AdminPermissions,
}

/// タスク権限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPermissions {
    pub create: bool,
    pub read: bool,
    pub update: bool,
    pub delete: bool,
}

/// チーム権限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPermissions {
    pub create: bool,
    pub manage: bool,
}

/// ユーザー権限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    pub manage: bool,
}

/// 管理者権限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPermissions {
    pub full_access: bool,
}

/// サブスクリプション階層別権限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierPermissions {
    pub tier: String,
    pub tier_level: u8,
    pub additional_permissions: AdditionalPermissions,
}

/// 追加権限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalPermissions {
    pub max_tasks: serde_json::Value, // Can be number or "unlimited"
    pub max_teams: serde_json::Value, // Can be number or "unlimited"
    pub max_team_members: Option<u32>,
    pub max_projects: Option<u32>,
    pub bulk_operations: bool,
    pub advanced_analytics: bool,
    pub api_access: bool,
    pub custom_integrations: bool,
    pub priority_support: bool,
}

/// サブスクリプション情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub applicable_tiers: Vec<String>,
    pub tier_independent: bool,
    pub available_tiers: Vec<String>,
    pub recommended_tier: String,
    pub tier_comparison: Vec<TierComparison>,
}

/// 階層比較情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierComparison {
    pub tier: String,
    pub monthly_price: f64,
    pub features: Vec<String>,
}

impl RoleWithSubscriptionResponse {
    pub fn from_role_with_tier(role: RoleWithPermissions, _tier: SubscriptionTier) -> Self {
        let is_admin = role.is_admin();
        let is_member = role.is_member();

        let base_permissions = BasePermissions {
            tasks: TaskPermissions {
                create: is_admin || is_member,
                read: is_admin || is_member,
                update: is_admin || is_member,
                delete: is_admin || is_member,
            },
            teams: TeamPermissions {
                create: is_admin,
                manage: is_admin,
            },
            users: UserPermissions { manage: is_admin },
            admin: AdminPermissions {
                full_access: is_admin,
            },
        };

        let subscription_based_permissions = SubscriptionTier::all()
            .into_iter()
            .map(|t| TierPermissions {
                tier: t.to_string(),
                tier_level: t.level(),
                additional_permissions: get_tier_permissions(t),
            })
            .collect();

        let tier_comparison = vec![
            TierComparison {
                tier: "free".to_string(),
                monthly_price: 0.0,
                features: vec![
                    "Basic task management".to_string(),
                    "Up to 5 team members".to_string(),
                    "1 project".to_string(),
                ],
            },
            TierComparison {
                tier: "pro".to_string(),
                monthly_price: 19.99,
                features: vec![
                    "Advanced task management".to_string(),
                    "Up to 20 team members".to_string(),
                    "10 projects".to_string(),
                    "Basic analytics".to_string(),
                    "API access".to_string(),
                ],
            },
            TierComparison {
                tier: "enterprise".to_string(),
                monthly_price: 99.99,
                features: vec![
                    "Unlimited task management".to_string(),
                    "Unlimited team members".to_string(),
                    "Unlimited projects".to_string(),
                    "Advanced analytics".to_string(),
                    "Full API access".to_string(),
                    "Custom integrations".to_string(),
                    "Priority support".to_string(),
                ],
            },
        ];

        let role_id = role.id;
        let role_name = role.name.as_str().to_string();
        let role_display_name = role.display_name.clone();
        let role_description = role.description.clone();
        let role_is_active = role.is_active;
        let role_created_at = Timestamp::from_datetime(role.created_at);
        let role_updated_at = Timestamp::from_datetime(role.updated_at);
        let recommended_tier = recommend_tier_for_role(&role).to_string();

        Self {
            id: role_id,
            name: role_name,
            display_name: role_display_name,
            description: role_description,
            is_active: role_is_active,
            created_at: role_created_at,
            updated_at: role_updated_at,
            permissions: RolePermissionsWithSubscription {
                base_permissions,
                subscription_based_permissions,
            },
            subscription_info: SubscriptionInfo {
                applicable_tiers: if is_admin {
                    vec!["all".to_string()]
                } else {
                    SubscriptionTier::all()
                        .into_iter()
                        .map(|t| t.to_string())
                        .collect()
                },
                tier_independent: is_admin,
                available_tiers: SubscriptionTier::all()
                    .into_iter()
                    .map(|t| t.to_string())
                    .collect(),
                recommended_tier,
                tier_comparison,
            },
        }
    }
}

fn get_tier_permissions(tier: SubscriptionTier) -> AdditionalPermissions {
    match tier {
        SubscriptionTier::Free => AdditionalPermissions {
            max_tasks: serde_json::json!(10),
            max_teams: serde_json::json!(1),
            max_team_members: Some(5),
            max_projects: Some(1),
            bulk_operations: false,
            advanced_analytics: false,
            api_access: false,
            custom_integrations: false,
            priority_support: false,
        },
        SubscriptionTier::Pro => AdditionalPermissions {
            max_tasks: serde_json::json!(100),
            max_teams: serde_json::json!(5),
            max_team_members: Some(20),
            max_projects: Some(10),
            bulk_operations: true,
            advanced_analytics: true,
            api_access: true,
            custom_integrations: false,
            priority_support: false,
        },
        SubscriptionTier::Enterprise => AdditionalPermissions {
            max_tasks: serde_json::json!("unlimited"),
            max_teams: serde_json::json!("unlimited"),
            max_team_members: None, // Unlimited
            max_projects: None,     // Unlimited
            bulk_operations: true,
            advanced_analytics: true,
            api_access: true,
            custom_integrations: true,
            priority_support: true,
        },
    }
}

fn recommend_tier_for_role(role: &RoleWithPermissions) -> SubscriptionTier {
    if role.is_admin() {
        SubscriptionTier::Enterprise
    } else if role.can_create_resource("task") {
        SubscriptionTier::Pro
    } else {
        SubscriptionTier::Free
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_role_list_query_default() {
        let query = AdminRoleListQuery::default();
        let (page, per_page) = query.pagination.get_pagination();
        assert_eq!(page, 1);
        assert_eq!(per_page, 20);
        assert_eq!(query.active_only, None);
    }

    #[test]
    fn test_get_tier_permissions() {
        let free_perms = get_tier_permissions(SubscriptionTier::Free);
        assert_eq!(free_perms.max_team_members, Some(5));
        assert_eq!(free_perms.max_projects, Some(1));
        assert!(!free_perms.advanced_analytics);

        let pro_perms = get_tier_permissions(SubscriptionTier::Pro);
        assert_eq!(pro_perms.max_team_members, Some(20));
        assert_eq!(pro_perms.max_projects, Some(10));
        assert!(pro_perms.advanced_analytics);
        assert!(pro_perms.api_access);

        let enterprise_perms = get_tier_permissions(SubscriptionTier::Enterprise);
        assert_eq!(enterprise_perms.max_team_members, None);
        assert_eq!(enterprise_perms.max_projects, None);
        assert!(enterprise_perms.custom_integrations);
        assert!(enterprise_perms.priority_support);
    }
}
