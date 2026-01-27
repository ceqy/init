//! OAuth 查询处理器

use std::sync::Arc;

use async_trait::async_trait;
use cuba_cqrs_core::QueryHandler;
use cuba_errors::AppResult;
use tracing::info;

use crate::application::queries::oauth::{
    GetOAuthClientByIdQuery, ListTenantOAuthClientsQuery,
    OAuthClientQueryResult, ListOAuthClientsResult,
};
use crate::domain::oauth::OAuthClientId;
use crate::domain::repositories::oauth::OAuthClientRepository;

/// 通过 ID 获取 OAuth 客户端处理器
pub struct GetOAuthClientByIdHandler {
    client_repository: Arc<dyn OAuthClientRepository>,
}

impl GetOAuthClientByIdHandler {
    pub fn new(client_repository: Arc<dyn OAuthClientRepository>) -> Self {
        Self { client_repository }
    }
}

#[async_trait]
impl QueryHandler<GetOAuthClientByIdQuery> for GetOAuthClientByIdHandler {
    async fn handle(&self, query: GetOAuthClientByIdQuery) -> AppResult<Option<OAuthClientQueryResult>> {
        info!(
            client_id = %query.client_id,
            tenant_id = %query.tenant_id,
            "Handling GetOAuthClientByIdQuery"
        );

        let client_id = OAuthClientId::from_string(&query.client_id)
            .map_err(|e| cuba_errors::AppError::validation(e.to_string()))?;

        let client = self.client_repository
            .find_by_id(&client_id, &query.tenant_id)
            .await?;

        Ok(client.map(|c| OAuthClientQueryResult {
            id: c.id.0.to_string(),
            client_id: c.id.0.to_string(),
            name: c.name,
            client_type: format!("{:?}", c.client_type),
            redirect_uris: c.redirect_uris,
            allowed_scopes: c.allowed_scopes,
            owner_id: c.owner_id.0.to_string(),
            tenant_id: c.tenant_id.0.to_string(),
            is_active: c.is_active,
            created_at: c.created_at,
        }))
    }
}

/// 获取租户的 OAuth 客户端列表处理器
pub struct ListTenantOAuthClientsHandler {
    client_repository: Arc<dyn OAuthClientRepository>,
}

impl ListTenantOAuthClientsHandler {
    pub fn new(client_repository: Arc<dyn OAuthClientRepository>) -> Self {
        Self { client_repository }
    }
}

#[async_trait]
impl QueryHandler<ListTenantOAuthClientsQuery> for ListTenantOAuthClientsHandler {
    async fn handle(&self, query: ListTenantOAuthClientsQuery) -> AppResult<ListOAuthClientsResult> {
        info!(
            tenant_id = %query.tenant_id,
            page = query.page,
            page_size = query.page_size,
            "Handling ListTenantOAuthClientsQuery"
        );

        let clients = self.client_repository
            .list_by_tenant(&query.tenant_id, query.page as i64, query.page_size as i64)
            .await?;

        let total = self.client_repository
            .count_by_tenant(&query.tenant_id)
            .await?;

        let results = clients.into_iter().map(|c| OAuthClientQueryResult {
            id: c.id.0.to_string(),
            client_id: c.id.0.to_string(),
            name: c.name,
            client_type: format!("{:?}", c.client_type),
            redirect_uris: c.redirect_uris,
            allowed_scopes: c.allowed_scopes,
            owner_id: c.owner_id.0.to_string(),
            tenant_id: c.tenant_id.0.to_string(),
            is_active: c.is_active,
            created_at: c.created_at,
        }).collect();

        Ok(ListOAuthClientsResult {
            clients: results,
            total: total as u64,
            page: query.page,
            page_size: query.page_size,
        })
    }
}
