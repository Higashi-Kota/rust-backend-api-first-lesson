pub mod requests;
pub mod responses;

// Re-export specific items for backward compatibility
// Requests
pub use requests::{
    AttachmentFilterDto, AttachmentSortBy, CreateShareLinkRequest, GenerateDownloadUrlRequest,
    GenerateUploadUrlRequest, SortOrder,
};

// Responses
pub use responses::{
    AttachmentDto, AttachmentUploadResponse, CreateShareLinkResponse, GenerateDownloadUrlResponse,
    GenerateUploadUrlResponse, ShareLinkDto, ShareLinkListResponse,
};
