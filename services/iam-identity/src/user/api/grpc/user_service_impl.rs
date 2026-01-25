//! UserService gRPC 实现
//!
//! 提供用户管理相关的 gRPC 接口

use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::info;

use cuba_auth_core::TokenService;
use cuba_common::{TenantId, UserId};

use crate::shared::domain::entities::{User, UserStatus};
use crate::shared::domain::repositories::UserRepository;
use crate::shared::domain::value_objects::{Email, HashedPassword, Username};

// 导入生成的 proto 代码
pub mod proto {
    include!("cuba.iam.user.rs");
}

use proto::{
    user_service_server::UserService, ActivateUserRequest, ActivateUserResponse,
    AssignRolesRequest, AssignRolesResponse, DeactivateUserRequest, DeactivateUserResponse,
    DeleteUserRequest, GetCurrentUserRequest, GetCurrentUserResponse, GetUserRequest,
    GetUserResponse, GetUserRolesRequest, GetUserRolesResponse, ListUsersRequest,
    ListUsersResponse, LockUserRequest, LockUserResponse, RegisterRequest, RegisterResponse,
    RemoveRolesRequest, RemoveRolesResponse, UnlockUserRequest, UnlockUserResponse,
    UpdateProfileRequest, UpdateProfileResponse, UpdateUserRequest, UpdateUserResponse,
};

/// UserService 实现
pub struct UserServiceImpl {
    user_repo: Arc<dyn UserRepository>,
    token_service: Arc<TokenService>,
}

impl UserServiceImpl {
    pub fn new(user_repo: Arc<dyn UserRepository>, token_service: Arc<TokenService>) -> Self {
        Self {
            user_repo,
            token_service,
        }
    }

    /// 将领域实体转换为 Proto User
    fn user_to_proto(&self, user: &User) -> proto::User {
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
            last_login_at: user.last_login_at.map(|dt| prost_types::Timestamp {
                seconds: dt.timestamp(),
                nanos: dt.timestamp_subsec_nanos() as i32,
            }),
            audit_info: Some(proto::AuditInfo {
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
        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let user = self
            .user_repo
            .find_by_id(&user_id)
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

        let user = self
            .user_repo
            .find_by_id(&user_id)
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
        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id)
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
        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id)
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
        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        self.user_repo
            .delete(&user_id)
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
        let req = request.into_inner();

        // 解析租户 ID
        let tenant_id = if !req.tenant_id.is_empty() {
            Some(
                TenantId::from_string(&req.tenant_id)
                    .map_err(|e| Status::invalid_argument(format!("Invalid tenant_id: {}", e)))?,
            )
        } else {
            None
        };

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
                tenant_id.as_ref(),
                status,
                search,
                &req.role_ids,
                page,
                page_size,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 转换为 proto
        let proto_users: Vec<proto::User> = users.iter().map(|u| self.user_to_proto(u)).collect();

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
        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id)
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
        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id)
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
        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id)
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
        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id)
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
        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id)
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
        let req = request.into_inner();

        let user_id = UserId::from_string(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        let mut user = self
            .user_repo
            .find_by_id(&user_id)
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
}
