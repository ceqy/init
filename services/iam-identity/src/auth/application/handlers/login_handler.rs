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
use crate::auth::domain::entities::Session;
use crate::auth::domain::repositories::SessionRepository;
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
    token_service: Arc<TokenService>,
    refresh_token_expires_in: i64,
}

impl LoginHandler {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        session_repo: Arc<dyn SessionRepository>,
        token_service: Arc<TokenService>,
        refresh_token_expires_in: i64,
    ) -> Self {
        Self {
            user_repo,
            session_repo,
            token_service,
            refresh_token_expires_in,
        }
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

        // 查找用户
        let user = self
            .user_repo
            .find_by_username(&username, &tenant_id)
            .await?
            .ok_or_else(|| AppError::unauthorized("Invalid credentials"))?;

        // 验证密码
        let valid = PasswordService::verify_password(&command.password, &user.password_hash)?;
        if !valid {
            return Err(AppError::unauthorized("Invalid credentials"));
        }

        // 检查用户状态
        if !user.is_active() {
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

        let mut session = Session::new(user.id.clone(), refresh_token_hash, expires_at);

        if let Some(device_info) = command.device_info {
            session = session.with_device_info(device_info);
        }
        if let Some(ip_address) = command.ip_address {
            session = session.with_ip_address(ip_address);
        }

        self.session_repo.save(&session).await?;

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
