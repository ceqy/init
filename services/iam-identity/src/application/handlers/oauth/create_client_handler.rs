use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use cuba_common::TenantId;
use cuba_cqrs_core::CommandHandler;
use cuba_errors::{AppError, AppResult};
use tracing::info;

use crate::application::commands::oauth::CreateClientCommand;
use crate::domain::oauth::{GrantType, OAuthClient, OAuthClientType};
use crate::infrastructure::events::{EventPublisher, IamDomainEvent};

use crate::domain::unit_of_work::UnitOfWorkFactory;

pub struct CreateClientHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CreateClientHandler {
    pub fn new(
        uow_factory: Arc<dyn UnitOfWorkFactory>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { uow_factory, event_publisher }
    }
}

#[async_trait]
impl CommandHandler<CreateClientCommand> for CreateClientHandler {
    async fn handle(&self, command: CreateClientCommand) -> AppResult<(String, Option<String>)> {
        info!("Creating OAuth client: {}", command.name);

        let tenant_id = TenantId::from_str(&command.tenant_id)
            .map_err(|e| AppError::validation(format!("Invalid tenant_id: {}", e)))?;

        // 开始事务
        let uow = self.uow_factory.begin().await?;

        // 查找所有者
        let (users, _) = uow.users().list(
            &tenant_id,
            None,  // status
            None,  // search
            &[],  // role_ids
            1,   // page
            1,   // page_size
        ).await.unwrap_or((vec![], 0));

        let owner_id = if users.is_empty() {
            // No users yet, create a temporary one for this tenant
            use crate::domain::user::User;
            use crate::domain::value_objects::{Email, Username};
            use crate::domain::services::auth::PasswordService;

            let username = Username::new(format!("admin_{}", &tenant_id.0.to_string()[..8]))
                .or_else(|_| Username::new("admin".to_string()))
                .map_err(|e| AppError::validation(format!("Failed to create username: {}", e)))?;
            let email = Email::new(format!("admin_{}@temp.local", &tenant_id.0.to_string()[..8]))
                .or_else(|_| Email::new("admin@temp.local".to_string()))
                .map_err(|e| AppError::validation(format!("Failed to create email: {}", e)))?;
            let password_hash = PasswordService::hash_password("temp_password_123")
                .map_err(|e| AppError::validation(format!("Failed to hash password: {}", e)))?;

            let user = User::new(username, email, password_hash, tenant_id.clone());
            uow.users().save(&user).await
                .map_err(|e| AppError::internal(format!("Failed to create default user: {}", e)))?;
            user.id.clone()
        } else {
            // Use the first user in the tenant
            users[0].id.clone()
        };

        let client_type = if command.public_client {
             OAuthClientType::Public
        } else {
             OAuthClientType::Confidential
        };

        let client_name = command.name.clone();
        let mut client = OAuthClient::new(
            tenant_id.clone(),
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
            client.set_client_secret(secret.clone()); 
            Some(secret)
        };

        uow.oauth_clients().save(&client).await?;

        // 提交事务
        uow.commit().await?;

        // 发布 OAuth 客户端创建事件
        let event = IamDomainEvent::OAuthClientCreated {
            client_id: client.id.0.to_string(),
            tenant_id: tenant_id.clone(),
            name: client_name,
            timestamp: Utc::now(),
        };
        self.event_publisher.publish(event).await;

        info!("OAuth client created: {}", client.id);

        Ok((client.id.0.to_string(), plain_secret))
    }
}
