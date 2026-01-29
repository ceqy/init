//! 登录日志仓储接口

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cuba_common::{TenantId, UserId};
use cuba_errors::AppResult;

use crate::domain::auth::{LoginLog, LoginLogId, LoginResult};

/// 登录日志仓储接口
#[async_trait]
pub trait LoginLogRepository: Send + Sync {
    /// 保存登录日志
    async fn save(&self, log: &LoginLog) -> AppResult<()>;

    /// 根据 ID 查找日志
    async fn find_by_id(
        &self,
        id: &LoginLogId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<LoginLog>>;

    /// 查询用户的登录历史
    async fn find_by_user_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        limit: i32,
    ) -> AppResult<Vec<LoginLog>>;

    /// 查询用户在指定时间范围内的登录历史
    async fn find_by_user_id_and_time_range(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> AppResult<Vec<LoginLog>>;

    /// 查询用户最近一次成功登录
    async fn find_last_successful_login(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<LoginLog>>;

    /// 查询用户从特定 IP 的登录历史
    async fn find_by_user_and_ip(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        ip_address: &str,
    ) -> AppResult<Vec<LoginLog>>;

    /// 查询用户从特定设备的登录历史
    async fn find_by_user_and_device_fingerprint(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        device_fingerprint: &str,
    ) -> AppResult<Vec<LoginLog>>;

    /// 统计用户在指定时间范围内的失败登录次数
    async fn count_failed_attempts(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        start_time: DateTime<Utc>,
    ) -> AppResult<i64>;

    /// 查询可疑登录
    async fn find_suspicious_logins(
        &self,
        tenant_id: &TenantId,
        start_time: DateTime<Utc>,
        limit: i32,
    ) -> AppResult<Vec<LoginLog>>;

    /// 分页查询登录日志
    async fn list(
        &self,
        tenant_id: &TenantId,
        user_id: Option<&UserId>,
        result: Option<LoginResult>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        page: i32,
        page_size: i32,
    ) -> AppResult<(Vec<LoginLog>, i64)>;

    /// 删除过期的登录日志（数据清理）
    async fn delete_older_than(
        &self,
        tenant_id: &TenantId,
        before: DateTime<Utc>,
    ) -> AppResult<u64>;
}
