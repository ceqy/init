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

use crate::auth::domain::entities::{Session as DomainSession, SessionId};
use crate::auth::domain::repositories::SessionRepository;
use crate::auth::infrastructure::cache::AuthCache;
use crate::shared::domain::repositories::UserRepository;
use crate::shared::domain::value_objects::{HashedPassword, Username};

/// AuthService 实现
pub struct AuthServiceImpl {
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn SessionRepository>,
    token_service: Arc<TokenService>,
    auth_cache: Arc<dyn AuthCache>,
    refresh_token_expires_in: i64,
}

impl AuthServiceImpl {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        session_repo: Arc<dyn SessionRepository>,
        token_service: Arc<TokenService>,
        auth_cache: Arc<dyn AuthCache>,
        refresh_token_expires_in: i64,
    ) -> Self {
        Self {
            user_repo,
            session_repo,
            token_service,
            auth_cache,
            refresh_token_expires_in,
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

        // TODO: 实现密码重置邮件发送
        Err(Status::unimplemented("Password reset not implemented yet"))
    }

    async fn reset_password(
        &self,
        request: Request<ResetPasswordRequest>,
    ) -> Result<Response<ResetPasswordResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(email = %req.email, "Reset password");

        // TODO: 实现密码重置
        Err(Status::unimplemented("Password reset not implemented yet"))
    }

    async fn get_current_user(
        &self,
        request: Request<GetCurrentUserRequest>,
    ) -> Result<Response<GetCurrentUserResponse>, Status> {
        let req = request.into_inner();
        tracing::debug!("Get current user request");

        // 1. 验证 token
        let claims = self
            .token_service
            .validate_token(&req.access_token)
            .map_err(|_| Status::unauthenticated("Invalid token"))?;

        // 检查 Token 是否在黑名单中
        if self
            .auth_cache
            .is_token_blacklisted(&claims.jti)
            .await
            .unwrap_or(false)
        {
            return Err(Status::unauthenticated("Token has been revoked"));
        }

        // 2. 获取用户 ID
        let user_id = claims
            .user_id()
            .map_err(|e| Status::internal(e.to_string()))?;

        let user_id_str = user_id.0.to_string();

        // 3. 尝试从缓存获取用户
        if let Ok(Some(cached_user)) = self.auth_cache.get_cached_user(&user_id_str).await {
            tracing::debug!(user_id = %user_id_str, "User found in cache");
            return Ok(Response::new(GetCurrentUserResponse {
                user: Some(user_to_proto(&cached_user)),
            }));
        }

        // 4. 从数据库获取用户
        let user = self
            .user_repo
            .find_by_id(&user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // 5. 缓存用户信息
        if let Err(e) = self.auth_cache.cache_user(&user).await {
            tracing::warn!(error = %e, "Failed to cache user");
        }

        Ok(Response::new(GetCurrentUserResponse {
            user: Some(user_to_proto(&user)),
        }))
    }

    async fn update_profile(
        &self,
        request: Request<UpdateProfileRequest>,
    ) -> Result<Response<UpdateProfileResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, "Update profile request");

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

        // 3. 更新字段
        if !req.display_name.is_empty() {
            user.display_name = Some(req.display_name);
        }
        if !req.phone.is_empty() {
            user.phone = Some(req.phone);
        }
        if !req.avatar_url.is_empty() {
            user.avatar_url = Some(req.avatar_url);
        }
        if !req.language.is_empty() {
            user.language = req.language;
        }
        if !req.timezone.is_empty() {
            user.timezone = req.timezone;
        }

        // 4. 保存更新
        self.user_repo
            .update(&user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 5. 清除用户缓存（确保下次获取最新数据）
        if let Err(e) = self.auth_cache.invalidate_user_cache(&req.user_id).await {
            tracing::warn!(error = %e, "Failed to invalidate user cache");
        }

        tracing::info!(user_id = %req.user_id, "Profile updated");

        Ok(Response::new(UpdateProfileResponse {
            user: Some(user_to_proto(&user)),
        }))
    }

    async fn enable2_fa(
        &self,
        request: Request<Enable2FaRequest>,
    ) -> Result<Response<Enable2FaResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, method = %req.method, "Enable 2FA request");

        // TODO: 实现 2FA
        Err(Status::unimplemented("2FA not implemented yet"))
    }

    async fn disable2_fa(
        &self,
        request: Request<Disable2FaRequest>,
    ) -> Result<Response<Disable2FaResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, "Disable 2FA request");

        // TODO: 实现 2FA
        Err(Status::unimplemented("2FA not implemented yet"))
    }

    async fn verify2_fa(
        &self,
        request: Request<Verify2FaRequest>,
    ) -> Result<Response<Verify2FaResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, "Verify 2FA request");

        // TODO: 实现 2FA
        Err(Status::unimplemented("2FA not implemented yet"))
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
}
