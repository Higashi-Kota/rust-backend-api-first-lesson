// task-backend/src/domain/permission.rs

use crate::domain::subscription_tier::SubscriptionTier;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use uuid::Uuid;

/// 権限スコープ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionScope {
    Own,          // 自分のデータのみ
    Team,         // チームのデータ
    Organization, // 組織全体
    Global,       // 全データ
}

impl PermissionScope {
    /// スコープレベルを数値で取得（高いほど広範囲）
    pub fn level(&self) -> u8 {
        match self {
            PermissionScope::Own => 1,
            PermissionScope::Team => 2,
            PermissionScope::Organization => 3,
            PermissionScope::Global => 4,
        }
    }

    /// 指定されたスコープ以上かチェック
    pub fn includes(&self, other: &PermissionScope) -> bool {
        self.level() >= other.level()
    }

    /// スコープの説明を取得
    pub fn description(&self) -> &str {
        match self {
            PermissionScope::Own => "Access to own resources only",
            PermissionScope::Team => "Access to team resources",
            PermissionScope::Organization => "Access to organization-wide resources",
            PermissionScope::Global => "Access to all system resources",
        }
    }
}

/// 権限クォータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionQuota {
    pub max_items: Option<u32>,  // 最大取得件数
    pub rate_limit: Option<u32>, // レート制限（回/分）
    pub features: Vec<String>,   // 利用可能機能
}

impl PermissionQuota {
    /// 新しいクォータを作成
    pub fn new(max_items: Option<u32>, rate_limit: Option<u32>, features: Vec<String>) -> Self {
        Self {
            max_items,
            rate_limit,
            features,
        }
    }

    /// 制限付きクォータを作成
    pub fn limited(max_items: u32, rate_limit: u32) -> Self {
        Self {
            max_items: Some(max_items),
            rate_limit: Some(rate_limit),
            features: vec!["basic_access".to_string()],
        }
    }

    /// 無制限クォータを作成
    pub fn unlimited() -> Self {
        Self {
            max_items: None,
            rate_limit: None,
            features: vec!["unlimited_access".to_string()],
        }
    }

    /// 指定した機能が利用可能かチェック
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.contains(&feature.to_string())
    }
}

/// 特権
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Privilege {
    pub name: String,                        // 特権名
    pub subscription_tier: SubscriptionTier, // 必要なサブスクリプション階層
    pub quota: Option<PermissionQuota>,      // クォータ制限
}

/// 権限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,       // リソース名 (e.g., "tasks", "users")
    pub action: String,         // アクション名 (e.g., "read", "write", "delete")
    pub scope: PermissionScope, // 権限スコープ
}

/// 権限チェック結果
#[derive(Debug, Clone)]
pub enum PermissionResult {
    Allowed {
        privilege: Option<Privilege>,
        scope: PermissionScope,
    },
    Denied {
        reason: String,
    },
}

impl PermissionResult {
    #[cfg(test)]
    pub fn new(
        base_permission: Option<Permission>,
        subscription_privilege: Option<Privilege>,
        _target_user_id: Option<Uuid>,
    ) -> Self {
        match base_permission {
            Some(permission) => Self::Allowed {
                privilege: subscription_privilege,
                scope: permission.scope,
            },
            None => Self::Denied {
                reason: "Insufficient permissions".to_string(),
            },
        }
    }

    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed { .. })
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, Self::Denied { .. })
    }

    pub fn get_scope(&self) -> Option<&PermissionScope> {
        match self {
            Self::Allowed { scope, .. } => Some(scope),
            Self::Denied { .. } => None,
        }
    }

    pub fn get_privilege(&self) -> Option<&Privilege> {
        match self {
            Self::Allowed { privilege, .. } => privilege.as_ref(),
            Self::Denied { .. } => None,
        }
    }

    pub fn get_denial_reason(&self) -> Option<&String> {
        match self {
            Self::Allowed { .. } => None,
            Self::Denied { reason } => Some(reason),
        }
    }

    /// 許可結果を作成
    pub fn allowed(privilege: Option<Privilege>, scope: PermissionScope) -> Self {
        Self::Allowed { privilege, scope }
    }

    /// 拒否結果を作成
    pub fn denied(reason: &str) -> Self {
        Self::Denied {
            reason: reason.to_string(),
        }
    }
}

impl Permission {
    pub fn new(resource: &str, action: &str, scope: PermissionScope) -> Self {
        Self {
            resource: resource.to_string(),
            action: action.to_string(),
            scope,
        }
    }

    /// 基本的な読み取り権限を作成
    pub fn read_own(resource: &str) -> Self {
        Self::new(resource, "read", PermissionScope::Own)
    }

    /// 基本的な書き込み権限を作成
    pub fn write_own(resource: &str) -> Self {
        Self::new(resource, "write", PermissionScope::Own)
    }

    /// 管理者権限を作成
    pub fn admin_global(resource: &str) -> Self {
        Self::new(resource, "admin", PermissionScope::Global)
    }

