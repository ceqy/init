//! PostgreSQL Unit of Work 实现
//!
//! 使用 SQLx Transaction 提供事务协调能力。

use async_trait::async_trait;
use cuba_errors::{AppError, AppResult};
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::repositories::auth::{
    BackupCodeRepository, LoginLogRepository, PasswordResetRepository, SessionRepository,
    WebAuthnCredentialRepository,
};
use crate::domain::repositories::oauth::{
    AccessTokenRepository, AuthorizationCodeRepository, OAuthClientRepository,
    RefreshTokenRepository,
};
use crate::domain::repositories::user::{
    EmailVerificationRepository, PhoneVerificationRepository, TenantRepository, UserRepository,
};
use crate::domain::unit_of_work::{UnitOfWork, UnitOfWorkFactory};

use super::tx_repositories::{
    TxAccessTokenRepository, TxAuthorizationCodeRepository, TxBackupCodeRepository,
    TxEmailVerificationRepository, TxLoginLogRepository, TxOAuthClientRepository,
    TxPasswordResetRepository, TxPhoneVerificationRepository, TxRefreshTokenRepository,
    TxSessionRepository, TxTenantRepository, TxUserRepository, TxWebAuthnCredentialRepository,
};

/// PostgreSQL Unit of Work 工厂
pub struct PostgresUnitOfWorkFactory {
    pool: PgPool,
}

impl PostgresUnitOfWorkFactory {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UnitOfWorkFactory for PostgresUnitOfWorkFactory {
    async fn begin(&self) -> AppResult<Box<dyn UnitOfWork>> {
        let tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        Ok(Box::new(PostgresUnitOfWork::new(tx)))
    }
}

/// PostgreSQL Unit of Work 实现
///
/// 持有一个事务和所有相关的 Repository 实例。
/// 所有 Repository 操作都在同一个事务中执行。
pub struct PostgresUnitOfWork {
    /// 使用 Arc<Mutex> 包装 Transaction，使其可以被多个 Repository 共享
    tx: Arc<Mutex<Option<Transaction<'static, Postgres>>>>,

    // 事务感知的 Repositories
    user_repo: TxUserRepository,
    tenant_repo: TxTenantRepository,
    email_verification_repo: TxEmailVerificationRepository,
    phone_verification_repo: TxPhoneVerificationRepository,
    session_repo: TxSessionRepository,
    backup_code_repo: TxBackupCodeRepository,
    login_log_repo: TxLoginLogRepository,
    password_reset_repo: TxPasswordResetRepository,
    webauthn_credential_repo: TxWebAuthnCredentialRepository,
    oauth_client_repo: TxOAuthClientRepository,
    access_token_repo: TxAccessTokenRepository,
    refresh_token_repo: TxRefreshTokenRepository,
    authorization_code_repo: TxAuthorizationCodeRepository,
}

impl PostgresUnitOfWork {
    fn new(tx: Transaction<'static, Postgres>) -> Self {
        let tx = Arc::new(Mutex::new(Some(tx)));

        Self {
            tx: tx.clone(),
            user_repo: TxUserRepository::new(tx.clone()),
            tenant_repo: TxTenantRepository::new(tx.clone()),
            email_verification_repo: TxEmailVerificationRepository::new(tx.clone()),
            phone_verification_repo: TxPhoneVerificationRepository::new(tx.clone()),
            session_repo: TxSessionRepository::new(tx.clone()),
            backup_code_repo: TxBackupCodeRepository::new(tx.clone()),
            login_log_repo: TxLoginLogRepository::new(tx.clone()),
            password_reset_repo: TxPasswordResetRepository::new(tx.clone()),
            webauthn_credential_repo: TxWebAuthnCredentialRepository::new(tx.clone()),
            oauth_client_repo: TxOAuthClientRepository::new(tx.clone()),
            access_token_repo: TxAccessTokenRepository::new(tx.clone()),
            refresh_token_repo: TxRefreshTokenRepository::new(tx.clone()),
            authorization_code_repo: TxAuthorizationCodeRepository::new(tx.clone()),
        }
    }
}

#[async_trait]
impl UnitOfWork for PostgresUnitOfWork {
    // ============ User Repositories ============

    fn users(&self) -> &dyn UserRepository {
        &self.user_repo
    }

    fn tenants(&self) -> &dyn TenantRepository {
        &self.tenant_repo
    }

    fn email_verifications(&self) -> &dyn EmailVerificationRepository {
        &self.email_verification_repo
    }

    fn phone_verifications(&self) -> &dyn PhoneVerificationRepository {
        &self.phone_verification_repo
    }

    // ============ Auth Repositories ============

    fn sessions(&self) -> &dyn SessionRepository {
        &self.session_repo
    }

    fn backup_codes(&self) -> &dyn BackupCodeRepository {
        &self.backup_code_repo
    }

    fn login_logs(&self) -> &dyn LoginLogRepository {
        &self.login_log_repo
    }

    fn password_resets(&self) -> &dyn PasswordResetRepository {
        &self.password_reset_repo
    }

    fn webauthn_credentials(&self) -> &dyn WebAuthnCredentialRepository {
        &self.webauthn_credential_repo
    }

    // ============ OAuth Repositories ============

    fn oauth_clients(&self) -> &dyn OAuthClientRepository {
        &self.oauth_client_repo
    }

    fn access_tokens(&self) -> &dyn AccessTokenRepository {
        &self.access_token_repo
    }

    fn refresh_tokens(&self) -> &dyn RefreshTokenRepository {
        &self.refresh_token_repo
    }

    fn authorization_codes(&self) -> &dyn AuthorizationCodeRepository {
        &self.authorization_code_repo
    }

    // ============ Transaction Control ============

    async fn commit(self: Box<Self>) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .take()
            .ok_or_else(|| AppError::internal("Transaction already consumed"))?;

        tx.commit()
            .await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    async fn rollback(self: Box<Self>) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .take()
            .ok_or_else(|| AppError::internal("Transaction already consumed"))?;

        tx.rollback()
            .await
            .map_err(|e| AppError::database(format!("Failed to rollback transaction: {}", e)))?;

        Ok(())
    }
}
