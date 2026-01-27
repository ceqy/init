//! 登录处理器

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use cuba_auth_core::TokenService;
use cuba_common::TenantId;
use cuba_cqrs_core::CommandHandler;
use cuba_errors::{AppError, AppResult};
use uuid::Uuid;

use crate::auth::application::commands::{LoginCommand, LoginResult};
use crate::auth::application::dto::TokenPair;
use crate::auth::application::services::{BruteForceProtectionService, SuspiciousLoginDetector};
use crate::auth::domain::entities::{DeviceInfo, LoginFailureReason, LoginLog, LoginResult as LogResult, Session};
use crate::auth::domain::repositories::{LoginLogRepository, SessionRepository};
use crate::auth::domain::services::PasswordService;
use crate::shared::domain::repositories::UserRepository;
use crate::shared::domain::value_objects::Username;

/// 简单的哈希函数（用于 refresh token）
fn sha256_simple(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

pub struct LoginHandler {
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn SessionRepository>,
    login_log_repo: Arc<dyn LoginLogRepository>,
    token_service: Arc<TokenService>,
    brute_force_protection: Arc<BruteForceProtectionService>,
    suspicious_detector: Arc<SuspiciousLoginDetector>,
    refresh_token_expires_in: i64,
}

impl LoginHandler {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        session_repo: Arc<dyn SessionRepository>,
        login_log_repo: Arc<dyn LoginLogRepository>,
        token_service: Arc<TokenService>,
        brute_force_protection: Arc<BruteForceProtectionService>,
        suspicious_detector: Arc<SuspiciousLoginDetector>,
        refresh_token_expires_in: i64,
    ) -> Self {
        Self {
            user_repo,
            session_repo,
            login_log_repo,
            token_service,
            brute_force_protection,
            suspicious_detector,
            refresh_token_expires_in,
        }
    }

    async fn log_login(&self, tenant_id: &TenantId, username: &str, user_id: Option<&cuba_common::UserId>, 
                       ip: &str, user_agent: &str, result: LogResult, failure_reason: Option<LoginFailureReason>,
                       is_suspicious: bool) -> AppResult<()> {
        let log = LoginLog {
            id: crate::auth::domain::entities::LoginLogId::new(),
            user_id: user_id.cloned(),
            tenant_id: tenant_id.clone(),
            username: username.to_string(),
            ip_address: ip.to_string(),
            user_agent: user_agent.to_string(),
            device_info: DeviceInfo::default(),
            result,
            failure_reason,
            country: None,
            city: None,
            is_suspicious,
            suspicious_reason: None,
            created_at: Utc::now(),
        };
        
        self.login_log_repo.save(&log).await
    }
}

#[async_trait]
impl CommandHandler<LoginCommand> for LoginHandler {
    async fn handle(&self, command: LoginCommand) -> AppResult<LoginResult> {
        let tenant_id = TenantId::from_uuid(
            Uuid::parse_str(&command.tenant_id)
                .map_err(|_| AppError::validation("Invalid tenant ID"))?,
        );

        let username = Username::new(&command.username)?;
        let ip = command.ip_address.as_deref().unwrap_or("unknown");
        let user_agent = command.device_info.as_deref().unwrap_or("unknown");

        // 查找用户
        let user = match self.user_repo.find_by_username(&username, &tenant_id).await? {
            Some(u) => u,
            None => {
                self.log_login(&tenant_id, &command.username, None, ip, user_agent, 
                              LogResult::Failed, Some(LoginFailureReason::InvalidCredentials), false).await?;
                return Err(AppError::unauthorized("Invalid credentials"));
            }
        };

        // 检测可疑登录
        let (is_suspicious, reasons) = self.suspicious_detector
            .is_suspicious(&user.id, &tenant_id, ip, None)
            .await?;

        if is_suspicious {
            tracing::warn!(
                user_id = %user.id,
                ip = %ip,
                reasons = ?reasons,
                "Suspicious login detected"
            );
        }

        // 检查账户锁定
        if self.brute_force_protection.is_locked(&user.id, &tenant_id).await? {
            self.log_login(&tenant_id, &command.username, Some(&user.id), ip, user_agent,
                          LogResult::Failed, Some(LoginFailureReason::AccountLocked), is_suspicious).await?;
            return Err(AppError::forbidden("Account is locked due to too many failed attempts"));
        }

        // 验证密码
        let valid = PasswordService::verify_password(&command.password, &user.password_hash)?;
        if !valid {
            self.log_login(&tenant_id, &command.username, Some(&user.id), ip, user_agent,
                          LogResult::Failed, Some(LoginFailureReason::InvalidCredentials), is_suspicious).await?;
            self.brute_force_protection.record_failed_attempt(&user.id, &tenant_id).await?;
            return Err(AppError::unauthorized("Invalid credentials"));
        }

        // 检查用户状态
        if !user.is_active() {
            self.log_login(&tenant_id, &command.username, Some(&user.id), ip, user_agent,
                          LogResult::Failed, Some(LoginFailureReason::AccountDisabled), is_suspicious).await?;
            return Err(AppError::forbidden("User account is not active"));
        }

        // 检查是否需要 2FA
        if user.two_factor_enabled {
            // 创建临时会话用于 2FA
            let session_id = Uuid::now_v7().to_string();
            return Ok(LoginResult {
                tokens: None,
                user_id: user.id.0.to_string(),
                require_2fa: true,
                session_id: Some(session_id),
            });
        }

        // 生成令牌
        let access_token = self.token_service.generate_access_token(
            &user.id,
            &user.tenant_id,
            vec![], // TODO: 从角色获取权限
            user.role_ids.clone(),
        )?;

        let refresh_token = self
            .token_service
            .generate_refresh_token(&user.id, &user.tenant_id)?;

        // 创建会话
        // 使用简单的哈希（生产环境应使用更安全的方式）
        let refresh_token_hash = format!("{:x}", sha256_simple(&refresh_token));
        let expires_at = Utc::now() + Duration::seconds(self.refresh_token_expires_in);

        let mut session = Session::new(user.id.clone(), user.tenant_id.clone(), refresh_token_hash, expires_at);

        if let Some(device_info) = command.device_info.clone() {
            session = session.with_device_info(device_info);
        }
        if let Some(ip_address) = command.ip_address.clone() {
            session = session.with_ip_address(ip_address);
        }

        self.session_repo.save(&session).await?;

        // 记录成功登录
        self.log_login(&tenant_id, &command.username, Some(&user.id), ip, user_agent,
                      LogResult::Success, None, is_suspicious).await?;
        self.brute_force_protection.record_successful_login(&user.id).await?;

        Ok(LoginResult {
            tokens: Some(TokenPair {
                access_token,
                refresh_token,
                expires_in: self.token_service.access_token_expires_in(),
                token_type: "Bearer".to_string(),
            }),
            user_id: user.id.0.to_string(),
            require_2fa: false,
            session_id: Some(session.id.0.to_string()),
        })
    }
}
