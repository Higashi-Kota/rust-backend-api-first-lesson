// tests/unit/organization_hierarchy/repository/department_member_repository_tests.rs

use crate::common::db::setup_test_db;
use sea_orm::*;
use task_backend::domain::department_member_model::{ActiveModel, DepartmentRole, Model};
use task_backend::repository::department_member_repository::DepartmentMemberRepository;
use uuid::Uuid;

async fn create_test_user(db: &DatabaseConnection) -> Uuid {
    use sea_orm::prelude::*;
    use task_backend::domain::user_model::ActiveModel as UserActiveModel;

    let user_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

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

async fn create_test_department(db: &DatabaseConnection, organization_id: Uuid) -> Uuid {
    use task_backend::domain::organization_department_model::ActiveModel as DeptActiveModel;
    use task_backend::repository::organization_department_repository::OrganizationDepartmentRepository;

    let dept_id = Uuid::new_v4();
    let active_model = DeptActiveModel {
        id: Set(dept_id),
        name: Set("Test Department".to_string()),
        description: Set(Some("Test department".to_string())),
        organization_id: Set(organization_id),
        parent_department_id: Set(None),
        hierarchy_level: Set(0),
        hierarchy_path: Set(format!("/{}", dept_id)),
        manager_user_id: Set(None),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    OrganizationDepartmentRepository::create(db, active_model)
        .await
        .unwrap();
    dept_id
}

async fn create_test_role2(db: &DatabaseConnection) -> Uuid {
    use sea_orm::prelude::*;
    use task_backend::domain::role_model::ActiveModel as RoleActiveModel;

    let role_id = Uuid::new_v4();
    let active_model = RoleActiveModel {
        id: Set(role_id),
        name: Set(format!("test_role_{}", role_id)),
        display_name: Set("Test Role".to_string()),
        description: Set(Some("Test role".to_string())),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    active_model.insert(db).await.unwrap();
    role_id
}

async fn create_test_user2(db: &DatabaseConnection) -> Uuid {
    use sea_orm::prelude::*;
    use task_backend::domain::user_model::ActiveModel as UserActiveModel;

    let user_id = Uuid::new_v4();
    let role_id = create_test_role2(db).await;

    let active_model = UserActiveModel {
        id: Set(user_id),
        username: Set(format!("testuser_{}", user_id)),
        email: Set(format!("test_{}@example.com", user_id)),
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

async fn create_test_member(
    db: &DatabaseConnection,
    department_id: Uuid,
    user_id: Uuid,
    role: DepartmentRole,
    added_by: Uuid,
) -> Model {
    let active_model = ActiveModel {
        id: Set(Uuid::new_v4()),
        department_id: Set(department_id),
        user_id: Set(user_id),
        role: Set(role.to_string()),
        is_active: Set(true),
        joined_at: Set(chrono::Utc::now()),
        added_by: Set(added_by),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    DepartmentMemberRepository::create(db, active_model)
        .await
        .unwrap()
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_department_member_creation_logic() {
    let db = setup_test_db().await;

    let organization_id = create_test_organization(&db).await;
    let department_id = create_test_department(&db, organization_id).await;
    let user_id = create_test_user2(&db).await;
    let added_by = create_test_user2(&db).await;

    let member = create_test_member(
        &db,
        department_id,
        user_id,
        DepartmentRole::Member,
        added_by,
    )
    .await;

    // 実際のロジックテスト: メンバー作成の検証
    assert_eq!(member.department_id, department_id);
    assert_eq!(member.user_id, user_id);
    assert_eq!(member.role, "member");
    assert_eq!(member.added_by, added_by);
    assert!(member.is_active);
}

#[tokio::test]
async fn test_department_role_string_conversion_logic() {
    // DepartmentRoleの文字列変換ロジックテスト
    assert_eq!(DepartmentRole::Manager.to_string(), "manager");
    assert_eq!(DepartmentRole::Lead.to_string(), "lead");
    assert_eq!(DepartmentRole::Member.to_string(), "member");
    assert_eq!(DepartmentRole::Viewer.to_string(), "viewer");
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_department_member_find_logic() {
    let db = setup_test_db().await;

    let organization_id = create_test_organization(&db).await;
    let department_id = create_test_department(&db, organization_id).await;
    let user_id = create_test_user2(&db).await;
    let added_by = create_test_user2(&db).await;

    let created_member =
        create_test_member(&db, department_id, user_id, DepartmentRole::Lead, added_by).await;

    let found_member = DepartmentMemberRepository::find_by_id(&db, created_member.id)
        .await
        .unwrap();

    // 検索ロジックの検証
    assert!(found_member.is_some());
    let member = found_member.unwrap();
    assert_eq!(member.id, created_member.id);
    assert_eq!(member.role, "lead");
}

#[tokio::test]
async fn test_department_role_from_str_logic() {
    // from_strメソッドのロジックテスト
    assert!(DepartmentRole::from_str("manager").is_ok());
    assert!(DepartmentRole::from_str("lead").is_ok());
    assert!(DepartmentRole::from_str("member").is_ok());
    assert!(DepartmentRole::from_str("viewer").is_ok());
    assert!(DepartmentRole::from_str("invalid").is_err());
}

#[tokio::test]
async fn test_department_role_permissions_logic() {
    // 権限チェックのロジックテスト
    assert!(DepartmentRole::Manager.has_management_permissions());
    assert!(DepartmentRole::Lead.has_management_permissions());
    assert!(!DepartmentRole::Member.has_management_permissions());
    assert!(!DepartmentRole::Viewer.has_management_permissions());

    assert!(DepartmentRole::Manager.can_modify_members());
    assert!(!DepartmentRole::Lead.can_modify_members());
    assert!(!DepartmentRole::Member.can_modify_members());
    assert!(!DepartmentRole::Viewer.can_modify_members());

    assert!(DepartmentRole::Manager.can_view_analytics());
    assert!(DepartmentRole::Lead.can_view_analytics());
    assert!(DepartmentRole::Member.can_view_analytics());
    assert!(!DepartmentRole::Viewer.can_view_analytics());
}

#[tokio::test]
async fn test_department_role_permission_levels_logic() {
    // get_permission_levelメソッドのロジックテスト
    assert_eq!(DepartmentRole::Manager.get_permission_level(), 4);
    assert_eq!(DepartmentRole::Lead.get_permission_level(), 3);
    assert_eq!(DepartmentRole::Member.get_permission_level(), 2);
    assert_eq!(DepartmentRole::Viewer.get_permission_level(), 1);
}

#[tokio::test]
async fn test_department_member_model_new_logic() {
    // Model::newメソッドのロジックテスト
    let department_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let added_by = Uuid::new_v4();
    let role = DepartmentRole::Lead;

    let member = Model::new(department_id, user_id, role.clone(), added_by);

    assert_eq!(member.department_id, department_id);
    assert_eq!(member.user_id, user_id);
    assert_eq!(member.role, role.to_string());
    assert_eq!(member.added_by, added_by);
    assert!(member.is_active);
    assert_eq!(member.get_role(), role);
}

#[tokio::test]
async fn test_department_member_model_update_role_logic() {
    // update_roleメソッドのロジックテスト
    let department_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let added_by = Uuid::new_v4();

    let mut member = Model::new(department_id, user_id, DepartmentRole::Member, added_by);
    let original_updated_at = member.updated_at;

    // 少し時間を進めるためにwait
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    member.update_role(DepartmentRole::Lead);

    assert_eq!(member.role, "lead");
    assert_eq!(member.get_role(), DepartmentRole::Lead);
    assert!(member.updated_at > original_updated_at);
}

#[tokio::test]
async fn test_department_member_model_activate_deactivate_logic() {
    // activate/deactivateメソッドのロジックテスト
    let department_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let added_by = Uuid::new_v4();

    let mut member = Model::new(department_id, user_id, DepartmentRole::Member, added_by);
    let original_updated_at = member.updated_at;

    // 少し時間を進めるためにwait
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Deactivate
    member.deactivate();
    assert!(!member.is_active);
    assert!(member.updated_at > original_updated_at);

    let deactivate_time = member.updated_at;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Activate
    member.activate();
    assert!(member.is_active);
    assert!(member.updated_at > deactivate_time);
}

#[tokio::test]
async fn test_department_member_model_permissions_logic() {
    // Modelの権限チェックメソッドのロジックテスト
    let department_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let added_by = Uuid::new_v4();

    let manager = Model::new(department_id, user_id, DepartmentRole::Manager, added_by);
    let lead = Model::new(department_id, user_id, DepartmentRole::Lead, added_by);
    let member = Model::new(department_id, user_id, DepartmentRole::Member, added_by);
    let viewer = Model::new(department_id, user_id, DepartmentRole::Viewer, added_by);

    // Management permissions
    assert!(manager.has_management_permissions());
    assert!(lead.has_management_permissions());
    assert!(!member.has_management_permissions());
    assert!(!viewer.has_management_permissions());

    // Member modification permissions
    assert!(manager.can_modify_members());
    assert!(!lead.can_modify_members());
    assert!(!member.can_modify_members());
    assert!(!viewer.can_modify_members());

    // Analytics viewing permissions
    assert!(manager.can_view_analytics());
    assert!(lead.can_view_analytics());
    assert!(member.can_view_analytics());
    assert!(!viewer.can_view_analytics());
}

#[tokio::test]
#[ignore] // Ignore due to database connection timeout issues and external table dependencies
async fn test_department_member_repository_comprehensive() {
    let db = setup_test_db().await;

    let organization_id = create_test_organization(&db).await;
    let department_id = create_test_department(&db, organization_id).await;
    let user1_id = create_test_user2(&db).await;
    let user2_id = create_test_user2(&db).await;
    let added_by = create_test_user2(&db).await;

    // Test all repository functions
    let member1 = create_test_member(
        &db,
        department_id,
        user1_id,
        DepartmentRole::Manager,
        added_by,
    )
    .await;
    let _member2 =
        create_test_member(&db, department_id, user2_id, DepartmentRole::Lead, added_by).await;

    // Test find_by_department_id
    let dept_members = DepartmentMemberRepository::find_by_department_id(&db, department_id)
        .await
        .unwrap();
    assert_eq!(dept_members.len(), 2);

    // Test find_by_user_id
    let user_memberships = DepartmentMemberRepository::find_by_user_id(&db, user1_id)
        .await
        .unwrap();
    assert_eq!(user_memberships.len(), 1);

    // Test find_by_department_and_user
    let specific_member =
        DepartmentMemberRepository::find_by_department_and_user(&db, department_id, user1_id)
            .await
            .unwrap();
    assert!(specific_member.is_some());

    // Test find_by_department_and_role
    let managers = DepartmentMemberRepository::find_by_department_and_role(
        &db,
        department_id,
        DepartmentRole::Manager,
    )
    .await
    .unwrap();
    assert_eq!(managers.len(), 1);

    // Test find_managers_by_department_ids
    let all_managers =
        DepartmentMemberRepository::find_managers_by_department_ids(&db, vec![department_id])
            .await
            .unwrap();
    assert_eq!(all_managers.len(), 1);

    // Test find_by_added_by
    let added_members = DepartmentMemberRepository::find_by_added_by(&db, added_by, Some(10))
        .await
        .unwrap();
    assert_eq!(added_members.len(), 2);

    // Test update_role_by_department_and_user
    let updated_member = DepartmentMemberRepository::update_role_by_department_and_user(
        &db,
        department_id,
        user2_id,
        DepartmentRole::Manager,
    )
    .await
    .unwrap();
    assert_eq!(updated_member.role, "manager");

    // Test deactivate_by_id
    DepartmentMemberRepository::deactivate_by_id(&db, member1.id)
        .await
        .unwrap();
    let deactivated_member = DepartmentMemberRepository::find_by_id(&db, member1.id)
        .await
        .unwrap()
        .unwrap();
    assert!(!deactivated_member.is_active);

    // Test activate_by_department_and_user
    let reactivated =
        DepartmentMemberRepository::activate_by_department_and_user(&db, department_id, user1_id)
            .await
            .unwrap();
    assert!(reactivated.is_some());
    assert!(reactivated.unwrap().is_active);

    // Test count_by_department_id
    let dept_count = DepartmentMemberRepository::count_by_department_id(&db, department_id)
        .await
        .unwrap();
    assert_eq!(dept_count, 2);

    // Test count_by_user_id
    let user_count = DepartmentMemberRepository::count_by_user_id(&db, user1_id)
        .await
        .unwrap();
    assert_eq!(user_count, 1);

    // Test is_member_of_department
    let is_member =
        DepartmentMemberRepository::is_member_of_department(&db, user1_id, department_id)
            .await
            .unwrap();
    assert!(is_member);

    // Test user_has_role_in_department
    let has_manager_role = DepartmentMemberRepository::user_has_role_in_department(
        &db,
        user2_id,
        department_id,
        DepartmentRole::Manager,
    )
    .await
    .unwrap();
    assert!(has_manager_role);

    // Test get_user_role_in_department
    let user_role =
        DepartmentMemberRepository::get_user_role_in_department(&db, user2_id, department_id)
            .await
            .unwrap();
    assert_eq!(user_role, Some(DepartmentRole::Manager));

    // Test find_all_user_departments_with_roles
    let user_dept_roles =
        DepartmentMemberRepository::find_all_user_departments_with_roles(&db, user1_id)
            .await
            .unwrap();
    assert_eq!(user_dept_roles.len(), 1);
    assert_eq!(user_dept_roles[0].0, department_id);
}
