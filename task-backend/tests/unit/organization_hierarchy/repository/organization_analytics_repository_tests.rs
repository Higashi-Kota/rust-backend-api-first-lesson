// tests/unit/organization_hierarchy/repository/organization_analytics_repository_tests.rs

use crate::common::db::setup_test_db;
use sea_orm::*;
use task_backend::domain::organization_analytics_model::{
    ActiveModel, AnalyticsType, MetricValue, Model, Period,
};
use task_backend::repository::organization_analytics_repository::OrganizationAnalyticsRepository;
use uuid::Uuid;

async fn create_test_role(db: &DatabaseConnection) -> Uuid {
    use sea_orm::prelude::*;
    use task_backend::domain::role_model::ActiveModel as RoleActiveModel;

    let role_id = Uuid::new_v4();
    let active_model = RoleActiveModel {
        id: Set(role_id),
        name: Set("test_role".to_string()),
        display_name: Set("Test Role".to_string()),
        description: Set(Some("Test role".to_string())),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    active_model.insert(db).await.unwrap();
    role_id
}

async fn create_test_user(db: &DatabaseConnection) -> Uuid {
    use sea_orm::prelude::*;
    use task_backend::domain::user_model::ActiveModel as UserActiveModel;

    let user_id = Uuid::new_v4();
    let role_id = create_test_role(db).await;

    let active_model = UserActiveModel {
        id: Set(user_id),
        username: Set("testuser".to_string()),
        email: Set("test@example.com".to_string()),
        password_hash: Set("hash".to_string()),
        subscription_tier: Set("free".to_string()),
        is_active: Set(true),
        email_verified: Set(false),
        last_login_at: Set(None),
        role_id: Set(role_id),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    active_model.insert(db).await.unwrap();
    user_id
}

async fn create_test_organization(db: &DatabaseConnection) -> Uuid {
    use sea_orm::prelude::*;
    use task_backend::domain::organization_model::ActiveModel as OrgActiveModel;

    let org_id = Uuid::new_v4();
    let user_id = create_test_user(db).await;

    let active_model = OrgActiveModel {
        id: Set(org_id),
        name: Set("Test Organization".to_string()),
        description: Set(Some("Test org".to_string())),
        owner_id: Set(user_id),
        subscription_tier: Set("free".to_string()),
        max_teams: Set(10),
        max_members: Set(100),
        settings_json: Set("{}".to_string()),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    active_model.insert(db).await.unwrap();
    org_id
}

async fn create_test_analytics(
    db: &DatabaseConnection,
    organization_id: Uuid,
    analytics_type: AnalyticsType,
    metric_name: &str,
) -> Model {
    let metric_value = MetricValue {
        value: 100.0,
        trend: Some(5.5),
        benchmark: Some(95.0),
        metadata: std::collections::HashMap::new(),
    };

    let active_model = ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(organization_id),
        department_id: Set(None),
        analytics_type: Set(analytics_type.to_string()),
        metric_name: Set(metric_name.to_string()),
        metric_value: Set(serde_json::to_value(&metric_value).unwrap()),
        period: Set(Period::Monthly.to_string()),
        period_start: Set(chrono::Utc::now() - chrono::Duration::days(30)),
        period_end: Set(chrono::Utc::now()),
        calculated_by: Set(create_test_user(db).await),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    OrganizationAnalyticsRepository::create(db, active_model)
        .await
        .unwrap()
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_analytics_creation_logic() {
    let db = setup_test_db().await;
    let organization_id = create_test_organization(&db).await;

    let analytics = create_test_analytics(
        &db,
        organization_id,
        AnalyticsType::Performance,
        "task_completion_rate",
    )
    .await;

    // 実際のロジックテスト: 分析データ作成の検証
    assert_eq!(analytics.organization_id, organization_id);
    assert_eq!(analytics.analytics_type, "performance");
    assert_eq!(analytics.metric_name, "task_completion_rate");
    assert_eq!(analytics.period, "monthly");
    assert!(analytics.department_id.is_none());
}

#[tokio::test]
async fn test_analytics_type_string_conversion_logic() {
    // AnalyticsTypeの文字列変換ロジックテスト
    assert_eq!(AnalyticsType::Performance.to_string(), "performance");
    assert_eq!(AnalyticsType::Productivity.to_string(), "productivity");
    assert_eq!(AnalyticsType::Engagement.to_string(), "engagement");
    assert_eq!(AnalyticsType::Quality.to_string(), "quality");
}

#[tokio::test]
async fn test_period_string_conversion_logic() {
    // Periodの文字列変換ロジックテスト
    assert_eq!(Period::Daily.to_string(), "daily");
    assert_eq!(Period::Weekly.to_string(), "weekly");
    assert_eq!(Period::Monthly.to_string(), "monthly");
    assert_eq!(Period::Quarterly.to_string(), "quarterly");
    assert_eq!(Period::Yearly.to_string(), "yearly");
}

#[tokio::test]
async fn test_metric_value_json_serialization_logic() {
    // MetricValueのJSON変換ロジックテスト
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("source".to_string(), serde_json::json!("automated"));

    let metric_value = MetricValue {
        value: 85.5,
        trend: Some(-2.1),
        benchmark: Some(88.0),
        metadata,
    };

    let json_value = serde_json::to_value(&metric_value).unwrap();
    assert!(json_value.is_object());
    assert_eq!(json_value["value"], 85.5);
    assert_eq!(json_value["benchmark"], 88.0);
    assert_eq!(json_value["trend"], -2.1);
}

#[tokio::test]
async fn test_analytics_type_from_str_logic() {
    // AnalyticsTypeのfrom_strメソッドのロジックテスト
    assert!(AnalyticsType::from_str("performance").is_ok());
    assert!(AnalyticsType::from_str("productivity").is_ok());
    assert!(AnalyticsType::from_str("engagement").is_ok());
    assert!(AnalyticsType::from_str("quality").is_ok());
    assert!(AnalyticsType::from_str("invalid").is_err());
}

#[tokio::test]
async fn test_period_from_str_logic() {
    // Periodのfrom_strメソッドのロジックテスト
    assert!(Period::from_str("daily").is_ok());
    assert!(Period::from_str("weekly").is_ok());
    assert!(Period::from_str("monthly").is_ok());
    assert!(Period::from_str("quarterly").is_ok());
    assert!(Period::from_str("yearly").is_ok());
    assert!(Period::from_str("invalid").is_err());
}

#[tokio::test]
async fn test_analytics_model_new_logic() {
    // Model::newメソッドのロジックテスト
    let organization_id = Uuid::new_v4();
    let department_id = Some(Uuid::new_v4());
    let calculated_by = Uuid::new_v4();

    let metric_value = MetricValue {
        value: 75.5,
        trend: Some(2.3),
        benchmark: Some(80.0),
        metadata: std::collections::HashMap::new(),
    };

    let period_start = chrono::Utc::now() - chrono::Duration::days(30);
    let period_end = chrono::Utc::now();

    let analytics = Model::new(
        organization_id,
        department_id,
        AnalyticsType::Performance,
        "task_completion_rate".to_string(),
        metric_value.clone(),
        Period::Monthly,
        period_start,
        period_end,
        calculated_by,
    );

    assert_eq!(analytics.organization_id, organization_id);
    assert_eq!(analytics.department_id, department_id);
    assert_eq!(analytics.analytics_type, "performance");
    assert_eq!(analytics.metric_name, "task_completion_rate");
    assert_eq!(analytics.period, "monthly");
    assert_eq!(analytics.calculated_by, calculated_by);
    assert_eq!(analytics.get_analytics_type(), AnalyticsType::Performance);
    assert_eq!(analytics.get_period(), Period::Monthly);
}

#[tokio::test]
async fn test_analytics_model_update_metric_value_logic() {
    // update_metric_valueメソッドのロジックテスト
    let organization_id = Uuid::new_v4();
    let calculated_by = Uuid::new_v4();

    let initial_metric = MetricValue {
        value: 70.0,
        trend: Some(1.0),
        benchmark: Some(75.0),
        metadata: std::collections::HashMap::new(),
    };

    let mut analytics = Model::new(
        organization_id,
        None,
        AnalyticsType::Productivity,
        "efficiency_score".to_string(),
        initial_metric,
        Period::Weekly,
        chrono::Utc::now() - chrono::Duration::days(7),
        chrono::Utc::now(),
        calculated_by,
    );

    let original_updated_at = analytics.updated_at;

    // 少し時間を進める
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let new_metric = MetricValue {
        value: 85.0,
        trend: Some(3.5),
        benchmark: Some(80.0),
        metadata: std::collections::HashMap::new(),
    };

    analytics.update_metric_value(new_metric.clone());

    let retrieved_metric = analytics.get_metric_value().unwrap();
    assert_eq!(retrieved_metric.value, 85.0);
    assert_eq!(retrieved_metric.trend, Some(3.5));
    assert!(analytics.updated_at > original_updated_at);
}

#[tokio::test]
async fn test_analytics_model_is_for_department_logic() {
    // is_for_department/is_organization_levelメソッドのロジックテスト
    let organization_id = Uuid::new_v4();
    let department_id = Uuid::new_v4();
    let calculated_by = Uuid::new_v4();

    let metric_value = MetricValue {
        value: 90.0,
        trend: None,
        benchmark: None,
        metadata: std::collections::HashMap::new(),
    };

    // Department level analytics
    let dept_analytics = Model::new(
        organization_id,
        Some(department_id),
        AnalyticsType::Quality,
        "error_rate".to_string(),
        metric_value.clone(),
        Period::Daily,
        chrono::Utc::now() - chrono::Duration::days(1),
        chrono::Utc::now(),
        calculated_by,
    );

    assert!(dept_analytics.is_for_department());
    assert!(!dept_analytics.is_organization_level());

    // Organization level analytics
    let org_analytics = Model::new(
        organization_id,
        None,
        AnalyticsType::Quality,
        "overall_score".to_string(),
        metric_value,
        Period::Daily,
        chrono::Utc::now() - chrono::Duration::days(1),
        chrono::Utc::now(),
        calculated_by,
    );

    assert!(!org_analytics.is_for_department());
    assert!(org_analytics.is_organization_level());
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_analytics_repository_comprehensive() {
    let db = setup_test_db().await;
    let organization_id = create_test_organization(&db).await;

    // Create test analytics data
    let analytics1 = create_test_analytics(
        &db,
        organization_id,
        AnalyticsType::Performance,
        "completion_rate",
    )
    .await;
    let analytics2 = create_test_analytics(
        &db,
        organization_id,
        AnalyticsType::Productivity,
        "efficiency",
    )
    .await;
    let _analytics3 = create_test_analytics(
        &db,
        organization_id,
        AnalyticsType::Performance,
        "quality_score",
    )
    .await;

    // Test find_by_organization_id
    let org_analytics =
        OrganizationAnalyticsRepository::find_by_organization_id(&db, organization_id, Some(10))
            .await
            .unwrap();
    assert_eq!(org_analytics.len(), 3);

    // Test find_by_organization_and_type
    let performance_analytics = OrganizationAnalyticsRepository::find_by_organization_and_type(
        &db,
        organization_id,
        AnalyticsType::Performance,
        Some(10),
    )
    .await
    .unwrap();
    assert_eq!(performance_analytics.len(), 2);

    // Test find_by_metric_name
    let completion_analytics = OrganizationAnalyticsRepository::find_by_metric_name(
        &db,
        organization_id,
        "completion_rate",
        Some(10),
    )
    .await
    .unwrap();
    assert_eq!(completion_analytics.len(), 1);

    // Test find_latest_by_organization
    let latest_analytics =
        OrganizationAnalyticsRepository::find_latest_by_organization(&db, organization_id, 2)
            .await
            .unwrap();
    assert_eq!(latest_analytics.len(), 2);

    // Test update_by_id
    let mut updated_analytics = analytics1.clone().into_active_model();
    updated_analytics.metric_name = sea_orm::Set("updated_metric".to_string());
    let updated =
        OrganizationAnalyticsRepository::update_by_id(&db, analytics1.id, updated_analytics)
            .await
            .unwrap();
    assert_eq!(updated.metric_name, "updated_metric");

    // Test count_by_organization_id
    let count = OrganizationAnalyticsRepository::count_by_organization_id(&db, organization_id)
        .await
        .unwrap();
    assert_eq!(count, 3);

    // Test find_analytics_summary
    let summary = OrganizationAnalyticsRepository::find_analytics_summary(
        &db,
        organization_id,
        Period::Monthly,
        10,
    )
    .await
    .unwrap();
    assert_eq!(summary.len(), 3);

    // Test exists_analytics_for_period
    let period_start = chrono::Utc::now() - chrono::Duration::days(30);
    let period_end = chrono::Utc::now();
    let exists = OrganizationAnalyticsRepository::exists_analytics_for_period(
        &db,
        organization_id,
        None,
        AnalyticsType::Performance,
        "completion_rate",
        period_start,
        period_end,
    )
    .await
    .unwrap();
    assert!(!exists); // Won't exist with exact dates

    // Test delete_by_id
    OrganizationAnalyticsRepository::delete_by_id(&db, analytics2.id)
        .await
        .unwrap();
    let deleted_check = OrganizationAnalyticsRepository::find_by_id(&db, analytics2.id)
        .await
        .unwrap();
    assert!(deleted_check.is_none());

    // Test delete_by_organization_id
    let deleted_count =
        OrganizationAnalyticsRepository::delete_by_organization_id(&db, organization_id)
            .await
            .unwrap();
    assert_eq!(deleted_count, 2); // Should delete remaining 2
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_analytics_repository_with_department() {
    let db = setup_test_db().await;
    let organization_id = create_test_organization(&db).await;

    // Create department-specific analytics
    let department_id = Uuid::new_v4();
    let metric_value = MetricValue {
        value: 95.0,
        trend: Some(1.5),
        benchmark: Some(90.0),
        metadata: std::collections::HashMap::new(),
    };

    let dept_analytics = ActiveModel {
        id: sea_orm::Set(Uuid::new_v4()),
        organization_id: sea_orm::Set(organization_id),
        department_id: sea_orm::Set(Some(department_id)),
        analytics_type: sea_orm::Set(AnalyticsType::Engagement.to_string()),
        metric_name: sea_orm::Set("team_collaboration".to_string()),
        metric_value: sea_orm::Set(serde_json::to_value(&metric_value).unwrap()),
        period: sea_orm::Set(Period::Quarterly.to_string()),
        period_start: sea_orm::Set(chrono::Utc::now() - chrono::Duration::days(90)),
        period_end: sea_orm::Set(chrono::Utc::now()),
        calculated_by: sea_orm::Set(create_test_user(&db).await),
        created_at: sea_orm::Set(chrono::Utc::now()),
        updated_at: sea_orm::Set(chrono::Utc::now()),
    };

    let _created_analytics = OrganizationAnalyticsRepository::create(&db, dept_analytics)
        .await
        .unwrap();

    // Test find_by_department_id
    let dept_analytics_result =
        OrganizationAnalyticsRepository::find_by_department_id(&db, department_id, Some(10))
            .await
            .unwrap();
    assert_eq!(dept_analytics_result.len(), 1);

    // Test find_by_department_and_type
    let dept_engagement_analytics = OrganizationAnalyticsRepository::find_by_department_and_type(
        &db,
        department_id,
        AnalyticsType::Engagement,
        Some(10),
    )
    .await
    .unwrap();
    assert_eq!(dept_engagement_analytics.len(), 1);

    // Test count_by_department_id
    let dept_count = OrganizationAnalyticsRepository::count_by_department_id(&db, department_id)
        .await
        .unwrap();
    assert_eq!(dept_count, 1);

    // Test delete_by_department_id
    let deleted_dept_count =
        OrganizationAnalyticsRepository::delete_by_department_id(&db, department_id)
            .await
            .unwrap();
    assert_eq!(deleted_dept_count, 1);
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_analytics_repository_date_filtering() {
    let db = setup_test_db().await;
    let organization_id = create_test_organization(&db).await;

    let now = chrono::Utc::now();
    let old_date = now - chrono::Duration::days(365);

    // Create old analytics
    let old_metric_value = MetricValue {
        value: 60.0,
        trend: None,
        benchmark: None,
        metadata: std::collections::HashMap::new(),
    };

    let old_analytics = ActiveModel {
        id: sea_orm::Set(Uuid::new_v4()),
        organization_id: sea_orm::Set(organization_id),
        department_id: sea_orm::Set(None),
        analytics_type: sea_orm::Set(AnalyticsType::Security.to_string()),
        metric_name: sea_orm::Set("vulnerability_count".to_string()),
        metric_value: sea_orm::Set(serde_json::to_value(&old_metric_value).unwrap()),
        period: sea_orm::Set(Period::Yearly.to_string()),
        period_start: sea_orm::Set(old_date - chrono::Duration::days(365)),
        period_end: sea_orm::Set(old_date),
        calculated_by: sea_orm::Set(create_test_user(&db).await),
        created_at: sea_orm::Set(old_date),
        updated_at: sea_orm::Set(old_date),
    };

    OrganizationAnalyticsRepository::create(&db, old_analytics)
        .await
        .unwrap();

    // Test find_by_organization_and_period
    let period_analytics = OrganizationAnalyticsRepository::find_by_organization_and_period(
        &db,
        organization_id,
        Period::Yearly,
        old_date - chrono::Duration::days(400),
        old_date + chrono::Duration::days(1),
    )
    .await
    .unwrap();
    assert_eq!(period_analytics.len(), 1);

    // Test delete_old_analytics
    let deleted_old_count = OrganizationAnalyticsRepository::delete_old_analytics(
        &db,
        now - chrono::Duration::days(180),
    )
    .await
    .unwrap();
    assert_eq!(deleted_old_count, 1);
}
