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
        belongs_to = "super::organization_model::Entity",
        from = "Column::OrganizationId",
        to = "super::organization_model::Column::Id"
    )]
    Organization,
    #[sea_orm(
        belongs_to = "super::organization_department_model::Entity",
        from = "Column::DepartmentId",
        to = "super::organization_department_model::Column::Id"
    )]
    Department,
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::CalculatedBy",
        to = "crate::domain::user_model::Column::Id"
    )]
    CalculatedByUser,
}

impl Related<super::organization_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Organization.def()
    }
}

impl Related<super::organization_department_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Department.def()
    }
}

impl Related<crate::domain::user_model::Entity> for Entity {
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

#[allow(dead_code)]
impl AnalyticsType {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "performance" => Ok(AnalyticsType::Performance),
            "productivity" => Ok(AnalyticsType::Productivity),
            "engagement" => Ok(AnalyticsType::Engagement),
            "quality" => Ok(AnalyticsType::Quality),
            "resource" => Ok(AnalyticsType::Resource),
            "user" => Ok(AnalyticsType::User),
            "security" => Ok(AnalyticsType::Security),
            "compliance" => Ok(AnalyticsType::Compliance),
            _ => Err(format!("Invalid analytics type: {}", s)),
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

#[allow(dead_code)]
impl Period {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "daily" => Ok(Period::Daily),
            "weekly" => Ok(Period::Weekly),
            "monthly" => Ok(Period::Monthly),
            "quarterly" => Ok(Period::Quarterly),
            "yearly" => Ok(Period::Yearly),
            _ => Err(format!("Invalid period: {}", s)),
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

#[allow(dead_code)]
impl Model {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        organization_id: Uuid,
        department_id: Option<Uuid>,
        analytics_type: AnalyticsType,
        metric_name: String,
        metric_value: MetricValue,
        period: Period,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        calculated_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            organization_id,
            department_id,
            analytics_type: analytics_type.to_string(),
            metric_name,
            metric_value: serde_json::to_value(metric_value).unwrap_or_default(),
            period: period.to_string(),
            period_start,
            period_end,
            calculated_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn get_analytics_type(&self) -> AnalyticsType {
        AnalyticsType::from(self.analytics_type.clone())
    }

    pub fn get_period(&self) -> Period {
        Period::from(self.period.clone())
    }

    pub fn get_metric_value(&self) -> Result<MetricValue, serde_json::Error> {
        serde_json::from_value(self.metric_value.clone())
    }

    pub fn update_metric_value(&mut self, metric_value: MetricValue) {
        self.metric_value = serde_json::to_value(metric_value).unwrap_or_default();
        self.updated_at = Utc::now();
    }

    pub fn is_for_department(&self) -> bool {
        self.department_id.is_some()
    }

