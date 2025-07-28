use crate::api::dto::common::PaginationQuery;
use crate::domain::subscription_tier::SubscriptionTier;
use crate::types::{query::SearchQuery, SortQuery};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 統一組織検索クエリ
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct OrganizationSearchQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    #[serde(flatten)]
    pub sort: SortQuery,

    pub search: Option<String>,
    pub name: Option<String>,
    pub owner_id: Option<Uuid>,
    pub subscription_tier: Option<SubscriptionTier>,
}

impl OrganizationSearchQuery {
    /// 許可されたソートフィールド
    pub fn allowed_sort_fields() -> &'static [&'static str] {
        &["name", "created_at", "updated_at", "owner_id"]
    }
}

impl SearchQuery for OrganizationSearchQuery {
    fn search_term(&self) -> Option<&str> {
        self.search.as_deref()
    }

    fn filters(&self) -> HashMap<String, String> {
        let mut filters = HashMap::new();

        if let Some(name) = &self.name {
            filters.insert("name".to_string(), name.clone());
        }
        if let Some(id) = &self.owner_id {
            filters.insert("owner_id".to_string(), id.to_string());
        }
        if let Some(tier) = &self.subscription_tier {
            filters.insert("subscription_tier".to_string(), format!("{:?}", tier));
        }

        filters
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SortOrder;

    #[test]
    fn test_organization_search_query_defaults() {
        let query = OrganizationSearchQuery::default();
        assert!(query.search.is_none());
        assert!(query.name.is_none());
        assert!(query.owner_id.is_none());
        assert!(query.subscription_tier.is_none());
        assert!(query.sort.sort_by.is_none());
        assert!(matches!(query.sort.sort_order, SortOrder::Asc));
    }
}
