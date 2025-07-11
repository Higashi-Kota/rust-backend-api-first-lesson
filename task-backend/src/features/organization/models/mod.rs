pub mod analytics;
pub mod department;
pub mod department_member;
pub mod organization;
pub mod organization_analytics;
// pub mod organization_department; // Commented out - using department instead

// Re-export commonly used types
// pub use analytics::{AnalyticsType, MetricValue, Period};
// pub use department_member::DepartmentRole;
// TODO: Phase 19で使用箇所が移行されたら#[allow(unused_imports)]を削除
#[allow(unused_imports)]
pub use organization::{Organization, OrganizationMember, OrganizationRole, OrganizationSettings};

// TODO: Phase 19で古い参照を削除後、以下の#[allow]を削除予定
#[allow(unused_imports)]
pub use analytics::Entity as AnalyticsEntity;
#[allow(unused_imports)]
pub use department::Entity as DepartmentEntity;
#[allow(unused_imports)]
pub use department_member::Entity as DepartmentMemberEntity;
#[allow(unused_imports)]
pub use organization::Entity as OrganizationEntity;
