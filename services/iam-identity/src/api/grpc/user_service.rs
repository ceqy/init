//! UserService gRPC 实现
//!
//! 提供用户管理相关的 gRPC 接口

use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::info;

use cuba_auth_core::{Claims, TokenService};
use cuba_common::{TenantId, UserId};
use cuba_cqrs_core::CommandHandler;

use crate::application::commands::user::{
    SendEmailVerificationCommand, SendPhoneVerificationCommand, VerifyEmailCommand,
    VerifyPhoneCommand,
};
use crate::application::handlers::user::{
    SendEmailVerificationHandler, SendPhoneVerificationHandler, VerifyEmailHandler,
    VerifyPhoneHandler,
};
use crate::domain::user::{User, UserStatus};
use crate::domain::repositories::user::UserRepository;
use crate::domain::value_objects::{Email, HashedPassword, Username};

use super::user_proto::{
    self, user_service_server::UserService, *,
};

/// UserService 实现
pub struct UserServiceImpl {
    user_repo: Arc<dyn UserRepository>,
    token_service: Arc<TokenService>,
    send_email_verification_handler: Arc<SendEmailVerificationHandler>,
    verify_email_handler: Arc<VerifyEmailHandler>,
    send_phone_verification_handler: Arc<SendPhoneVerificationHandler>,
    verify_phone_handler: Arc<VerifyPhoneHandler>,
}

