//! 数据库连接池监控

use sqlx::PgPool;
use metrics::{gauge, counter, histogram};
use std::time::Instant;

/// 数据库监控工具
pub struct DbMetrics;

impl DbMetrics {
    /// 记录连接池状态
    pub fn record_pool_state(pool: &PgPool, pool_name: &str) {
        gauge!("db_pool_size", "pool" => pool_name.to_string())
            .set(pool.size() as f64);
        gauge!("db_pool_idle", "pool" => pool_name.to_string())
            .set(pool.num_idle() as f64);
    }

    /// 记录查询（计时）
    pub fn record_query(start: Instant, table: &str, operation: &str) {
        histogram!(
            "db_query_duration_ms",
            "table" => table.to_string(),
            "operation" => operation.to_string()
        ).record(start.elapsed().as_millis() as f64);
        
        counter!(
            "db_queries_total",
            "table" => table.to_string(),
            "operation" => operation.to_string()
        ).increment(1);
    }

    /// 记录查询错误
    pub fn record_error(table: &str, operation: &str) {
        counter!(
            "db_query_errors_total",
            "table" => table.to_string(),
            "operation" => operation.to_string()
        ).increment(1);
    }
}

/// 用于计时的守卫结构
pub struct QueryTimer {
    start: Instant,
    table: String,
    operation: String,
}

impl QueryTimer {
    pub fn new(table: &str, operation: &str) -> Self {
        Self {
            start: Instant::now(),
            table: table.to_string(),
            operation: operation.to_string(),
        }
    }

    pub fn finish(self) {
        DbMetrics::record_query(self.start, &self.table, &self.operation);
    }

    pub fn finish_with_error(self) {
        DbMetrics::record_query(self.start, &self.table, &self.operation);
        DbMetrics::record_error(&self.table, &self.operation);
    }
}

// 使用示例：
// let timer = QueryTimer::new("roles", "find_by_id");
// let result = sqlx::query_as::<_, RoleRow>(...).fetch_optional(&pool).await;
// if result.is_err() { timer.finish_with_error(); } else { timer.finish(); }
