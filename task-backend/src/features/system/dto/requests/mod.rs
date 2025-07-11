use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsQueryParams {
    pub include_cpu: Option<bool>,
    pub include_memory: Option<bool>,
    pub include_disk: Option<bool>,
    pub include_network: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogQueryParams {
    pub level: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
