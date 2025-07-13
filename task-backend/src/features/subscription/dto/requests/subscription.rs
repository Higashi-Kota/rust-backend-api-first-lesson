// task-backend/src/features/subscription/dto/requests/subscription.rs

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::core::subscription_tier::SubscriptionTier;

/// サブスクリプション階層変更リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ChangeSubscriptionRequest {
    #[validate(length(
        min = 3,
        max = 20,
        message = "Tier must be between 3 and 20 characters"
    ))]
    pub new_tier: String,

    #[validate(length(max = 500, message = "Reason must be 500 characters or less"))]
    pub reason: Option<String>,
}

/// サブスクリプション階層アップグレードリクエスト  
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpgradeSubscriptionRequest {
    #[validate(custom(function = "validate_upgrade_tier"))]
    pub target_tier: SubscriptionTier,

    #[validate(length(max = 500, message = "Reason must be 500 characters or less"))]
    pub reason: Option<String>,
}

/// サブスクリプション階層ダウングレードリクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct DowngradeSubscriptionRequest {
    #[validate(custom(function = "validate_downgrade_tier"))]
    pub target_tier: SubscriptionTier,

    #[validate(length(max = 500, message = "Reason must be 500 characters or less"))]
    pub reason: Option<String>,
}

/// 管理者用サブスクリプション変更リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AdminChangeSubscriptionRequest {
    #[validate(length(
        min = 3,
        max = 20,
        message = "Tier must be between 3 and 20 characters"
    ))]
    pub new_tier: String,

    #[validate(length(max = 500, message = "Reason must be 500 characters or less"))]
    pub reason: Option<String>,

    pub force_change: Option<bool>,
}

// Validation helper functions
fn validate_upgrade_tier(tier: &SubscriptionTier) -> Result<(), validator::ValidationError> {
    if *tier == SubscriptionTier::Free {
        return Err(validator::ValidationError::new(
            "Cannot upgrade to Free tier",
        ));
    }
    Ok(())
}

fn validate_downgrade_tier(tier: &SubscriptionTier) -> Result<(), validator::ValidationError> {
    if *tier == SubscriptionTier::Enterprise {
        return Err(validator::ValidationError::new(
            "Cannot downgrade from Enterprise tier",
        ));
    }
    Ok(())
}
