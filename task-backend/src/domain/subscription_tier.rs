// task-backend/src/domain/subscription_tier.rs

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// サブスクリプション階層
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionTier {
    Free,
    Pro,
    Enterprise,
}

impl SubscriptionTier {
    /// 文字列からSubscriptionTierに変換
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "free" => Some(Self::Free),
            "pro" => Some(Self::Pro),
            "enterprise" => Some(Self::Enterprise),
            _ => None,
        }
    }

    /// SubscriptionTierを文字列として取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Pro => "pro",
            Self::Enterprise => "enterprise",
        }
    }

    /// 階層レベルを数値で取得
    pub fn level(&self) -> u8 {
        match self {
            Self::Free => 1,
            Self::Pro => 2,
            Self::Enterprise => 3,
        }
    }

    /// 指定した階層以上かチェック
    pub fn is_at_least(&self, other: &Self) -> bool {
        self.level() >= other.level()
    }
    /// 全ての有効な階層を取得
    pub fn all() -> Vec<Self> {
        vec![Self::Free, Self::Pro, Self::Enterprise]
    }
}

impl FromStr for SubscriptionTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "free" => Ok(Self::Free),
            "pro" => Ok(Self::Pro),
            "enterprise" => Ok(Self::Enterprise),
            _ => Err(format!("Invalid subscription tier: {}", s)),
        }
    }
}

impl Default for SubscriptionTier {
    fn default() -> Self {
        Self::Free
    }
}

impl std::fmt::Display for SubscriptionTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            SubscriptionTier::from_str("free"),
            Some(SubscriptionTier::Free)
        );
        assert_eq!(
            SubscriptionTier::from_str("FREE"),
            Some(SubscriptionTier::Free)
        );
        assert_eq!(
            SubscriptionTier::from_str("pro"),
            Some(SubscriptionTier::Pro)
        );
        assert_eq!(
            SubscriptionTier::from_str("PRO"),
            Some(SubscriptionTier::Pro)
        );
        assert_eq!(
            SubscriptionTier::from_str("enterprise"),
            Some(SubscriptionTier::Enterprise)
        );
        assert_eq!(
            SubscriptionTier::from_str("ENTERPRISE"),
            Some(SubscriptionTier::Enterprise)
        );
        assert_eq!(SubscriptionTier::from_str("invalid"), None);
    }

    #[test]
    fn test_to_string() {
        assert_eq!(SubscriptionTier::Free.to_string(), "free");
        assert_eq!(SubscriptionTier::Pro.to_string(), "pro");
        assert_eq!(SubscriptionTier::Enterprise.to_string(), "enterprise");
    }

    #[test]
    fn test_level() {
        assert_eq!(SubscriptionTier::Free.level(), 1);
        assert_eq!(SubscriptionTier::Pro.level(), 2);
        assert_eq!(SubscriptionTier::Enterprise.level(), 3);
    }

    #[test]
    fn test_is_at_least() {
        assert!(SubscriptionTier::Enterprise.is_at_least(&SubscriptionTier::Free));
        assert!(SubscriptionTier::Enterprise.is_at_least(&SubscriptionTier::Pro));
        assert!(SubscriptionTier::Enterprise.is_at_least(&SubscriptionTier::Enterprise));

        assert!(SubscriptionTier::Pro.is_at_least(&SubscriptionTier::Free));
        assert!(SubscriptionTier::Pro.is_at_least(&SubscriptionTier::Pro));
        assert!(!SubscriptionTier::Pro.is_at_least(&SubscriptionTier::Enterprise));

        assert!(SubscriptionTier::Free.is_at_least(&SubscriptionTier::Free));
        assert!(!SubscriptionTier::Free.is_at_least(&SubscriptionTier::Pro));
        assert!(!SubscriptionTier::Free.is_at_least(&SubscriptionTier::Enterprise));
    }

    #[test]
    fn test_default() {
        assert_eq!(SubscriptionTier::default(), SubscriptionTier::Free);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", SubscriptionTier::Free), "free");
        assert_eq!(format!("{}", SubscriptionTier::Pro), "pro");
        assert_eq!(format!("{}", SubscriptionTier::Enterprise), "enterprise");
    }
}
