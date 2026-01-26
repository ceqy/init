use std::sync::Arc;

use async_trait::async_trait;
use cuba_common::TenantId;
use cuba_cqrs_core::CommandHandler;
use cuba_errors::{AppError, AppResult};
use tracing::info;

use crate::oauth::application::commands::CreateClientCommand;
use crate::oauth::domain::entities::{GrantType, OAuthClient, OAuthClientType};
use cuba_common::{UserId}; // Need UserId for owner_id
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

        // Default owner_id if not present (TODO: should come from command context/claims)
        let owner_id = UserId::new();

        let client_type = if command.public_client {
             OAuthClientType::Public
        } else {
             OAuthClientType::Confidential
        };

        let mut client = OAuthClient::new(
            tenant_id,
            owner_id,
            command.name,
            client_type,
            command.redirect_uris,
        ).map_err(|e| AppError::validation(format!("Failed to create client: {}", e)))?;
        
        // Handle scopes and grant_types if provided (overwrite defaults)
        if !command.scopes.is_empty() {
             client.scopes = command.scopes;
        }

        // Handle grant types string to enum
        if !command.grant_types.is_empty() {
             client.grant_types = command.grant_types.iter().map(|s| {
                 match s.as_str() {
                    "authorization_code" => GrantType::AuthorizationCode,
                    "client_credentials" => GrantType::ClientCredentials,
                    "refresh_token" => GrantType::RefreshToken,
                    "implicit" => GrantType::Implicit,
                    "password" => GrantType::Password,
                    _ => GrantType::AuthorizationCode,
                 }
             }).collect();
        }

        let plain_secret = if command.public_client {
            None
        } else {
            // Use provided secret or generate new one
            let secret = command.client_secret.unwrap_or_else(|| {
                use rand::Rng;
                let random_bytes = rand::thread_rng().r#gen::<[u8; 32]>();
                hex::encode(random_bytes)
            });
            // Hash secret (simplified for now, should use argon2/bcrypt)
            // Ideally inject a password service
            client.set_client_secret(secret.clone()); 
            Some(secret)
        };

        self.client_repo.save(&client).await?;

        info!("OAuth client created: {}", client.id);

        Ok((client.id.0.to_string(), plain_secret))
    }
}
