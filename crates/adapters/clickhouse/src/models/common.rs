//! 通用分析模型
//!
//! 定义通用的分析数据结构

use clickhouse::Row;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 时间序列数据点
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 值
    pub value: f64,
}

/// 计数时间序列
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct CountTimeSeries {
    /// 时间桶
    pub time_bucket: chrono::DateTime<chrono::Utc>,
    /// 计数
    pub count: u64,
}

/// 分组计数
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct GroupedCount {
    /// 分组键
    pub group_key: String,
    /// 计数
    pub count: u64,
}

/// 租户统计
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct TenantStats {
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 指标名称
    pub metric_name: String,
    /// 值
    pub value: f64,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 用户活动记录
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct UserActivity {
    /// 用户 ID
    pub user_id: Uuid,
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 活动类型
    pub activity_type: String,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 会话 ID
    pub session_id: Option<String>,
    /// 元数据
    pub metadata: String,
}

/// 性能指标
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct PerformanceMetric {
    /// 服务名称
    pub service_name: String,
    /// 操作名称
    pub operation_name: String,
    /// 持续时间（毫秒）
    pub duration_ms: f64,
    /// 状态码
    pub status_code: i32,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 追踪 ID
    pub trace_id: Option<String>,
}

/// 聚合结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    /// 总数
    pub count: u64,
    /// 总和
    pub sum: f64,
    /// 平均值
    pub avg: f64,
    /// 最小值
    pub min: f64,
    /// 最大值
    pub max: f64,
}

/// 百分位数结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentileResult {
    /// P50
    pub p50: f64,
    /// P90
    pub p90: f64,
    /// P95
    pub p95: f64,
    /// P99
    pub p99: f64,
}

/// 时间粒度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeGranularity {
    /// 分钟
    Minute,
    /// 小时
    Hour,
    /// 天
    Day,
    /// 周
    Week,
    /// 月
    Month,
}

impl TimeGranularity {
    /// 转换为 ClickHouse 函数
    pub fn to_clickhouse_func(&self) -> &'static str {
        match self {
            Self::Minute => "toStartOfMinute",
            Self::Hour => "toStartOfHour",
            Self::Day => "toStartOfDay",
            Self::Week => "toStartOfWeek",
            Self::Month => "toStartOfMonth",
        }
    }
}

/// 排序方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortDirection {
    /// 升序
    #[default]
    Asc,
    /// 降序
    Desc,
}

impl SortDirection {
    /// 转换为 SQL
    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_granularity() {
        assert_eq!(TimeGranularity::Hour.to_clickhouse_func(), "toStartOfHour");
        assert_eq!(TimeGranularity::Day.to_clickhouse_func(), "toStartOfDay");
    }

    #[test]
    fn test_sort_direction() {
        assert_eq!(SortDirection::Asc.to_sql(), "ASC");
        assert_eq!(SortDirection::Desc.to_sql(), "DESC");
    }

    #[test]
    fn test_aggregation_result() {
        let result = AggregationResult {
            count: 100,
            sum: 1000.0,
            avg: 10.0,
            min: 1.0,
            max: 50.0,
        };

        assert_eq!(result.count, 100);
        assert_eq!(result.avg, 10.0);
    }
}
