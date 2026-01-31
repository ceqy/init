//! 审计日志模型
//!
//! ClickHouse 审计日志表对应的数据结构

use clickhouse::Row;
use cuba_ports::AuditLogEntry;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 审计日志行（ClickHouse 格式）
///
/// 对应 ClickHouse 表结构：
/// ```sql
/// CREATE TABLE audit_logs (
///     id UUID,
///     timestamp DateTime64(3),
///     tenant_id UUID,
///     user_id UUID,
///     action LowCardinality(String),
///     resource_type LowCardinality(String),
///     resource_id String,
///     old_value String,
///     new_value String,
///     ip_address String,
///     user_agent String,
///     metadata String,
///     date Date DEFAULT toDate(timestamp)
/// ) ENGINE = MergeTree()
/// PARTITION BY toYYYYMM(date)
/// ORDER BY (tenant_id, timestamp, action)
/// TTL date + INTERVAL 2 YEAR;
/// ```
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct AuditLogRow {
    /// 唯一标识
    pub id: Uuid,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 用户 ID
    pub user_id: Uuid,
    /// 操作类型
    pub action: String,
    /// 资源类型
    pub resource_type: String,
    /// 资源 ID
    pub resource_id: String,
    /// 旧值（JSON）
    pub old_value: String,
    /// 新值（JSON）
    pub new_value: String,
    /// IP 地址
    pub ip_address: String,
    /// User Agent
    pub user_agent: String,
    /// 元数据（JSON）
    pub metadata: String,
}

impl From<AuditLogEntry> for AuditLogRow {
    fn from(entry: AuditLogEntry) -> Self {
        Self {
            id: entry.id,
            timestamp: entry.timestamp,
            tenant_id: entry.tenant_id,
            user_id: entry.user_id,
            action: entry.action,
            resource_type: entry.resource_type,
            resource_id: entry.resource_id,
            old_value: entry.old_value.unwrap_or_default(),
            new_value: entry.new_value.unwrap_or_default(),
            ip_address: entry.ip_address.unwrap_or_default(),
            user_agent: entry.user_agent.unwrap_or_default(),
            metadata: entry.metadata.unwrap_or_default(),
        }
    }
}

impl From<AuditLogRow> for AuditLogEntry {
    fn from(row: AuditLogRow) -> Self {
        Self {
            id: row.id,
            timestamp: row.timestamp,
            tenant_id: row.tenant_id,
            user_id: row.user_id,
            action: row.action,
            resource_type: row.resource_type,
            resource_id: row.resource_id,
            old_value: if row.old_value.is_empty() {
                None
            } else {
                Some(row.old_value)
            },
            new_value: if row.new_value.is_empty() {
                None
            } else {
                Some(row.new_value)
            },
            ip_address: if row.ip_address.is_empty() {
                None
            } else {
                Some(row.ip_address)
            },
            user_agent: if row.user_agent.is_empty() {
                None
            } else {
                Some(row.user_agent)
            },
            metadata: if row.metadata.is_empty() {
                None
            } else {
                Some(row.metadata)
            },
        }
    }
}

/// 审计日志统计
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct AuditLogStats {
    /// 操作类型
    pub action: String,
    /// 计数
    pub count: u64,
}

/// 按用户的审计日志统计
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct AuditLogByUserStats {
    /// 用户 ID
    pub user_id: Uuid,
    /// 操作类型
    pub action: String,
    /// 计数
    pub count: u64,
}

/// 按资源类型的审计日志统计
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct AuditLogByResourceStats {
    /// 资源类型
    pub resource_type: String,
    /// 操作类型
    pub action: String,
    /// 计数
    pub count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_row_from_entry() {
        let entry = AuditLogEntry::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "CREATE",
            "User",
            "user-123",
        )
        .with_changes(None, Some(r#"{"name": "test"}"#.to_string()))
        .with_request_info(Some("192.168.1.1".to_string()), Some("Mozilla/5.0".to_string()));

        let row = AuditLogRow::from(entry.clone());

        assert_eq!(row.id, entry.id);
        assert_eq!(row.action, "CREATE");
        assert_eq!(row.resource_type, "User");
        assert_eq!(row.old_value, "");
        assert_eq!(row.new_value, r#"{"name": "test"}"#);
        assert_eq!(row.ip_address, "192.168.1.1");
    }

    #[test]
    fn test_audit_log_entry_from_row() {
        let row = AuditLogRow {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            action: "UPDATE".to_string(),
            resource_type: "Order".to_string(),
            resource_id: "order-456".to_string(),
            old_value: r#"{"status": "pending"}"#.to_string(),
            new_value: r#"{"status": "completed"}"#.to_string(),
            ip_address: "".to_string(),
            user_agent: "".to_string(),
            metadata: "".to_string(),
        };

        let entry = AuditLogEntry::from(row);

        assert_eq!(entry.action, "UPDATE");
        assert!(entry.old_value.is_some());
        assert!(entry.new_value.is_some());
        assert!(entry.ip_address.is_none());
        assert!(entry.user_agent.is_none());
    }
}
