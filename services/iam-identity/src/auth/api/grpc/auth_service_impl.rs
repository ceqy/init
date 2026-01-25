//! AuthService gRPC 实现

#[allow(clippy::all)]
mod proto {
    include!("cuba.iam.auth.rs");
}

pub use proto::auth_service_server::{AuthService, AuthServiceServer};
pub use proto::*;

use std::sync::Arc;

use chrono::{Duration, Utc};
use cuba_auth_core::TokenService;
use cuba_common::{TenantId, UserId};
use prost_types::Timestamp;
use sha2::{Digest, Sha256};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::auth::domain::entities::{PasswordResetToken, Session as DomainSession, SessionId};
use crate::auth::domain::repositories::{
    BackupCodeRepository, PasswordResetRepository, SessionRepository,
};
use crate::auth::domain::services::{BackupCodeService, TotpService};
use crate::auth::infrastructure::cache::AuthCache;
use crate::shared::domain::repositories::UserRepository;
use crate::shared::domain::value_objects::{Email, HashedPassword, Username};
use cuba_adapter_email::EmailSender;
use cuba_config::PasswordResetConfig;

/// AuthService 实现
pub struct AuthServiceImpl {
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn SessionRepository>,
    backup_code_repo: Arc<dyn BackupCodeRepository>,
    password_reset_repo: Arc<dyn PasswordResetRepository>,
    token_service: Arc<TokenService>,
    totp_service: Arc<TotpService>,
    email_sender: Arc<dyn EmailSender>,
    auth_cache: Arc<dyn AuthCache>,
    refresh_token_expires_in: i64,
    password_reset_config: PasswordResetConfig,
}

impl AuthServiceImpl {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        session_repo: Arc<dyn SessionRepository>,
        backup_code_repo: Arc<dyn BackupCodeRepository>,
        password_reset_repo: Arc<dyn PasswordResetRepository>,
        token_service: Arc<TokenService>,
        totp_service: Arc<TotpService>,
        email_sender: Arc<dyn EmailSender>,
        auth_cache: Arc<dyn AuthCache>,
        refresh_token_expires_in: i64,
        password_reset_config: PasswordResetConfig,
    ) -> Self {
        Self {
            user_repo,
            session_repo,
            backup_code_repo,
            password_reset_repo,
            token_service,
            totp_service,
            email_sender,
            auth_cache,
            refresh_token_expires_in,
            password_reset_config,
        }
    }
}

/// 计算 SHA256 哈希
fn sha256_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// 将 Domain User 转换为 Proto User
fn user_to_proto(user: &crate::shared::domain::entities::User) -> proto::User {
    proto::User {
        id: user.id.0.to_string(),
        username: user.username.as_str().to_string(),
        email: user.email.as_str().to_string(),
        display_name: user.display_name.clone().unwrap_or_default(),
        phone: user.phone.clone().unwrap_or_default(),
        avatar_url: user.avatar_url.clone().unwrap_or_default(),
        tenant_id: user.tenant_id.0.to_string(),
        role_ids: user.role_ids.clone(),
        status: format!("{:?}", user.status),
        language: user.language.clone(),
        timezone: user.timezone.clone(),
        two_factor_enabled: user.two_factor_enabled,
        last_login_at: user.last_login_at.map(|dt| Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }),
        audit_info: Some(proto::AuditInfo {
            created_at: Some(Timestamp {
                seconds: user.audit_info.created_at.timestamp(),
                nanos: user.audit_info.created_at.timestamp_subsec_nanos() as i32,
            }),
            created_by: user
                .audit_info
                .created_by
                .as_ref()
                .map(|u| u.0.to_string())
                .unwrap_or_default(),
            updated_at: Some(Timestamp {
                seconds: user.audit_info.updated_at.timestamp(),
                nanos: user.audit_info.updated_at.timestamp_subsec_nanos() as i32,
            }),
            updated_by: user
                .audit_info
                .updated_by
                .as_ref()
                .map(|u| u.0.to_string())
                .unwrap_or_default(),
        }),
    }
}

