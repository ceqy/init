use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use cuba_common::TenantId;
use cuba_cqrs_core::CommandHandler;
use cuba_errors::{AppError, AppResult};
use tracing::info;

use crate::application::commands::oauth::TokenCommand;
use crate::domain::oauth::OAuthClientId;
use crate::domain::services::oauth::OAuthService;

pub struct TokenHandler {
    oauth_service: Arc<OAuthService>,
}

impl TokenHandler {
    pub fn new(oauth_service: Arc<OAuthService>) -> Self {
        Self { oauth_service }
    }
}

#[async_trait]
impl CommandHandler<TokenCommand> for TokenHandler {
    async fn handle(&self, command: TokenCommand) -> AppResult<(String, String, i64)> {
        info!("Processing token request: grant_type={}", command.grant_type);

        let client_id = OAuthClientId::from_str(&command.client_id)
            .map_err(|e| AppError::validation(format!("Invalid client_id: {}", e)))?;

        let tenant_id = TenantId::from_str(&command.tenant_id)
            .map_err(|e| AppError::validation(format!("Invalid tenant_id: {}", e)))?;

        let (access_token, refresh_token) = match command.grant_type.as_str() {
            "authorization_code" => {
                let code = command
                    .code
                    .ok_or_else(|| AppError::validation("code is required"))?;
                let redirect_uri = command
                    .redirect_uri
                    .ok_or_else(|| AppError::validation("redirect_uri is required"))?;

                self.oauth_service
                    .exchange_code_for_token(
                        &code,
                        &client_id,
                        &tenant_id,
                        &redirect_uri,
                        command.code_verifier.as_deref(),
                    )
                    .await?
            }
            "refresh_token" => {
                let refresh_token = command
                    .refresh_token
                    .ok_or_else(|| AppError::validation("refresh_token is required"))?;

                self.oauth_service
                    .refresh_access_token(&refresh_token, &client_id, &tenant_id)
                    .await?
            }
            _ => return Err(AppError::validation("Unsupported grant type")),
        };

        info!("Token issued for client: {}", client_id);

        Ok((access_token, refresh_token, 3600))
    }
}
