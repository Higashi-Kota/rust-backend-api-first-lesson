// task-backend/src/domain/user_consent_model.rs

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User consent types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter)]
pub enum ConsentType {
    DataProcessing,
    Marketing,
    Analytics,
    ThirdPartySharing,
}

// Conversion implementations for ConsentType
impl From<ConsentType> for String {
    fn from(consent_type: ConsentType) -> Self {
        match consent_type {
            ConsentType::DataProcessing => "data_processing".to_string(),
            ConsentType::Marketing => "marketing".to_string(),
            ConsentType::Analytics => "analytics".to_string(),
            ConsentType::ThirdPartySharing => "third_party_sharing".to_string(),
        }
    }
}

impl TryFrom<String> for ConsentType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "data_processing" => Ok(ConsentType::DataProcessing),
            "marketing" => Ok(ConsentType::Marketing),
            "analytics" => Ok(ConsentType::Analytics),
            "third_party_sharing" => Ok(ConsentType::ThirdPartySharing),
            _ => Err(format!("Invalid consent type: {}", value)),
        }
    }
}

/// User consent model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_consents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub consent_type: String,
    pub is_granted: bool,
    pub granted_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::UserId",
        to = "crate::domain::user_model::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<crate::domain::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Create a new consent record
    pub fn new(
        user_id: Uuid,
        consent_type: ConsentType,
        is_granted: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            consent_type: consent_type.into(),
            is_granted,
            granted_at: if is_granted { Some(now) } else { None },
            revoked_at: if !is_granted { Some(now) } else { None },
            ip_address,
            user_agent,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get consent type enum
    pub fn get_consent_type(&self) -> Result<ConsentType, String> {
        self.consent_type.clone().try_into()
    }
}

impl ConsentType {
    /// Get display name for consent type
    pub fn display_name(&self) -> &'static str {
        match self {
            ConsentType::DataProcessing => "Essential Data Processing",
            ConsentType::Marketing => "Marketing Communications",
            ConsentType::Analytics => "Analytics and Performance",
            ConsentType::ThirdPartySharing => "Third-Party Data Sharing",
        }
    }

    /// Get description for consent type
    pub fn description(&self) -> &'static str {
        match self {
            ConsentType::DataProcessing => {
                "Process your data to provide core functionality of the service"
            }
            ConsentType::Marketing => "Send you promotional emails and marketing communications",
            ConsentType::Analytics => "Collect and analyze usage data to improve our services",
            ConsentType::ThirdPartySharing => "Share your data with trusted third-party partners",
        }
    }

    /// Check if consent is required
    pub fn is_required(&self) -> bool {
        matches!(self, ConsentType::DataProcessing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consent_creation() {
        let user_id = Uuid::new_v4();
        let consent = Model::new(
            user_id,
            ConsentType::Marketing,
            true,
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        );

        assert_eq!(consent.user_id, user_id);
        assert_eq!(consent.get_consent_type().unwrap(), ConsentType::Marketing);
        assert!(consent.is_granted);
        assert!(consent.granted_at.is_some());
        assert!(consent.revoked_at.is_none());
    }

    #[test]
    fn test_consent_state_changes() {
        // 拒否状態の同意を作成
        let consent_denied = Model::new(Uuid::new_v4(), ConsentType::Analytics, false, None, None);
        assert!(!consent_denied.is_granted);
        assert!(consent_denied.granted_at.is_none());
        assert!(consent_denied.revoked_at.is_some());

        // 承認状態の同意を作成
        let consent_granted = Model::new(
            Uuid::new_v4(),
            ConsentType::Analytics,
            true,
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        );
        assert!(consent_granted.is_granted);
        assert!(consent_granted.granted_at.is_some());
        assert!(consent_granted.revoked_at.is_none());

        // 承認後に拒否された同意をシミュレート
        // 新しいModelを作成して、拒否状態をテスト
        let consent_revoked = Model {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            consent_type: ConsentType::Analytics.into(),
            is_granted: false,
            granted_at: Some(Utc::now() - chrono::Duration::hours(1)), // 過去に承認
            revoked_at: Some(Utc::now()),                              // 現在拒否
            ip_address: Some("192.168.1.2".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            created_at: Utc::now() - chrono::Duration::hours(2),
            updated_at: Utc::now(),
        };
        assert!(!consent_revoked.is_granted);
        assert!(consent_revoked.granted_at.is_some());
        assert!(consent_revoked.revoked_at.is_some());
    }

    #[test]
    fn test_consent_type_properties() {
        assert!(ConsentType::DataProcessing.is_required());
        assert!(!ConsentType::Marketing.is_required());
        assert!(!ConsentType::Analytics.is_required());
        assert!(!ConsentType::ThirdPartySharing.is_required());

        assert_eq!(
            ConsentType::Marketing.display_name(),
            "Marketing Communications"
        );
        assert_eq!(
            ConsentType::Analytics.description(),
            "Collect and analyze usage data to improve our services"
        );
    }
}
