//! Admin DTOs module
//! 
//! 管理者向けのリクエスト/レスポンスDTO定義

pub mod requests;
pub mod responses;

// Re-export for convenience
// TODO: Phase 19で古い参照を削除後、#[allow]を削除予定
#[allow(unused_imports)]
pub use requests::*;
#[allow(unused_imports)]
pub use responses::*;