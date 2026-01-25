//! Metrics 采集器
//!
//! 定期采集和更新业务指标

use std::sync::Arc;
use std::time::Duration;
use sqlx::PgPool;
use tracing::{debug, error};

use super::metrics::*;

/// Metrics 采集器
pub struct MetricsCollector {
    pool: PgPool,
    interval: Duration,
}

impl MetricsCollector {
    /// 创建新的 Metrics 采集器
    pub fn new(pool: PgPool, interval: Duration) -> Self {
        Self { pool, interval }
    }

    /// 启动后台采集任务
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(self.interval);
            loop {
                ticker.tick().await;

                if let Err(e) = self.collect_metrics().await {
                    error!(error = %e, "Failed to collect metrics");
                }
            }
        })
    }

    /// 采集所有指标
    async fn collect_metrics(&self) -> Result<(), sqlx::Error> {
        // 采集用户指标
        self.collect_user_metrics().await?;

        // 采集会话指标
        self.collect_session_metrics().await?;

        // 采集 2FA 指标
        self.collect_2fa_metrics().await?;

        // 采集 OAuth 指标
        self.collect_oauth_metrics().await?;

        // 采集租户指标
        self.collect_tenant_metrics().await?;

        // 采集登录成功率
        self.collect_login_success_rate().await?;

        debug!("Metrics collected successfully");
        Ok(())
    }

    /// 采集用户指标
    async fn collect_user_metrics(&self) -> Result<(), sqlx::Error> {
        // 总用户数
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        set_total_users(total.0);

        // 活跃用户数
        let active: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE status = 'Active'")
            .fetch_one(&self.pool)
            .await?;
        set_active_users(active.0);

        Ok(())
    }

    /// 采集会话指标
    async fn collect_session_metrics(&self) -> Result<(), sqlx::Error> {
        // 活跃会话数
        let active: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sessions WHERE revoked_at IS NULL AND expires_at > NOW()"
        )
        .fetch_one(&self.pool)
        .await?;
        set_active_sessions(active.0);

        Ok(())
    }

    /// 采集 2FA 指标
    async fn collect_2fa_metrics(&self) -> Result<(), sqlx::Error> {
        // 启用 2FA 的用户数
        let enabled: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE totp_enabled = true OR backup_codes_enabled = true"
        )
        .fetch_one(&self.pool)
        .await?;
        set_2fa_enabled_users(enabled.0);

        // 计算 2FA 使用率
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE status = 'Active'")
            .fetch_one(&self.pool)
            .await?;

        let rate = calculate_2fa_usage_rate(enabled.0, total.0);
        set_2fa_usage_rate(rate);

        Ok(())
    }

    /// 采集 OAuth 指标
    async fn collect_oauth_metrics(&self) -> Result<(), sqlx::Error> {
        // 活跃的 OAuth Client 数量
        let active: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM oauth_clients WHERE is_active = true"
        )
        .fetch_one(&self.pool)
        .await?;
        set_active_oauth_clients(active.0);

        Ok(())
    }

    /// 采集租户指标
    async fn collect_tenant_metrics(&self) -> Result<(), sqlx::Error> {
        // 总租户数
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tenants")
            .fetch_one(&self.pool)
            .await?;
        set_total_tenants(total.0);

        // 活跃租户数
        let active: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tenants WHERE status = 'Active'")
            .fetch_one(&self.pool)
            .await?;
        set_active_tenants(active.0);

        Ok(())
    }

    /// 采集登录成功率（最近 1 小时）
    async fn collect_login_success_rate(&self) -> Result<(), sqlx::Error> {
        let result: (i64, i64) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(*) FILTER (WHERE success = true) as success_count,
                COUNT(*) as total_count
            FROM login_logs
            WHERE created_at > NOW() - INTERVAL '1 hour'
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let rate = calculate_login_success_rate(result.0, result.1);
        set_login_success_rate(rate);

        Ok(())
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        // 默认每 30 秒采集一次
        Self::new(
            PgPool::connect("").unwrap(), // 需要实际的连接
            Duration::from_secs(30),
        )
    }
}
