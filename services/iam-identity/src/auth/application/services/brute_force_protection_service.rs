//! 暴力破解防护服务

use std::sync::Arc;
use chrono::{Duration, Utc};
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use cuba_ports::CachePort;

use crate::auth::domain::repositories::LoginLogRepository;
use crate::shared::domain::repositories::UserRepository;

/// 暴力破解防护配置
#[derive(Debug, Clone)]
pub struct BruteForceConfig {
    /// 最大失败次数
    pub max_failed_attempts: i32,
    /// 锁定时间（分钟）
    pub lockout_duration_minutes: i64,
    /// 检查时间窗口（分钟）
    pub check_window_minutes: i64,
}

impl Default for BruteForceConfig {
    fn default() -> Self {
        Self {
            max_failed_attempts: 5,
            lockout_duration_minutes: 30,
            check_window_minutes: 15,
        }
    }
}

/// 暴力破解防护服务
pub struct BruteForceProtectionService {
    login_log_repo: Arc<dyn LoginLogRepository>,
    user_repo: Arc<dyn UserRepository>,
    cache: Arc<dyn CachePort>,
    config: BruteForceConfig,
}

impl BruteForceProtectionService {
    pub fn new(
        login_log_repo: Arc<dyn LoginLogRepository>,
        user_repo: Arc<dyn UserRepository>,
        cache: Arc<dyn CachePort>,
        config: BruteForceConfig,
    ) -> Self {
        Self {
            login_log_repo,
            user_repo,
            cache,
            config,
        }
    }

    /// 检查用户是否被锁定
    pub async fn is_locked(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<bool> {
        // 1. 检查缓存
        let cache_key = format!("account_lock:{}", user_id);
        if let Some(locked) = self.cache.get::<bool>(&cache_key).await? {
            return Ok(locked);
        }

        // 2. 检查数据库
        let user = self.user_repo.find_by_id(user_id, tenant_id).await?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        let is_locked = user.is_locked();
        
        // 缓存结果
        if is_locked {
            self.cache.set(&cache_key, &true, 300).await?;
        }

        Ok(is_locked)
    }

    /// 记录登录失败并检查是否需要锁定
    pub async fn record_failed_attempt(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<()> {
        let start_time = Utc::now() - Duration::minutes(self.config.check_window_minutes);
        
        // 统计失败次数
        let failed_count = self.login_log_repo
            .count_failed_attempts(user_id, tenant_id, start_time)
            .await?;

        // 如果超过限制，锁定账户
        if failed_count >= self.config.max_failed_attempts as i64 {
            let mut user = self.user_repo.find_by_id(user_id, tenant_id).await?
                .ok_or_else(|| AppError::not_found("User not found"))?;

            user.lock_account(
                self.config.lockout_duration_minutes,
                "Too many failed login attempts".to_string()
            );
            
            self.user_repo.update(&user).await?;

            // 更新缓存
            let cache_key = format!("account_lock:{}", user_id);
            self.cache.set(&cache_key, &true, (self.config.lockout_duration_minutes * 60) as usize).await?;
        }

        Ok(())
    }

    /// 记录登录成功并清除失败计数
    pub async fn record_successful_login(&self, user_id: &UserId) -> AppResult<()> {
        // 清除锁定缓存
        let cache_key = format!("account_lock:{}", user_id);
        self.cache.delete(&cache_key).await?;

        Ok(())
    }

    /// 获取剩余尝试次数
    pub async fn get_remaining_attempts(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<i32> {
        let start_time = Utc::now() - Duration::minutes(self.config.check_window_minutes);
        
        let failed_count = self.login_log_repo
            .count_failed_attempts(user_id, tenant_id, start_time)
            .await?;

        let remaining = self.config.max_failed_attempts - failed_count as i32;
        Ok(remaining.max(0))
    }

    /// 手动解锁账户
    pub async fn unlock_account(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<()> {
        let mut user = self.user_repo.find_by_id(user_id, tenant_id).await?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        user.unlock_account();
        self.user_repo.update(&user).await?;

        // 清除缓存
        let cache_key = format!("account_lock:{}", user_id);
        self.cache.delete(&cache_key).await?;

        Ok(())
    }
}
