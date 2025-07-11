pub mod dto;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod services;
pub mod usecases;

// 後方互換性のための再エクスポート
// TODO: Phase 19で古い参照を削除後、#[allow(ambiguous_glob_reexports)]を削除
// #[allow(ambiguous_glob_reexports)]
// pub use dto::*;
// #[allow(ambiguous_glob_reexports)]
// pub use models::*;
// #[allow(ambiguous_glob_reexports)]
// pub use repositories::*;
// #[allow(ambiguous_glob_reexports)]
// pub use services::*;

// ハンドラーの再エクスポート
// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除
// Removed: organization_hierarchy_routes was deleted as unused
#[allow(unused_imports)]
pub use handlers::organization::organization_router_with_state;