/// 将 Domain Session 转换为 Proto Session
fn session_to_proto(session: &DomainSession, is_current: bool) -> proto::Session {
    proto::Session {
        id: session.id.0.to_string(),
        user_id: session.user_id.0.to_string(),
        device_info: session.device_info.clone().unwrap_or_default(),
        ip_address: session.ip_address.clone().unwrap_or_default(),
        user_agent: session.user_agent.clone().unwrap_or_default(),
        is_current,
        created_at: Some(Timestamp {
            seconds: session.created_at.timestamp(),
            nanos: session.created_at.timestamp_subsec_nanos() as i32,
        }),
        expires_at: Some(Timestamp {
            seconds: session.expires_at.timestamp(),
            nanos: session.expires_at.timestamp_subsec_nanos() as i32,
        }),
        last_activity_at: Some(Timestamp {
            seconds: session.last_activity_at.timestamp(),
            nanos: session.last_activity_at.timestamp_subsec_nanos() as i32,
        }),
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

        // 1. 解析租户 ID
        let tenant_id = TenantId::from_uuid(
            Uuid::parse_str(&req.tenant_id)
                .map_err(|_| Status::invalid_argument("Invalid tenant ID"))?,
        );

        // 2. 构建用户名值对象
        let username = Username::new(&req.username)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // 3. 查找用户
        let user = self
            .user_repo
            .find_by_username(&username, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::unauthenticated("Invalid credentials"))?;

        // 4. 验证密码
        let valid = user
            .password_hash
            .verify(&req.password)
            .map_err(|e| Status::internal(e.to_string()))?;

        if !valid {
            tracing::warn!(username = %req.username, "Invalid password");
            return Err(Status::unauthenticated("Invalid credentials"));
        }

        // 5. 检查用户状态
        if !user.is_active() {
            tracing::warn!(username = %req.username, status = ?user.status, "User not active");
            return Err(Status::permission_denied("User account is not active"));
        }

        // 6. 检查是否需要 2FA
        if user.two_factor_enabled {
            // 创建临时会话用于 2FA 验证
            let session_id = SessionId::new();
            tracing::info!(username = %req.username, "2FA required");
            return Ok(Response::new(LoginResponse {
                require_2fa: true,
                session_id: session_id.0.to_string(),
                ..Default::default()
            }));
        }

        // 7. 生成令牌
        let access_token = self
            .token_service
            .generate_access_token(&user.id, &user.tenant_id, vec![], user.role_ids.clone())
            .map_err(|e| Status::internal(e.to_string()))?;

        let refresh_token = self
            .token_service
            .generate_refresh_token(&user.id, &user.tenant_id)
            .map_err(|e| Status::internal(e.to_string()))?;

        // 8. 创建会话
        let refresh_token_hash = sha256_hash(&refresh_token);
        let expires_at = Utc::now() + Duration::seconds(self.refresh_token_expires_in);

        let mut session = DomainSession::new(user.id.clone(), refresh_token_hash, expires_at);

        if !req.device_info.is_empty() {
            session = session.with_device_info(&req.device_info);
        }
        if !req.ip_address.is_empty() {
            session = session.with_ip_address(&req.ip_address);
        }

        self.session_repo
            .save(&session)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 9. 更新用户最后登录时间
        let mut updated_user = user.clone();
        updated_user.record_login();
        let _ = self.user_repo.update(&updated_user).await;

        tracing::info!(username = %req.username, user_id = %user.id.0, "Login successful");

        Ok(Response::new(LoginResponse {
            access_token,
            refresh_token,
            expires_in: self.token_service.access_token_expires_in(),
            token_type: "Bearer".to_string(),
            user: Some(user_to_proto(&user)),
            require_2fa: false,
            session_id: session.id.0.to_string(),
        }))
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(logout_all = %req.logout_all_devices, "Logout request");

        // 验证 token 获取用户信息
        let claims = self
            .token_service
            .validate_token(&req.access_token)
            .map_err(|_| Status::unauthenticated("Invalid token"))?;

        let user_id = claims
            .user_id()
            .map_err(|e| Status::internal(e.to_string()))?;

        // 计算 token 剩余有效期（秒）
        let now = Utc::now().timestamp();
        let ttl_secs = (claims.exp - now).max(0) as u64;

        if req.logout_all_devices {
            // 撤销所有会话
            self.session_repo
                .revoke_all_by_user_id(&user_id)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

            // 将用户所有 Token 加入黑名单
            self.auth_cache
                .blacklist_user_tokens(&user_id.0.to_string(), self.refresh_token_expires_in as u64)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

            // 清除用户缓存
            self.auth_cache
                .invalidate_user_cache(&user_id.0.to_string())
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

            tracing::info!(user_id = %user_id.0, "All sessions revoked and tokens blacklisted");
        } else {
            // 只将当前 Token 加入黑名单
            self.auth_cache
                .blacklist_token(&claims.jti, ttl_secs)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
            tracing::info!(jti = %claims.jti, "Token blacklisted");
        }

        Ok(Response::new(LogoutResponse { success: true }))
    }

    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<RefreshTokenResponse>, Status> {
        let req = request.into_inner();
        tracing::info!("Refresh token request");

        // 1. 验证 refresh token
        let claims = self
            .token_service
            .validate_token(&req.refresh_token)
            .map_err(|_| Status::unauthenticated("Invalid refresh token"))?;

        // 2. 计算 token hash 并查找 session
        let token_hash = sha256_hash(&req.refresh_token);
        let session = self
            .session_repo
            .find_by_refresh_token_hash(&token_hash)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::unauthenticated("Session not found"))?;

        // 3. 检查 session 是否有效
        if !session.is_valid() {
            return Err(Status::unauthenticated("Session expired or revoked"));
        }

        // 4. 获取用户信息
        let user = self
            .user_repo
            .find_by_id(&session.user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // 5. 生成新的 tokens
        let new_access_token = self
            .token_service
            .generate_access_token(&user.id, &user.tenant_id, vec![], user.role_ids.clone())
            .map_err(|e| Status::internal(e.to_string()))?;

        let new_refresh_token = self
            .token_service
            .generate_refresh_token(&user.id, &user.tenant_id)
            .map_err(|e| Status::internal(e.to_string()))?;

        // 6. 更新 session
        let new_hash = sha256_hash(&new_refresh_token);
        let new_expires = Utc::now() + Duration::seconds(self.refresh_token_expires_in);
        let mut updated_session = session;
        updated_session.refresh(new_hash, new_expires);
        self.session_repo
            .update(&updated_session)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        tracing::info!(user_id = %claims.sub, "Token refreshed");

        Ok(Response::new(RefreshTokenResponse {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            expires_in: self.token_service.access_token_expires_in(),
        }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();
        tracing::debug!("Validate token request");

        match self.token_service.validate_token(&req.access_token) {
            Ok(claims) => {
                // 检查 Token 是否在黑名单中
                if self
                    .auth_cache
                    .is_token_blacklisted(&claims.jti)
                    .await
                    .unwrap_or(false)
                {
                    tracing::debug!(jti = %claims.jti, "Token is blacklisted");
                    return Ok(Response::new(ValidateTokenResponse {
                        valid: false,
                        ..Default::default()
                    }));
                }

                // 检查用户的所有 Token 是否被撤销
                if self
                    .auth_cache
                    .is_user_tokens_blacklisted(&claims.sub)
                    .await
                    .unwrap_or(false)
                {
                    tracing::debug!(user_id = %claims.sub, "User tokens are blacklisted");
                    return Ok(Response::new(ValidateTokenResponse {
                        valid: false,
                        ..Default::default()
                    }));
                }

                Ok(Response::new(ValidateTokenResponse {
                    valid: true,
                    user_id: claims.sub,
                    tenant_id: claims.tenant_id,
                    permissions: claims.permissions,
                    expires_at: Some(Timestamp {
                        seconds: claims.exp,
                        nanos: 0,
                    }),
                }))
            }
            Err(_) => Ok(Response::new(ValidateTokenResponse {
                valid: false,
                ..Default::default()
            })),
        }
    }

    async fn change_password(
        &self,
        request: Request<ChangePasswordRequest>,
    ) -> Result<Response<ChangePasswordResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, "Change password request");

        // 1. 解析用户 ID
        let user_id = UserId::from_uuid(
            Uuid::parse_str(&req.user_id)
                .map_err(|_| Status::invalid_argument("Invalid user ID"))?,
        );

        // 2. 获取用户
        let mut user = self
            .user_repo
            .find_by_id(&user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // 3. 验证旧密码
        let valid = user
            .password_hash
            .verify(&req.old_password)
            .map_err(|e| Status::internal(e.to_string()))?;

        if !valid {
            return Err(Status::unauthenticated("Invalid old password"));
        }

        // 4. 哈希新密码
        let new_hash = HashedPassword::from_plain(&req.new_password)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // 5. 更新密码
        user.update_password(new_hash);
        self.user_repo
            .update(&user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 6. 撤销所有会话（安全考虑）
        self.session_repo
            .revoke_all_by_user_id(&user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 7. 将用户所有 Token 加入黑名单
        self.auth_cache
            .blacklist_user_tokens(&req.user_id, self.refresh_token_expires_in as u64)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 8. 清除用户缓存
        self.auth_cache
            .invalidate_user_cache(&req.user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        tracing::info!(user_id = %req.user_id, "Password changed successfully, all tokens invalidated");

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

        // 1. 解析邮箱
        let email = Email::new(&req.email)
            .map_err(|e| Status::invalid_argument(format!("Invalid email: {}", e)))?;

        // 2. 查找用户（使用租户 ID）
        let tenant_id = TenantId::from_uuid(
            Uuid::parse_str(&req.tenant_id)
                .map_err(|_| Status::invalid_argument("Invalid tenant ID"))?,
        );

        let user = self
            .user_repo
            .find_by_email(&email, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 如果用户不存在，也返回成功（安全考虑，不泄露用户是否存在）
        if user.is_none() {
            tracing::warn!(email = %req.email, "User not found for password reset");
            return Ok(Response::new(RequestPasswordResetResponse {
                success: true,
                message: "If the email exists, a password reset link has been sent.".to_string(),
            }));
        }

        let user = user.unwrap();

        // 3. 检查限流（防止滥用）
        let unused_count = self
            .password_reset_repo
            .count_unused_by_user_id(&user.id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if unused_count >= self.password_reset_config.max_requests_per_hour as i64 {
            tracing::warn!(user_id = %user.id, "Too many password reset requests");
            return Err(Status::resource_exhausted(
                "Too many password reset requests. Please try again later.",
            ));
        }

        // 4. 生成重置令牌
        let token = Uuid::new_v4().to_string();
        let token_hash = sha256_hash(&token);

        // 5. 创建并保存令牌实体
        let reset_token = PasswordResetToken::new(
            user.id.clone(),
            token_hash,
            self.password_reset_config.token_expires_minutes,
        );

        self.password_reset_repo
            .save(&reset_token)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 6. 构建重置链接
        let reset_link = format!(
            "{}?token={}",
            self.password_reset_config.reset_link_base_url, token
        );

        // 7. 发送邮件
        let subject = "密码重置请求 - Cuba ERP";
        let user_name = user.display_name.as_deref().unwrap_or(user.username.as_str());
        
        // 构建邮件内容
        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"></head>
<body>
<h2>密码重置</h2>
<p>您好，{}！</p>
<p>我们收到了您的密码重置请求。请点击下面的链接重置您的密码：</p>
<p><a href="{}">{}</a></p>
<p><strong>此链接将在 {} 分钟后失效。</strong></p>
<p>如果您没有请求重置密码，请忽略此邮件。</p>
<hr>
<p><small>此邮件由 Cuba ERP 系统自动发送，请勿回复。</small></p>
</body>
</html>"#,
            user_name, reset_link, reset_link, self.password_reset_config.token_expires_minutes
        );

        let text_body = format!(
            "密码重置\n\n您好，{}！\n\n我们收到了您的密码重置请求。请访问以下链接重置您的密码：\n\n{}\n\n此链接将在 {} 分钟后失效。\n\n如果您没有请求重置密码，请忽略此邮件。\n\n---\n此邮件由 Cuba ERP 系统自动发送，请勿回复。",
            user_name, reset_link, self.password_reset_config.token_expires_minutes
        );

        self.email_sender
            .send_html_email(&req.email, subject, &html_body, Some(&text_body))
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to send password reset email");
                Status::internal("Failed to send email")
            })?;

        tracing::info!(user_id = %user.id, email = %req.email, "Password reset email sent");

        Ok(Response::new(RequestPasswordResetResponse {
            success: true,
            message: "If the email exists, a password reset link has been sent.".to_string(),
        }))
    }

    async fn reset_password(
        &self,
        request: Request<ResetPasswordRequest>,
    ) -> Result<Response<ResetPasswordResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(email = %req.email, "Reset password");

        // 1. 计算令牌哈希
        let token_hash = sha256_hash(&req.reset_token);

        // 2. 查找令牌
        let token = self
            .password_reset_repo
            .find_by_token_hash(&token_hash)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Invalid or expired reset token"))?;

        // 3. 验证令牌有效性
        if !token.is_valid() {
            if token.used {
                return Err(Status::invalid_argument("Reset token has already been used"));
            }
            if token.is_expired() {
                return Err(Status::invalid_argument("Reset token has expired"));
            }
            return Err(Status::invalid_argument("Invalid reset token"));
        }

        // 4. 获取用户
        let mut user = self
            .user_repo
            .find_by_id(&token.user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // 5. 验证邮箱匹配（额外的安全检查）
        if user.email.as_str() != req.email {
            tracing::warn!(
                user_id = %user.id,
                provided_email = %req.email,
                actual_email = %user.email.as_str(),
                "Email mismatch in password reset"
            );
            return Err(Status::invalid_argument("Invalid reset token"));
        }

        // 6. 验证新密码强度（基本验证）
        if req.new_password.len() < 8 {
            return Err(Status::invalid_argument(
                "Password must be at least 8 characters long",
            ));
        }

        // 7. 哈希新密码
        let new_password_hash = HashedPassword::from_plain(&req.new_password)
            .map_err(|e| Status::internal(format!("Failed to hash password: {}", e)))?;

        // 8. 更新密码
        user.update_password(new_password_hash);
        self.user_repo
            .update(&user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 9. 标记令牌为已使用
        self.password_reset_repo
            .mark_as_used(&token.id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 10. 撤销所有会话（安全考虑）
        self.session_repo
            .revoke_all_by_user_id(&user.id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 11. 将用户所有 Token 加入黑名单
        self.auth_cache
            .blacklist_user_tokens(&user.id.0.to_string(), self.refresh_token_expires_in as u64)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 12. 清除用户缓存
        self.auth_cache
            .invalidate_user_cache(&user.id.0.to_string())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        tracing::info!(
            user_id = %user.id,
            email = %req.email,
            "Password reset successful, all sessions revoked"
        );

        Ok(Response::new(ResetPasswordResponse {
            success: true,
            message: "Password has been reset successfully. Please login with your new password.".to_string(),
        }))
    }

    async fn get_active_sessions(
        &self,
        request: Request<GetActiveSessionsRequest>,
    ) -> Result<Response<GetActiveSessionsResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, "Get active sessions request");

        let user_id = UserId::from_uuid(
            Uuid::parse_str(&req.user_id)
                .map_err(|_| Status::invalid_argument("Invalid user ID"))?,
        );

        let sessions = self
            .session_repo
            .find_active_by_user_id(&user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_sessions: Vec<Session> = sessions
            .into_iter()
            .map(|s| session_to_proto(&s, false))
            .collect();

        Ok(Response::new(GetActiveSessionsResponse {
            sessions: proto_sessions,
        }))
    }

    async fn revoke_session(
        &self,
        request: Request<RevokeSessionRequest>,
    ) -> Result<Response<RevokeSessionResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, session_id = %req.session_id, "Revoke session request");

        let session_id = SessionId(
            Uuid::parse_str(&req.session_id)
                .map_err(|_| Status::invalid_argument("Invalid session ID"))?,
        );

        // 获取并验证会话属于该用户
        let session = self
            .session_repo
            .find_by_id(&session_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Session not found"))?;

        if session.user_id.0.to_string() != req.user_id {
            return Err(Status::permission_denied(
                "Cannot revoke other user's session",
            ));
        }

        // 撤销会话
        let mut session = session;
        session.revoke();
        self.session_repo
            .update(&session)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        tracing::info!(session_id = %req.session_id, "Session revoked");

        Ok(Response::new(RevokeSessionResponse { success: true }))
    }

    async fn enable2_fa(
        &self,
        request: Request<Enable2FaRequest>,
    ) -> Result<Response<Enable2FaResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, method = %req.method, "Enable 2FA request");

        // 1. 解析用户 ID
        let user_id = UserId::from_string(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

        // 2. 获取用户
        let mut user = self
            .user_repo
            .find_by_id(&user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // 3. 检查是否已启用
        if user.two_factor_enabled {
            return Err(Status::already_exists("2FA already enabled"));
        }

        // 4. 生成 TOTP secret
        let secret = self
            .totp_service
            .generate_secret()
            .map_err(|e| Status::internal(e.to_string()))?;

        // 5. 生成 QR 码 URL
        let qr_code_url = self
            .totp_service
            .generate_qr_code_url(user.username.as_str(), &secret)
            .map_err(|e| Status::internal(e.to_string()))?;

        // 6. 如果提供了验证码，验证并启用
        if !req.verification_code.is_empty() {
            let valid = self
                .totp_service
                .verify_code(user.username.as_str(), &secret, &req.verification_code)
                .map_err(|e| Status::internal(e.to_string()))?;

            if !valid {
                return Err(Status::invalid_argument("Invalid verification code"));
            }

            // 7. 生成备份码
            let backup_codes = BackupCodeService::generate_codes();
            let backup_code_entities: Vec<crate::auth::domain::entities::BackupCode> = backup_codes
                .iter()
                .map(|code| {
                    let hash = BackupCodeService::hash_code(code);
                    crate::auth::domain::entities::BackupCode::new(user_id.clone(), hash)
                })
                .collect();

            // 8. 保存备份码
            self.backup_code_repo
                .save_batch(&backup_code_entities)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

            // 9. 启用 2FA
            user.enable_2fa(secret.clone());
            self.user_repo
                .update(&user)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

            tracing::info!(user_id = %req.user_id, "2FA enabled successfully");

            return Ok(Response::new(Enable2FaResponse {
                secret,
                qr_code_url,
                backup_codes,
                enabled: true,
            }));
        }

        // 如果没有提供验证码，只返回 QR 码（第一步）
        Ok(Response::new(Enable2FaResponse {
            secret,
            qr_code_url,
            backup_codes: vec![],
            enabled: false,
        }))
    }

    async fn verify2_fa(
        &self,
        request: Request<Verify2FaRequest>,
    ) -> Result<Response<Verify2FaResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, "Verify 2FA request");

        // 1. 解析用户 ID
        let user_id = UserId::from_string(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

        // 2. 获取用户
        let user = self
            .user_repo
            .find_by_id(&user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // 3. 检查是否启用 2FA
        if !user.two_factor_enabled {
            return Err(Status::failed_precondition("2FA not enabled"));
        }

        let secret = user
            .two_factor_secret
            .as_ref()
            .ok_or_else(|| Status::internal("2FA secret not found"))?;

        // 4. 验证 TOTP 码
        let totp_valid = self
            .totp_service
            .verify_code(user.username.as_str(), secret, &req.code)
            .map_err(|e| Status::internal(e.to_string()))?;

        if totp_valid {
            // TOTP 验证成功，生成令牌
            let access_token = self
                .token_service
                .generate_access_token(&user.id, &user.tenant_id, vec![], user.role_ids.clone())
                .map_err(|e| Status::internal(e.to_string()))?;

            let refresh_token = self
                .token_service
                .generate_refresh_token(&user.id, &user.tenant_id)
                .map_err(|e| Status::internal(e.to_string()))?;

            tracing::info!(user_id = %req.user_id, "2FA verified with TOTP");

            return Ok(Response::new(Verify2FaResponse {
                success: true,
                access_token,
                refresh_token,
                expires_in: self.token_service.access_token_expires_in(),
            }));
        }

        // 5. TOTP 验证失败，尝试备份码
        let backup_codes = self
            .backup_code_repo
            .find_available_by_user_id(&user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        for mut backup_code in backup_codes {
            if BackupCodeService::verify_code(&req.code, &backup_code.code_hash) {
                // 备份码验证成功
                backup_code.mark_as_used();
                self.backup_code_repo
                    .update(&backup_code)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;

                // 生成令牌
                let access_token = self
                    .token_service
                    .generate_access_token(&user.id, &user.tenant_id, vec![], user.role_ids.clone())
                    .map_err(|e| Status::internal(e.to_string()))?;

                let refresh_token = self
                    .token_service
                    .generate_refresh_token(&user.id, &user.tenant_id)
                    .map_err(|e| Status::internal(e.to_string()))?;

                tracing::info!(user_id = %req.user_id, "2FA verified with backup code");

                return Ok(Response::new(Verify2FaResponse {
                    success: true,
                    access_token,
                    refresh_token,
                    expires_in: self.token_service.access_token_expires_in(),
                }));
            }
        }

        // 验证失败
        Err(Status::unauthenticated("Invalid 2FA code"))
    }

    async fn disable2_fa(
        &self,
        request: Request<Disable2FaRequest>,
    ) -> Result<Response<Disable2FaResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, "Disable 2FA request");

        // 1. 解析用户 ID
        let user_id = UserId::from_string(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

        // 2. 获取用户
        let mut user = self
            .user_repo
            .find_by_id(&user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // 3. 验证密码
        let valid = user
            .password_hash
            .verify(&req.password)
            .map_err(|e| Status::internal(e.to_string()))?;

        if !valid {
            return Err(Status::unauthenticated("Invalid password"));
        }

        // 4. 禁用 2FA
        user.disable_2fa();
        self.user_repo
            .update(&user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 5. 删除所有备份码
        self.backup_code_repo
            .delete_by_user_id(&user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        tracing::info!(user_id = %req.user_id, "2FA disabled successfully");

        Ok(Response::new(Disable2FaResponse {
            success: true,
            message: "2FA disabled successfully".to_string(),
        }))
    }
}
