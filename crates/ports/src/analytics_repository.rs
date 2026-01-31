//! 分析 Repository trait 定义
//!
//! 定义用于分析数据存储的抽象接口

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cuba_errors::AppResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 分析查询选项
#[derive(Debug, Clone, Default)]
pub struct AnalyticsQueryOptions {
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    /// 租户 ID
    pub tenant_id: Option<Uuid>,
    /// 分页偏移
    pub offset: u64,
    /// 分页大小
    pub limit: u64,
    /// 排序字段
    pub order_by: Option<String>,
    /// 是否降序
    pub descending: bool,
}

impl AnalyticsQueryOptions {
    /// 创建新的查询选项
    pub fn new() -> Self {
        Self {
            limit: 100,
            ..Default::default()
        }
    }

    /// 设置时间范围
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// 设置租户
    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// 设置分页
    pub fn with_pagination(mut self, offset: u64, limit: u64) -> Self {
        self.offset = offset;
        self.limit = limit;
        self
    }

    /// 设置排序
    pub fn with_order(mut self, field: impl Into<String>, descending: bool) -> Self {
        self.order_by = Some(field.into());
        self.descending = descending;
        self
    }
}

/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// 唯一标识
    pub id: Uuid,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
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
    pub old_value: Option<String>,
    /// 新值（JSON）
    pub new_value: Option<String>,
    /// IP 地址
    pub ip_address: Option<String>,
    /// User Agent
    pub user_agent: Option<String>,
    /// 元数据（JSON）
    pub metadata: Option<String>,
}

impl AuditLogEntry {
    /// 创建新的审计日志条目
    pub fn new(
        tenant_id: Uuid,
        user_id: Uuid,
        action: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            timestamp: Utc::now(),
            tenant_id,
            user_id,
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            old_value: None,
            new_value: None,
            ip_address: None,
            user_agent: None,
            metadata: None,
        }
    }

    /// 设置变更值
    pub fn with_changes(mut self, old_value: Option<String>, new_value: Option<String>) -> Self {
        self.old_value = old_value;
        self.new_value = new_value;
        self
    }

    /// 设置请求信息
    pub fn with_request_info(
        mut self,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self
    }

    /// 设置元数据
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// 审计日志过滤器
#[derive(Debug, Clone, Default)]
pub struct AuditLogFilter {
    /// 用户 ID
    pub user_id: Option<Uuid>,
    /// 操作类型
    pub action: Option<String>,
    /// 资源类型
    pub resource_type: Option<String>,
    /// 资源 ID
    pub resource_id: Option<String>,
}

impl AuditLogFilter {
    /// 创建新的过滤器
    pub fn new() -> Self {
        Self::default()
    }

    /// 按用户过滤
    pub fn by_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// 按操作过滤
    pub fn by_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// 按资源类型过滤
    pub fn by_resource_type(mut self, resource_type: impl Into<String>) -> Self {
        self.resource_type = Some(resource_type.into());
        self
    }

    /// 按资源 ID 过滤
    pub fn by_resource(mut self, resource_id: impl Into<String>) -> Self {
        self.resource_id = Some(resource_id.into());
        self
    }
}

/// 业务事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessEvent {
    /// 唯一标识
    pub id: Uuid,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
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
    pub metadata: Option<String>,
}

impl BusinessEvent {
    /// 创建新的业务事件
    pub fn new(
        tenant_id: Uuid,
        event_type: impl Into<String>,
        aggregate_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        version: u64,
        payload: String,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            timestamp: Utc::now(),
            tenant_id,
            event_type: event_type.into(),
            aggregate_type: aggregate_type.into(),
            aggregate_id: aggregate_id.into(),
            version,
            payload,
            metadata: None,
        }
    }

    /// 设置元数据
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// 事件趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTrend {
    /// 时间桶
    pub time_bucket: DateTime<Utc>,
    /// 事件类型
    pub event_type: String,
    /// 计数
    pub count: u64,
}

/// 审计日志 Repository trait
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    /// 记录单条审计日志
    async fn log(&self, entry: AuditLogEntry) -> AppResult<()>;

    /// 批量记录审计日志
    async fn log_batch(&self, entries: Vec<AuditLogEntry>) -> AppResult<u64>;

    /// 查询审计日志
    async fn query(
        &self,
        filter: AuditLogFilter,
        options: &AnalyticsQueryOptions,
    ) -> AppResult<Vec<AuditLogEntry>>;

    /// 统计审计日志数量
    async fn count(
        &self,
        filter: AuditLogFilter,
        options: &AnalyticsQueryOptions,
    ) -> AppResult<u64>;
}

/// 业务事件 Repository trait
#[async_trait]
pub trait BusinessEventRepository: Send + Sync {
    /// 记录单个事件
    async fn record(&self, event: BusinessEvent) -> AppResult<()>;

    /// 批量记录事件
    async fn record_batch(&self, events: Vec<BusinessEvent>) -> AppResult<u64>;

    /// 查询事件
    async fn query(&self, options: &AnalyticsQueryOptions) -> AppResult<Vec<BusinessEvent>>;

    /// 按聚合查询事件
    async fn query_by_aggregate(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
        options: &AnalyticsQueryOptions,
    ) -> AppResult<Vec<BusinessEvent>>;

    /// 趋势分析
    async fn trend_analysis(
        &self,
        event_type: &str,
        options: &AnalyticsQueryOptions,
    ) -> AppResult<Vec<EventTrend>>;
}

/// 通用分析 Repository trait
#[async_trait]
pub trait AnalyticsRepository: Send + Sync {
    /// 关联的记录类型
    type Record: Send + Sync;

    /// 批量插入记录
    async fn insert_batch(&self, records: Vec<Self::Record>) -> AppResult<u64>;

    /// 按时间范围查询
    async fn query_by_time_range(
        &self,
        options: &AnalyticsQueryOptions,
    ) -> AppResult<Vec<Self::Record>>;

    /// 统计记录数量
    async fn count(&self, options: &AnalyticsQueryOptions) -> AppResult<u64>;
}
