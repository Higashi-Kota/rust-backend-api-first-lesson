// task-backend/src/middleware/subscription_guard.rs

use crate::domain::subscription_tier::SubscriptionTier;
use crate::error::{AppError, AppResult};

/// 機能制限をチェックするためのヘルパー関数
pub fn check_feature_limit(
    user_tier: &SubscriptionTier,
    current_usage: usize,
    feature: &str,
) -> AppResult<()> {
    let limit = match (user_tier, feature) {
        (SubscriptionTier::Free, "teams") => 1,
        (SubscriptionTier::Pro, "teams") => 5,
        (SubscriptionTier::Enterprise, "teams") => usize::MAX,

        (SubscriptionTier::Free, "team_members") => 3,
        (SubscriptionTier::Pro, "team_members") => 10,
        (SubscriptionTier::Enterprise, "team_members") => usize::MAX,

        (SubscriptionTier::Free, "tasks") => 100,
        (SubscriptionTier::Pro, "tasks") => 1000,
        (SubscriptionTier::Enterprise, "tasks") => usize::MAX,

        (SubscriptionTier::Free, "api_calls_per_day") => 1000,
        (SubscriptionTier::Pro, "api_calls_per_day") => 10000,
        (SubscriptionTier::Enterprise, "api_calls_per_day") => usize::MAX,

        _ => return Ok(()), // 未定義の機能は制限なし
    };

    if current_usage >= limit {
        return Err(AppError::Forbidden(format!(
            "You have reached the {} limit for your {} plan. Current: {}, Limit: {}",
            feature,
            user_tier.as_str(),
            current_usage,
            limit
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_feature_limit() {
        // Free tier tests
        assert!(check_feature_limit(&SubscriptionTier::Free, 0, "teams").is_ok());
        assert!(check_feature_limit(&SubscriptionTier::Free, 1, "teams").is_err());

        assert!(check_feature_limit(&SubscriptionTier::Free, 2, "team_members").is_ok());
        assert!(check_feature_limit(&SubscriptionTier::Free, 3, "team_members").is_err());

        // Pro tier tests
        assert!(check_feature_limit(&SubscriptionTier::Pro, 4, "teams").is_ok());
        assert!(check_feature_limit(&SubscriptionTier::Pro, 5, "teams").is_err());

        // Enterprise tier tests
        assert!(check_feature_limit(&SubscriptionTier::Enterprise, 1000000, "teams").is_ok());
        assert!(check_feature_limit(&SubscriptionTier::Enterprise, 1000000, "tasks").is_ok());

        // Unknown feature
        assert!(check_feature_limit(&SubscriptionTier::Free, 100, "unknown_feature").is_ok());
    }
}
