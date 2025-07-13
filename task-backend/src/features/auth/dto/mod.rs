pub mod requests;
pub mod responses;

// Re-export specific items for backward compatibility
// Requests
pub use requests::{
    DeleteAccountRequest, EmailVerificationRequest, PasswordChangeRequest, PasswordResetRequest,
    PasswordResetRequestRequest, RefreshTokenRequest, ResendVerificationEmailRequest,
    SigninRequest, SignupRequest,
};

// Responses
pub use responses::{
    AccountDeletionResponse, AuthResponse, AuthStatusResponse, CurrentUserResponse,
    EmailVerificationResponse, LogoutResponse, PasswordChangeResponse,
    PasswordResetRequestResponse, PasswordResetResponse, ResendVerificationEmailResponse,
    TokenRefreshResponse,
};