    /// リソースとアクションが一致するかチェック
    pub fn matches(&self, resource: &str, action: &str) -> bool {
        self.resource == resource && self.action == action
    }
}

impl Privilege {
    pub fn new(name: &str, tier: SubscriptionTier, quota: Option<PermissionQuota>) -> Self {
        Self {
            name: name.to_string(),
            subscription_tier: tier,
            quota,
        }
    }

    /// Free階層の基本特権を作成
    pub fn free_basic(name: &str, max_items: u32, rate_limit: u32) -> Self {
        Self::new(
            name,
            SubscriptionTier::Free,
            Some(PermissionQuota {
                max_items: Some(max_items),
                rate_limit: Some(rate_limit),
                features: vec!["basic_access".to_string()],
            }),
        )
    }

    /// Pro階層の拡張特権を作成
    pub fn pro_advanced(name: &str, max_items: u32, rate_limit: u32, features: Vec<&str>) -> Self {
        Self::new(
            name,
            SubscriptionTier::Pro,
            Some(PermissionQuota {
                max_items: Some(max_items),
                rate_limit: Some(rate_limit),
                features: features.iter().map(|f| (*f).to_string()).collect(),
            }),
        )
    }

    /// Enterprise階層の無制限特権を作成
    pub fn enterprise_unlimited(name: &str, features: Vec<&str>) -> Self {
        Self::new(
            name,
            SubscriptionTier::Enterprise,
            Some(PermissionQuota {
                max_items: None,
                rate_limit: None,
                features: features.iter().map(|f| (*f).to_string()).collect(),
            }),
        )
    }

    /// 特権が指定した階層で利用可能かチェック
    pub fn is_available_for_tier(&self, tier: &SubscriptionTier) -> bool {
        tier.is_at_least(&self.subscription_tier)
    }

    /// 特権で利用可能な最大アイテム数を取得
    pub fn get_max_items(&self) -> Option<u32> {
        self.quota.as_ref().and_then(|q| q.max_items)
    }

    /// 特権のレート制限を取得
    pub fn get_rate_limit(&self) -> Option<u32> {
        self.quota.as_ref().and_then(|q| q.rate_limit)
    }

    /// 特権で利用可能な機能をチェック
    pub fn has_feature(&self, feature: &str) -> bool {
        self.quota
            .as_ref()
            .is_some_and(|q| q.features.contains(&feature.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_creation() {
        let permission = Permission::read_own("tasks");
        assert_eq!(permission.resource, "tasks");
        assert_eq!(permission.action, "read");
        assert_eq!(permission.scope, PermissionScope::Own);
    }

    #[test]
    fn test_privilege_free_basic() {
        let privilege = Privilege::free_basic("task_access", 100, 10);
        assert_eq!(privilege.name, "task_access");
        assert_eq!(privilege.subscription_tier, SubscriptionTier::Free);
        assert_eq!(privilege.get_max_items(), Some(100));
        assert_eq!(privilege.get_rate_limit(), Some(10));
        assert!(privilege.has_feature("basic_access"));
    }

    #[test]
    fn test_privilege_availability() {
        let free_privilege = Privilege::free_basic("basic", 100, 10);
        let pro_privilege = Privilege::pro_advanced("advanced", 1000, 100, vec!["export"]);
        let enterprise_privilege = Privilege::enterprise_unlimited("unlimited", vec!["bulk_ops"]);

        // Free tier can only access free privileges
        assert!(free_privilege.is_available_for_tier(&SubscriptionTier::Free));
        assert!(!pro_privilege.is_available_for_tier(&SubscriptionTier::Free));
        assert!(!enterprise_privilege.is_available_for_tier(&SubscriptionTier::Free));

        // Pro tier can access free and pro privileges
        assert!(free_privilege.is_available_for_tier(&SubscriptionTier::Pro));
        assert!(pro_privilege.is_available_for_tier(&SubscriptionTier::Pro));
        assert!(!enterprise_privilege.is_available_for_tier(&SubscriptionTier::Pro));

        // Enterprise tier can access all privileges
        assert!(free_privilege.is_available_for_tier(&SubscriptionTier::Enterprise));
        assert!(pro_privilege.is_available_for_tier(&SubscriptionTier::Enterprise));
        assert!(enterprise_privilege.is_available_for_tier(&SubscriptionTier::Enterprise));
    }

    #[test]
    fn test_permission_result() {
        let permission = Some(Permission::read_own("tasks"));
        let privilege = Some(Privilege::free_basic("task_access", 100, 10));

        let result = PermissionResult::new(permission, privilege, None);
        assert!(result.is_allowed());
        assert_eq!(result.get_scope(), Some(&PermissionScope::Own));
        assert!(result.get_privilege().is_some());

        let denied_result = PermissionResult::new(None, None, None);
        assert!(denied_result.is_denied());
        assert_eq!(
            denied_result.get_denial_reason(),
            Some(&"Insufficient permissions".to_string())
        );
    }
}
