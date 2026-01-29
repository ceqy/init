//! 登录处理器

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use cuba_auth_core::TokenService;
use cuba_common::TenantId;
use cuba_cqrs_core::CommandHandler;
use cuba_errors::{AppError, AppResult};
use uuid::Uuid;

use crate::application::commands::auth::{LoginCommand, LoginResult};
use crate::application::dto::auth::TokenPair;
use crate::domain::auth::{
    DeviceInfo, LoginFailureReason, LoginLog, LoginResult as LogResult, Session,
};
use crate::domain::services::auth::PasswordService;
use crate::domain::services::auth::{BruteForceProtectionService, SuspiciousLoginDetector};
use crate::domain::value_objects::Username;
use crate::infrastructure::events::{EventPublisher, IamDomainEvent};

/// 简单的哈希函数（用于 refresh token）
fn sha256_simple(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

use crate::domain::unit_of_work::{UnitOfWork, UnitOfWorkFactory};

pub struct LoginHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
    token_service: Arc<TokenService>,
    brute_force_protection: Arc<BruteForceProtectionService>,
    suspicious_detector: Arc<SuspiciousLoginDetector>,
    event_publisher: Arc<dyn EventPublisher>,
    refresh_token_expires_in: i64,
}

impl LoginHandler {
    pub fn new(
        uow_factory: Arc<dyn UnitOfWorkFactory>,
        token_service: Arc<TokenService>,
        brute_force_protection: Arc<BruteForceProtectionService>,
        suspicious_detector: Arc<SuspiciousLoginDetector>,
        event_publisher: Arc<dyn EventPublisher>,
        refresh_token_expires_in: i64,
    ) -> Self {
        Self {
            uow_factory,
            token_service,
            brute_force_protection,
            suspicious_detector,
            event_publisher,
            refresh_token_expires_in,
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn log_login(
        &self,
        uow: &dyn UnitOfWork,
        tenant_id: &TenantId,
        username: &str,
        user_id: Option<&cuba_common::UserId>,
        ip: &str,
        user_agent: &str,
        result: LogResult,
        failure_reason: Option<LoginFailureReason>,
        is_suspicious: bool,
    ) -> AppResult<()> {
        let log = LoginLog {
            id: crate::domain::auth::LoginLogId::new(),
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

        uow.login_logs().save(&log).await
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

        // 开始事务
        let uow = self.uow_factory.begin().await?;

        // 查找用户
        let user = match uow.users().find_by_username(&username, &tenant_id).await? {
            Some(u) => u,
            None => {
                self.log_login(
                    uow.as_ref(),
                    &tenant_id,
                    &command.username,
                    None,
                    ip,
                    user_agent,
                    LogResult::Failed,
                    Some(LoginFailureReason::InvalidCredentials),
                    false,
                )
                .await?;
                uow.commit().await?; // 即使失败也记录日志
                return Err(AppError::unauthorized("Invalid credentials"));
            }
        };

        // 检测可疑登录
        let (is_suspicious, reasons) = self
            .suspicious_detector
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
        if self
            .brute_force_protection
            .is_locked(&user.id, &tenant_id)
            .await?
        {
            self.log_login(
                uow.as_ref(),
                &tenant_id,
                &command.username,
                Some(&user.id),
                ip,
                user_agent,
                LogResult::Failed,
                Some(LoginFailureReason::AccountLocked),
                is_suspicious,
            )
            .await?;
            uow.commit().await?;
            return Err(AppError::forbidden(
                "Account is locked due to too many failed attempts",
            ));
        }

        // 验证密码
        let valid = PasswordService::verify_password(&command.password, &user.password_hash)?;
        if !valid {
            self.log_login(
                uow.as_ref(),
                &tenant_id,
                &command.username,
                Some(&user.id),
                ip,
                user_agent,
                LogResult::Failed,
                Some(LoginFailureReason::InvalidCredentials),
                is_suspicious,
            )
            .await?;
            self.brute_force_protection
                .record_failed_attempt(&user.id, &tenant_id)
                .await?;
            uow.commit().await?;
            return Err(AppError::unauthorized("Invalid credentials"));
        }

        // 检查用户状态
        if !user.is_active() {
            self.log_login(
                uow.as_ref(),
                &tenant_id,
                &command.username,
                Some(&user.id),
                ip,
                user_agent,
                LogResult::Failed,
                Some(LoginFailureReason::AccountDisabled),
                is_suspicious,
            )
            .await?;
            uow.commit().await?;
            return Err(AppError::forbidden("User account is not active"));
        }

        // 检查是否需要 2FA
        if user.two_factor_enabled {
            // 创建临时会话用于 2FA
            let session_id = Uuid::now_v7().to_string();
            // 注意：此处可能也需要记录日志或保存临时会话状态
            uow.commit().await?;
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
            vec![],
            user.role_ids.clone(),
        )?;

        let refresh_token = self
            .token_service
            .generate_refresh_token(&user.id, &user.tenant_id)?;

        // 创建会话
        let refresh_token_hash = format!("{:x}", sha256_simple(&refresh_token));
        let expires_at = Utc::now() + Duration::seconds(self.refresh_token_expires_in);

        let mut session = Session::new(
            user.id.clone(),
            user.tenant_id.clone(),
            refresh_token_hash,
            expires_at,
        );

        if let Some(device_info) = command.device_info.clone() {
            session = session.with_device_info(device_info);
        }
        if let Some(ip_address) = command.ip_address.clone() {
            session = session.with_ip_address(ip_address);
        }

        uow.sessions().save(&session).await?;

        // 记录成功登录
        self.log_login(
            uow.as_ref(),
            &tenant_id,
            &command.username,
            Some(&user.id),
            ip,
            user_agent,
            LogResult::Success,
            None,
            is_suspicious,
        )
        .await?;
        self.brute_force_protection
            .record_successful_login(&user.id)
            .await?;

        // 提交事务
        uow.commit().await?;

        // 发布用户登录事件（在事务提交后）
        let login_event = IamDomainEvent::UserLoggedIn {
            user_id: user.id.clone(),
            tenant_id: user.tenant_id.clone(),
            ip_address: command.ip_address.clone(),
            user_agent: command.device_info.clone(),
            timestamp: Utc::now(),
        };
        self.event_publisher.publish(login_event).await;

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
