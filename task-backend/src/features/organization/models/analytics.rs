use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "organization_analytics")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub organization_id: Uuid,
    pub department_id: Option<Uuid>,
    pub analytics_type: String,
    pub metric_name: String,
    pub metric_value: JsonValue,
    pub period: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub calculated_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::organization::Entity",
        from = "Column::OrganizationId",
        to = "super::organization::Column::Id"
    )]
    Organization,
    #[sea_orm(
        belongs_to = "super::department::Entity",
        from = "Column::DepartmentId",
        to = "super::department::Column::Id"
    )]
    Department,
    #[sea_orm(
        belongs_to = "crate::features::user::models::user::Entity",
        from = "Column::CalculatedBy",
        to = "crate::features::user::models::user::Column::Id"
    )]
    CalculatedByUser,
}

impl Related<super::organization::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Organization.def()
    }
}

impl Related<super::department::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Department.def()
    }
}

impl Related<crate::features::user::models::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CalculatedByUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalyticsType {
    Performance,
    Productivity,
    Engagement,
    Quality,
    Resource,
    User,
    Security,
    Compliance,
}

impl std::fmt::Display for AnalyticsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalyticsType::Performance => write!(f, "performance"),
            AnalyticsType::Productivity => write!(f, "productivity"),
            AnalyticsType::Engagement => write!(f, "engagement"),
            AnalyticsType::Quality => write!(f, "quality"),
            AnalyticsType::Resource => write!(f, "resource"),
            AnalyticsType::User => write!(f, "user"),
            AnalyticsType::Security => write!(f, "security"),
            AnalyticsType::Compliance => write!(f, "compliance"),
        }
    }
}

impl From<String> for AnalyticsType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "performance" => AnalyticsType::Performance,
            "productivity" => AnalyticsType::Productivity,
            "engagement" => AnalyticsType::Engagement,
            "quality" => AnalyticsType::Quality,
            "resource" => AnalyticsType::Resource,
            "user" => AnalyticsType::User,
            "security" => AnalyticsType::Security,
            "compliance" => AnalyticsType::Compliance,
            _ => AnalyticsType::Performance,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Period {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

impl std::fmt::Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Period::Daily => write!(f, "daily"),
            Period::Weekly => write!(f, "weekly"),
            Period::Monthly => write!(f, "monthly"),
            Period::Quarterly => write!(f, "quarterly"),
            Period::Yearly => write!(f, "yearly"),
        }
    }
}

impl From<String> for Period {
    fn from(value: String) -> Self {
        match value.as_str() {
            "daily" => Period::Daily,
            "weekly" => Period::Weekly,
            "monthly" => Period::Monthly,
            "quarterly" => Period::Quarterly,
            "yearly" => Period::Yearly,
            _ => Period::Monthly,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub value: f64,
    pub trend: Option<f64>,
    pub benchmark: Option<f64>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Model {
    pub fn get_analytics_type(&self) -> AnalyticsType {
        AnalyticsType::from(self.analytics_type.clone())
    }

    pub fn get_period(&self) -> Period {
        Period::from(self.period.clone())
    }

    pub fn get_metric_value(&self) -> Result<MetricValue, serde_json::Error> {
        serde_json::from_value(self.metric_value.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsInput {
    pub organization_id: Uuid,
    pub department_id: Option<Uuid>,
    pub analytics_type: AnalyticsType,
    pub metric_name: String,
    pub metric_value: JsonValue,
    pub period: Period,
    pub period_start: chrono::NaiveDateTime,
    pub period_end: chrono::NaiveDateTime,
    pub calculated_by: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_type_display() {
        assert_eq!(AnalyticsType::Performance.to_string(), "performance");
        assert_eq!(AnalyticsType::Productivity.to_string(), "productivity");
        assert_eq!(AnalyticsType::Engagement.to_string(), "engagement");
        assert_eq!(AnalyticsType::Quality.to_string(), "quality");
        assert_eq!(AnalyticsType::Resource.to_string(), "resource");
        assert_eq!(AnalyticsType::User.to_string(), "user");
        assert_eq!(AnalyticsType::Security.to_string(), "security");
        assert_eq!(AnalyticsType::Compliance.to_string(), "compliance");
    }

    #[test]
    fn test_analytics_type_from_string() {
        assert_eq!(
            AnalyticsType::from("performance".to_string()),
            AnalyticsType::Performance
        );
        assert_eq!(
            AnalyticsType::from("productivity".to_string()),
            AnalyticsType::Productivity
        );
        assert_eq!(
            AnalyticsType::from("engagement".to_string()),
            AnalyticsType::Engagement
        );
        assert_eq!(
            AnalyticsType::from("quality".to_string()),
            AnalyticsType::Quality
        );
        assert_eq!(
            AnalyticsType::from("resource".to_string()),
            AnalyticsType::Resource
        );
        assert_eq!(AnalyticsType::from("user".to_string()), AnalyticsType::User);
        assert_eq!(
            AnalyticsType::from("security".to_string()),
            AnalyticsType::Security
        );
        assert_eq!(
            AnalyticsType::from("compliance".to_string()),
            AnalyticsType::Compliance
        );
        assert_eq!(
            AnalyticsType::from("invalid".to_string()),
            AnalyticsType::Performance
        );
    }

    #[test]
    fn test_period_display() {
        assert_eq!(Period::Daily.to_string(), "daily");
        assert_eq!(Period::Weekly.to_string(), "weekly");
        assert_eq!(Period::Monthly.to_string(), "monthly");
        assert_eq!(Period::Quarterly.to_string(), "quarterly");
        assert_eq!(Period::Yearly.to_string(), "yearly");
    }

    #[test]
    fn test_period_from_string() {
        assert_eq!(Period::from("daily".to_string()), Period::Daily);
        assert_eq!(Period::from("weekly".to_string()), Period::Weekly);
        assert_eq!(Period::from("monthly".to_string()), Period::Monthly);
        assert_eq!(Period::from("quarterly".to_string()), Period::Quarterly);
        assert_eq!(Period::from("yearly".to_string()), Period::Yearly);
        assert_eq!(Period::from("invalid".to_string()), Period::Monthly);
    }

    #[test]
    fn test_metric_value_serialization() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::json!("automated"));
        metadata.insert("confidence".to_string(), serde_json::json!(0.95));

        let metric_value = MetricValue {
            value: 92.3,
            trend: Some(3.1),
            benchmark: Some(89.0),
            metadata,
        };

        // Test that serialization and deserialization work
        let json_value = serde_json::to_value(&metric_value).unwrap();
        let deserialized: MetricValue = serde_json::from_value(json_value).unwrap();

        assert_eq!(deserialized.value, metric_value.value);
        assert_eq!(deserialized.trend, metric_value.trend);
        assert_eq!(deserialized.benchmark, metric_value.benchmark);
        assert_eq!(deserialized.metadata.len(), metric_value.metadata.len());
    }
}
