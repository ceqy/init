use std::sync::Arc;

use async_trait::async_trait;
use cuba_common::TenantId;
use cuba_cqrs_core::CommandHandler;
use cuba_errors::{AppError, AppResult};
use tracing::info;

use crate::oauth::application::commands::CreateClientCommand;
use crate::oauth::domain::entities::OAuthClient;
use crate::oauth::domain::repositories::OAuthClientRepository;

pub struct CreateClientHandler {
    client_repo: Arc<dyn OAuthClientRepository>,
}

impl CreateClientHandler {
    pub fn new(client_repo: Arc<dyn OAuthClientRepository>) -> Self {
        Self { client_repo }
    }
}

#[async_trait]
impl CommandHandler<CreateClientCommand> for CreateClientHandler {
    async fn handle(&self, command: CreateClientCommand) -> AppResult<(String, Option<String>)> {
        info!("Creating OAuth client: {}", command.name);

        let tenant_id = TenantId::from_string(&command.tenant_id)
            .map_err(|e| AppError::validation(format!("Invalid tenant_id: {}", e)))?;

        let (client, plain_secret) = OAuthClient::create(
            command.name,
            tenant_id,
            command.redirect_uris,
            command.grant_types,
            command.scopes,
            command.client_secret,
            command.public_client,
        )?;

        self.client_repo.save(&client).await?;

        info!("OAuth client created: {}", client.id);

        Ok((client.id.0.to_string(), plain_secret))
    }
}
