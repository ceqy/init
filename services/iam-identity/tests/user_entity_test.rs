//! Repository 集成测试
//!
//! 这些测试需要数据库连接，使用 mock 或真实数据库

use cuba_common::TenantId;

// 导入 Repository 和实体
use iam_identity::domain::user::User;
use iam_identity::domain::value_objects::{Email, HashedPassword, Username};

/// 测试辅助：创建测试用户
fn create_test_user(tenant_id: TenantId) -> User {
    let username = Username::new(format!(
        "testuser_{}",
        uuid::Uuid::new_v4().to_string()[..8].to_string()
    ))
    .expect("Valid username");
    let email = Email::new(format!(
        "test_{}@example.com",
        uuid::Uuid::new_v4().to_string()[..8].to_string()
    ))
    .expect("Valid email");
    let password_hash =
        HashedPassword::from_hash("$argon2id$v=19$m=65536,t=3,p=4$test_hash".to_string());

    User::new(username, email, password_hash, tenant_id)
}

/// 测试用户实体创建
#[test]
fn test_user_entity_creation() {
    let tenant_id = TenantId::new();
    let user = create_test_user(tenant_id.clone());

    assert!(!user.id.0.is_nil());
    assert_eq!(user.tenant_id, tenant_id);
    assert!(!user.email_verified);
    assert!(!user.two_factor_enabled);
}

/// 测试用户状态转换
#[test]
fn test_user_status_transitions() {
    let tenant_id = TenantId::new();
    let mut user = create_test_user(tenant_id);

    // 初始状态
    assert!(!user.is_active());

    // 激活用户
    user.activate();
    assert!(user.is_active());

    // 停用用户
    user.deactivate();
    assert!(!user.is_active());

    // 锁定用户
    user.activate();
    user.lock();
    assert!(!user.is_active());
}

/// 测试用户登录记录
#[test]
fn test_user_login_tracking() {
    let tenant_id = TenantId::new();
    let mut user = create_test_user(tenant_id);

    assert!(user.last_login_at.is_none());

    user.record_login();
    assert!(user.last_login_at.is_some());
}

/// 测试 2FA 功能
#[test]
fn test_user_2fa() {
    let tenant_id = TenantId::new();
    let mut user = create_test_user(tenant_id);

    assert!(!user.two_factor_enabled);
    assert!(user.two_factor_secret.is_none());

    user.enable_2fa("JBSWY3DPEHPK3PXP".to_string());
    assert!(user.two_factor_enabled);
    assert!(user.two_factor_secret.is_some());

    user.disable_2fa();
    assert!(!user.two_factor_enabled);
    assert!(user.two_factor_secret.is_none());
}

/// 测试密码更新
#[test]
fn test_user_password_update() {
    let tenant_id = TenantId::new();
    let mut user = create_test_user(tenant_id);

    let old_change_time = user.last_password_change_at;

    let new_hash = HashedPassword::from_hash("$argon2id$v=19$m=65536,t=3,p=4$new_hash".to_string());
    user.update_password(new_hash);

    assert!(user.last_password_change_at > old_change_time);
}

/// 测试角色管理
#[test]
fn test_user_role_management() {
    let tenant_id = TenantId::new();
    let mut user = create_test_user(tenant_id);

    assert!(user.role_ids.is_empty());

    user.add_role("admin".to_string());
    assert_eq!(user.role_ids.len(), 1);

    // 重复添加不应增加
    user.add_role("admin".to_string());
    assert_eq!(user.role_ids.len(), 1);

    user.add_role("user".to_string());
    assert_eq!(user.role_ids.len(), 2);

    user.remove_role("admin");
    assert_eq!(user.role_ids.len(), 1);
    assert!(!user.role_ids.contains(&"admin".to_string()));
}

/// 测试邮箱验证
#[test]
fn test_user_email_verification() {
    let tenant_id = TenantId::new();
    let mut user = create_test_user(tenant_id);

    assert!(!user.is_email_verified());
    assert!(user.email_verified_at.is_none());

    user.mark_email_verified();

    assert!(user.is_email_verified());
    assert!(user.email_verified_at.is_some());
}

/// 测试账户锁定
#[test]
fn test_user_account_locking() {
    let tenant_id = TenantId::new();
    let mut user = create_test_user(tenant_id);
    user.activate();

    assert!(!user.is_locked());

    // 锁定账户
    user.lock_account(30, "Test lock".to_string());
    assert!(user.is_locked());
    assert!(user.lock_reason.is_some());
    assert!(user.get_lock_remaining_seconds().unwrap() > 0);

    // 解锁账户
    user.unlock_account();
    assert!(!user.is_locked());
    assert!(user.lock_reason.is_none());
}

/// 测试登录失败记录
#[test]
fn test_user_login_failure_tracking() {
    let tenant_id = TenantId::new();
    let mut user = create_test_user(tenant_id);
    user.activate();

    assert_eq!(user.failed_login_count, 0);

    for _ in 0..5 {
        user.record_login_failure();
    }

    assert_eq!(user.failed_login_count, 5);
    assert!(user.last_failed_login_at.is_some());

    user.clear_login_failures();
    assert_eq!(user.failed_login_count, 0);
    assert!(user.last_failed_login_at.is_none());
}
