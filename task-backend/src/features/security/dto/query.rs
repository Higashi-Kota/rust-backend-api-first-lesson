// task-backend/src/features/security/dto/query.rs

use serde::Deserialize;

/// 権限検索パラメータ
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Used in query extraction by axum handlers
pub struct PermissionQuery {
    pub resource: Option<String>,
    pub action: Option<String>,
}

/// 機能検索パラメータ
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Used in query extraction by axum handlers
pub struct FeatureQuery {
    pub category: Option<String>,
}
