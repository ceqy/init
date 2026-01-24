//! AuthService gRPC 实现

#[allow(clippy::all)]
mod proto {
    include!("cuba.iam.auth.rs");
}

pub use proto::auth_service_server::{AuthService, AuthServiceServer};
pub use proto::*;

use tonic::{Request, Response, Status};

/// AuthService 实现
#[derive(Debug, Default)]
pub struct AuthServiceImpl {}

impl AuthServiceImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(username = %req.username, tenant_id = %req.tenant_id, "Login attempt");

        // TODO: 实现真正的登录逻辑
        Ok(Response::new(LoginResponse {
            access_token: "mock_access_token".to_string(),
            refresh_token: "mock_refresh_token".to_string(),
            expires_in: 3600,
            token_type: "Bearer".to_string(),
            user: Some(User {
                id: "user_001".to_string(),
                username: req.username,
                email: "user@example.com".to_string(),
                display_name: "Test User".to_string(),
                phone: "".to_string(),
                avatar_url: "".to_string(),
                tenant_id: req.tenant_id,
                role_ids: vec!["admin".to_string()],
                status: "active".to_string(),
                language: "zh-CN".to_string(),
                timezone: "Asia/Shanghai".to_string(),
                two_factor_enabled: false,
                last_login_at: None,
                audit_info: None,
            }),
            require_2fa: false,
            session_id: "".to_string(),
        }))
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(logout_all = %req.logout_all_devices, "Logout request");

        Ok(Response::new(LogoutResponse { success: true }))
    }

    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<RefreshTokenResponse>, Status> {
        let _req = request.into_inner();
        tracing::info!("Refresh token request");

        Ok(Response::new(RefreshTokenResponse {
            access_token: "new_mock_access_token".to_string(),
            refresh_token: "new_mock_refresh_token".to_string(),
            expires_in: 3600,
        }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let _req = request.into_inner();
        tracing::info!("Validate token request");

        Ok(Response::new(ValidateTokenResponse {
            valid: true,
            user_id: "user_001".to_string(),
            tenant_id: "tenant_001".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            expires_at: None,
        }))
    }

    async fn change_password(
        &self,
        request: Request<ChangePasswordRequest>,
    ) -> Result<Response<ChangePasswordResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, "Change password request");

        Ok(Response::new(ChangePasswordResponse {
            success: true,
            message: "Password changed successfully".to_string(),
        }))
    }

    async fn request_password_reset(
        &self,
        request: Request<RequestPasswordResetRequest>,
    ) -> Result<Response<RequestPasswordResetResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(email = %req.email, "Request password reset");

        Ok(Response::new(RequestPasswordResetResponse {
            success: true,
            message: "Password reset email sent".to_string(),
        }))
    }

    async fn reset_password(
        &self,
        request: Request<ResetPasswordRequest>,
    ) -> Result<Response<ResetPasswordResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(email = %req.email, "Reset password");

        Ok(Response::new(ResetPasswordResponse {
            success: true,
            message: "Password reset successfully".to_string(),
        }))
    }

    async fn get_current_user(
        &self,
        request: Request<GetCurrentUserRequest>,
    ) -> Result<Response<GetCurrentUserResponse>, Status> {
        let _req = request.into_inner();
        tracing::info!("Get current user request");

        Ok(Response::new(GetCurrentUserResponse {
            user: Some(User {
                id: "user_001".to_string(),
                username: "testuser".to_string(),
                email: "user@example.com".to_string(),
                display_name: "Test User".to_string(),
                phone: "".to_string(),
                avatar_url: "".to_string(),
                tenant_id: "tenant_001".to_string(),
                role_ids: vec!["admin".to_string()],
                status: "active".to_string(),
                language: "zh-CN".to_string(),
                timezone: "Asia/Shanghai".to_string(),
                two_factor_enabled: false,
                last_login_at: None,
                audit_info: None,
            }),
        }))
    }

    async fn update_profile(
        &self,
        request: Request<UpdateProfileRequest>,
    ) -> Result<Response<UpdateProfileResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, "Update profile request");

        Ok(Response::new(UpdateProfileResponse {
            user: Some(User {
                id: req.user_id,
                username: "testuser".to_string(),
                email: req.email,
                display_name: req.display_name,
                phone: req.phone,
                avatar_url: req.avatar_url,
                tenant_id: "tenant_001".to_string(),
                role_ids: vec!["admin".to_string()],
                status: "active".to_string(),
                language: req.language,
                timezone: req.timezone,
                two_factor_enabled: false,
                last_login_at: None,
                audit_info: None,
            }),
        }))
    }

    async fn enable2_fa(
        &self,
        request: Request<Enable2FaRequest>,
    ) -> Result<Response<Enable2FaResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, method = %req.method, "Enable 2FA request");

        Ok(Response::new(Enable2FaResponse {
            secret: "MOCK_SECRET_KEY".to_string(),
            qr_code_url: "otpauth://totp/Example:user@example.com?secret=MOCK_SECRET_KEY&issuer=Example".to_string(),
            backup_codes: vec![
                "12345678".to_string(),
                "23456789".to_string(),
                "34567890".to_string(),
            ],
        }))
    }

    async fn disable2_fa(
        &self,
        request: Request<Disable2FaRequest>,
    ) -> Result<Response<Disable2FaResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, "Disable 2FA request");

        Ok(Response::new(Disable2FaResponse { success: true }))
    }

    async fn verify2_fa(
        &self,
        request: Request<Verify2FaRequest>,
    ) -> Result<Response<Verify2FaResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, "Verify 2FA request");

        Ok(Response::new(Verify2FaResponse {
            success: true,
            access_token: "mock_access_token_after_2fa".to_string(),
            refresh_token: "mock_refresh_token_after_2fa".to_string(),
        }))
    }

    async fn get_active_sessions(
        &self,
        request: Request<GetActiveSessionsRequest>,
    ) -> Result<Response<GetActiveSessionsResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, "Get active sessions request");

        Ok(Response::new(GetActiveSessionsResponse {
            sessions: vec![Session {
                id: "session_001".to_string(),
                user_id: req.user_id,
                device_info: "Chrome on macOS".to_string(),
                ip_address: "127.0.0.1".to_string(),
                user_agent: "Mozilla/5.0".to_string(),
                is_current: true,
                created_at: None,
                expires_at: None,
                last_activity_at: None,
            }],
        }))
    }

    async fn revoke_session(
        &self,
        request: Request<RevokeSessionRequest>,
    ) -> Result<Response<RevokeSessionResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, session_id = %req.session_id, "Revoke session request");

        Ok(Response::new(RevokeSessionResponse { success: true }))
    }
}
