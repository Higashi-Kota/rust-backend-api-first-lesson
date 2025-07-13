// task-backend/src/features/user/dto/mod.rs

pub mod requests;
pub mod responses;

// Re-export all request DTOs
pub use requests::{
    BulkUserOperation, BulkUserOperationsRequest, ResendVerificationEmailRequest,
    SubscriptionQuery, UpdateAccountStatusRequest, UpdateEmailRequest, UpdateProfileRequest,
    UpdateUserSettingsRequest, UpdateUsernameRequest, UserSearchQuery, VerifyEmailRequest,
};

// Re-export all response DTOs
pub use responses::{
    AccountStatusUpdateResponse, BulkOperationResponse, BulkOperationResult,
    EmailVerificationHistoryItem, EmailVerificationHistoryResponse, EmailVerificationResponse,
    NotificationSettings, PendingEmailVerificationResponse, ProfileUpdateResponse, RoleUserStats,
    SecuritySettings, SubscriptionAnalytics, SubscriptionAnalyticsResponse, TokenStatusResponse,
    UserActivityStatsResponse, UserAdditionalInfo, UserAnalyticsResponse, UserListResponse,
    UserPermissionsResponse, UserPreferences, UserProfileResponse, UserSettingsResponse,
    UserStatsResponse, UserSummary, UserWithRoleResponse,
};
