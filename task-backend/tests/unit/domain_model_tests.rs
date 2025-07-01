// tests/unit/domain_model_tests.rs

use task_backend::domain::organization_model::{
    Model as OrganizationModel, Organization, OrganizationSettings,
};
use task_backend::domain::subscription_tier::SubscriptionTier;
use uuid::Uuid;

#[tokio::test]
async fn test_organization_model_new_logic() {
    // Model::newメソッドのロジックテスト
    let _name = "Test Organization".to_string();
    let owner_id = Uuid::new_v4();
    let description = Some("Test description".to_string());
    let settings = OrganizationSettings {
        allow_public_teams: true,
        require_approval_for_new_members: false,
        enable_single_sign_on: true,
        default_team_subscription_tier: SubscriptionTier::Pro,
    };

    let mut org = Organization::new(
        "Test Org".to_string(),
        description.clone(),
        owner_id,
        SubscriptionTier::Pro,
    );
    org.settings = settings;

    let model = OrganizationModel::from_organization(&org);

    assert_eq!(model.name, "Test Org");
    assert_eq!(model.owner_id, owner_id);
    assert_eq!(model.subscription_tier, "pro");
    assert_eq!(model.description, description);
    assert_eq!(model.max_teams, 20);
    assert_eq!(model.max_members, 100);
    assert!(model.id != Uuid::nil());

    let retrieved_settings = model.get_settings().unwrap();
    assert!(retrieved_settings.allow_public_teams);
    assert!(!retrieved_settings.require_approval_for_new_members);
    assert!(retrieved_settings.enable_single_sign_on);
    assert_eq!(
        retrieved_settings.default_team_subscription_tier,
        SubscriptionTier::Pro
    );
}

#[tokio::test]
async fn test_organization_model_get_subscription_tier_logic() {
    // get_subscription_tierメソッドのロジックテスト
    let owner_id = Uuid::new_v4();

    let free_org_struct = Organization::new(
        "Free Org".to_string(),
        None,
        owner_id,
        SubscriptionTier::Free,
    );
    let free_org = OrganizationModel::from_organization(&free_org_struct);

    let pro_org_struct =
        Organization::new("Pro Org".to_string(), None, owner_id, SubscriptionTier::Pro);
    let pro_org = OrganizationModel::from_organization(&pro_org_struct);

    let enterprise_org_struct = Organization::new(
        "Enterprise Org".to_string(),
        None,
        owner_id,
        SubscriptionTier::Enterprise,
    );
    let enterprise_org = OrganizationModel::from_organization(&enterprise_org_struct);

    assert_eq!(free_org.get_subscription_tier(), SubscriptionTier::Free);
    assert_eq!(pro_org.get_subscription_tier(), SubscriptionTier::Pro);
    assert_eq!(
        enterprise_org.get_subscription_tier(),
        SubscriptionTier::Enterprise
    );
}

#[tokio::test]
async fn test_organization_model_get_settings_logic() {
    // get_settingsメソッドのロジックテスト
    let owner_id = Uuid::new_v4();

    let custom_settings = OrganizationSettings {
        allow_public_teams: false,
        require_approval_for_new_members: true,
        enable_single_sign_on: false,
        default_team_subscription_tier: SubscriptionTier::Enterprise,
    };

    let mut org_struct = Organization::new(
        "Test Org".to_string(),
        None,
        owner_id,
        SubscriptionTier::Enterprise,
    );
    org_struct.settings = custom_settings.clone();
    let org = OrganizationModel::from_organization(&org_struct);

    let retrieved_settings = org.get_settings().unwrap();
    assert!(!retrieved_settings.allow_public_teams);
    assert!(retrieved_settings.require_approval_for_new_members);
    assert!(!retrieved_settings.enable_single_sign_on);
    assert_eq!(
        retrieved_settings.default_team_subscription_tier,
        SubscriptionTier::Enterprise
    );
}

#[tokio::test]
async fn test_organization_model_update_settings_logic() {
    // update_settingsメソッドのロジックテスト
    let owner_id = Uuid::new_v4();

    let org_struct = Organization::new(
        "Test Org".to_string(),
        None,
        owner_id,
        SubscriptionTier::Pro,
    );
    let mut org = OrganizationModel::from_organization(&org_struct);

    let original_updated_at = org.updated_at;

    // 少し時間を進める
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let new_settings = OrganizationSettings {
        allow_public_teams: true,
        require_approval_for_new_members: false,
        enable_single_sign_on: true,
        default_team_subscription_tier: SubscriptionTier::Enterprise,
    };

    org.update_settings(new_settings.clone());

    let retrieved_settings = org.get_settings().unwrap();
    assert!(retrieved_settings.allow_public_teams);
    assert!(!retrieved_settings.require_approval_for_new_members);
    assert!(retrieved_settings.enable_single_sign_on);
    assert_eq!(
        retrieved_settings.default_team_subscription_tier,
        SubscriptionTier::Enterprise
    );
    assert!(org.updated_at > original_updated_at);
}

#[tokio::test]
async fn test_organization_model_update_subscription_tier_logic() {
    // update_subscription_tierメソッドのロジックテスト
    let owner_id = Uuid::new_v4();

    let org_struct = Organization::new(
        "Test Org".to_string(),
        None,
        owner_id,
        SubscriptionTier::Free,
    );
    let mut org = OrganizationModel::from_organization(&org_struct);

    // Initial state - Free tier
    assert_eq!(org.get_subscription_tier(), SubscriptionTier::Free);
    assert_eq!(org.max_teams, 3);
    assert_eq!(org.max_members, 10);

    let original_updated_at = org.updated_at;

    // 少し時間を進める
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Upgrade to Pro
    org.update_subscription_tier(SubscriptionTier::Pro);

    assert_eq!(org.get_subscription_tier(), SubscriptionTier::Pro);
    assert_eq!(org.max_teams, 20);
    assert_eq!(org.max_members, 100);
    assert!(org.updated_at > original_updated_at);

    let pro_updated_at = org.updated_at;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Upgrade to Enterprise
    org.update_subscription_tier(SubscriptionTier::Enterprise);

    assert_eq!(org.get_subscription_tier(), SubscriptionTier::Enterprise);
    assert_eq!(org.max_teams, 100);
    assert_eq!(org.max_members, 1000);
    assert!(org.updated_at > pro_updated_at);
}