impl UserServiceImpl {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        token_service: Arc<TokenService>,
        send_email_verification_handler: Arc<SendEmailVerificationHandler>,
        verify_email_handler: Arc<VerifyEmailHandler>,
        send_phone_verification_handler: Arc<SendPhoneVerificationHandler>,
        verify_phone_handler: Arc<VerifyPhoneHandler>,
    ) -> Self {
        Self {
            user_repo,
            token_service,
            send_email_verification_handler,
            verify_email_handler,
            send_phone_verification_handler,
            verify_phone_handler,
        }
    }

    /// 将领域实体转换为 Proto User
    fn user_to_proto(&self, user: &User) -> user_proto::User {
        user_proto::User {
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
            last_login_at: user.last_login_at.map(|dt| prost_types::Timestamp {
                seconds: dt.timestamp(),
                nanos: dt.timestamp_subsec_nanos() as i32,
            }),
            audit_info: Some(user_proto::AuditInfo {
                created_at: Some(prost_types::Timestamp {
                    seconds: user.audit_info.created_at.timestamp(),
                    nanos: user.audit_info.created_at.timestamp_subsec_nanos() as i32,
                }),
                created_by: user
                    .audit_info
                    .created_by
                    .as_ref()
                    .map(|id| id.0.to_string())
                    .unwrap_or_default(),
                updated_at: Some(prost_types::Timestamp {
                    seconds: user.audit_info.updated_at.timestamp(),
                    nanos: user.audit_info.updated_at.timestamp_subsec_nanos() as i32,
                }),
                updated_by: user
                    .audit_info
                    .updated_by
                    .as_ref()
                    .map(|id| id.0.to_string())
                    .unwrap_or_default(),
            }),
        }
    }

    /// 从请求 metadata 中验证 token 并获取 claims
    fn validate_request_token<T>(&self, request: &Request<T>) -> Result<Claims, Status> {
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|t| t.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("Missing or invalid token"))?;

        self.token_service
            .validate_token(token)
            .map_err(|_| Status::unauthenticated("Invalid token"))
    }
}

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    /// 注册新用户
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        info!("Registering new user: {}", req.username);

        // 解析租户 ID
        let tenant_id = TenantId::from_string(&req.tenant_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid tenant_id: {}", e)))?;

        // 创建值对象
        let username = Username::new(&req.username)
            .map_err(|e| Status::invalid_argument(format!("Invalid username: {}", e)))?;

        let email = Email::new(&req.email)
            .map_err(|e| Status::invalid_argument(format!("Invalid email: {}", e)))?;

        // 检查用户名和邮箱是否已存在
        if self
            .user_repo
            .exists_by_username(&username, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
        {
            return Err(Status::already_exists("Username already exists"));
        }

        if self
            .user_repo
            .exists_by_email(&email, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
        {
            return Err(Status::already_exists("Email already exists"));
        }

        // 哈希密码
        let password_hash = HashedPassword::from_plain(&req.password)
            .map_err(|e| Status::internal(format!("Failed to hash password: {}", e)))?;

        // 创建用户
        let mut user = User::new(username, email, password_hash, tenant_id);

        if !req.display_name.is_empty() {
            user.display_name = Some(req.display_name);
        }

        if !req.phone.is_empty() {
            user.phone = Some(req.phone);
        }

        // 分配角色
        for role_id in req.role_ids {
            user.add_role(role_id);
        }

        // 保存用户
        self.user_repo
            .save(&user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        info!("User registered successfully: {}", user.id);

        Ok(Response::new(RegisterResponse {
            user_id: user.id.0.to_string(),
            user: Some(self.user_to_proto(&user)),
        }))
    }

    /// 获取用户信息
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let claims = self.validate_request_token(&request)?;
        let tenant_id = TenantId::from_string(&claims.tenant_id)
            .map_err(|_| Status::internal("Invalid tenant ID"))?;

        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let user = self
            .user_repo
            .find_by_id(&user_id, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        Ok(Response::new(GetUserResponse {
            user: Some(self.user_to_proto(&user)),
        }))
    }

    /// 获取当前用户（从 AuthService 迁移）
    async fn get_current_user(
        &self,
        request: Request<GetCurrentUserRequest>,
    ) -> Result<Response<GetCurrentUserResponse>, Status> {
        let req = request.into_inner();

        // 验证令牌并获取用户 ID
        let claims = self
            .token_service
            .validate_token(&req.access_token)
            .map_err(|e| Status::unauthenticated(format!("Invalid token: {}", e)))?;

        let user_id = UserId::from_string(&claims.sub)
            .map_err(|e| Status::internal(format!("Invalid user_id in token: {}", e)))?;

        let tenant_id = TenantId::from_string(&claims.tenant_id)
            .map_err(|_| Status::internal("Invalid tenant ID"))?;

        let user = self
            .user_repo
            .find_by_id(&user_id, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        Ok(Response::new(GetCurrentUserResponse {
            user: Some(self.user_to_proto(&user)),
        }))
    }

    /// 更新用户信息
    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        let claims = self.validate_request_token(&request)?;
        let tenant_id = TenantId::from_string(&claims.tenant_id)
            .map_err(|_| Status::internal("Invalid tenant ID"))?;

        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // 更新字段
        if !req.username.is_empty() {
            user.username = Username::new(&req.username)
                .map_err(|e| Status::invalid_argument(format!("Invalid username: {}", e)))?;
        }

        if !req.email.is_empty() {
            user.email = Email::new(&req.email)
                .map_err(|e| Status::invalid_argument(format!("Invalid email: {}", e)))?;
        }

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

        if !req.status.is_empty() {
            user.status = match req.status.as_str() {
                "Active" => UserStatus::Active,
                "Inactive" => UserStatus::Inactive,
                "Locked" => UserStatus::Locked,
                "PendingVerification" => UserStatus::PendingVerification,
                _ => return Err(Status::invalid_argument("Invalid status")),
            };
        }

        self.user_repo
            .update(&user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateUserResponse {
            user: Some(self.user_to_proto(&user)),
        }))
    }

    /// 更新个人资料（从 AuthService 迁移）
    async fn update_profile(
        &self,
        request: Request<UpdateProfileRequest>,
    ) -> Result<Response<UpdateProfileResponse>, Status> {
        let claims = self.validate_request_token(&request)?;
        let tenant_id = TenantId::from_string(&claims.tenant_id)
            .map_err(|_| Status::internal("Invalid tenant ID"))?;

        // 验证用户只能更新自己的资料
        let req = request.into_inner();
        if claims.sub != req.user_id {
            return Err(Status::permission_denied("Cannot update other user's profile"));
        }

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // 只更新个人资料相关字段
        if !req.display_name.is_empty() {
            user.display_name = Some(req.display_name);
        }

        if !req.email.is_empty() {
            user.email = Email::new(&req.email)
                .map_err(|e| Status::invalid_argument(format!("Invalid email: {}", e)))?;
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

        self.user_repo
            .update(&user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateProfileResponse {
            user: Some(self.user_to_proto(&user)),
        }))
    }

    /// 删除用户
    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<()>, Status> {
        let claims = self.validate_request_token(&request)?;
        let tenant_id = TenantId::from_string(&claims.tenant_id)
            .map_err(|_| Status::internal("Invalid tenant ID"))?;

        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        self.user_repo
            .delete(&user_id, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        info!("User deleted: {}", user_id);

        Ok(Response::new(()))
    }

    /// 用户列表查询
    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let claims = self.validate_request_token(&request)?;
        let tenant_id = TenantId::from_string(&claims.tenant_id)
            .map_err(|_| Status::internal("Invalid tenant ID"))?;

        let req = request.into_inner();

        // 覆盖请求中的 tenant_id (或验证必须匹配)
        if !req.tenant_id.is_empty() && req.tenant_id != tenant_id.0.to_string() {
              // 简单实现：忽略请求中的 tenant_id，强制使用 token 中的
        }

        // 解析状态
        let status = if !req.status.is_empty() {
            Some(req.status.as_str())
        } else {
            None
        };

        // 解析搜索关键词
        let search = if !req.search.is_empty() {
            Some(req.search.as_str())
        } else {
            None
        };

        // 分页参数
        let page = if req.page > 0 { req.page } else { 1 };
        let page_size = if req.page_size > 0 && req.page_size <= 100 {
            req.page_size
        } else {
            20
        };

        // 查询用户列表
        let (users, total) = self
            .user_repo
            .list(
                &tenant_id,
                status,
                search,
                &req.role_ids,
                page,
                page_size,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 转换为 proto
        let proto_users: Vec<user_proto::User> = users.iter().map(|u| self.user_to_proto(u)).collect();

        // 计算总页数
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as i32;

        Ok(Response::new(ListUsersResponse {
            users: proto_users,
            page,
            page_size,
            total,
            total_pages,
        }))
    }

    /// 激活用户
    async fn activate_user(
        &self,
        request: Request<ActivateUserRequest>,
    ) -> Result<Response<ActivateUserResponse>, Status> {
        let claims = self.validate_request_token(&request)?;
        let tenant_id = TenantId::from_string(&claims.tenant_id)
            .map_err(|_| Status::internal("Invalid tenant ID"))?;

        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        user.activate();

        self.user_repo
            .update(&user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ActivateUserResponse {
            user: Some(self.user_to_proto(&user)),
        }))
    }

    /// 停用用户
    async fn deactivate_user(
        &self,
        request: Request<DeactivateUserRequest>,
    ) -> Result<Response<DeactivateUserResponse>, Status> {
        let claims = self.validate_request_token(&request)?;
        let tenant_id = TenantId::from_string(&claims.tenant_id)
            .map_err(|_| Status::internal("Invalid tenant ID"))?;

        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        user.deactivate();

        self.user_repo
            .update(&user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        info!("User deactivated: {}, reason: {}", user_id, req.reason);

        Ok(Response::new(DeactivateUserResponse {
            user: Some(self.user_to_proto(&user)),
        }))
    }

    /// 锁定用户
    async fn lock_user(
        &self,
        request: Request<LockUserRequest>,
    ) -> Result<Response<LockUserResponse>, Status> {
        let claims = self.validate_request_token(&request)?;
        let tenant_id = TenantId::from_string(&claims.tenant_id)
            .map_err(|_| Status::internal("Invalid tenant ID"))?;

        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        user.lock();

        self.user_repo
            .update(&user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        info!("User locked: {}, reason: {}", user_id, req.reason);

        Ok(Response::new(LockUserResponse {
            user: Some(self.user_to_proto(&user)),
        }))
    }

    /// 解锁用户
    async fn unlock_user(
        &self,
        request: Request<UnlockUserRequest>,
    ) -> Result<Response<UnlockUserResponse>, Status> {
        let claims = self.validate_request_token(&request)?;
        let tenant_id = TenantId::from_string(&claims.tenant_id)
            .map_err(|_| Status::internal("Invalid tenant ID"))?;

        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        user.activate(); // 解锁后激活

        self.user_repo
            .update(&user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        info!("User unlocked: {}", user_id);

        Ok(Response::new(UnlockUserResponse {
            user: Some(self.user_to_proto(&user)),
        }))
    }

    /// 分配角色
    async fn assign_roles(
        &self,
        request: Request<AssignRolesRequest>,
    ) -> Result<Response<AssignRolesResponse>, Status> {
        let claims = self.validate_request_token(&request)?;
        let tenant_id = TenantId::from_string(&claims.tenant_id)
            .map_err(|_| Status::internal("Invalid tenant ID"))?;

        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        for role_id in req.role_ids {
            user.add_role(role_id);
        }

        self.user_repo
            .update(&user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(AssignRolesResponse {
            user: Some(self.user_to_proto(&user)),
        }))
    }

    /// 移除角色
    async fn remove_roles(
        &self,
        request: Request<RemoveRolesRequest>,
    ) -> Result<Response<RemoveRolesResponse>, Status> {
        let claims = self.validate_request_token(&request)?;
        let tenant_id = TenantId::from_string(&claims.tenant_id)
            .map_err(|_| Status::internal("Invalid tenant ID"))?;

        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        for role_id in &req.role_ids {
            user.remove_role(role_id);
        }

        self.user_repo
            .update(&user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RemoveRolesResponse {
            user: Some(self.user_to_proto(&user)),
        }))
    }

    /// 获取用户角色
    async fn get_user_roles(
        &self,
        _request: Request<GetUserRolesRequest>,
    ) -> Result<Response<GetUserRolesResponse>, Status> {
        // TODO: 需要实现角色查询逻辑，可能需要调用 RBAC 服务
        Err(Status::unimplemented("GetUserRoles not yet implemented"))
    }

    /// 发送邮箱验证码
    async fn send_email_verification(
        &self,
        request: Request<SendEmailVerificationRequest>,
    ) -> Result<Response<SendEmailVerificationResponse>, Status> {
        // 从 metadata 中获取 tenant_id - 必须在 into_inner 之前
        let tenant_id = request
            .metadata()
            .get("tenant-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        
        let req = request.into_inner();
        info!("Sending email verification for user: {}", req.user_id);

        if tenant_id.is_empty() {
            return Err(Status::invalid_argument("tenant_id is required in metadata"));
        }

        // 创建命令
        let command = SendEmailVerificationCommand {
            user_id: req.user_id,
            tenant_id,
        };

        // 执行命令
        let result = self
            .send_email_verification_handler
            .handle(command)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SendEmailVerificationResponse {
            success: result.success,
            message: result.message,
            expires_in_seconds: result.expires_in_seconds as i32,
        }))
    }

    /// 验证邮箱
    async fn verify_email(
        &self,
        request: Request<VerifyEmailRequest>,
    ) -> Result<Response<VerifyEmailResponse>, Status> {
        // 从 metadata 中获取 tenant_id - 必须在 into_inner 之前
        let tenant_id = request
            .metadata()
            .get("tenant-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        
        let req = request.into_inner();
        info!("Verifying email for user: {}", req.user_id);

        if tenant_id.is_empty() {
            return Err(Status::invalid_argument("tenant_id is required in metadata"));
        }

        // 创建命令
        let command = VerifyEmailCommand {
            user_id: req.user_id,
            code: req.code,
            tenant_id,
        };

        // 执行命令
        let result = self
            .verify_email_handler
            .handle(command)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(VerifyEmailResponse {
            success: result.success,
            message: result.message,
        }))
    }

    /// 发送手机验证码
    async fn send_phone_verification(
        &self,
        request: Request<SendPhoneVerificationRequest>,
    ) -> Result<Response<SendPhoneVerificationResponse>, Status> {
        // 从 metadata 中获取 tenant_id - 必须在 into_inner 之前
        let tenant_id = request
            .metadata()
            .get("tenant-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        
        let req = request.into_inner();
        info!("Sending phone verification for user: {}", req.user_id);

        if tenant_id.is_empty() {
            return Err(Status::invalid_argument("tenant_id is required in metadata"));
        }

        // 创建命令
        let command = SendPhoneVerificationCommand {
            user_id: req.user_id,
            tenant_id,
        };

        // 执行命令
        let result = self
            .send_phone_verification_handler
            .handle(command)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SendPhoneVerificationResponse {
            success: result.success,
            message: result.message,
            expires_in_seconds: result.expires_in_seconds as i32,
        }))
    }

    /// 验证手机
    async fn verify_phone(
        &self,
        request: Request<VerifyPhoneRequest>,
    ) -> Result<Response<VerifyPhoneResponse>, Status> {
        // 从 metadata 中获取 tenant_id - 必须在 into_inner 之前
        let tenant_id = request
            .metadata()
            .get("tenant-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        
        let req = request.into_inner();
        info!("Verifying phone for user: {}", req.user_id);

        if tenant_id.is_empty() {
            return Err(Status::invalid_argument("tenant_id is required in metadata"));
        }

        // 创建命令
        let command = VerifyPhoneCommand {
            user_id: req.user_id,
            code: req.code,
            tenant_id,
        };

        // 执行命令
        let result = self
            .verify_phone_handler
            .handle(command)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(VerifyPhoneResponse {
            success: result.success,
            message: result.message,
        }))
    }
}
