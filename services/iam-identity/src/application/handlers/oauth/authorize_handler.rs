use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_cqrs_core::CommandHandler;
use cuba_errors::{AppError, AppResult};
use tracing::info;

use crate::application::commands::oauth::AuthorizeCommand;
use crate::domain::oauth::OAuthClientId;
use crate::domain::services::oauth::OAuthService;

pub struct AuthorizeHandler {
    oauth_service: Arc<OAuthService>,
}

impl AuthorizeHandler {
    pub fn new(oauth_service: Arc<OAuthService>) -> Self {
        Self { oauth_service }
    }
}

#[async_trait]
impl CommandHandler<AuthorizeCommand> for AuthorizeHandler {
    async fn handle(&self, command: AuthorizeCommand) -> AppResult<(String, Option<String>)> {
        info!("Authorizing client: {}", command.client_id);

        if command.response_type != "code" {
            return Err(AppError::validation("Only 'code' response type is supported"));
        }

        let client_id = OAuthClientId::from_str(&command.client_id)
            .map_err(|e| AppError::validation(format!("Invalid client_id: {}", e)))?;

        let user_id = UserId::from_str(&command.user_id)
            .map_err(|e| AppError::validation(format!("Invalid user_id: {}", e)))?;

        let tenant_id = TenantId::from_str(&command.tenant_id)
            .map_err(|e| AppError::validation(format!("Invalid tenant_id: {}", e)))?;

        let code = self
            .oauth_service
            .create_authorization_code(
                &client_id,
                &user_id,
                &tenant_id,
                command.redirect_uri,
                command.scope.split_whitespace().map(|s| s.to_string()).collect(),
                command.code_challenge,
                command.code_challenge_method,
            )
            .await?;

        info!("Authorization code created for client: {}", client_id);

        Ok((code, command.state))
    }
}