#[tokio::test]
async fn test_organization_model_can_add_teams_logic() {
    // can_add_teamsメソッドのロジックテスト
    let owner_id = Uuid::new_v4();

    let free_org_struct = Organization::new(
        "Free Org".to_string(),
        None,
        owner_id,
        SubscriptionTier::Free,
    );
    let free_org = OrganizationModel::from_organization(&free_org_struct);

    let pro_org_struct =
        Organization::new("Pro Org".to_string(), None, owner_id, SubscriptionTier::Pro);
    let pro_org = OrganizationModel::from_organization(&pro_org_struct);

    let enterprise_org_struct = Organization::new(
        "Enterprise Org".to_string(),
        None,
        owner_id,
        SubscriptionTier::Enterprise,
    );
    let enterprise_org = OrganizationModel::from_organization(&enterprise_org_struct);

    // Test Free tier limits (max 3 teams)
    assert!(free_org.can_add_teams(0));
    assert!(free_org.can_add_teams(2));
    assert!(!free_org.can_add_teams(3));
    assert!(!free_org.can_add_teams(5));

    // Test Pro tier limits (max 20 teams)
    assert!(pro_org.can_add_teams(10));
    assert!(pro_org.can_add_teams(19));
    assert!(!pro_org.can_add_teams(20));
    assert!(!pro_org.can_add_teams(25));

    // Test Enterprise tier limits (max 100 teams)
    assert!(enterprise_org.can_add_teams(50));
    assert!(enterprise_org.can_add_teams(99));
    assert!(!enterprise_org.can_add_teams(100));
    assert!(!enterprise_org.can_add_teams(150));
}

#[tokio::test]
async fn test_organization_model_can_add_members_logic() {
    // can_add_membersメソッドのロジックテスト
    let owner_id = Uuid::new_v4();

    let free_org_struct = Organization::new(
        "Free Org".to_string(),
        None,
        owner_id,
        SubscriptionTier::Free,
    );
    let free_org = OrganizationModel::from_organization(&free_org_struct);

    let pro_org_struct =
        Organization::new("Pro Org".to_string(), None, owner_id, SubscriptionTier::Pro);
    let pro_org = OrganizationModel::from_organization(&pro_org_struct);

    let enterprise_org_struct = Organization::new(
        "Enterprise Org".to_string(),
        None,
        owner_id,
        SubscriptionTier::Enterprise,
    );
    let enterprise_org = OrganizationModel::from_organization(&enterprise_org_struct);

    // Test Free tier limits (max 10 members)
    assert!(free_org.can_add_members(5));
    assert!(free_org.can_add_members(9));
    assert!(!free_org.can_add_members(10));
    assert!(!free_org.can_add_members(15));

    // Test Pro tier limits (max 100 members)
    assert!(pro_org.can_add_members(50));
    assert!(pro_org.can_add_members(99));
    assert!(!pro_org.can_add_members(100));
    assert!(!pro_org.can_add_members(150));

    // Test Enterprise tier limits (max 1000 members)
    assert!(enterprise_org.can_add_members(500));
    assert!(enterprise_org.can_add_members(999));
    assert!(!enterprise_org.can_add_members(1000));
    assert!(!enterprise_org.can_add_members(1500));
}

#[tokio::test]
async fn test_organization_settings_default_logic() {
    // OrganizationSettings::defaultのロジックテスト
    let default_settings = OrganizationSettings::default();

    assert!(!default_settings.allow_public_teams);
    assert!(default_settings.require_approval_for_new_members);
    assert!(!default_settings.enable_single_sign_on);
    assert_eq!(
        default_settings.default_team_subscription_tier,
        SubscriptionTier::Free
    );
}

#[tokio::test]
async fn test_organization_model_conversion_methods_logic() {
    // to_organization/from_organizationメソッドのロジックテスト
    let owner_id = Uuid::new_v4();

    let org_struct = Organization::new(
        "Test Org".to_string(),
        Some("Test description".to_string()),
        owner_id,
        SubscriptionTier::Pro,
    );
    let model = OrganizationModel::from_organization(&org_struct);

    // Test conversion to Organization struct
    let organization = model.to_organization().unwrap();
    assert_eq!(organization.id, model.id);
    assert_eq!(organization.name, model.name);
    assert_eq!(organization.description, model.description);
    assert_eq!(organization.owner_id, model.owner_id);
    assert_eq!(organization.subscription_tier, SubscriptionTier::Pro);
    assert_eq!(organization.max_teams, 20);
    assert_eq!(organization.max_members, 100);

    // Test conversion back to Model
    let converted_model = OrganizationModel::from_organization(&organization);
    assert_eq!(converted_model.id, organization.id);
    assert_eq!(converted_model.name, organization.name);
    assert_eq!(converted_model.description, organization.description);
    assert_eq!(converted_model.owner_id, organization.owner_id);
    assert_eq!(converted_model.subscription_tier, "pro");
    assert_eq!(converted_model.max_teams, 20);
    assert_eq!(converted_model.max_members, 100);
}
