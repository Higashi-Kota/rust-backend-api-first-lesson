// tests/unit/organization_hierarchy/repository/organization_department_repository_tests.rs

use crate::common::db::setup_test_db;
use sea_orm::*;
use task_backend::domain::organization_department_model::{ActiveModel, Model};
use task_backend::repository::organization_department_repository::OrganizationDepartmentRepository;
use uuid::Uuid;

// テスト用の最小限のロール作成
async fn create_minimal_test_role(db: &DatabaseConnection) -> Uuid {
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

// テスト用の最小限のユーザー作成
async fn create_minimal_test_user(db: &DatabaseConnection) -> Uuid {
    use sea_orm::prelude::*;
    use task_backend::domain::user_model::ActiveModel as UserActiveModel;

    let user_id = Uuid::new_v4();
    let role_id = create_minimal_test_role(db).await;

    let active_model = UserActiveModel {
        id: Set(user_id),
        username: Set(format!("testuser_{}", user_id)),
        email: Set(format!("test_{}@example.com", user_id)),
        password_hash: Set("test_hash".to_string()),
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
    // 実際のユーザーを作成して外部キー制約を満たす
    let owner_id = create_minimal_test_user(db).await;

    let active_model = OrgActiveModel {
        id: Set(org_id),
        name: Set("Test Organization".to_string()),
        description: Set(Some("Test org".to_string())),
        owner_id: Set(owner_id),
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

async fn create_test_department(
    db: &DatabaseConnection,
    name: &str,
    organization_id: Uuid,
    parent_id: Option<Uuid>,
) -> Model {
    let department_id = Uuid::new_v4();
    let active_model = ActiveModel {
        id: Set(department_id),
        name: Set(name.to_string()),
        description: Set(Some(format!("Test department: {}", name))),
        organization_id: Set(organization_id),
        parent_department_id: Set(parent_id),
        hierarchy_level: Set(if parent_id.is_some() { 1 } else { 0 }),
        hierarchy_path: Set(if let Some(parent) = parent_id {
            format!("/{}/{}", parent, department_id)
        } else {
            format!("/{}", department_id)
        }),
        manager_user_id: Set(Some(Uuid::new_v4())),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    OrganizationDepartmentRepository::create(db, active_model)
        .await
        .unwrap()
}

#[tokio::test]
#[ignore] // 一時的にスキップ：依存関係の問題を修正中
async fn test_department_creation_logic() {
    let db = setup_test_db().await;
    let organization_id = create_test_organization(&db).await;

    let department = create_test_department(&db, "Engineering", organization_id, None).await;

    // 実際のロジックテスト: 作成されたデータの検証
    assert_eq!(department.name, "Engineering");
    assert_eq!(department.organization_id, organization_id);
    assert_eq!(department.hierarchy_level, 0);
    assert!(department.is_active);
    assert!(department.parent_department_id.is_none());
    assert!(department.hierarchy_path.starts_with('/'));
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_department_hierarchy_logic() {
    let db = setup_test_db().await;
    let organization_id = create_test_organization(&db).await;

    let parent_dept = create_test_department(&db, "Engineering", organization_id, None).await;
    let child_dept =
        create_test_department(&db, "Frontend Team", organization_id, Some(parent_dept.id)).await;

    // 階層構造のロジック検証
    assert_eq!(parent_dept.hierarchy_level, 0);
    assert_eq!(child_dept.hierarchy_level, 1);
    assert_eq!(child_dept.parent_department_id, Some(parent_dept.id));
    assert!(child_dept
        .hierarchy_path
        .contains(&parent_dept.id.to_string()));
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_department_find_by_id_logic() {
    let db = setup_test_db().await;
    let organization_id = create_test_organization(&db).await;

    let created_dept = create_test_department(&db, "Marketing", organization_id, None).await;

    let found_dept = OrganizationDepartmentRepository::find_by_id(&db, created_dept.id)
        .await
        .unwrap();

    // 検索ロジックの検証
    assert!(found_dept.is_some());
    let dept = found_dept.unwrap();
    assert_eq!(dept.id, created_dept.id);
    assert_eq!(dept.name, "Marketing");
    assert_eq!(dept.organization_id, organization_id);
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_department_not_found_logic() {
    let db = setup_test_db().await;

    let non_existent_id = Uuid::new_v4();
    let result = OrganizationDepartmentRepository::find_by_id(&db, non_existent_id)
        .await
        .unwrap();

    // 存在しないIDの検索ロジック検証
    assert!(result.is_none());
}

#[tokio::test]
async fn test_department_model_new_logic() {
    // Model::newメソッドのロジックテスト
    let name = "Engineering Department".to_string();
    let organization_id = Uuid::new_v4();
    let parent_id = Some(Uuid::new_v4());
    let manager_id = Some(Uuid::new_v4());
    let description = Some("Software engineering team".to_string());

    let department = Model::new(
        name.clone(),
        organization_id,
        parent_id,
        manager_id,
        description.clone(),
    );

    assert_eq!(department.name, name);
    assert_eq!(department.organization_id, organization_id);
    assert_eq!(department.parent_department_id, parent_id);
    assert_eq!(department.manager_user_id, manager_id);
    assert_eq!(department.description, description);
    assert_eq!(department.hierarchy_level, 1); // Since parent exists
    assert!(department.is_active);
    assert!(department.hierarchy_path.starts_with('/'));
}

#[tokio::test]
async fn test_department_model_hierarchy_methods_logic() {
    // Hierarchy関連メソッドのロジックテスト
    let name = "Root Department".to_string();
    let organization_id = Uuid::new_v4();

    // Root department
    let mut root_dept = Model::new(name.clone(), organization_id, None, None, None);
    assert_eq!(root_dept.hierarchy_level, 0);
    assert!(root_dept.is_root_department());

    // Test update_hierarchy_path
    root_dept.update_hierarchy_path(None);
    assert_eq!(root_dept.hierarchy_path, format!("/{}", root_dept.id));

    // Child department
    let child_name = "Child Department".to_string();
    let mut child_dept = Model::new(child_name, organization_id, Some(root_dept.id), None, None);
    child_dept.update_hierarchy_path(Some(&root_dept.hierarchy_path));

    assert!(!child_dept.is_root_department());
    assert!(child_dept.is_child_of(root_dept.id));
    assert!(child_dept
        .hierarchy_path
        .contains(&root_dept.id.to_string()));

    // Test update_hierarchy_level
    child_dept.update_hierarchy_level(2);
    assert_eq!(child_dept.hierarchy_level, 2);

    // Test get_path_components
    let path_components = child_dept.get_path_components();
    assert!(path_components.len() >= 2);
    assert!(path_components.contains(&root_dept.id.to_string()));
    assert!(path_components.contains(&child_dept.id.to_string()));
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_department_repository_comprehensive() {
    let db = setup_test_db().await;
    let organization_id = create_test_organization(&db).await;

    // Create test departments
    let root_dept = create_test_department(&db, "Engineering", organization_id, None).await;
    let child_dept =
        create_test_department(&db, "Frontend", organization_id, Some(root_dept.id)).await;
    let another_dept = create_test_department(&db, "Marketing", organization_id, None).await;

    // Test find_by_id
    let found_dept = OrganizationDepartmentRepository::find_by_id(&db, root_dept.id)
        .await
        .unwrap();
    assert!(found_dept.is_some());
    assert_eq!(found_dept.unwrap().name, "Engineering");

    // Test find_by_organization_id
    let org_depts = OrganizationDepartmentRepository::find_by_organization_id(&db, organization_id)
        .await
        .unwrap();
    assert_eq!(org_depts.len(), 3);

    // Test find_children_by_parent_id
    let child_depts =
        OrganizationDepartmentRepository::find_children_by_parent_id(&db, root_dept.id)
            .await
            .unwrap();
    assert_eq!(child_depts.len(), 1);
    assert_eq!(child_depts[0].name, "Frontend");

    // Test find_root_departments_by_organization_id
    let root_depts = OrganizationDepartmentRepository::find_root_departments_by_organization_id(
        &db,
        organization_id,
    )
    .await
    .unwrap();
    assert_eq!(root_depts.len(), 2); // Engineering and Marketing

    // Test find_by_name_and_organization
    let found_by_name = OrganizationDepartmentRepository::find_by_name_and_organization(
        &db,
        "Marketing",
        organization_id,
        None,
    )
    .await
    .unwrap();
    assert!(found_by_name.is_some());
    assert_eq!(found_by_name.unwrap().id, another_dept.id);

    // Test find_by_manager_id
    if let Some(manager_id) = root_dept.manager_user_id {
        let managed_depts = OrganizationDepartmentRepository::find_by_manager_id(&db, manager_id)
            .await
            .unwrap();
        assert!(!managed_depts.is_empty());
    }

    // Test find_hierarchy_by_organization_id
    let hierarchy_depts =
        OrganizationDepartmentRepository::find_hierarchy_by_organization_id(&db, organization_id)
            .await
            .unwrap();
    assert_eq!(hierarchy_depts.len(), 3);

    // Test find_by_hierarchy_path_prefix
    let path_prefix_depts =
        OrganizationDepartmentRepository::find_by_hierarchy_path_prefix(&db, organization_id, "/")
            .await
            .unwrap();
    assert_eq!(path_prefix_depts.len(), 3); // All departments

    // Test count_by_organization_id
    let dept_count =
        OrganizationDepartmentRepository::count_by_organization_id(&db, organization_id)
            .await
            .unwrap();
    assert_eq!(dept_count, 3);

    // Test update_by_id
    let mut updated_dept = child_dept.clone().into_active_model();
    updated_dept.name = sea_orm::Set("Updated Frontend".to_string());
    let updated = OrganizationDepartmentRepository::update_by_id(&db, child_dept.id, updated_dept)
        .await
        .unwrap();
    assert_eq!(updated.name, "Updated Frontend");

    // Test update_hierarchy_paths_batch
    let batch_updates = vec![(child_dept.id, "/new/path".to_string(), 3)];
    OrganizationDepartmentRepository::update_hierarchy_paths_batch(&db, batch_updates)
        .await
        .unwrap();
    let updated_hierarchy = OrganizationDepartmentRepository::find_by_id(&db, child_dept.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_hierarchy.hierarchy_path, "/new/path");
    assert_eq!(updated_hierarchy.hierarchy_level, 3);

    // Test exists_circular_dependency
    let circular_exists = OrganizationDepartmentRepository::exists_circular_dependency(
        &db,
        root_dept.id,
        child_dept.id,
    )
    .await
    .unwrap();
    assert!(!circular_exists); // Child cannot be parent of root

    // Test delete_by_id (soft delete)
    OrganizationDepartmentRepository::delete_by_id(&db, another_dept.id)
        .await
        .unwrap();
    let deleted_check = OrganizationDepartmentRepository::find_by_id(&db, another_dept.id)
        .await
        .unwrap();
    // Department should still exist but be inactive
    assert!(deleted_check.is_some());
    assert!(!deleted_check.unwrap().is_active);
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_department_repository_additional_methods() {
    let db = setup_test_db().await;
    let organization_id = create_test_organization(&db).await;

    // Create departments with different names and hierarchy
    let dept1 = create_test_department(&db, "Development Team", organization_id, None).await;
    let dept2 = create_test_department(&db, "Quality Assurance", organization_id, None).await;
    let dept3 =
        create_test_department(&db, "DevOps Engineering", organization_id, Some(dept1.id)).await;

    // Test find_by_name_and_organization with parent
    let child_dept_search = OrganizationDepartmentRepository::find_by_name_and_organization(
        &db,
        "DevOps Engineering",
        organization_id,
        Some(dept1.id),
    )
    .await
    .unwrap();
    assert!(child_dept_search.is_some());
    assert_eq!(child_dept_search.unwrap().id, dept3.id);

    // Test find_by_name_and_organization without parent (should not find DevOps)
    let root_search = OrganizationDepartmentRepository::find_by_name_and_organization(
        &db,
        "DevOps Engineering",
        organization_id,
        None,
    )
    .await
    .unwrap();
    assert!(root_search.is_none());

    // Test circular dependency check
    let circular_check =
        OrganizationDepartmentRepository::exists_circular_dependency(&db, dept1.id, dept3.id)
            .await
            .unwrap();
    assert!(!circular_check); // Child cannot be ancestor of parent

    // Test hierarchy path prefix search
    let hierarchy_descendants = OrganizationDepartmentRepository::find_by_hierarchy_path_prefix(
        &db,
        organization_id,
        &dept1.hierarchy_path,
    )
    .await
    .unwrap();
    assert!(hierarchy_descendants.len() >= 2); // Should include dept1 and its children

    // Clean up
    OrganizationDepartmentRepository::delete_by_id(&db, dept1.id)
        .await
        .unwrap();
    OrganizationDepartmentRepository::delete_by_id(&db, dept2.id)
        .await
        .unwrap();
    OrganizationDepartmentRepository::delete_by_id(&db, dept3.id)
        .await
        .unwrap();
}
