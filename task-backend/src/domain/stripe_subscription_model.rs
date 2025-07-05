// src/domain/stripe_subscription_model.rs

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "stripe_subscriptions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    pub user_id: Uuid,

    #[sea_orm(unique)]
    pub stripe_subscription_id: String,

    pub stripe_price_id: String,

    pub status: String,

    #[sea_orm(nullable)]
    pub current_period_start: Option<DateTime<Utc>>,

    #[sea_orm(nullable)]
    pub current_period_end: Option<DateTime<Utc>>,

    #[sea_orm(nullable)]
    pub cancel_at: Option<DateTime<Utc>>,

    #[sea_orm(nullable)]
    pub canceled_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::UserId",
        to = "crate::domain::user_model::Column::Id"
    )]
    User,
}

impl Related<crate::domain::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// サブスクリプションステータス
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionStatus {
    Active,
    Canceled,
    PastDue,
    Unpaid,
    Trialing,
    Incomplete,
    IncompleteExpired,
}

impl FromStr for SubscriptionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(SubscriptionStatus::Active),
            "canceled" => Ok(SubscriptionStatus::Canceled),
            "past_due" => Ok(SubscriptionStatus::PastDue),
            "unpaid" => Ok(SubscriptionStatus::Unpaid),
            "trialing" => Ok(SubscriptionStatus::Trialing),
            "incomplete" => Ok(SubscriptionStatus::Incomplete),
            "incomplete_expired" => Ok(SubscriptionStatus::IncompleteExpired),
            _ => Err(format!("Invalid subscription status: {}", s)),
        }
    }
}
