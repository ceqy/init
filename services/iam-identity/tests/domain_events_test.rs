//! 领域事件测试

use iam_identity::domain::events::{
    UserCreated, UserLoggedIn, UserLoggedOut, PasswordChanged, 
    TwoFactorEnabled, TwoFactorDisabled,
};
use cuba_event_core::DomainEvent;

#[test]
fn test_user_created_event() {
    let event = UserCreated {
        user_id: "user-123".to_string(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        tenant_id: "tenant-456".to_string(),
    };
    
    assert_eq!(event.event_type(), "UserCreated");
    assert_eq!(event.aggregate_type(), "User");
    assert_eq!(event.aggregate_id(), "user-123");
}

#[test]
fn test_user_logged_in_event() {
    let event = UserLoggedIn {
        user_id: "user-123".to_string(),
        session_id: "session-789".to_string(),
        ip_address: Some("192.168.1.1".to_string()),
        device_info: Some("Chrome/120".to_string()),
    };
    
    assert_eq!(event.event_type(), "UserLoggedIn");
    assert_eq!(event.aggregate_type(), "User");
    assert_eq!(event.aggregate_id(), "user-123");
}

#[test]
fn test_user_logged_out_event() {
    let event = UserLoggedOut {
        user_id: "user-123".to_string(),
        session_id: "session-789".to_string(),
    };
    
    assert_eq!(event.event_type(), "UserLoggedOut");
    assert_eq!(event.aggregate_type(), "User");
}

#[test]
fn test_password_changed_event() {
    let event = PasswordChanged {
        user_id: "user-123".to_string(),
    };
    
    assert_eq!(event.event_type(), "PasswordChanged");
    assert_eq!(event.aggregate_type(), "User");
}

#[test]
fn test_two_factor_enabled_event() {
    let event = TwoFactorEnabled {
        user_id: "user-123".to_string(),
        method: "totp".to_string(),
    };
    
    assert_eq!(event.event_type(), "TwoFactorEnabled");
    assert_eq!(event.aggregate_type(), "User");
}

#[test]
fn test_two_factor_disabled_event() {
    let event = TwoFactorDisabled {
        user_id: "user-123".to_string(),
    };
    
    assert_eq!(event.event_type(), "TwoFactorDisabled");
    assert_eq!(event.aggregate_type(), "User");
}
