// src/repository/stripe_payment_history_repository.rs

use crate::db;
use crate::domain::stripe_payment_history_model::{
    self, ActiveModel as PaymentHistoryActiveModel, Entity as PaymentHistoryEntity,
};
use chrono::Utc;
use sea_orm::entity::*;
use sea_orm::{DbConn, DbErr, Order, PaginatorTrait, QueryFilter, QueryOrder, Set};
use uuid::Uuid;

#[derive(Debug)]
pub struct StripePaymentHistoryRepository {
    db: DbConn,
    schema: Option<String>,
}

impl StripePaymentHistoryRepository {
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

    /// ユーザーIDで支払い履歴を検索（ページネーション付き）
    pub async fn find_by_user_id_paginated(
        &self,
        user_id: Uuid,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<stripe_payment_history_model::Model>, u64), DbErr> {
        self.prepare_connection().await?;

        let paginator = PaymentHistoryEntity::find()
            .filter(stripe_payment_history_model::Column::UserId.eq(user_id))
            .order_by(stripe_payment_history_model::Column::CreatedAt, Order::Desc)
            .paginate(&self.db, per_page);

        let total_pages = paginator.num_pages().await?;
        let items = paginator.fetch_page(page).await?;

        Ok((items, total_pages))
    }

    /// 支払い履歴を作成
    pub async fn create(
        &self,
        create_payment: CreatePaymentHistory,
    ) -> Result<stripe_payment_history_model::Model, DbErr> {
        self.prepare_connection().await?;

        let new_payment = PaymentHistoryActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(create_payment.user_id),
            stripe_payment_intent_id: Set(create_payment.stripe_payment_intent_id),
            stripe_invoice_id: Set(create_payment.stripe_invoice_id),
            amount: Set(create_payment.amount),
            currency: Set(create_payment.currency),
            status: Set(create_payment.status),
            description: Set(create_payment.description),
            paid_at: Set(create_payment.paid_at),
            created_at: Set(Utc::now()),
        };

        new_payment.insert(&self.db).await
    }
}

/// 支払い履歴作成用構造体
#[derive(Debug)]
pub struct CreatePaymentHistory {
    pub user_id: Uuid,
    pub stripe_payment_intent_id: Option<String>,
    pub stripe_invoice_id: Option<String>,
    pub amount: i32,
    pub currency: String,
    pub status: String,
    pub description: Option<String>,
    pub paid_at: Option<chrono::DateTime<Utc>>,
}
