//! 值对象单元测试

use iam_identity::domain::value_objects::{Email, Username};

#[test]
fn test_valid_email() {
    let email = Email::new("test@example.com");
    assert!(email.is_ok());
    assert_eq!(email.unwrap().as_str(), "test@example.com");
}

#[test]
fn test_invalid_email_no_at() {
    let email = Email::new("invalid-email");
    assert!(email.is_err());
}

#[test]
fn test_invalid_email_no_domain() {
    let email = Email::new("test@");
    assert!(email.is_err());
}

#[test]
fn test_invalid_email_empty() {
    let email = Email::new("");
    assert!(email.is_err());
}

#[test]
fn test_valid_username() {
    let username = Username::new("johndoe");
    assert!(username.is_ok());
    assert_eq!(username.unwrap().as_str(), "johndoe");
}

#[test]
fn test_valid_username_with_numbers() {
    let username = Username::new("john123");
    assert!(username.is_ok());
}

#[test]
fn test_valid_username_with_underscore() {
    let username = Username::new("john_doe");
    assert!(username.is_ok());
}

#[test]
fn test_invalid_username_too_short() {
    let username = Username::new("ab");
    assert!(username.is_err());
}

#[test]
fn test_invalid_username_empty() {
    let username = Username::new("");
    assert!(username.is_err());
}
