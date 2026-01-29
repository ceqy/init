//! 用户查询处理器

use std::sync::Arc;

use async_trait::async_trait;
use cuba_cqrs_core::QueryHandler;
use cuba_errors::AppResult;
use tracing::info;

use crate::application::queries::user::{
    GetUserByEmailQuery, GetUserByIdQuery, GetUserByUsernameQuery, UserQueryResult,
};
use crate::domain::repositories::user::UserRepository;
use crate::domain::value_objects::{Email, Username};

/// 通过 ID 获取用户处理器
pub struct GetUserByIdHandler {
    user_repository: Arc<dyn UserRepository>,
}

impl GetUserByIdHandler {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }
}

#[async_trait]
impl QueryHandler<GetUserByIdQuery> for GetUserByIdHandler {
    async fn handle(&self, query: GetUserByIdQuery) -> AppResult<Option<UserQueryResult>> {
        info!(
            user_id = %query.user_id,
            tenant_id = %query.tenant_id,
            "Handling GetUserByIdQuery"
        );

        let user = self
            .user_repository
            .find_by_id(&query.user_id, &query.tenant_id)
            .await?;

        Ok(user.map(|u| UserQueryResult {
            id: u.id.0.to_string(),
            username: u.username.as_str().to_string(),
            email: u.email.as_str().to_string(),
            display_name: u.display_name.clone(),
            status: format!("{}", u.status),
            tenant_id: u.tenant_id.0.to_string(),
            email_verified: u.email_verified,
            phone_verified: u.phone_verified,
            two_factor_enabled: u.two_factor_enabled,
            created_at: u.audit_info.created_at,
            last_login_at: u.last_login_at,
        }))
    }
}

/// 通过用户名获取用户处理器
pub struct GetUserByUsernameHandler {
    user_repository: Arc<dyn UserRepository>,
}

impl GetUserByUsernameHandler {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }
}

#[async_trait]
impl QueryHandler<GetUserByUsernameQuery> for GetUserByUsernameHandler {
    async fn handle(&self, query: GetUserByUsernameQuery) -> AppResult<Option<UserQueryResult>> {
        info!(
            username = %query.username,
            tenant_id = %query.tenant_id,
            "Handling GetUserByUsernameQuery"
        );

        // 转换为 Username 值对象
        let username = Username::new(&query.username)
            .map_err(|e| cuba_errors::AppError::validation(e.to_string()))?;

        let user = self
            .user_repository
            .find_by_username(&username, &query.tenant_id)
            .await?;

        Ok(user.map(|u| UserQueryResult {
            id: u.id.0.to_string(),
            username: u.username.as_str().to_string(),
            email: u.email.as_str().to_string(),
            display_name: u.display_name.clone(),
            status: format!("{}", u.status),
            tenant_id: u.tenant_id.0.to_string(),
            email_verified: u.email_verified,
            phone_verified: u.phone_verified,
            two_factor_enabled: u.two_factor_enabled,
            created_at: u.audit_info.created_at,
            last_login_at: u.last_login_at,
        }))
    }
}

/// 通过邮箱获取用户处理器
pub struct GetUserByEmailHandler {
    user_repository: Arc<dyn UserRepository>,
}

impl GetUserByEmailHandler {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }
}

#[async_trait]
impl QueryHandler<GetUserByEmailQuery> for GetUserByEmailHandler {
    async fn handle(&self, query: GetUserByEmailQuery) -> AppResult<Option<UserQueryResult>> {
        info!(
            email = %query.email,
            tenant_id = %query.tenant_id,
            "Handling GetUserByEmailQuery"
        );

        // 转换为 Email 值对象
        let email = Email::new(&query.email)
            .map_err(|e| cuba_errors::AppError::validation(e.to_string()))?;

        let user = self
            .user_repository
            .find_by_email(&email, &query.tenant_id)
            .await?;

        Ok(user.map(|u| UserQueryResult {
            id: u.id.0.to_string(),
            username: u.username.as_str().to_string(),
            email: u.email.as_str().to_string(),
            display_name: u.display_name.clone(),
            status: format!("{}", u.status),
            tenant_id: u.tenant_id.0.to_string(),
            email_verified: u.email_verified,
            phone_verified: u.phone_verified,
            two_factor_enabled: u.two_factor_enabled,
            created_at: u.audit_info.created_at,
            last_login_at: u.last_login_at,
        }))
    }
}
