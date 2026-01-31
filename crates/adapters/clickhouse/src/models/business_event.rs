//! 业务事件模型
//!
//! ClickHouse 业务事件表对应的数据结构

use clickhouse::Row;
use ports::BusinessEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 业务事件行（ClickHouse 格式）
///
/// 对应 ClickHouse 表结构：
/// ```sql
/// CREATE TABLE business_events (
///     id UUID,
///     timestamp DateTime64(3),
///     tenant_id UUID,
///     event_type LowCardinality(String),
///     aggregate_type LowCardinality(String),
///     aggregate_id String,
///     version UInt64,
///     payload String,
///     metadata String,
///     date Date DEFAULT toDate(timestamp)
/// ) ENGINE = MergeTree()
/// PARTITION BY toYYYYMM(date)
/// ORDER BY (tenant_id, aggregate_type, aggregate_id, version)
/// TTL date + INTERVAL 3 YEAR;
/// ```
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct BusinessEventRow {
    /// 唯一标识
    pub id: Uuid,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 事件类型
    pub event_type: String,
    /// 聚合类型
    pub aggregate_type: String,
    /// 聚合 ID
    pub aggregate_id: String,
    /// 版本号
    pub version: u64,
    /// 负载（JSON）
    pub payload: String,
    /// 元数据（JSON）
    pub metadata: String,
}

impl From<BusinessEvent> for BusinessEventRow {
    fn from(event: BusinessEvent) -> Self {
        Self {
            id: event.id,
            timestamp: event.timestamp,
            tenant_id: event.tenant_id,
            event_type: event.event_type,
            aggregate_type: event.aggregate_type,
            aggregate_id: event.aggregate_id,
            version: event.version,
            payload: event.payload,
            metadata: event.metadata.unwrap_or_default(),
        }
    }
}

impl From<BusinessEventRow> for BusinessEvent {
    fn from(row: BusinessEventRow) -> Self {
        Self {
            id: row.id,
            timestamp: row.timestamp,
            tenant_id: row.tenant_id,
            event_type: row.event_type,
            aggregate_type: row.aggregate_type,
            aggregate_id: row.aggregate_id,
            version: row.version,
            payload: row.payload,
            metadata: if row.metadata.is_empty() {
                None
            } else {
                Some(row.metadata)
            },
        }
    }
}

/// 事件类型统计
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct EventTypeStats {
    /// 事件类型
    pub event_type: String,
    /// 计数
    pub count: u64,
}

/// 聚合事件统计
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct AggregateEventStats {
    /// 聚合类型
    pub aggregate_type: String,
    /// 事件类型
    pub event_type: String,
    /// 计数
    pub count: u64,
}

/// 每小时事件统计
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct HourlyEventStats {
    /// 小时
    pub hour: chrono::DateTime<chrono::Utc>,
    /// 事件类型
    pub event_type: String,
    /// 计数
    pub count: u64,
}

/// 每日事件统计
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct DailyEventStats {
    /// 日期
    pub date: chrono::NaiveDate,
    /// 事件类型
    pub event_type: String,
    /// 计数
    pub count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_event_row_from_event() {
        let event = BusinessEvent::new(
            Uuid::new_v4(),
            "OrderCreated",
            "Order",
            "order-123",
            1,
            r#"{"amount": 100}"#.to_string(),
        )
        .with_metadata(r#"{"source": "api"}"#.to_string());

        let row = BusinessEventRow::from(event.clone());

        assert_eq!(row.id, event.id);
        assert_eq!(row.event_type, "OrderCreated");
        assert_eq!(row.aggregate_type, "Order");
        assert_eq!(row.version, 1);
        assert_eq!(row.metadata, r#"{"source": "api"}"#);
    }

    #[test]
    fn test_business_event_from_row() {
        let row = BusinessEventRow {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            tenant_id: Uuid::new_v4(),
            event_type: "OrderShipped".to_string(),
            aggregate_type: "Order".to_string(),
            aggregate_id: "order-456".to_string(),
            version: 2,
            payload: r#"{"tracking_number": "ABC123"}"#.to_string(),
            metadata: "".to_string(),
        };

        let event = BusinessEvent::from(row);

        assert_eq!(event.event_type, "OrderShipped");
        assert_eq!(event.version, 2);
        assert!(event.metadata.is_none());
    }
}
