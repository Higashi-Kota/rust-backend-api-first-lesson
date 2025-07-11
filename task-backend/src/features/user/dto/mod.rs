// task-backend/src/features/user/dto/mod.rs

pub mod requests;
pub mod responses;

// Re-export all request DTOs
pub use requests::{
    BulkUserOperation, BulkUserOperationsRequest, ResendVerificationEmailRequest, SortOrder,
    SubscriptionQuery, UpdateAccountStatusRequest, UpdateEmailRequest, UpdateProfileRequest,
    UpdateUserSettingsRequest, UpdateUsernameRequest, UserSearchQuery, UserSortField,
    VerifyEmailRequest,
};

// Re-export all response DTOs
pub use responses::{
    AccountRestriction, AccountStatus, AccountStatusUpdateResponse, BulkOperationResponse,
    BulkOperationResult, EmailVerificationHistoryItem, EmailVerificationHistoryResponse,
    EmailVerificationResponse, NotificationSettings, PendingEmailVerificationResponse,
    ProfileUpdateResponse, RestrictionType, RoleUserStats, SecuritySettings, SubscriptionAnalytics,
    SubscriptionAnalyticsResponse, TokenStatusResponse, UserActivityStats,
    UserActivityStatsResponse, UserAdditionalInfo, UserAnalyticsResponse, UserListResponse,
    UserPermissionsResponse, UserPreferences, UserProfileResponse, UserSettingsDto,
    UserSettingsResponse, UserStatsResponse, UserSummary, UserWithRoleResponse,
    UsersByLanguageResponse, UsersWithNotificationResponse,
};

// Re-export test helpers for tests
#[cfg(test)]
pub use requests::test_helpers;
