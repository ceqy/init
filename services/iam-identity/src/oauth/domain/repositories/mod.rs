//! OAuth 仓储接口

pub mod access_token_repository;
pub mod authorization_code_repository;
pub mod oauth_client_repository;
pub mod refresh_token_repository;

pub use access_token_repository::AccessTokenRepository;
pub use authorization_code_repository::AuthorizationCodeRepository;
pub use oauth_client_repository::OAuthClientRepository;
pub use refresh_token_repository::RefreshTokenRepository;
