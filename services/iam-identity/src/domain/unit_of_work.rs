//! Unit of Work 模式
//!
//! 提供跨多个 Repository 的事务协调能力，确保操作的原子性。

use async_trait::async_trait;
use errors::AppResult;

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

/// Unit of Work trait
///
/// 协调多个 Repository 在同一事务中的操作。
///
/// # 使用示例
///
/// ```ignore
/// let uow = uow_factory.begin().await?;
///
/// // 所有操作在同一事务中
/// uow.users().save(&user).await?;
/// uow.sessions().save(&session).await?;
///
/// // 提交事务
/// uow.commit().await?;
/// ```
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    // ============ User Repositories ============

    /// 获取用户 Repository
    fn users(&self) -> &dyn UserRepository;

    /// 获取租户 Repository
    fn tenants(&self) -> &dyn TenantRepository;

    /// 获取邮箱验证 Repository
    fn email_verifications(&self) -> &dyn EmailVerificationRepository;

    /// 获取手机验证 Repository
    fn phone_verifications(&self) -> &dyn PhoneVerificationRepository;

    // ============ Auth Repositories ============

    /// 获取会话 Repository
    fn sessions(&self) -> &dyn SessionRepository;

    /// 获取备份码 Repository
    fn backup_codes(&self) -> &dyn BackupCodeRepository;

    /// 获取登录日志 Repository
    fn login_logs(&self) -> &dyn LoginLogRepository;

    /// 获取密码重置 Repository
    fn password_resets(&self) -> &dyn PasswordResetRepository;

    /// 获取 WebAuthn 凭证 Repository
    fn webauthn_credentials(&self) -> &dyn WebAuthnCredentialRepository;

    // ============ OAuth Repositories ============

    /// 获取 OAuth 客户端 Repository
    fn oauth_clients(&self) -> &dyn OAuthClientRepository;

    /// 获取访问令牌 Repository
    fn access_tokens(&self) -> &dyn AccessTokenRepository;

    /// 获取刷新令牌 Repository
    fn refresh_tokens(&self) -> &dyn RefreshTokenRepository;

    /// 获取授权码 Repository
    fn authorization_codes(&self) -> &dyn AuthorizationCodeRepository;

    // ============ Transaction Control ============

    /// 提交事务
    ///
    /// 成功时所有更改将持久化，失败时自动回滚。
    async fn commit(self: Box<Self>) -> AppResult<()>;

    /// 回滚事务
    ///
    /// 撤销所有未提交的更改。
    async fn rollback(self: Box<Self>) -> AppResult<()>;
}

/// Unit of Work 工厂 trait
///
/// 用于创建新的 UnitOfWork 实例。
#[async_trait]
pub trait UnitOfWorkFactory: Send + Sync {
    /// 开始新的事务
    async fn begin(&self) -> AppResult<Box<dyn UnitOfWork>>;
}
