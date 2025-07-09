// task-backend/src/api/dto/common.rs

// ページネーション関連の型を shared::types::pagination から再エクスポート
#[allow(unused_imports)]
pub use crate::shared::types::pagination::{PaginatedResponse, PaginationMeta, PaginationQuery};

// 共通型を shared::types::common から再エクスポート
#[allow(unused_imports)]
pub use crate::shared::types::common::{ApiResponse, OperationResult};
