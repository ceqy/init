//! PostgreSQL 事件存储
//!
//! 将领域事件持久化到 PostgreSQL 数据库

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::event_publisher::{EventPublisher, IamDomainEvent};

/// PostgreSQL 事件存储
pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    /// 创建新的事件存储实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 提取事件的聚合 ID
    fn extract_aggregate_id(event: &IamDomainEvent) -> String {
        match event {
            IamDomainEvent::UserCreated { user_id, .. } => user_id.0.to_string(),
            IamDomainEvent::UserLoggedIn { user_id, .. } => user_id.0.to_string(),
            IamDomainEvent::UserLoggedOut { user_id, .. } => user_id.0.to_string(),
            IamDomainEvent::PasswordChanged { user_id, .. } => user_id.0.to_string(),
            IamDomainEvent::TwoFactorEnabled { user_id, .. } => user_id.0.to_string(),
            IamDomainEvent::TwoFactorDisabled { user_id, .. } => user_id.0.to_string(),
            IamDomainEvent::OAuthClientCreated { client_id, .. } => client_id.clone(),
            IamDomainEvent::SessionCreated { session_id, .. } => session_id.clone(),
            IamDomainEvent::SessionRevoked { session_id, .. } => session_id.clone(),
            IamDomainEvent::UserUpdated { user_id, .. } => user_id.0.to_string(),
            IamDomainEvent::UserProfileUpdated { user_id, .. } => user_id.0.to_string(),
        }
    }

    /// 提取事件的租户 ID
    fn extract_tenant_id(event: &IamDomainEvent) -> Uuid {
        match event {
            IamDomainEvent::UserCreated { tenant_id, .. } => tenant_id.0,
            IamDomainEvent::UserLoggedIn { tenant_id, .. } => tenant_id.0,
            IamDomainEvent::UserLoggedOut { tenant_id, .. } => tenant_id.0,
            IamDomainEvent::PasswordChanged { tenant_id, .. } => tenant_id.0,
            IamDomainEvent::TwoFactorEnabled { tenant_id, .. } => tenant_id.0,
            IamDomainEvent::TwoFactorDisabled { tenant_id, .. } => tenant_id.0,
            IamDomainEvent::OAuthClientCreated { tenant_id, .. } => tenant_id.0,
            IamDomainEvent::SessionCreated { tenant_id, .. } => tenant_id.0,
            IamDomainEvent::SessionRevoked { tenant_id, .. } => tenant_id.0,
            IamDomainEvent::UserUpdated { tenant_id, .. } => tenant_id.0,
            IamDomainEvent::UserProfileUpdated { tenant_id, .. } => tenant_id.0,
        }
    }
}

#[async_trait]
impl EventPublisher for PostgresEventStore {
    async fn publish(&self, event: IamDomainEvent) {
        let event_type = event.event_type();
        let aggregate_type = event.aggregate_type();
        let aggregate_id = Self::extract_aggregate_id(&event);
        let tenant_id = Self::extract_tenant_id(&event);
        let created_at = event.timestamp();

        // 序列化事件为 JSON
        let payload = match serde_json::to_value(&event) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "Failed to serialize event");
                return;
            }
        };

        // 插入数据库
        let result = sqlx::query(
            r#"
            INSERT INTO domain_events (event_type, aggregate_type, aggregate_id, tenant_id, payload, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(event_type)
        .bind(aggregate_type)
        .bind(&aggregate_id)
        .bind(tenant_id)
        .bind(&payload)
        .bind(created_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                tracing::info!(
                    event_type = event_type,
                    aggregate_type = aggregate_type,
                    aggregate_id = %aggregate_id,
                    "Domain event persisted"
                );
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    event_type = event_type,
                    "Failed to persist domain event"
                );
            }
        }
    }

    async fn publish_all(&self, events: Vec<IamDomainEvent>) {
        for event in events {
            self.publish(event).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_aggregate_id() {
        use chrono::Utc;
        use common::{TenantId, UserId};

        let user_id = UserId::new();
        let tenant_id = TenantId::new();

        let event = IamDomainEvent::UserCreated {
            user_id: user_id.clone(),
            tenant_id: tenant_id.clone(),
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            timestamp: Utc::now(),
        };

        assert_eq!(
            PostgresEventStore::extract_aggregate_id(&event),
            user_id.0.to_string()
        );
        assert_eq!(PostgresEventStore::extract_tenant_id(&event), tenant_id.0);
    }
}
