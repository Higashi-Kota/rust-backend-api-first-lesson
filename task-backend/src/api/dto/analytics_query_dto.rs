use crate::api::dto::common::PaginationQuery;
use crate::types::{optional_timestamp, SortQuery};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// 統計期間パラメータ
#[derive(Debug, Clone, Default, Deserialize, Serialize, Validate)]
pub struct StatsPeriodQuery {
    #[validate(range(min = 1, max = 365, message = "Days must be between 1 and 365"))]
    pub days: Option<u32>,
    pub include_trends: Option<bool>,
    pub detailed: Option<bool>,
}

/// 統一サブスクリプション履歴検索クエリ
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SubscriptionHistorySearchQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    #[serde(flatten)]
    pub sort: SortQuery,

    pub search: Option<String>,
    pub tier: Option<String>,
    pub user_id: Option<Uuid>,
    #[serde(default, with = "optional_timestamp")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(default, with = "optional_timestamp")]
    pub end_date: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SortOrder;

    #[test]
    fn test_stats_period_query_validation() {
        let query = StatsPeriodQuery {
            days: Some(30),
            include_trends: Some(true),
            detailed: Some(false),
        };
        assert!(query.validate().is_ok());

        let invalid_query = StatsPeriodQuery {
            days: Some(500),
            ..Default::default()
        };
        assert!(invalid_query.validate().is_err());
    }

    #[test]
    fn test_subscription_history_search_query_defaults() {
        let query = SubscriptionHistorySearchQuery::default();
        assert!(query.search.is_none());
        assert!(query.tier.is_none());
        assert!(query.user_id.is_none());
        assert!(query.start_date.is_none());
        assert!(query.end_date.is_none());
        assert!(query.sort.sort_by.is_none());
        assert!(matches!(query.sort.sort_order, SortOrder::Asc));
    }
}
