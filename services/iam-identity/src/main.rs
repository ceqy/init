#![allow(dead_code)]
#![allow(unused_imports)]

//! IAM Identity Service - 身份服务入口
//!
//! 使用 cuba-bootstrap 统一启动模式

mod api;
mod application;
mod config;
mod domain;
mod error;
mod infrastructure;

use std::sync::Arc;

use api::grpc::{
    auth_service::AuthServiceImpl,
    oauth_service::OAuthServiceImpl,
    user_service::UserServiceImpl,
    auth_proto::auth_service_server::AuthServiceServer,
    oauth_proto::o_auth_service_server::OAuthServiceServer,
    user_proto::user_service_server::UserServiceServer,
};
// Auth handlers are used indirectly via the service - keeping module export
use application::handlers::oauth::{AuthorizeHandler, CreateClientHandler, TokenHandler};
use application::handlers::user::{
    SendEmailVerificationHandler, SendPhoneVerificationHandler, VerifyEmailHandler,
    VerifyPhoneHandler,
};
use domain::repositories::auth::{
    BackupCodeRepository, PasswordResetRepository, SessionRepository, WebAuthnCredentialRepository,
};
use domain::repositories::oauth::{
    AccessTokenRepository, AuthorizationCodeRepository, OAuthClientRepository, RefreshTokenRepository,
};
use domain::repositories::user::{EmailVerificationRepository, PhoneVerificationRepository, UserRepository};
use domain::services::auth::{TotpService, WebAuthnService};
use domain::services::oauth::OAuthService;
use domain::services::user::{EmailVerificationService, PhoneVerificationService, SmsSender};
use infrastructure::cache::{AuthCache, RedisAuthCache};
use infrastructure::persistence::auth::{
    PostgresBackupCodeRepository, PostgresPasswordResetRepository, PostgresSessionRepository,
    PostgresWebAuthnCredentialRepository,
};
use infrastructure::persistence::oauth::{
    PostgresAccessTokenRepository, PostgresAuthorizationCodeRepository, PostgresOAuthClientRepository,
    PostgresRefreshTokenRepository,
};
use infrastructure::persistence::user::{
    PostgresEmailVerificationRepository, PostgresPhoneVerificationRepository, PostgresUserRepository,
};
use infrastructure::events::{EventPublisher, LoggingEventPublisher};
use async_trait::async_trait;
use cuba_adapter_email::{EmailClient, EmailSender};
use cuba_bootstrap::{run_with_services, Infrastructure};

use cuba_ports::CachePort;
use tonic::transport::Server;
use tonic_reflection::server::Builder as ReflectionBuilder;

// Temporary NoOpSmsSender implementation for compilation
struct NoOpSmsSender;

