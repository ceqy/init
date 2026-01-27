//! 获取用户查询

use cuba_common::{TenantId, UserId};
use cuba_cqrs_core::Query;

/// 通过 ID 获取用户查询
#[derive(Debug, Clone)]
pub struct GetUserByIdQuery {
    pub user_id: UserId,
    pub tenant_id: TenantId,
}

impl Query for GetUserByIdQuery {
    type Result = Option<UserQueryResult>;
}

/// 通过用户名获取用户查询
#[derive(Debug, Clone)]
pub struct GetUserByUsernameQuery {
    pub username: String,
    pub tenant_id: TenantId,
}

impl Query for GetUserByUsernameQuery {
    type Result = Option<UserQueryResult>;
}

/// 通过邮箱获取用户查询
#[derive(Debug, Clone)]
pub struct GetUserByEmailQuery {
    pub email: String,
    pub tenant_id: TenantId,
}

impl Query for GetUserByEmailQuery {
    type Result = Option<UserQueryResult>;
}

/// 获取当前用户查询
#[derive(Debug, Clone)]
pub struct GetCurrentUserQuery {
    pub access_token: String,
}

impl Query for GetCurrentUserQuery {
    type Result = Option<UserQueryResult>;
}

/// 用户列表查询
#[derive(Debug, Clone)]
pub struct ListUsersQuery {
    pub tenant_id: TenantId,
    pub page: u32,
    pub page_size: u32,
    pub status_filter: Option<String>,
    pub search: Option<String>,
}

impl Query for ListUsersQuery {
    type Result = ListUsersResult;
}

/// 用户查询结果
#[derive(Debug, Clone)]
pub struct UserQueryResult {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub status: String,
    pub tenant_id: String,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub two_factor_enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 用户列表查询结果
#[derive(Debug, Clone)]
pub struct ListUsersResult {
    pub users: Vec<UserQueryResult>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}
