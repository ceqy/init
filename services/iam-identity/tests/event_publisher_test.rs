//! 事件发布器测试

use chrono::Utc;
use common::{TenantId, UserId};
use iam_identity::infrastructure::events::{
    EventPublisher, IamDomainEvent, InMemoryEventBus, LoggingEventPublisher, NoOpEventPublisher,
};

fn create_test_event() -> IamDomainEvent {
    IamDomainEvent::UserCreated {
        user_id: UserId::new(),
        tenant_id: TenantId::new(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        timestamp: Utc::now(),
    }
}

#[tokio::test]
async fn test_in_memory_event_bus_publish() {
    let bus = InMemoryEventBus::new();

    let event = create_test_event();
    bus.publish(event).await;

    let events = bus.get_events().await;
    assert_eq!(events.len(), 1);
}

#[tokio::test]
async fn test_in_memory_event_bus_publish_all() {
    let bus = InMemoryEventBus::new();

    let events = vec![
        IamDomainEvent::UserCreated {
            user_id: UserId::new(),
            tenant_id: TenantId::new(),
            username: "user1".to_string(),
            email: "user1@example.com".to_string(),
            timestamp: Utc::now(),
        },
        IamDomainEvent::UserCreated {
            user_id: UserId::new(),
            tenant_id: TenantId::new(),
            username: "user2".to_string(),
            email: "user2@example.com".to_string(),
            timestamp: Utc::now(),
        },
    ];

    bus.publish_all(events).await;

    let stored = bus.get_events().await;
    assert_eq!(stored.len(), 2);
}

#[tokio::test]
async fn test_noop_event_publisher() {
    let publisher = NoOpEventPublisher;

    let event = create_test_event();
    publisher.publish(event).await;
    // NoOp 不保存事件，只是忽略
}

#[tokio::test]
async fn test_logging_event_publisher() {
    let publisher = LoggingEventPublisher;

    let event = create_test_event();
    publisher.publish(event).await;
    // 日志发布器只记录，不保存
}

#[tokio::test]
async fn test_in_memory_event_bus_default() {
    let bus = InMemoryEventBus::default();

    let event = create_test_event();
    bus.publish(event).await;

    let events = bus.get_events().await;
    assert_eq!(events.len(), 1);
}

#[tokio::test]
async fn test_in_memory_event_bus_clear() {
    let bus = InMemoryEventBus::new();

    bus.publish(create_test_event()).await;
    bus.publish(create_test_event()).await;
    assert_eq!(bus.get_events().await.len(), 2);

    bus.clear().await;
    assert_eq!(bus.get_events().await.len(), 0);
}

#[tokio::test]
async fn test_iam_domain_event_types() {
    let user_created = IamDomainEvent::UserCreated {
        user_id: UserId::new(),
        tenant_id: TenantId::new(),
        username: "test".to_string(),
        email: "test@test.com".to_string(),
        timestamp: Utc::now(),
    };
    assert_eq!(user_created.event_type(), "UserCreated");
    assert_eq!(user_created.aggregate_type(), "User");

    let logged_in = IamDomainEvent::UserLoggedIn {
        user_id: UserId::new(),
        tenant_id: TenantId::new(),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: None,
        timestamp: Utc::now(),
    };
    assert_eq!(logged_in.event_type(), "UserLoggedIn");
    assert_eq!(logged_in.aggregate_type(), "User");

    let password_changed = IamDomainEvent::PasswordChanged {
        user_id: UserId::new(),
        tenant_id: TenantId::new(),
        timestamp: Utc::now(),
    };
    assert_eq!(password_changed.event_type(), "PasswordChanged");
}
