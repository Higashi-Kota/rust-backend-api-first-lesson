// src/repository/stripe_subscription_repository.rs

use crate::db;
use crate::domain::stripe_subscription_model::{
    self, ActiveModel as SubscriptionActiveModel, Entity as SubscriptionEntity,
};
use chrono::Utc;
use sea_orm::entity::*;
use sea_orm::{DbConn, DbErr, QueryFilter, Set};
use uuid::Uuid;

#[derive(Debug)]
pub struct StripeSubscriptionRepository {
    db: DbConn,
    schema: Option<String>,
}

impl StripeSubscriptionRepository {
    pub fn new(db: DbConn) -> Self {
        Self { db, schema: None }
    }

    // スキーマを設定するヘルパーメソッド
    async fn prepare_connection(&self) -> Result<(), DbErr> {
        if let Some(schema) = &self.schema {
            db::set_schema(&self.db, schema).await?;
        }
        Ok(())
    }

    // --- 基本CRUD操作 ---

    /// Stripe Subscription IDでサブスクリプションを検索
    pub async fn find_by_stripe_subscription_id(
        &self,
        stripe_subscription_id: &str,
    ) -> Result<Option<stripe_subscription_model::Model>, DbErr> {
        self.prepare_connection().await?;
        SubscriptionEntity::find()
            .filter(
                stripe_subscription_model::Column::StripeSubscriptionId.eq(stripe_subscription_id),
            )
            .one(&self.db)
            .await
    }

    /// サブスクリプションを作成
    pub async fn create(
        &self,
        create_subscription: CreateStripeSubscription,
    ) -> Result<stripe_subscription_model::Model, DbErr> {
        self.prepare_connection().await?;

        let new_subscription = SubscriptionActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(create_subscription.user_id),
            stripe_subscription_id: Set(create_subscription.stripe_subscription_id),
            stripe_price_id: Set(create_subscription.stripe_price_id),
            status: Set(create_subscription.status),
            current_period_start: Set(create_subscription.current_period_start),
            current_period_end: Set(create_subscription.current_period_end),
            cancel_at: Set(create_subscription.cancel_at),
            canceled_at: Set(create_subscription.canceled_at),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        new_subscription.insert(&self.db).await
    }

    /// サブスクリプションを更新
    pub async fn update(
        &self,
        id: Uuid,
        update_subscription: UpdateStripeSubscription,
    ) -> Result<stripe_subscription_model::Model, DbErr> {
        self.prepare_connection().await?;

        let subscription = SubscriptionEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Subscription not found".to_string()))?;

        let mut active_model: SubscriptionActiveModel = subscription.into();

        if let Some(status) = update_subscription.status {
            active_model.status = Set(status);
        }
        if let Some(stripe_price_id) = update_subscription.stripe_price_id {
            active_model.stripe_price_id = Set(stripe_price_id);
        }
        if let Some(current_period_start) = update_subscription.current_period_start {
            active_model.current_period_start = Set(Some(current_period_start));
        }
        if let Some(current_period_end) = update_subscription.current_period_end {
            active_model.current_period_end = Set(Some(current_period_end));
        }
        if let Some(cancel_at) = update_subscription.cancel_at {
            active_model.cancel_at = Set(Some(cancel_at));
        }
        if let Some(canceled_at) = update_subscription.canceled_at {
            active_model.canceled_at = Set(Some(canceled_at));
        }

        active_model.updated_at = Set(Utc::now());
        active_model.update(&self.db).await
    }
}

/// サブスクリプション作成用構造体
#[derive(Debug)]
pub struct CreateStripeSubscription {
    pub user_id: Uuid,
    pub stripe_subscription_id: String,
    pub stripe_price_id: String,
    pub status: String,
    pub current_period_start: Option<chrono::DateTime<Utc>>,
    pub current_period_end: Option<chrono::DateTime<Utc>>,
    pub cancel_at: Option<chrono::DateTime<Utc>>,
    pub canceled_at: Option<chrono::DateTime<Utc>>,
}

/// サブスクリプション更新用構造体
#[derive(Debug, Default)]
pub struct UpdateStripeSubscription {
    pub status: Option<String>,
    pub stripe_price_id: Option<String>,
    pub current_period_start: Option<chrono::DateTime<Utc>>,
    pub current_period_end: Option<chrono::DateTime<Utc>>,
    pub cancel_at: Option<chrono::DateTime<Utc>>,
    pub canceled_at: Option<chrono::DateTime<Utc>>,
}