    pub fn is_organization_level(&self) -> bool {
        self.department_id.is_none()
    }
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
    fn test_analytics_type_from_str() {
        assert_eq!(
            AnalyticsType::from_str("performance").unwrap(),
            AnalyticsType::Performance
        );
        assert_eq!(
            AnalyticsType::from_str("productivity").unwrap(),
            AnalyticsType::Productivity
        );
        assert_eq!(
            AnalyticsType::from_str("engagement").unwrap(),
            AnalyticsType::Engagement
        );
        assert_eq!(
            AnalyticsType::from_str("quality").unwrap(),
            AnalyticsType::Quality
        );
        assert_eq!(
            AnalyticsType::from_str("resource").unwrap(),
            AnalyticsType::Resource
        );
        assert_eq!(
            AnalyticsType::from_str("user").unwrap(),
            AnalyticsType::User
        );
        assert_eq!(
            AnalyticsType::from_str("security").unwrap(),
            AnalyticsType::Security
        );
        assert_eq!(
            AnalyticsType::from_str("compliance").unwrap(),
            AnalyticsType::Compliance
        );
        assert!(AnalyticsType::from_str("invalid").is_err());
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
    fn test_period_from_str() {
        assert_eq!(Period::from_str("daily").unwrap(), Period::Daily);
        assert_eq!(Period::from_str("weekly").unwrap(), Period::Weekly);
        assert_eq!(Period::from_str("monthly").unwrap(), Period::Monthly);
        assert_eq!(Period::from_str("quarterly").unwrap(), Period::Quarterly);
        assert_eq!(Period::from_str("yearly").unwrap(), Period::Yearly);
        assert!(Period::from_str("invalid").is_err());
    }

    #[test]
    fn test_analytics_model_new() {
        let organization_id = Uuid::new_v4();
        let department_id = Some(Uuid::new_v4());
        let calculated_by = Uuid::new_v4();

        let metric_value = MetricValue {
            value: 95.5,
            trend: Some(2.5),
            benchmark: Some(90.0),
            metadata: HashMap::new(),
        };

        let period_start = Utc::now() - chrono::Duration::days(30);
        let period_end = Utc::now();

        let model = Model::new(
            organization_id,
            department_id,
            AnalyticsType::Performance,
            "task_completion_rate".to_string(),
            metric_value,
            Period::Monthly,
            period_start,
            period_end,
            calculated_by,
        );

        assert_eq!(model.organization_id, organization_id);
        assert_eq!(model.department_id, department_id);
        assert_eq!(model.get_analytics_type(), AnalyticsType::Performance);
        assert_eq!(model.metric_name, "task_completion_rate");
        assert_eq!(model.get_period(), Period::Monthly);
        assert_eq!(model.calculated_by, calculated_by);
        assert_eq!(model.period_start, period_start);
        assert_eq!(model.period_end, period_end);
    }

    #[test]
    fn test_analytics_model_getters() {
        let organization_id = Uuid::new_v4();
        let calculated_by = Uuid::new_v4();

        let metric_value = MetricValue {
            value: 85.0,
            trend: Some(-1.2),
            benchmark: Some(88.0),
            metadata: HashMap::new(),
        };

        let period_start = Utc::now() - chrono::Duration::days(7);
        let period_end = Utc::now();

        let model = Model::new(
            organization_id,
            None,
            AnalyticsType::Quality,
            "code_quality_score".to_string(),
            metric_value.clone(),
            Period::Weekly,
            period_start,
            period_end,
            calculated_by,
        );

        assert_eq!(model.get_analytics_type(), AnalyticsType::Quality);
        assert_eq!(model.get_period(), Period::Weekly);

        let retrieved_metric = model.get_metric_value().unwrap();
        assert_eq!(retrieved_metric.value, metric_value.value);
        assert_eq!(retrieved_metric.trend, metric_value.trend);
        assert_eq!(retrieved_metric.benchmark, metric_value.benchmark);
    }

    #[test]
    fn test_analytics_model_department_checks() {
        let organization_id = Uuid::new_v4();
        let department_id = Uuid::new_v4();
        let calculated_by = Uuid::new_v4();

        let metric_value = MetricValue {
            value: 75.0,
            trend: None,
            benchmark: None,
            metadata: HashMap::new(),
        };

        let period_start = Utc::now() - chrono::Duration::days(1);
        let period_end = Utc::now();

        // Department-level analytics
        let dept_model = Model::new(
            organization_id,
            Some(department_id),
            AnalyticsType::Engagement,
            "employee_engagement".to_string(),
            metric_value.clone(),
            Period::Daily,
            period_start,
            period_end,
            calculated_by,
        );

        assert!(dept_model.is_for_department());
        assert!(!dept_model.is_organization_level());

        // Organization-level analytics
        let org_model = Model::new(
            organization_id,
            None,
            AnalyticsType::Engagement,
            "overall_engagement".to_string(),
            metric_value,
            Period::Daily,
            period_start,
            period_end,
            calculated_by,
        );

        assert!(!org_model.is_for_department());
        assert!(org_model.is_organization_level());
    }

    #[test]
    fn test_analytics_model_update_metric_value() {
        let organization_id = Uuid::new_v4();
        let calculated_by = Uuid::new_v4();

        let initial_metric = MetricValue {
            value: 80.0,
            trend: Some(0.0),
            benchmark: Some(82.0),
            metadata: HashMap::new(),
        };

        let updated_metric = MetricValue {
            value: 85.0,
            trend: Some(5.0),
            benchmark: Some(82.0),
            metadata: HashMap::new(),
        };

        let period_start = Utc::now() - chrono::Duration::days(30);
        let period_end = Utc::now();

        let mut model = Model::new(
            organization_id,
            None,
            AnalyticsType::Productivity,
            "team_velocity".to_string(),
            initial_metric,
            Period::Monthly,
            period_start,
            period_end,
            calculated_by,
        );

        // Update the metric value
        model.update_metric_value(updated_metric.clone());

        let retrieved_metric = model.get_metric_value().unwrap();
        assert_eq!(retrieved_metric.value, updated_metric.value);
        assert_eq!(retrieved_metric.trend, updated_metric.trend);
        assert_eq!(retrieved_metric.benchmark, updated_metric.benchmark);
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
