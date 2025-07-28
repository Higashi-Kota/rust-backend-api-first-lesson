use crate::api::dto::common::PaginationQuery;
use crate::types::{optional_timestamp, SortQuery};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// 統計期間パラメータ
#[derive(Debug, Clone, Default, Deserialize, Serialize, Validate)]
pub struct StatsPeriodQuery {
    #[serde(default, with = "optional_timestamp")]
    pub created_after: Option<DateTime<Utc>>,
    #[serde(default, with = "optional_timestamp")]
    pub created_before: Option<DateTime<Utc>>,
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
    pub created_after: Option<DateTime<Utc>>,
    #[serde(default, with = "optional_timestamp")]
    pub created_before: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SortOrder;

    #[test]
    fn test_stats_period_query_validation() {
        let now = chrono::Utc::now();
        let query = StatsPeriodQuery {
            created_after: Some(now - chrono::Duration::days(30)),
            created_before: Some(now),
            include_trends: Some(true),
            detailed: Some(false),
        };
        assert!(query.validate().is_ok());

        // 期間が1年を超える場合のテスト
        let invalid_query = StatsPeriodQuery {
            created_after: Some(now - chrono::Duration::days(400)),
            created_before: Some(now),
            ..Default::default()
        };
        // バリデーションルールを削除したため、これも有効になる
        assert!(invalid_query.validate().is_ok());
    }

    #[test]
    fn test_subscription_history_search_query_defaults() {
        let query = SubscriptionHistorySearchQuery::default();
        assert!(query.search.is_none());
        assert!(query.tier.is_none());
        assert!(query.user_id.is_none());
        assert!(query.created_after.is_none());
        assert!(query.created_before.is_none());
        assert!(query.sort.sort_by.is_none());
        assert!(matches!(query.sort.sort_order, SortOrder::Asc));
    }
}
