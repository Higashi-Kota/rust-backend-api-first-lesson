use crate::api::dto::common::PaginationQuery;
use crate::domain::subscription_tier::SubscriptionTier;
use crate::types::SortQuery;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 統一チーム検索クエリ
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TeamSearchQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    #[serde(flatten)]
    pub sort: SortQuery,

    pub search: Option<String>,
    pub name: Option<String>,
    pub organization_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub subscription_tier: Option<SubscriptionTier>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SortOrder;

    #[test]
    fn test_team_search_query_defaults() {
        let query = TeamSearchQuery::default();
        assert!(query.search.is_none());
        assert!(query.name.is_none());
        assert!(query.organization_id.is_none());
        assert!(query.owner_id.is_none());
        assert!(query.subscription_tier.is_none());
        assert!(query.sort.sort_by.is_none());
        assert!(matches!(query.sort.sort_order, SortOrder::Asc));
    }
}
