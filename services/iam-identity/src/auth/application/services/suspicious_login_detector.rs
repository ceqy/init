//! 可疑登录检测服务

use std::sync::Arc;
use chrono::{Utc, Timelike, Duration};
use cuba_common::{TenantId, UserId};
use cuba_errors::AppResult;

use crate::domain::repositories::auth::LoginLogRepository;

/// 可疑登录检测服务
pub struct SuspiciousLoginDetector {
    login_log_repo: Arc<dyn LoginLogRepository>,
}

impl SuspiciousLoginDetector {
    pub fn new(login_log_repo: Arc<dyn LoginLogRepository>) -> Self {
        Self { login_log_repo }
    }

    /// 检测是否为可疑登录
    pub async fn is_suspicious(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        ip_address: &str,
        country: Option<&str>,
    ) -> AppResult<(bool, Vec<String>)> {
        let mut reasons = Vec::new();

        // 1. 检查新 IP
        let recent_logins = self.login_log_repo
            .find_by_user_id(user_id, tenant_id, 10)
            .await?;

        let known_ips: Vec<String> = recent_logins
            .iter()
            .map(|log| log.ip_address.clone())
            .collect();

        if !known_ips.is_empty() && !known_ips.contains(&ip_address.to_string()) {
            reasons.push("New IP address".to_string());
        }

        // 2. 检查新国家
        if let Some(country) = country {
            let known_countries: Vec<String> = recent_logins
                .iter()
                .filter_map(|log| log.country.clone())
                .collect();

            if !known_countries.is_empty() && !known_countries.contains(&country.to_string()) {
                reasons.push("New country".to_string());
            }
        }

        // 3. 检查短时间内多次失败
        let recent_time = Utc::now() - Duration::minutes(5);
        let recent_failed = self.login_log_repo
            .count_failed_attempts(user_id, tenant_id, recent_time)
            .await?;

        if recent_failed >= 3 {
            reasons.push("Multiple failed attempts in short time".to_string());
        }

        // 4. 检查异常时间登录（凌晨2-6点）
        let hour = Utc::now().hour();
        if hour >= 2 && hour < 6 {
            reasons.push("Unusual login time".to_string());
        }

        Ok((!reasons.is_empty(), reasons))
    }

    /// 标记登录为可疑
    pub async fn mark_suspicious(&self, _login_log_id: &str) -> AppResult<()> {
        // TODO: 更新 login_log 的 is_suspicious 字段
        Ok(())
    }
}
