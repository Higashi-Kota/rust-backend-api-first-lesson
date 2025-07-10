//! Admin usecases module
//! 
//! 管理者向けの複雑なビジネスロジックを実装

pub mod organization_management;
pub mod analytics_operations;
pub mod user_management;
pub mod subscription_management;

// Re-export for convenience
// TODO: Phase 19で古い参照を削除後、#[allow]を削除予定
#[allow(unused_imports)]
pub use organization_management::OrganizationManagementUseCase;
#[allow(unused_imports)]
pub use analytics_operations::AnalyticsOperationsUseCase;
#[allow(unused_imports)]
pub use user_management::UserManagementUseCase;
#[allow(unused_imports)]
pub use subscription_management::SubscriptionManagementUseCase;