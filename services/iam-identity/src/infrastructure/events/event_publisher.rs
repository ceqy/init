//! 事件发布器
//!
//! 提供领域事件的发布功能

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cuba_common::{TenantId, UserId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// IAM 领域事件枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IamDomainEvent {
    UserCreated {
        user_id: UserId,
        tenant_id: TenantId,
        username: String,
        email: String,
        timestamp: DateTime<Utc>,
    },
    UserLoggedIn {
        user_id: UserId,
        tenant_id: TenantId,
        ip_address: Option<String>,
        user_agent: Option<String>,
        timestamp: DateTime<Utc>,
    },
    UserLoggedOut {
        user_id: UserId,
        tenant_id: TenantId,
        session_id: String,
        timestamp: DateTime<Utc>,
    },
    PasswordChanged {
        user_id: UserId,
        tenant_id: TenantId,
        timestamp: DateTime<Utc>,
    },
    TwoFactorEnabled {
        user_id: UserId,
        tenant_id: TenantId,
        method: String,
        timestamp: DateTime<Utc>,
    },
    TwoFactorDisabled {
        user_id: UserId,
        tenant_id: TenantId,
        timestamp: DateTime<Utc>,
    },
    OAuthClientCreated {
        client_id: String,
        tenant_id: TenantId,
        name: String,
        timestamp: DateTime<Utc>,
    },
    SessionCreated {
        session_id: String,
        user_id: UserId,
        tenant_id: TenantId,
        timestamp: DateTime<Utc>,
    },
    SessionRevoked {
        session_id: String,
        user_id: UserId,
        tenant_id: TenantId,
        timestamp: DateTime<Utc>,
    },
    UserUpdated {
        user_id: UserId,
        tenant_id: TenantId,
        updated_fields: Vec<String>,
        timestamp: DateTime<Utc>,
    },
    UserProfileUpdated {
        user_id: UserId,
        tenant_id: TenantId,
        updated_fields: Vec<String>,
        timestamp: DateTime<Utc>,
    },
}

impl IamDomainEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::UserCreated { .. } => "UserCreated",
            Self::UserLoggedIn { .. } => "UserLoggedIn",
            Self::UserLoggedOut { .. } => "UserLoggedOut",
            Self::PasswordChanged { .. } => "PasswordChanged",
            Self::TwoFactorEnabled { .. } => "TwoFactorEnabled",
            Self::TwoFactorDisabled { .. } => "TwoFactorDisabled",
            Self::OAuthClientCreated { .. } => "OAuthClientCreated",
            Self::SessionCreated { .. } => "SessionCreated",
            Self::SessionRevoked { .. } => "SessionRevoked",
            Self::UserUpdated { .. } => "UserUpdated",
            Self::UserProfileUpdated { .. } => "UserProfileUpdated",
        }
    }

    pub fn aggregate_type(&self) -> &'static str {
        match self {
            Self::UserCreated { .. } 
            | Self::UserLoggedIn { .. }
            | Self::UserLoggedOut { .. }
            | Self::PasswordChanged { .. }
            | Self::TwoFactorEnabled { .. }
            | Self::TwoFactorDisabled { .. }
            | Self::UserUpdated { .. }
            | Self::UserProfileUpdated { .. } => "User",
            Self::OAuthClientCreated { .. } => "OAuthClient",
            Self::SessionCreated { .. }
            | Self::SessionRevoked { .. } => "Session",
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::UserCreated { timestamp, .. }
            | Self::UserLoggedIn { timestamp, .. }
            | Self::UserLoggedOut { timestamp, .. }
            | Self::PasswordChanged { timestamp, .. }
            | Self::TwoFactorEnabled { timestamp, .. }
            | Self::TwoFactorDisabled { timestamp, .. }
            | Self::OAuthClientCreated { timestamp, .. }
            | Self::SessionCreated { timestamp, .. }
            | Self::SessionRevoked { timestamp, .. } 
            | Self::UserUpdated { timestamp, .. }
            | Self::UserProfileUpdated { timestamp, .. } => *timestamp,
        }
    }
}

/// 事件发布器 trait
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// 发布单个事件
    async fn publish(&self, event: IamDomainEvent);
    
    /// 批量发布事件
    async fn publish_all(&self, events: Vec<IamDomainEvent>);
}

/// 内存事件总线实现
pub struct InMemoryEventBus {
    events: Arc<RwLock<Vec<IamDomainEvent>>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 获取所有发布的事件（用于测试）
    pub async fn get_events(&self) -> Vec<IamDomainEvent> {
        self.events.read().await.clone()
    }

    /// 清空事件（用于测试）
    pub async fn clear(&self) {
        self.events.write().await.clear();
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventBus {
    async fn publish(&self, event: IamDomainEvent) {
        tracing::info!(
            event_type = event.event_type(),
            aggregate_type = event.aggregate_type(),
            "Domain event published"
        );
        self.events.write().await.push(event);
    }
    
    async fn publish_all(&self, events: Vec<IamDomainEvent>) {
        for event in events {
            self.publish(event).await;
        }
    }
}

/// NoOp 事件发布器
pub struct NoOpEventPublisher;

#[async_trait]
impl EventPublisher for NoOpEventPublisher {
    async fn publish(&self, _event: IamDomainEvent) {}
    
    async fn publish_all(&self, _events: Vec<IamDomainEvent>) {}
}

/// 日志事件发布器
pub struct LoggingEventPublisher;

#[async_trait]
impl EventPublisher for LoggingEventPublisher {
    async fn publish(&self, event: IamDomainEvent) {
        tracing::info!(
            event_type = event.event_type(),
            aggregate_type = event.aggregate_type(),
            "Domain event: {}",
            event.event_type()
        );
    }
    
    async fn publish_all(&self, events: Vec<IamDomainEvent>) {
        for event in events {
            self.publish(event).await;
        }
    }
}
