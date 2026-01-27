//! 事件发布器测试

use iam_identity::infrastructure::events::{
    EventPublisher, InMemoryEventBus, LoggingEventPublisher, NoOpEventPublisher,
};
use iam_identity::domain::events::UserCreated;

#[tokio::test]
async fn test_in_memory_event_bus_publish() {
    let bus = InMemoryEventBus::new();
    
    let event = UserCreated {
        user_id: "user-123".to_string(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        tenant_id: "tenant-456".to_string(),
    };
    
    let result = bus.publish(event).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_in_memory_event_bus_publish_all() {
    let bus = InMemoryEventBus::new();
    
    let events = vec![
        UserCreated {
            user_id: "user-1".to_string(),
            username: "user1".to_string(),
            email: "user1@example.com".to_string(),
            tenant_id: "tenant-1".to_string(),
        },
        UserCreated {
            user_id: "user-2".to_string(),
            username: "user2".to_string(),
            email: "user2@example.com".to_string(),
            tenant_id: "tenant-1".to_string(),
        },
    ];
    
    let result = bus.publish_all(events).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_noop_event_publisher() {
    let publisher = NoOpEventPublisher;
    
    let event = UserCreated {
        user_id: "user-123".to_string(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        tenant_id: "tenant-456".to_string(),
    };
    
    let result = publisher.publish(event).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_logging_event_publisher() {
    let publisher = LoggingEventPublisher;
    
    let event = UserCreated {
        user_id: "user-123".to_string(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        tenant_id: "tenant-456".to_string(),
    };
    
    let result = publisher.publish(event).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_in_memory_event_bus_default() {
    let bus = InMemoryEventBus::default();
    
    let event = UserCreated {
        user_id: "user-123".to_string(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        tenant_id: "tenant-456".to_string(),
    };
    
    let result = bus.publish(event).await;
    assert!(result.is_ok());
}
