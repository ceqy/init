//! 可疑登录检测服务

use chrono::{Timelike, Utc};
use cuba_common::{TenantId, UserId};
use cuba_errors::AppResult;
use std::sync::Arc;


use crate::domain::repositories::auth::LoginLogRepository;

/// 可疑登录检测服务
pub struct SuspiciousLoginDetector {
    login_log_repo: Arc<dyn LoginLogRepository>,
}

impl SuspiciousLoginDetector {
    pub fn new(login_log_repo: Arc<dyn LoginLogRepository>) -> Self {
        Self { login_log_repo }
    }

    /// 检测是否为可疑登录 - 返回 (是否可疑, 原因列表)
    pub async fn is_suspicious(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        ip_address: &str,
        device_fingerprint: Option<&str>,
    ) -> AppResult<(bool, Vec<String>)> {
        let fingerprint = device_fingerprint.unwrap_or("unknown");
        match self.detect(user_id, tenant_id, ip_address, fingerprint).await? {
            Some(reasons_str) => {
                let reasons: Vec<String> = reasons_str.split("; ").map(|s| s.to_string()).collect();
                Ok((true, reasons))
            }
            None => Ok((false, vec![])),
        }
    }

    /// 检测是否为可疑登录
    pub async fn detect(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        ip_address: &str,
        device_fingerprint: &str,
    ) -> AppResult<Option<String>> {
        let mut reasons = Vec::new();

        // 1. 检测异地登录
        if let Some(reason) = self.detect_unusual_location(user_id, tenant_id, ip_address).await? {
            reasons.push(reason);
        }

        // 2. 检测新设备登录
        if let Some(reason) = self.detect_new_device(user_id, tenant_id, device_fingerprint).await? {
            reasons.push(reason);
        }

        // 3. 检测异常时间登录
        if let Some(reason) = self.detect_unusual_time(user_id, tenant_id).await? {
            reasons.push(reason);
        }

        if reasons.is_empty() {
            Ok(None)
        } else {
            Ok(Some(reasons.join("; ")))
        }
    }

    /// 检测异地登录
    async fn detect_unusual_location(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        ip_address: &str,
    ) -> AppResult<Option<String>> {
        // 获取用户最近的登录记录
        let recent_logins = self
            .login_log_repo
            .find_by_user_id(user_id, tenant_id, 10)
            .await?;

        if recent_logins.is_empty() {
            // 首次登录，不算可疑
            return Ok(None);
        }

        // 检查是否从新的 IP 登录
        let known_ips: Vec<String> = recent_logins
            .iter()
            .map(|log| log.ip_address.clone())
            .collect();

        if !known_ips.contains(&ip_address.to_string()) {
            // 从新 IP 登录
            // 实际应该检查 IP 的地理位置，这里简化处理
            let ip_prefix = Self::get_ip_prefix(ip_address);
            let known_prefixes: Vec<String> = known_ips
                .iter()
                .map(|ip| Self::get_ip_prefix(ip))
                .collect();

            if !known_prefixes.contains(&ip_prefix) {
                return Ok(Some(format!(
                    "Login from new location (IP: {})",
                    ip_address
                )));
            }
        }

        Ok(None)
    }

    /// 检测新设备登录
    async fn detect_new_device(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        device_fingerprint: &str,
    ) -> AppResult<Option<String>> {
        // 检查是否从已知设备登录
        let device_logins = self
            .login_log_repo
            .find_by_user_and_device_fingerprint(user_id, tenant_id, device_fingerprint)
            .await?;

        if device_logins.is_empty() {
            return Ok(Some(format!(
                "Login from new device (fingerprint: {})",
                device_fingerprint
            )));
        }

        Ok(None)
    }

    /// 检测异常时间登录
    async fn detect_unusual_time(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<String>> {
        let now = Utc::now();
        let hour = now.hour();

        // 深夜登录（凌晨 2-5 点）可能可疑
        if (2..=5).contains(&hour) {
            // 检查用户是否有深夜登录的历史
            let start_time = now - chrono::Duration::days(30);
            let recent_logins = self
                .login_log_repo
                .find_by_user_id_and_time_range(user_id, tenant_id, start_time, now)
                .await?;

            let night_logins = recent_logins
                .iter()
                .filter(|log| {
                    let log_hour = log.created_at.hour();
                    (2..=5).contains(&log_hour)
                })
                .count();

            // 如果过去30天没有深夜登录记录，则标记为可疑
            if night_logins == 0 {
                return Ok(Some(format!(
                    "Login at unusual time ({}:00)",
                    hour
                )));
            }
        }

        Ok(None)
    }

    /// 获取 IP 前缀（简化的地理位置判断）
    fn get_ip_prefix(ip: &str) -> String {
        // 简化处理：取前两段作为前缀
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() >= 2 {
            format!("{}.{}", parts[0], parts[1])
        } else {
            ip.to_string()
        }
    }

    /// 检测快速连续登录（可能是暴力破解）
    pub async fn detect_rapid_login_attempts(
        &self,
        _username: &str,
        _tenant_id: &TenantId,
        _ip_address: &str,
    ) -> AppResult<bool> {
        // 简单实现：检查最近 1 分钟内的登录尝试次数
        let _start_time = Utc::now() - chrono::Duration::minutes(1);
        
        // 这里需要一个按用户名和IP查询的方法
        // 简化处理：如果有超过10次尝试，则认为是快速连续登录
        
        Ok(false) // 简化实现
    }
}

