// tests/unit/organization_hierarchy/repository/permission_matrix_repository_tests.rs

use crate::common::db::setup_test_db;
use sea_orm::*;
use task_backend::domain::permission_matrix_model::{
    ActiveModel, ComplianceSettings, EntityType, InheritanceSettings, Model, PermissionMatrix,
};
use task_backend::repository::permission_matrix_repository::PermissionMatrixRepository;
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

async fn create_test_permission_matrix(
    db: &DatabaseConnection,
    entity_type: EntityType,
    entity_id: Uuid,
) -> Model {
    let matrix_data = PermissionMatrix {
        tasks: std::collections::HashMap::from([
            ("create".to_string(), true),
            ("read".to_string(), true),
            ("update".to_string(), false),
            ("delete".to_string(), false),
        ]),
        analytics: std::collections::HashMap::from([
            ("view".to_string(), true),
            ("export".to_string(), false),
        ]),
        administration: std::collections::HashMap::from([
            ("manage_users".to_string(), false),
            ("manage_settings".to_string(), false),
        ]),
    };

    let active_model = ActiveModel {
        id: Set(Uuid::new_v4()),
        entity_type: Set(entity_type.to_string()),
        entity_id: Set(entity_id),
        matrix_version: Set("v1.0".to_string()),
        matrix_data: Set(serde_json::to_value(&matrix_data).unwrap()),
        inheritance_settings: Set(None),
        compliance_settings: Set(None),
        updated_by: Set(create_test_user(db).await),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    PermissionMatrixRepository::create(db, active_model)
        .await
        .unwrap()
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_permission_matrix_creation_logic() {
    let db = setup_test_db().await;
    let entity_id = Uuid::new_v4();

    let matrix = create_test_permission_matrix(&db, EntityType::Organization, entity_id).await;

    // 実際のロジックテスト: 権限マトリックス作成の検証
    assert_eq!(matrix.entity_type, "organization");
    assert_eq!(matrix.entity_id, entity_id);
    assert_eq!(matrix.matrix_version, "v1.0");
    assert!(matrix.is_active);
    assert!(matrix.matrix_data.is_object());
}

#[tokio::test]
async fn test_entity_type_string_conversion_logic() {
    // EntityTypeの文字列変換ロジックテスト
    assert_eq!(EntityType::Organization.to_string(), "organization");
    assert_eq!(EntityType::Department.to_string(), "department");
    assert_eq!(EntityType::Team.to_string(), "team");
    assert_eq!(EntityType::User.to_string(), "user");
}

#[tokio::test]
async fn test_permission_matrix_json_serialization_logic() {
    // PermissionMatrixのJSON変換ロジックテスト
    let matrix = PermissionMatrix {
        tasks: std::collections::HashMap::from([
            ("create".to_string(), true),
            ("read".to_string(), true),
        ]),
        analytics: std::collections::HashMap::from([("view".to_string(), false)]),
        administration: std::collections::HashMap::new(),
    };

    let json_value = serde_json::to_value(&matrix).unwrap();
    assert!(json_value.is_object());
    assert!(json_value["tasks"].is_object());
    assert!(json_value["analytics"].is_object());
    assert!(json_value["administration"].is_object());
    assert_eq!(json_value["tasks"]["create"], true);
    assert_eq!(json_value["analytics"]["view"], false);
}

#[tokio::test]
async fn test_inheritance_settings_logic() {
    // InheritanceSettingsの論理テスト
    let settings = InheritanceSettings {
        inherit_from_parent: true,
        allow_override: false,
        inheritance_priority: 1,
    };

    // 継承設定のロジック検証
    assert!(settings.inherit_from_parent);
    assert!(!settings.allow_override);
    assert_eq!(settings.inheritance_priority, 1);
}

#[tokio::test]
async fn test_compliance_settings_logic() {
    // ComplianceSettingsの論理テスト
    let settings = ComplianceSettings {
        audit_required: true,
        approval_workflow: false,
        retention_period_days: 365,
        compliance_level: "high".to_string(),
    };

    // コンプライアンス設定のロジック検証
    assert!(settings.audit_required);
    assert!(!settings.approval_workflow);
    assert_eq!(settings.retention_period_days, 365);
    assert_eq!(settings.compliance_level, "high");
}

#[tokio::test]
async fn test_entity_type_from_str_logic() {
    // EntityTypeのfrom_strメソッドのロジックテスト
    assert!(EntityType::from_str("organization").is_ok());
    assert!(EntityType::from_str("department").is_ok());
    assert!(EntityType::from_str("team").is_ok());
    assert!(EntityType::from_str("user").is_ok());
    assert!(EntityType::from_str("invalid").is_err());
}

#[tokio::test]
async fn test_permission_matrix_model_new_logic() {
    // Model::newメソッドのロジックテスト
    let entity_id = Uuid::new_v4();
    let updated_by = Uuid::new_v4();

    let matrix_data = PermissionMatrix {
        tasks: std::collections::HashMap::from([
            ("create".to_string(), true),
            ("read".to_string(), true),
            ("update".to_string(), false),
        ]),
        analytics: std::collections::HashMap::from([("view".to_string(), true)]),
        administration: std::collections::HashMap::new(),
    };

    let inheritance_settings = InheritanceSettings {
        inherit_from_parent: true,
        allow_override: false,
        inheritance_priority: 2,
    };

    let compliance_settings = ComplianceSettings {
        audit_required: true,
        approval_workflow: false,
        retention_period_days: 90,
        compliance_level: "medium".to_string(),
    };

    let model = Model::new(
        EntityType::Department,
        entity_id,
        matrix_data.clone(),
        updated_by,
        Some(inheritance_settings.clone()),
        Some(compliance_settings.clone()),
    );

    assert_eq!(model.entity_type, "department");
    assert_eq!(model.entity_id, entity_id);
    assert_eq!(model.updated_by, updated_by);
    assert_eq!(model.matrix_version, "v1.0");
    assert!(model.is_active);
    assert_eq!(model.get_entity_type(), EntityType::Department);

    let retrieved_matrix = model.get_permission_matrix().unwrap();
    assert_eq!(retrieved_matrix.tasks.get("create"), Some(&true));
    assert_eq!(retrieved_matrix.tasks.get("update"), Some(&false));

    let retrieved_inheritance = model.get_inheritance_settings().unwrap().unwrap();
    assert!(retrieved_inheritance.inherit_from_parent);
    assert_eq!(retrieved_inheritance.inheritance_priority, 2);

    let retrieved_compliance = model.get_compliance_settings().unwrap().unwrap();
    assert!(retrieved_compliance.audit_required);
    assert_eq!(retrieved_compliance.retention_period_days, 90);
}

#[tokio::test]
async fn test_permission_matrix_model_update_methods_logic() {
    // update関連メソッドのロジックテスト
    let entity_id = Uuid::new_v4();
    let updated_by = Uuid::new_v4();

    let initial_matrix = PermissionMatrix {
        tasks: std::collections::HashMap::from([("create".to_string(), false)]),
        analytics: std::collections::HashMap::new(),
        administration: std::collections::HashMap::new(),
    };

    let mut model = Model::new(
        EntityType::User,
        entity_id,
        initial_matrix,
        updated_by,
        None,
        None,
    );

    let original_updated_at = model.updated_at;
    let original_version = model.matrix_version.clone();

    // 少し時間を進める
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Test update_matrix_data
    let new_matrix = PermissionMatrix {
        tasks: std::collections::HashMap::from([
            ("create".to_string(), true),
            ("read".to_string(), true),
        ]),
        analytics: std::collections::HashMap::from([("view".to_string(), false)]),
        administration: std::collections::HashMap::new(),
    };

    model.update_matrix_data(new_matrix.clone());

    let retrieved_matrix = model.get_permission_matrix().unwrap();
    assert_eq!(retrieved_matrix.tasks.get("create"), Some(&true));
    assert_eq!(retrieved_matrix.tasks.get("read"), Some(&true));
    assert_eq!(retrieved_matrix.analytics.get("view"), Some(&false));
    assert!(model.updated_at > original_updated_at);

    // Test update_inheritance_settings
    let new_inheritance = InheritanceSettings {
        inherit_from_parent: false,
        allow_override: true,
        inheritance_priority: 5,
    };

    model.update_inheritance_settings(new_inheritance.clone());

    let retrieved_inheritance = model.get_inheritance_settings().unwrap().unwrap();
    assert!(!retrieved_inheritance.inherit_from_parent);
    assert!(retrieved_inheritance.allow_override);
    assert_eq!(retrieved_inheritance.inheritance_priority, 5);

    // Test increment_version
    model.increment_version();

    assert_ne!(model.matrix_version, original_version);
    assert!(model.matrix_version.starts_with("v"));
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_permission_matrix_repository_comprehensive() {
    let db = setup_test_db().await;

    let entity1_id = Uuid::new_v4();
    let entity2_id = Uuid::new_v4();

    // Create test permission matrices
    let matrix1 = create_test_permission_matrix(&db, EntityType::Organization, entity1_id).await;
    let matrix2 = create_test_permission_matrix(&db, EntityType::Department, entity2_id).await;

    // Test find_by_id
    let found_matrix = PermissionMatrixRepository::find_by_id(&db, matrix1.id)
        .await
        .unwrap();
    assert!(found_matrix.is_some());
    assert_eq!(found_matrix.unwrap().entity_type, "organization");

    // Test find_by_entity
    let entity_matrix =
        PermissionMatrixRepository::find_by_entity(&db, EntityType::Department, entity2_id)
            .await
            .unwrap();
    assert!(entity_matrix.is_some());
    assert_eq!(entity_matrix.unwrap().id, matrix2.id);

    // Test find_by_entity_type
    let org_matrices =
        PermissionMatrixRepository::find_by_entity_type(&db, EntityType::Organization)
            .await
            .unwrap();
    assert_eq!(org_matrices.len(), 1);

    let dept_matrices =
        PermissionMatrixRepository::find_by_entity_type(&db, EntityType::Department)
            .await
            .unwrap();
    assert_eq!(dept_matrices.len(), 1);

    // Test find_organization_matrices
    let org_matrix_list =
        PermissionMatrixRepository::find_organization_matrices(&db, vec![entity1_id])
            .await
            .unwrap();
    assert_eq!(org_matrix_list.len(), 1);
    assert!(org_matrix_list[0].is_active);

    // Test find_department_matrices
    let dept_matrix_list =
        PermissionMatrixRepository::find_department_matrices(&db, vec![entity2_id])
            .await
            .unwrap();
    assert_eq!(dept_matrix_list.len(), 1);

    // Test find_matrices_by_version
    let version_matrices = PermissionMatrixRepository::find_matrices_by_version(&db, "v1.0")
        .await
        .unwrap();
    assert_eq!(version_matrices.len(), 2); // Both test matrices use v1.0

    // Test find_by_updated_by
    let updated_by_matrices =
        PermissionMatrixRepository::find_by_updated_by(&db, matrix1.updated_by)
            .await
            .unwrap();
    assert!(!updated_by_matrices.is_empty());

    // Test find_latest_by_entity_type
    let latest_org_matrices =
        PermissionMatrixRepository::find_latest_by_entity_type(&db, EntityType::Organization, 10)
            .await
            .unwrap();
    assert_eq!(latest_org_matrices.len(), 1);

    let latest_dept_matrices =
        PermissionMatrixRepository::find_latest_by_entity_type(&db, EntityType::Department, 10)
            .await
            .unwrap();
    assert_eq!(latest_dept_matrices.len(), 1);

    // Test entity_has_matrix
    let has_matrix_org =
        PermissionMatrixRepository::entity_has_matrix(&db, EntityType::Organization, entity1_id)
            .await
            .unwrap();
    assert!(has_matrix_org);

    let no_matrix_entity =
        PermissionMatrixRepository::entity_has_matrix(&db, EntityType::User, Uuid::new_v4())
            .await
            .unwrap();
    assert!(!no_matrix_entity);

    // Test update_by_id
    let mut updated_matrix = matrix1.clone().into_active_model();
    updated_matrix.matrix_version = sea_orm::Set("v2.0".to_string());
    let updated = PermissionMatrixRepository::update_by_id(&db, matrix1.id, updated_matrix)
        .await
        .unwrap();
    assert_eq!(updated.matrix_version, "v2.0");

    // Test update_by_entity (should deactivate old and create new)
    let new_matrix_data = PermissionMatrix {
        tasks: std::collections::HashMap::from([
            ("create".to_string(), false),
            ("read".to_string(), true),
        ]),
        analytics: std::collections::HashMap::new(),
        administration: std::collections::HashMap::new(),
    };

    let new_active_model = ActiveModel {
        id: sea_orm::Set(Uuid::new_v4()),
        entity_type: sea_orm::Set(EntityType::Department.to_string()),
        entity_id: sea_orm::Set(entity2_id),
        matrix_version: sea_orm::Set("v2.5".to_string()),
        matrix_data: sea_orm::Set(serde_json::to_value(&new_matrix_data).unwrap()),
        inheritance_settings: sea_orm::Set(None),
        compliance_settings: sea_orm::Set(None),
        updated_by: sea_orm::Set(create_test_user(&db).await),
        is_active: sea_orm::Set(true),
        created_at: sea_orm::Set(chrono::Utc::now()),
        updated_at: sea_orm::Set(chrono::Utc::now()),
    };

    let updated_by_entity = PermissionMatrixRepository::update_by_entity(
        &db,
        EntityType::Department,
        entity2_id,
        new_active_model,
    )
    .await
    .unwrap();
    assert_eq!(updated_by_entity.matrix_version, "v2.5");

    // Check that old matrix was deactivated
    let old_matrix = PermissionMatrixRepository::find_by_id(&db, matrix2.id)
        .await
        .unwrap()
        .unwrap();
    assert!(!old_matrix.is_active);

    // Test deactivate_by_entity
    PermissionMatrixRepository::deactivate_by_entity(&db, EntityType::Department, entity2_id)
        .await
        .unwrap();
    let current_matrix =
        PermissionMatrixRepository::find_by_entity(&db, EntityType::Department, entity2_id)
            .await
            .unwrap();
    assert!(current_matrix.is_none()); // Should be None since active matrix was deactivated

    // Test count_by_entity_type
    let org_count = PermissionMatrixRepository::count_by_entity_type(&db, EntityType::Organization)
        .await
        .unwrap();
    assert_eq!(org_count, 1); // Only org matrix is still active

    let dept_count = PermissionMatrixRepository::count_by_entity_type(&db, EntityType::Department)
        .await
        .unwrap();
    assert_eq!(dept_count, 0); // Department matrix was deactivated

    // Test delete_by_id (soft delete)
    PermissionMatrixRepository::delete_by_id(&db, matrix1.id)
        .await
        .unwrap();
    let deleted_check = PermissionMatrixRepository::find_by_id(&db, matrix1.id)
        .await
        .unwrap()
        .unwrap();
    assert!(!deleted_check.is_active); // Should exist but be inactive
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_permission_matrix_repository_inheritance_and_compliance() {
    let db = setup_test_db().await;

    let entity_id = Uuid::new_v4();

    // Create permission matrix with inheritance and compliance settings
    let matrix_data = PermissionMatrix {
        tasks: std::collections::HashMap::from([
            ("create".to_string(), true),
            ("read".to_string(), true),
        ]),
        analytics: std::collections::HashMap::new(),
        administration: std::collections::HashMap::new(),
    };

    let inheritance_settings = InheritanceSettings {
        inherit_from_parent: true,
        allow_override: false,
        inheritance_priority: 3,
    };

    let compliance_settings = ComplianceSettings {
        audit_required: true,
        approval_workflow: true,
        retention_period_days: 365,
        compliance_level: "high".to_string(),
    };

    let active_model = ActiveModel {
        id: sea_orm::Set(Uuid::new_v4()),
        entity_type: sea_orm::Set(EntityType::Team.to_string()),
        entity_id: sea_orm::Set(entity_id),
        matrix_version: sea_orm::Set("v1.5".to_string()),
        matrix_data: sea_orm::Set(serde_json::to_value(&matrix_data).unwrap()),
        inheritance_settings: sea_orm::Set(Some(
            serde_json::to_value(&inheritance_settings).unwrap(),
        )),
        compliance_settings: sea_orm::Set(Some(
            serde_json::to_value(&compliance_settings).unwrap(),
        )),
        updated_by: sea_orm::Set(create_test_user(&db).await),
        is_active: sea_orm::Set(true),
        created_at: sea_orm::Set(chrono::Utc::now()),
        updated_at: sea_orm::Set(chrono::Utc::now()),
    };

    let created_matrix = PermissionMatrixRepository::create(&db, active_model)
        .await
        .unwrap();

    // Test that matrices with inheritance and compliance settings can be retrieved and have correct data
    let retrieved_matrix = PermissionMatrixRepository::find_by_id(&db, created_matrix.id)
        .await
        .unwrap()
        .unwrap();

    // Verify inheritance settings
    let inheritance = retrieved_matrix
        .get_inheritance_settings()
        .unwrap()
        .unwrap();
    assert!(inheritance.inherit_from_parent);
    assert!(!inheritance.allow_override);
    assert_eq!(inheritance.inheritance_priority, 3);

    // Verify compliance settings
    let compliance = retrieved_matrix.get_compliance_settings().unwrap().unwrap();
    assert!(compliance.audit_required);
    assert!(compliance.approval_workflow);
    assert_eq!(compliance.retention_period_days, 365);
    assert_eq!(compliance.compliance_level, "high");

    // Test find_team_matrices (since this matrix is for a team)
    let team_matrices = PermissionMatrixRepository::find_team_matrices(&db, vec![entity_id])
        .await
        .unwrap();
    assert_eq!(team_matrices.len(), 1);
    assert_eq!(team_matrices[0].id, created_matrix.id);

    // Test find_matrices_by_version with the specific version
    let version_matrices = PermissionMatrixRepository::find_matrices_by_version(&db, "v1.5")
        .await
        .unwrap();
    assert_eq!(version_matrices.len(), 1);
    assert_eq!(version_matrices[0].id, created_matrix.id);

    // Clean up
    PermissionMatrixRepository::delete_by_id(&db, created_matrix.id)
        .await
        .unwrap();
}