#[async_trait]
impl SmsSender for NoOpSmsSender {
    async fn send_verification_code(&self, _phone: &str, _code: &str) -> cuba_errors::AppResult<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_with_services("config", |infra: Infrastructure, mut server: Server| async move {
        // 从 Infrastructure 获取资源
        let pool = infra.postgres_pool();
        let token_service = infra.token_service();
        let config = infra.config();

        // 组装 Cache（依赖 CachePort trait）
        let cache: Arc<dyn CachePort> = Arc::new(infra.redis_cache());
        let auth_cache: Arc<dyn AuthCache> = Arc::new(RedisAuthCache::new(cache));

        // 组装事件发布器
        let event_publisher: Arc<dyn EventPublisher> = Arc::new(LoggingEventPublisher);

        // 组装 TOTP 服务
        let totp_service = Arc::new(TotpService::new("Cuba ERP".to_string()));

        // 组装邮件客户端
        let email_config = cuba_adapter_email::EmailConfig {
            smtp_host: config.email.smtp_host.clone(),
            smtp_port: config.email.smtp_port,
            username: config.email.username.clone(),
            password: config.email.password.clone(),
            from_email: config.email.from_email.clone(),
            from_name: config.email.from_name.clone(),
            use_tls: config.email.use_tls,
            timeout_secs: config.email.timeout_secs,
        };
        let email_client = Arc::new(EmailClient::new(email_config));
        let email_sender: Arc<dyn EmailSender> = email_client;

        // 密码重置配置
        let password_reset_config = config.password_reset.clone();

        // 组装 Repositories（依赖 domain trait）
        let user_repo: Arc<dyn UserRepository> =
            Arc::new(PostgresUserRepository::new(pool.clone()));
        let session_repo: Arc<dyn SessionRepository> =
            Arc::new(PostgresSessionRepository::new(pool.clone()));
        let backup_code_repo: Arc<dyn BackupCodeRepository> =
            Arc::new(PostgresBackupCodeRepository::new(pool.clone()));
        let password_reset_repo: Arc<dyn PasswordResetRepository> =
            Arc::new(PostgresPasswordResetRepository::new(pool.clone()));
        let webauthn_credential_repo: Arc<dyn WebAuthnCredentialRepository> =
            Arc::new(PostgresWebAuthnCredentialRepository::new(pool.clone()));

        // 组装 WebAuthn 服务
        let rp_id = config.webauthn.rp_id.clone();
        let rp_origin = config.webauthn.rp_origin
            .parse()
            .map_err(|e| cuba_errors::AppError::internal(format!("Invalid RP origin: {}", e)))?;
        
        let webauthn_service = Arc::new(
            WebAuthnService::new(rp_id, rp_origin, webauthn_credential_repo)
                .map_err(|e| cuba_errors::AppError::internal(format!("Failed to create WebAuthn service: {}", e)))?,
        );

        // 组装 AuthService
        let auth_service = AuthServiceImpl::new(
            user_repo.clone(),
            session_repo,
            backup_code_repo,
            password_reset_repo,
            token_service.clone(),
            totp_service,
            webauthn_service,
            email_sender.clone(),
            auth_cache,
            config.jwt.refresh_expires_in as i64,
            password_reset_config,
        );

        // 组装 UserService
        // 组装 User 相关的 Repositories 和 Services
        let email_verification_repo: Arc<dyn EmailVerificationRepository> =
            Arc::new(PostgresEmailVerificationRepository::new(pool.clone()));
        let phone_verification_repo: Arc<dyn PhoneVerificationRepository> =
            Arc::new(PostgresPhoneVerificationRepository::new(pool.clone()));

        let email_verification_service = Arc::new(EmailVerificationService::new(
            email_verification_repo.clone(),
            user_repo.clone(),
            email_sender.clone(),
        ));

        let sms_sender = Arc::new(NoOpSmsSender);
        let phone_verification_service = Arc::new(PhoneVerificationService::new(
             phone_verification_repo.clone(),
             user_repo.clone(),
             sms_sender,
        ));
        
        // Handlers
        let send_email_verification_handler = Arc::new(SendEmailVerificationHandler::new(
            email_verification_service.clone(),
        ));
        let verify_email_handler = Arc::new(VerifyEmailHandler::new(
            email_verification_service.clone(),
        ));
        let send_phone_verification_handler = Arc::new(SendPhoneVerificationHandler::new(
            phone_verification_service.clone(),
        ));
        let verify_phone_handler = Arc::new(VerifyPhoneHandler::new(
            phone_verification_service.clone(),
        ));

        // 组装 UserService
        let user_service = UserServiceImpl::new(
            user_repo.clone(),
            token_service.clone(),
            send_email_verification_handler,
            verify_email_handler,
            send_phone_verification_handler,
            verify_phone_handler,
        );

        // 组装 OAuth Repositories
        let oauth_client_repo: Arc<dyn OAuthClientRepository> =
            Arc::new(PostgresOAuthClientRepository::new(pool.clone()));
        let authorization_code_repo: Arc<dyn AuthorizationCodeRepository> =
            Arc::new(PostgresAuthorizationCodeRepository::new(pool.clone()));
        let access_token_repo: Arc<dyn AccessTokenRepository> =
            Arc::new(PostgresAccessTokenRepository::new(pool.clone()));
        let refresh_token_repo: Arc<dyn RefreshTokenRepository> =
            Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));

        // 组装 OAuthService
        let oauth_service = Arc::new(OAuthService::new(
            oauth_client_repo.clone(),
            authorization_code_repo,
            access_token_repo,
            refresh_token_repo,
        ));

        // 组装 OAuthServiceImpl
        let create_client_handler = Arc::new(CreateClientHandler::new(
            oauth_client_repo.clone(),
            user_repo.clone(),
            event_publisher.clone(),
        ));
        let authorize_handler = Arc::new(AuthorizeHandler::new(oauth_service.clone()));
        let token_handler = Arc::new(TokenHandler::new(oauth_service.clone()));

        let oauth_service_impl = OAuthServiceImpl::new(oauth_client_repo, oauth_service, create_client_handler, authorize_handler, token_handler);

        // 注册多个服务并启动
        let addr = format!("{}:{}", config.server.host, config.server.port)
            .parse()
            .map_err(|e| cuba_errors::AppError::internal(format!("Invalid address: {}", e)))?;

        // 构建反射服务
        let reflection_service = ReflectionBuilder::configure()
            .register_encoded_file_descriptor_set(api::grpc::auth_proto::FILE_DESCRIPTOR_SET)
            .register_encoded_file_descriptor_set(api::grpc::user_proto::FILE_DESCRIPTOR_SET)
            .register_encoded_file_descriptor_set(api::grpc::oauth_proto::FILE_DESCRIPTOR_SET)
            .build_v1()
            .map_err(|e| cuba_errors::AppError::internal(format!("Failed to build reflection service: {}", e)))?;

        server
            .add_service(AuthServiceServer::new(auth_service))
            .add_service(UserServiceServer::new(user_service))
            .add_service(OAuthServiceServer::new(oauth_service_impl))
            .add_service(reflection_service)
            .serve_with_shutdown(addr, cuba_bootstrap::shutdown_signal())
            .await
            .map_err(|e| cuba_errors::AppError::internal(format!("Server error: {}", e)))?;

        Ok(())
    })
    .await
}
