use crate::api::dto::common::PaginationMeta;
use crate::api::dto::task_dto::{PaginatedTasksDto, TaskDto};
use serde::{Deserialize, Serialize};

/// Dynamic permission response types based on subscription tier
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "tier", rename_all = "PascalCase")]
pub enum DynamicTaskResponse {
    /// Free tier response - limited features
    Free {
        tasks: Vec<TaskDto>,
        quota_info: String,
        limit_reached: bool,
    },
    /// Pro tier response - enhanced features
    Pro {
        tasks: Vec<TaskDto>,
        features: Vec<String>,
        export_available: bool,
    },
    /// Enterprise tier response - full features
    Enterprise {
        #[serde(flatten)]
        data: PaginatedTasksDto,
        bulk_operations: bool,
        unlimited_access: bool,
    },
    /// Limited response variant
    Limited {
        items: Vec<TaskDto>,
        pagination: PaginationMeta,
    },
    /// Enhanced response variant
    Enhanced {
        items: Vec<TaskDto>,
        pagination: PaginationMeta,
    },
    /// Unlimited response variant
    Unlimited {
        items: Vec<TaskDto>,
        pagination: PaginationMeta,
    },
}

/// Subscription tier enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionTier {
    Free,
    Pro,
    Enterprise,
}

impl Default for SubscriptionTier {
    fn default() -> Self {
        Self::Free
    }
}

/// Task limits per subscription tier
pub struct TierLimits {
    pub max_tasks: Option<usize>,
}

impl TierLimits {
    pub fn for_tier(tier: SubscriptionTier) -> Self {
        match tier {
            SubscriptionTier::Free => Self {
                max_tasks: Some(10),
            },
            SubscriptionTier::Pro => Self {
                max_tasks: Some(1000),
            },
            SubscriptionTier::Enterprise => Self { max_tasks: None },
        }
    }
}
