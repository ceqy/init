use std::str::FromStr;
use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::info;

use cuba_common::TenantId;
use cuba_cqrs_core::CommandHandler;

use crate::application::commands::oauth::{AuthorizeCommand, CreateClientCommand, TokenCommand};
use crate::application::handlers::oauth::{
    AuthorizeHandler, CreateClientHandler, TokenHandler,
};
use crate::domain::oauth::OAuthClientId;
use crate::domain::repositories::oauth::OAuthClientRepository;
use crate::domain::services::oauth::OAuthService;

use super::oauth_proto::{
    o_auth_service_server::OAuthService as OAuthServiceTrait, AuthorizeRequest, AuthorizeResponse,
    CreateClientRequest, CreateClientResponse, DeleteClientRequest, GetClientRequest,
    GetClientResponse, IntrospectTokenRequest, IntrospectTokenResponse, ListClientsRequest,
    ListClientsResponse, RefreshTokenRequest, RevokeTokenRequest, TokenRequest, TokenResponse,
    UpdateClientRequest, UpdateClientResponse,
};

pub struct OAuthServiceImpl {
    client_repo: Arc<dyn OAuthClientRepository>,
    oauth_service: Arc<OAuthService>,
    create_client_handler: Arc<CreateClientHandler>,
    authorize_handler: Arc<AuthorizeHandler>,
    token_handler: Arc<TokenHandler>,
}

impl OAuthServiceImpl {
    pub fn new(
        client_repo: Arc<dyn OAuthClientRepository>,
        oauth_service: Arc<OAuthService>,
        create_client_handler: Arc<CreateClientHandler>,
        authorize_handler: Arc<AuthorizeHandler>,
        token_handler: Arc<TokenHandler>,
    ) -> Self {
        Self {
            client_repo,
            oauth_service,
            create_client_handler,
            authorize_handler,
            token_handler,
        }
    }

