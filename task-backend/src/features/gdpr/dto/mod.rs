pub mod requests;
pub mod responses;

// Re-export specific items for backward compatibility
// Requests
pub use requests::{
    ConsentUpdateRequest, DataDeletionRequest, DataExportRequest, SingleConsentUpdateRequest,
};

// Responses
pub use responses::{
    ComplianceStatusResponse, ConsentHistoryEntry, ConsentHistoryResponse, ConsentStatus,
    ConsentStatusResponse, DataDeletionResponse, DataExportResponse, DeletedRecordsSummary,
    SubscriptionHistoryExport, TaskDataExport, TeamDataExport, UserDataExport,
};
