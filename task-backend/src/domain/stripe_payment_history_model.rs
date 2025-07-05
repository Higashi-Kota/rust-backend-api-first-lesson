// src/domain/stripe_payment_history_model.rs

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "stripe_payment_history")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    pub user_id: Uuid,

    #[sea_orm(unique, nullable)]
    pub stripe_payment_intent_id: Option<String>,

    #[sea_orm(unique, nullable)]
    pub stripe_invoice_id: Option<String>,

    pub amount: i32,

    #[sea_orm(default_value = "jpy")]
    pub currency: String,

    pub status: String,

    #[sea_orm(nullable)]
    pub description: Option<String>,

    #[sea_orm(nullable)]
    pub paid_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
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

/// 支払いステータス
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentStatus {
    Succeeded,
    Failed,
    Pending,
    Canceled,
    RequiresAction,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Succeeded => "succeeded",
            PaymentStatus::Failed => "failed",
            PaymentStatus::Pending => "pending",
            PaymentStatus::Canceled => "canceled",
            PaymentStatus::RequiresAction => "requires_action",
        }
    }
}

impl FromStr for PaymentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "succeeded" => Ok(PaymentStatus::Succeeded),
            "failed" => Ok(PaymentStatus::Failed),
            "pending" => Ok(PaymentStatus::Pending),
            "canceled" => Ok(PaymentStatus::Canceled),
            "requires_action" => Ok(PaymentStatus::RequiresAction),
            _ => Err(format!("Invalid payment status: {}", s)),
        }
    }
}
