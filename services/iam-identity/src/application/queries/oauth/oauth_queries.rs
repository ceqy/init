//! OAuth 查询

use cuba_common::TenantId;
use cuba_cqrs_core::Query;

/// 通过 ID 获取 OAuth 客户端查询
#[derive(Debug, Clone)]
pub struct GetOAuthClientByIdQuery {
    pub client_id: String,
    pub tenant_id: TenantId,
}

impl Query for GetOAuthClientByIdQuery {
    type Result = Option<OAuthClientQueryResult>;
}

/// 获取用户的 OAuth 客户端列表查询
#[derive(Debug, Clone)]
pub struct ListUserOAuthClientsQuery {
    pub owner_id: String,
    pub tenant_id: TenantId,
    pub page: u32,
    pub page_size: u32,
}

impl Query for ListUserOAuthClientsQuery {
    type Result = ListOAuthClientsResult;
}

/// 获取租户的 OAuth 客户端列表查询
#[derive(Debug, Clone)]
pub struct ListTenantOAuthClientsQuery {
    pub tenant_id: TenantId,
    pub page: u32,
    pub page_size: u32,
}

impl Query for ListTenantOAuthClientsQuery {
    type Result = ListOAuthClientsResult;
}

/// 验证 OAuth 客户端查询
#[derive(Debug, Clone)]
pub struct ValidateOAuthClientQuery {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
}

impl Query for ValidateOAuthClientQuery {
    type Result = ValidateOAuthClientResult;
}

/// 获取用户授权的应用列表查询
#[derive(Debug, Clone)]
pub struct ListUserAuthorizedAppsQuery {
    pub user_id: String,
    pub tenant_id: TenantId,
}

impl Query for ListUserAuthorizedAppsQuery {
    type Result = Vec<AuthorizedAppResult>;
}

/// OAuth 客户端查询结果
#[derive(Debug, Clone)]
pub struct OAuthClientQueryResult {
    pub id: String,
    pub client_id: String,
    pub name: String,
    pub client_type: String,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub owner_id: String,
    pub tenant_id: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// OAuth 客户端列表结果
#[derive(Debug, Clone)]
pub struct ListOAuthClientsResult {
    pub clients: Vec<OAuthClientQueryResult>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

/// 验证 OAuth 客户端结果
#[derive(Debug, Clone)]
pub struct ValidateOAuthClientResult {
    pub valid: bool,
    pub client: Option<OAuthClientQueryResult>,
    pub error: Option<String>,
}

/// 用户授权的应用结果
#[derive(Debug, Clone)]
pub struct AuthorizedAppResult {
    pub client_id: String,
    pub client_name: String,
    pub scopes: Vec<String>,
    pub authorized_at: chrono::DateTime<chrono::Utc>,
}
