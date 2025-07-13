// task-backend/src/domain/login_attempt_model.rs

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "login_attempts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub email: String,
    pub user_id: Option<Uuid>,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    #[sea_orm(nullable)]
    pub device_type: Option<String>,
    #[sea_orm(nullable)]
    pub browser_name: Option<String>,
    #[sea_orm(nullable)]
    pub country: Option<String>,
    pub suspicious_score: i32,
    pub attempted_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::features::user::models::user::Entity",
        from = "Column::UserId",
        to = "crate::features::user::models::user::Column::Id"
    )]
    User,
}

impl Related<crate::features::user::models::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            attempted_at: Set(Utc::now()),
            suspicious_score: Set(0), // デフォルトは0
            ..ActiveModelTrait::default()
        }
    }
}

/// ログイン失敗理由
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginFailureReason {
    InvalidCredentials,
    AccountLocked,
    AccountInactive,
    EmailNotVerified,
    TooManyAttempts,
    Other,
}

impl Model {
    /// 成功したログイン試行を記録
    pub fn successful(
        email: String,
        user_id: Uuid,
        ip_address: String,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            user_id: Some(user_id),
            success: true,
            failure_reason: None,
            ip_address,
            user_agent,
            device_type: None,
            browser_name: None,
            country: None,
            suspicious_score: 0,
            attempted_at: Utc::now(),
        }
    }

    /// 失敗したログイン試行を記録
    pub fn failed(
        email: String,
        user_id: Option<Uuid>,
        failure_reason: String,
        ip_address: String,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            user_id,
            success: false,
            failure_reason: Some(failure_reason),
            ip_address,
            user_agent,
            device_type: None,
            browser_name: None,
            country: None,
            suspicious_score: 0,
            attempted_at: Utc::now(),
        }
    }
}
