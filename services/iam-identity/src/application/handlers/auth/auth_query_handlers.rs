//! 认证查询处理器

use std::sync::Arc;

use async_trait::async_trait;
use cuba_cqrs_core::QueryHandler;
use cuba_errors::AppResult;
use tracing::info;

use crate::application::queries::auth::{
    ListUserSessionsQuery, SessionQueryResult,
    GetUser2FAStatusQuery, User2FAStatusResult,
    GetBackupCodeCountQuery, BackupCodeCountResult,
};
use crate::domain::auth::Session;
use crate::domain::repositories::auth::{SessionRepository, BackupCodeRepository, WebAuthnCredentialRepository};

/// 获取用户会话列表处理器
pub struct ListUserSessionsHandler {
    session_repository: Arc<dyn SessionRepository>,
}

impl ListUserSessionsHandler {
    pub fn new(session_repository: Arc<dyn SessionRepository>) -> Self {
        Self { session_repository }
    }
}

#[async_trait]
impl QueryHandler<ListUserSessionsQuery> for ListUserSessionsHandler {
    async fn handle(&self, query: ListUserSessionsQuery) -> AppResult<Vec<SessionQueryResult>> {
        info!(
            user_id = %query.user_id,
            tenant_id = %query.tenant_id,
            include_expired = query.include_expired,
            "Handling ListUserSessionsQuery"
        );

        let user_id = cuba_common::UserId::from_uuid(
            uuid::Uuid::parse_str(&query.user_id)
                .map_err(|e| cuba_errors::AppError::validation(e.to_string()))?
        );

        // 获取活跃会话
        let sessions: Vec<Session> = self.session_repository
            .find_active_by_user_id(&user_id, &query.tenant_id)
            .await?;

        Ok(sessions.into_iter().map(|s| SessionQueryResult {
            id: s.id.0.to_string(),
            user_id: s.user_id.0.to_string(),
            tenant_id: s.tenant_id.0.to_string(),
            device_info: s.device_info.clone(),
            ip_address: s.ip_address.clone(),
            user_agent: s.user_agent.clone(),
            is_active: !s.is_expired() && !s.revoked,
            created_at: s.created_at,
            expires_at: s.expires_at,
            last_activity_at: Some(s.last_activity_at),
        }).collect())
    }
}

/// 获取用户备份码数量处理器
pub struct GetBackupCodeCountHandler {
    backup_code_repository: Arc<dyn BackupCodeRepository>,
}

impl GetBackupCodeCountHandler {
    pub fn new(backup_code_repository: Arc<dyn BackupCodeRepository>) -> Self {
        Self { backup_code_repository }
    }
}

#[async_trait]
impl QueryHandler<GetBackupCodeCountQuery> for GetBackupCodeCountHandler {
    async fn handle(&self, query: GetBackupCodeCountQuery) -> AppResult<BackupCodeCountResult> {
        info!(
            user_id = %query.user_id,
            tenant_id = %query.tenant_id,
            "Handling GetBackupCodeCountQuery"
        );

        let user_id = cuba_common::UserId::from_uuid(
            uuid::Uuid::parse_str(&query.user_id)
                .map_err(|e| cuba_errors::AppError::validation(e.to_string()))?
        );

        let remaining = self.backup_code_repository
            .count_available_by_user_id(&user_id, &query.tenant_id)
            .await? as u32;

        // 假设总共有 10 个备份码
        let total = 10u32;
        let used = total.saturating_sub(remaining);

        Ok(BackupCodeCountResult {
            total,
            used,
            remaining,
        })
    }
}

/// 获取用户 2FA 状态处理器
pub struct GetUser2FAStatusHandler {
    backup_code_repository: Arc<dyn BackupCodeRepository>,
    webauthn_repository: Arc<dyn WebAuthnCredentialRepository>,
}

impl GetUser2FAStatusHandler {
    pub fn new(
        backup_code_repository: Arc<dyn BackupCodeRepository>,
        webauthn_repository: Arc<dyn WebAuthnCredentialRepository>,
    ) -> Self {
        Self { 
            backup_code_repository,
            webauthn_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetUser2FAStatusQuery> for GetUser2FAStatusHandler {
    async fn handle(&self, query: GetUser2FAStatusQuery) -> AppResult<User2FAStatusResult> {
        info!(
            user_id = %query.user_id,
            tenant_id = %query.tenant_id,
            "Handling GetUser2FAStatusQuery"
        );

        let user_id = cuba_common::UserId::from_uuid(
            uuid::Uuid::parse_str(&query.user_id)
                .map_err(|e| cuba_errors::AppError::validation(e.to_string()))?
        );

        // 检查备份码
        let backup_count = self.backup_code_repository
            .count_available_by_user_id(&user_id, &query.tenant_id)
            .await?;
        let has_backup_codes = backup_count > 0;

        // 检查 WebAuthn
        let webauthn_credentials = self.webauthn_repository
            .find_by_user_id(&user_id, &query.tenant_id)
            .await?;
        let webauthn_enabled = !webauthn_credentials.is_empty();

        Ok(User2FAStatusResult {
            // FUTURE: 注入 UserRepository 以获取 user.two_factor_enabled 的真实值
            // 当前需要配合 GetUser2FAStatusQuery 中传入 totp_enabled 参数
            totp_enabled: false,
            webauthn_enabled,
            backup_codes_available: has_backup_codes,
            preferred_method: if webauthn_enabled {
                Some("webauthn".to_string())
            } else {
                None
            },
        })
    }
}
