// task-backend/src/domain/subscription_history_model.rs
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// サブスクリプション履歴エンティティ
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "subscription_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    pub user_id: Uuid,

    #[sea_orm(nullable)]
    pub previous_tier: Option<String>,

    pub new_tier: String,

    pub changed_at: DateTime<Utc>,

    #[sea_orm(nullable)]
    pub changed_by: Option<Uuid>,

    #[sea_orm(nullable)]
    pub reason: Option<String>,

    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user_model::Entity",
        from = "Column::UserId",
        to = "super::user_model::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::user_model::Entity",
        from = "Column::ChangedBy",
        to = "super::user_model::Column::Id"
    )]
    ChangedByUser,
}

impl Related<super::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// サブスクリプション履歴のビジネスロジック
#[allow(dead_code)]
impl Model {
    /// 新しいサブスクリプション変更履歴を作成
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        user_id: Uuid,
        previous_tier: Option<String>,
        new_tier: String,
        changed_by: Option<Uuid>,
        reason: Option<String>,
    ) -> ActiveModel {
        ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            previous_tier: Set(previous_tier),
            new_tier: Set(new_tier),
            changed_at: Set(Utc::now()),
            changed_by: Set(changed_by),
            reason: Set(reason),
            created_at: Set(Utc::now()),
        }
    }

    /// 階層が実際に変更されたかチェック
    pub fn is_tier_changed(&self) -> bool {
        match &self.previous_tier {
            Some(prev) => prev != &self.new_tier,
            None => true, // 初回設定の場合は変更とみなす
        }
    }

    /// 階層がアップグレードされたかチェック
    pub fn is_upgrade(&self) -> bool {
        match &self.previous_tier {
            Some(prev) => {
                let prev_level = tier_to_level(prev);
                let new_level = tier_to_level(&self.new_tier);
                new_level > prev_level
            }
            None => self.new_tier != "free", // 初回でFree以外ならアップグレード
        }
    }

    /// 階層がダウングレードされたかチェック
    pub fn is_downgrade(&self) -> bool {
        match &self.previous_tier {
            Some(prev) => {
                let prev_level = tier_to_level(prev);
                let new_level = tier_to_level(&self.new_tier);
                new_level < prev_level
            }
            None => false, // 初回設定はダウングレードではない
        }
    }

    /// 変更理由の設定
    #[allow(dead_code)]
    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
}

/// サブスクリプション階層を数値レベルに変換
fn tier_to_level(tier: &str) -> u8 {
    match tier.to_lowercase().as_str() {
        "free" => 1,
        "pro" => 2,
        "enterprise" => 3,
        _ => 0, // 不明な階層
    }
}

/// サブスクリプション変更の詳細情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionChangeInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub previous_tier: Option<String>,
    pub new_tier: String,
    pub changed_at: DateTime<Utc>,
    pub changed_by: Option<Uuid>,
    pub reason: Option<String>,
    pub is_upgrade: bool,
    pub is_downgrade: bool,
}

impl From<Model> for SubscriptionChangeInfo {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            previous_tier: model.previous_tier.clone(),
            new_tier: model.new_tier.clone(),
            changed_at: model.changed_at,
            changed_by: model.changed_by,
            reason: model.reason.clone(),
            is_upgrade: model.is_upgrade(),
            is_downgrade: model.is_downgrade(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_to_level() {
        assert_eq!(tier_to_level("free"), 1);
        assert_eq!(tier_to_level("pro"), 2);
        assert_eq!(tier_to_level("enterprise"), 3);
        assert_eq!(tier_to_level("unknown"), 0);
    }

    #[test]
    fn test_is_tier_changed() {
        let history = Model {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            previous_tier: Some("free".to_string()),
            new_tier: "pro".to_string(),
            changed_at: Utc::now(),
            changed_by: None,
            reason: None,
            created_at: Utc::now(),
        };

        assert!(history.is_tier_changed());

        let no_change = Model {
            previous_tier: Some("pro".to_string()),
            new_tier: "pro".to_string(),
            ..history
        };

        assert!(!no_change.is_tier_changed());
    }

    #[test]
    fn test_is_upgrade() {
        let upgrade = Model {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            previous_tier: Some("free".to_string()),
            new_tier: "pro".to_string(),
            changed_at: Utc::now(),
            changed_by: None,
            reason: None,
            created_at: Utc::now(),
        };

        assert!(upgrade.is_upgrade());

        let downgrade = Model {
            previous_tier: Some("pro".to_string()),
            new_tier: "free".to_string(),
            ..upgrade
        };

        assert!(!downgrade.is_upgrade());
        assert!(downgrade.is_downgrade());
    }

    #[test]
    fn test_initial_subscription() {
        let initial_free = Model {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            previous_tier: None,
            new_tier: "free".to_string(),
            changed_at: Utc::now(),
            changed_by: None,
            reason: None,
            created_at: Utc::now(),
        };

        assert!(initial_free.is_tier_changed());
        assert!(!initial_free.is_upgrade());
        assert!(!initial_free.is_downgrade());

        let initial_pro = Model {
            new_tier: "pro".to_string(),
            ..initial_free
        };

        assert!(initial_pro.is_upgrade());
    }
}