    fn extract_tenant_id(request: &Request<impl std::fmt::Debug>) -> Result<String, Status> {
        request
            .metadata()
            .get("tenant-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .ok_or_else(|| Status::invalid_argument("tenant_id is required in metadata"))
    }
}

#[tonic::async_trait]
impl OAuthServiceTrait for OAuthServiceImpl {
    async fn create_client(
        &self,
        request: Request<CreateClientRequest>,
    ) -> Result<Response<CreateClientResponse>, Status> {
        let tenant_id = Self::extract_tenant_id(&request)?;
        let req = request.into_inner();

        info!("Creating OAuth client: {}", req.name);

        let command = CreateClientCommand {
            name: req.name,
            redirect_uris: req.redirect_uris,
            grant_types: req.grant_types,
            scopes: req.scopes,
            client_secret: if req.client_secret.is_empty() {
                None
            } else {
                Some(req.client_secret)
            },
            public_client: req.public_client,
            tenant_id: tenant_id.clone(),
        };

        let (client_id, client_secret) = self
            .create_client_handler
            .handle(command)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let client = self
            .client_repo
            .find_by_id(
                &OAuthClientId::from_str(&client_id).unwrap(),
                &TenantId::from_str(&tenant_id.clone()).unwrap(),
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Client not found"))?;

        Ok(Response::new(CreateClientResponse {
            client_id,
            client_secret: client_secret.unwrap_or_default(),
            client: Some(super::oauth_proto::OAuthClient {
                id: client.id.0.to_string(),
                name: client.name,
                redirect_uris: client.redirect_uris,
                grant_types: client.grant_types.iter().map(|g| g.to_string()).collect(),
                scopes: client.scopes,
                public_client: client.public_client,
                tenant_id: client.tenant_id.0.to_string(),
                created_at: Some(prost_types::Timestamp {
                    seconds: client.created_at.timestamp(),
                    nanos: client.created_at.timestamp_subsec_nanos() as i32,
                }),
            }),
        }))
    }

    async fn get_client(
        &self,
        request: Request<GetClientRequest>,
    ) -> Result<Response<GetClientResponse>, Status> {
        let tenant_id = Self::extract_tenant_id(&request)?;
        let req = request.into_inner();

        let client_id = OAuthClientId::from_str(&req.client_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid client_id: {}", e)))?;

        let tenant_id = TenantId::from_str(&tenant_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid tenant_id: {}", e)))?;

        let client = self
            .client_repo
            .find_by_id(&client_id, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Client not found"))?;

        Ok(Response::new(GetClientResponse {
            client: Some(super::oauth_proto::OAuthClient {
                id: client.id.0.to_string(),
                name: client.name,
                redirect_uris: client.redirect_uris,
                grant_types: client.grant_types.iter().map(|g| g.to_string()).collect(),
                scopes: client.scopes,
                public_client: client.public_client,
                tenant_id: client.tenant_id.0.to_string(),
                created_at: Some(prost_types::Timestamp {
                    seconds: client.created_at.timestamp(),
                    nanos: client.created_at.timestamp_subsec_nanos() as i32,
                }),
            }),
        }))
    }

    async fn update_client(
        &self,
        request: Request<UpdateClientRequest>,
    ) -> Result<Response<UpdateClientResponse>, Status> {
        let tenant_id = Self::extract_tenant_id(&request)?;
        let req = request.into_inner();

        let client_id = OAuthClientId::from_str(&req.client_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid client_id: {}", e)))?;

        let tenant_id = TenantId::from_str(&tenant_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid tenant_id: {}", e)))?;

        let mut client = self
            .client_repo
            .find_by_id(&client_id, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Client not found"))?;

        client.name = req.name;
        client.redirect_uris = req.redirect_uris;
        if !req.grant_types.is_empty() {
             use crate::domain::oauth::GrantType;
             client.grant_types = req.grant_types.iter().map(|s| {
                 match s.as_str() {
                    "authorization_code" => GrantType::AuthorizationCode,
                    "client_credentials" => GrantType::ClientCredentials,
                    "refresh_token" => GrantType::RefreshToken,
                    "implicit" => GrantType::Implicit,
                    "password" => GrantType::Password,
                    _ => GrantType::AuthorizationCode, // Fallback or Error
                 }
             }).collect();
        }
        client.scopes = req.scopes;
        client.updated_at = chrono::Utc::now();

        self.client_repo
            .update(&client)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateClientResponse {
            client: Some(super::oauth_proto::OAuthClient {
                id: client.id.0.to_string(),
                name: client.name,
                redirect_uris: client.redirect_uris,
                grant_types: client.grant_types.iter().map(|g| g.to_string()).collect(),
                scopes: client.scopes,
                public_client: client.public_client,
                tenant_id: client.tenant_id.0.to_string(),
                created_at: Some(prost_types::Timestamp {
                    seconds: client.created_at.timestamp(),
                    nanos: client.created_at.timestamp_subsec_nanos() as i32,
                }),
            }),
        }))
    }

    async fn delete_client(
        &self,
        request: Request<DeleteClientRequest>,
    ) -> Result<Response<()>, Status> {
        let tenant_id = Self::extract_tenant_id(&request)?;
        let req = request.into_inner();

        let client_id = OAuthClientId::from_str(&req.client_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid client_id: {}", e)))?;

        let tenant_id = TenantId::from_str(&tenant_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid tenant_id: {}", e)))?;

        self.client_repo
            .delete(&client_id, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn list_clients(
        &self,
        request: Request<ListClientsRequest>,
    ) -> Result<Response<ListClientsResponse>, Status> {
        let tenant_id = Self::extract_tenant_id(&request)?;
        let req = request.into_inner();

        let tenant_id = TenantId::from_str(&tenant_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid tenant_id: {}", e)))?;

        let page = if req.page > 0 { req.page as i64 } else { 1 };
        let page_size = if req.page_size > 0 && req.page_size <= 100 {
            req.page_size as i64
        } else {
            20
        };

        let clients = self
            .client_repo
            .list_by_tenant(&tenant_id, page, page_size)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let total = self
            .client_repo
            .count_by_tenant(&tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_clients = clients
            .into_iter()
            .map(|client| super::oauth_proto::OAuthClient {
                id: client.id.0.to_string(),
                name: client.name,
                redirect_uris: client.redirect_uris,
                grant_types: client.grant_types.iter().map(|g| g.to_string()).collect(),
                scopes: client.scopes,
                public_client: client.public_client,
                tenant_id: client.tenant_id.0.to_string(),
                created_at: Some(prost_types::Timestamp {
                    seconds: client.created_at.timestamp(),
                    nanos: client.created_at.timestamp_subsec_nanos() as i32,
                }),
            })
            .collect();

        Ok(Response::new(ListClientsResponse {
            clients: proto_clients,
            total,
        }))
    }

    async fn authorize(
        &self,
        request: Request<AuthorizeRequest>,
    ) -> Result<Response<AuthorizeResponse>, Status> {
        let tenant_id = Self::extract_tenant_id(&request)?;
        let req = request.into_inner();

        info!("Authorize request for client: {}", req.client_id);

        let command = AuthorizeCommand {
            client_id: req.client_id,
            redirect_uri: req.redirect_uri,
            response_type: req.response_type,
            scope: req.scope,
            state: if req.state.is_empty() {
                None
            } else {
                Some(req.state)
            },
            code_challenge: if req.code_challenge.is_empty() {
                None
            } else {
                Some(req.code_challenge)
            },
            code_challenge_method: if req.code_challenge_method.is_empty() {
                None
            } else {
                Some(req.code_challenge_method)
            },
            user_id: req.user_id,
            tenant_id,
        };

        let (code, state) = self
            .authorize_handler
            .handle(command)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(AuthorizeResponse {
            code,
            state: state.unwrap_or_default(),
        }))
    }

    async fn token(
        &self,
        request: Request<TokenRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        let tenant_id = Self::extract_tenant_id(&request)?;
        let req = request.into_inner();

        info!("Token request: grant_type={}", req.grant_type);

        let command = TokenCommand {
            grant_type: req.grant_type,
            code: if req.code.is_empty() {
                None
            } else {
                Some(req.code)
            },
            redirect_uri: if req.redirect_uri.is_empty() {
                None
            } else {
                Some(req.redirect_uri)
            },
            client_id: req.client_id,
            client_secret: if req.client_secret.is_empty() {
                None
            } else {
                Some(req.client_secret)
            },
            code_verifier: if req.code_verifier.is_empty() {
                None
            } else {
                Some(req.code_verifier)
            },
            refresh_token: if req.refresh_token.is_empty() {
                None
            } else {
                Some(req.refresh_token)
            },
            scope: if req.scope.is_empty() {
                None
            } else {
                Some(req.scope)
            },
            tenant_id,
        };

        let (access_token, refresh_token, expires_in) = self
            .token_handler
            .handle(command)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in,
            refresh_token,
            scope: "".to_string(),
        }))
    }

    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        let tenant_id = Self::extract_tenant_id(&request)?;
        let req = request.into_inner();

        let command = TokenCommand {
            grant_type: "refresh_token".to_string(),
            code: None,
            redirect_uri: None,
            client_id: req.client_id,
            client_secret: if req.client_secret.is_empty() {
                None
            } else {
                Some(req.client_secret)
            },
            code_verifier: None,
            refresh_token: Some(req.refresh_token),
            scope: if req.scope.is_empty() {
                None
            } else {
                Some(req.scope)
            },
            tenant_id,
        };

        let (access_token, refresh_token, expires_in) = self
            .token_handler
            .handle(command)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in,
            refresh_token,
            scope: "".to_string(),
        }))
    }

    async fn revoke_token(
        &self,
        request: Request<RevokeTokenRequest>,
    ) -> Result<Response<()>, Status> {
        let tenant_id = Self::extract_tenant_id(&request)?;
        let req = request.into_inner();

        let tenant_id = TenantId::from_str(&tenant_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid tenant_id: {}", e)))?;

        self.oauth_service
            .revoke_token(&req.token, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn introspect_token(
        &self,
        request: Request<IntrospectTokenRequest>,
    ) -> Result<Response<IntrospectTokenResponse>, Status> {
        let tenant_id = Self::extract_tenant_id(&request)?;
        let req = request.into_inner();

        let tenant_id = TenantId::from_str(&tenant_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid tenant_id: {}", e)))?;

        let token = self
            .oauth_service
            .introspect_token(&req.token, &tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if let Some(token) = token {
            Ok(Response::new(IntrospectTokenResponse {
                active: true,
                scope: token.scopes.join(" "),
                client_id: token.client_id.0.to_string(),
                username: "".to_string(),
                token_type: "Bearer".to_string(),
                exp: token.expires_at.timestamp(),
                iat: token.created_at.timestamp(),
                sub: token.user_id.map(|u| u.0.to_string()).unwrap_or_default(),
            }))
        } else {
            Ok(Response::new(IntrospectTokenResponse {
                active: false,
                scope: "".to_string(),
                client_id: "".to_string(),
                username: "".to_string(),
                token_type: "".to_string(),
                exp: 0,
                iat: 0,
                sub: "".to_string(),
            }))
        }
    }
}
