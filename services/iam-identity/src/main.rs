//! IAM Identity Service - 身份服务入口
//!
//! 使用 cuba-bootstrap 统一启动模式

mod api;
mod application;
mod auth;
mod config;
mod domain;
mod error;
mod infrastructure;
mod oauth;
mod shared;
mod user;

use std::sync::Arc;

use auth::api::grpc::{AuthServiceImpl, AuthServiceServer};
use auth::domain::repositories::{
    BackupCodeRepository, PasswordResetRepository, SessionRepository, WebAuthnCredentialRepository,
};
use auth::domain::services::{TotpService, WebAuthnService};
use auth::infrastructure::cache::{AuthCache, RedisAuthCache};
use auth::infrastructure::persistence::{
    PostgresBackupCodeRepository, PostgresPasswordResetRepository, PostgresSessionRepository,
    PostgresWebAuthnCredentialRepository,
};
use async_trait::async_trait;
use cuba_adapter_email::{EmailClient, EmailSender};
use cuba_bootstrap::{run_with_services, Infrastructure};
use cuba_config::PasswordResetConfig;
use cuba_ports::CachePort;
use shared::application::handlers::{
    SendEmailVerificationHandler, SendPhoneVerificationHandler, VerifyEmailHandler,
    VerifyPhoneHandler,
};
use shared::domain::repositories::{EmailVerificationRepository, PhoneVerificationRepository, UserRepository};
use shared::domain::services::{EmailVerificationService, PhoneVerificationService, SmsSender};
use shared::infrastructure::persistence::{
    PostgresEmailVerificationRepository, PostgresPhoneVerificationRepository, PostgresUserRepository,
};
use tonic::transport::Server;
use tonic_reflection::server::Builder as ReflectionBuilder;
use user::api::grpc::{proto::user_service_server::UserServiceServer, UserServiceImpl};

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
            user_repo, 
            token_service,
            send_email_verification_handler,
            verify_email_handler,
            send_phone_verification_handler,
            verify_phone_handler,
        );

        // 注册多个服务并启动
        let addr = format!("{}:{}", config.server.host, config.server.port)
            .parse()
            .map_err(|e| cuba_errors::AppError::internal(format!("Invalid address: {}", e)))?;

        // 构建反射服务
        let reflection_service = ReflectionBuilder::configure()
            .register_encoded_file_descriptor_set(auth::api::grpc::proto::FILE_DESCRIPTOR_SET)
            .register_encoded_file_descriptor_set(user::api::grpc::proto::FILE_DESCRIPTOR_SET)
            .build_v1()
            .map_err(|e| cuba_errors::AppError::internal(format!("Failed to build reflection service: {}", e)))?;

        server
            .add_service(AuthServiceServer::new(auth_service))
            .add_service(UserServiceServer::new(user_service))
            .add_service(reflection_service)
            .serve_with_shutdown(addr, cuba_bootstrap::shutdown_signal())
            .await
            .map_err(|e| cuba_errors::AppError::internal(format!("Server error: {}", e)))?;

        Ok(())
    })
    .await
}
