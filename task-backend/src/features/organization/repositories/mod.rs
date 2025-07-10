pub mod analytics;
pub mod department;
pub mod department_member;
pub mod organization;

// Re-export repository types
// TODO: Phase 19で古い参照を削除後、以下の#[allow]を削除予定
#[allow(unused_imports)]
pub use analytics::AnalyticsRepository;
#[allow(unused_imports)]
pub use department::DepartmentRepository;
#[allow(unused_imports)]
pub use department_member::DepartmentMemberRepository;
#[allow(unused_imports)]
pub use organization::OrganizationRepository;
