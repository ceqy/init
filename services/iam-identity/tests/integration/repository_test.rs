//! Repository 集成测试
//!
//! 测试所有 Repository 实现的数据库操作

use chrono::{Duration, Utc};
use cuba_common::{TenantId, UserId};
use sqlx::PgPool;
use uuid::Uuid;

use iam_identity::auth::domain::entities::{BackupCode, PasswordResetToken, Session, WebAuthnCredential};
use iam_identity::auth::domain::repositories::{
    BackupCodeRepository, PasswordResetRepository, SessionRepository, WebAuthnCredentialRepository,
};
use iam_identity::auth::infrastructure::persistence::{
    PostgresBackupCodeRepository, PostgresPasswordResetRepository, PostgresSessionRepository,
    PostgresWebAuthnCredentialRepository,
};
use iam_identity::shared::domain::entities::User;
use iam_identity::shared::domain::repositories::UserRepository;
use iam_identity::shared::domain::value_objects::{Email, Password, Username};
use iam_identity::shared::infrastructure::persistence::PostgresUserRepository;

// ============================================================
// 测试辅助函数
// ============================================================

fn create_test_tenant_id() -> TenantId {
    TenantId(Uuid::now_v7())
}

fn create_test_user_id() -> UserId {
    UserId(Uuid::now_v7())
}

fn create_test_user(tenant_id: TenantId) -> User {
    let username = Username::new(&format!("testuser_{}", Uuid::now_v7())).unwrap();
    let email = Email::new(&format!("test_{}@example.com", Uuid::now_v7())).unwrap();
    let password = Password::new("SecurePass123!").unwrap();

    User::new(username, email, password, tenant_id).unwrap()
}

// ============================================================
// UserRepository 集成测试
// ============================================================

#[sqlx::test]
async fn test_user_repository_save_and_find_by_id(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user = create_test_user(tenant_id.clone());
    let user_id = user.id.clone();

    // 保存用户
    let save_result = repo.save(&user).await;
    assert!(save_result.is_ok(), "Failed to save user: {:?}", save_result.err());

    // 查找用户
    let found = repo.find_by_id(&user_id, &tenant_id).await.unwrap();
    assert!(found.is_some(), "User not found");
    
    let found_user = found.unwrap();
    assert_eq!(found_user.id, user_id);
    assert_eq!(found_user.username, user.username);
    assert_eq!(found_user.email, user.email);
}

#[sqlx::test]
async fn test_user_repository_find_by_username(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user = create_test_user(tenant_id.clone());
    let username = user.username.clone();

    repo.save(&user).await.unwrap();

    // 查找用户
    let found = repo.find_by_username(&username, &tenant_id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().username, username);
}

#[sqlx::test]
async fn test_user_repository_find_by_email(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user = create_test_user(tenant_id.clone());
    let email = user.email.clone();

    repo.save(&user).await.unwrap();

    // 查找用户
    let found = repo.find_by_email(&email, &tenant_id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().email, email);
}

#[sqlx::test]
async fn test_user_repository_update(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let mut user = create_test_user(tenant_id.clone());

    repo.save(&user).await.unwrap();

    // 更新用户
    user.activate().unwrap();
    let update_result = repo.update(&user).await;
    assert!(update_result.is_ok());

    // 验证更新
    let found = repo.find_by_id(&user.id, &tenant_id).await.unwrap().unwrap();
    assert!(found.is_active());
}

#[sqlx::test]
async fn test_user_repository_delete(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user = create_test_user(tenant_id.clone());
    let user_id = user.id.clone();

    repo.save(&user).await.unwrap();

    // 删除用户
    let delete_result = repo.delete(&user_id, &tenant_id).await;
    assert!(delete_result.is_ok());

    // 验证已删除
    let found = repo.find_by_id(&user_id, &tenant_id).await.unwrap();
    assert!(found.is_none());
}

#[sqlx::test]
async fn test_user_repository_exists_by_username(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user = create_test_user(tenant_id.clone());
    let username = user.username.clone();

    // 保存前不存在
    let exists_before = repo.exists_by_username(&username, &tenant_id).await.unwrap();
    assert!(!exists_before);

    repo.save(&user).await.unwrap();

    // 保存后存在
    let exists_after = repo.exists_by_username(&username, &tenant_id).await.unwrap();
    assert!(exists_after);
}

#[sqlx::test]
async fn test_user_repository_exists_by_email(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user = create_test_user(tenant_id.clone());
    let email = user.email.clone();

    // 保存前不存在
    let exists_before = repo.exists_by_email(&email, &tenant_id).await.unwrap();
    assert!(!exists_before);

    repo.save(&user).await.unwrap();

    // 保存后存在
    let exists_after = repo.exists_by_email(&email, &tenant_id).await.unwrap();
    assert!(exists_after);
}

#[sqlx::test]
async fn test_user_repository_count_by_tenant(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool);
    let tenant_id = create_test_tenant_id();

    // 初始计数为 0
    let count_before = repo.count_by_tenant(&tenant_id).await.unwrap();
    assert_eq!(count_before, 0);

    // 创建 3 个用户
    for _ in 0..3 {
        let user = create_test_user(tenant_id.clone());
        repo.save(&user).await.unwrap();
    }

    // 计数应为 3
    let count_after = repo.count_by_tenant(&tenant_id).await.unwrap();
    assert_eq!(count_after, 3);
}

#[sqlx::test]
async fn test_user_repository_tenant_isolation(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool);
    let tenant1 = create_test_tenant_id();
    let tenant2 = create_test_tenant_id();

    let user1 = create_test_user(tenant1.clone());
    let user2 = create_test_user(tenant2.clone());

    repo.save(&user1).await.unwrap();
    repo.save(&user2).await.unwrap();

    // 租户 1 不能访问租户 2 的用户
    let found = repo.find_by_id(&user2.id, &tenant1).await.unwrap();
    assert!(found.is_none());

    // 租户 2 不能访问租户 1 的用户
    let found = repo.find_by_id(&user1.id, &tenant2).await.unwrap();
    assert!(found.is_none());
}

// ============================================================
// SessionRepository 集成测试
// ============================================================

fn create_test_session(user_id: UserId, tenant_id: TenantId) -> Session {
    Session::new(
        user_id,
        tenant_id,
        "test_token_hash".to_string(),
        "127.0.0.1".to_string(),
        "Test User Agent".to_string(),
    )
}

#[sqlx::test]
async fn test_session_repository_save_and_find_by_id(pool: PgPool) {
    let repo = PostgresSessionRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();
    let session = create_test_session(user_id, tenant_id.clone());
    let session_id = session.id.clone();

    // 保存会话
    let save_result = repo.save(&session).await;
    assert!(save_result.is_ok());

    // 查找会话
    let found = repo.find_by_id(&session_id, &tenant_id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, session_id);
}

#[sqlx::test]
async fn test_session_repository_find_by_refresh_token_hash(pool: PgPool) {
    let repo = PostgresSessionRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();
    let session = create_test_session(user_id, tenant_id.clone());
    let token_hash = session.refresh_token_hash.clone();

    repo.save(&session).await.unwrap();

    // 根据 token hash 查找
    let found = repo.find_by_refresh_token_hash(&token_hash, &tenant_id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().refresh_token_hash, token_hash);
}

#[sqlx::test]
async fn test_session_repository_find_active_by_user_id(pool: PgPool) {
    let repo = PostgresSessionRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();

    // 创建 3 个活跃会话
    for i in 0..3 {
        let mut session = create_test_session(user_id.clone(), tenant_id.clone());
        session.refresh_token_hash = format!("token_hash_{}", i);
        repo.save(&session).await.unwrap();
    }

    // 查找用户的所有活跃会话
    let sessions = repo.find_active_by_user_id(&user_id, &tenant_id).await.unwrap();
    assert_eq!(sessions.len(), 3);
}

#[sqlx::test]
async fn test_session_repository_update(pool: PgPool) {
    let repo = PostgresSessionRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();
    let mut session = create_test_session(user_id, tenant_id.clone());

    repo.save(&session).await.unwrap();

    // 更新会话
    session.revoke();
    let update_result = repo.update(&session).await;
    assert!(update_result.is_ok());

    // 验证更新
    let found = repo.find_by_id(&session.id, &tenant_id).await.unwrap().unwrap();
    assert!(found.is_revoked());
}

#[sqlx::test]
async fn test_session_repository_delete(pool: PgPool) {
    let repo = PostgresSessionRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();
    let session = create_test_session(user_id, tenant_id.clone());
    let session_id = session.id.clone();

    repo.save(&session).await.unwrap();

    // 删除会话
    let delete_result = repo.delete(&session_id, &tenant_id).await;
    assert!(delete_result.is_ok());

    // 验证已删除
    let found = repo.find_by_id(&session_id, &tenant_id).await.unwrap();
    assert!(found.is_none());
}

#[sqlx::test]
async fn test_session_repository_revoke_all_by_user_id(pool: PgPool) {
    let repo = PostgresSessionRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();

    // 创建 3 个会话
    for i in 0..3 {
        let mut session = create_test_session(user_id.clone(), tenant_id.clone());
        session.refresh_token_hash = format!("token_hash_{}", i);
        repo.save(&session).await.unwrap();
    }

    // 撤销所有会话
    let revoke_result = repo.revoke_all_by_user_id(&user_id, &tenant_id).await;
    assert!(revoke_result.is_ok());

    // 验证所有会话都被撤销
    let sessions = repo.find_active_by_user_id(&user_id, &tenant_id).await.unwrap();
    assert_eq!(sessions.len(), 0);
}

#[sqlx::test]
async fn test_session_repository_cleanup_expired(pool: PgPool) {
    let repo = PostgresSessionRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();

    // 创建一个过期的会话
    let mut session = create_test_session(user_id, tenant_id.clone());
    session.expires_at = Utc::now() - Duration::hours(1);
    repo.save(&session).await.unwrap();

    // 清理过期会话
    let cleaned = repo.cleanup_expired(&tenant_id).await.unwrap();
    assert_eq!(cleaned, 1);

    // 验证已清理
    let found = repo.find_by_id(&session.id, &tenant_id).await.unwrap();
    assert!(found.is_none());
}

#[sqlx::test]
async fn test_session_repository_tenant_isolation(pool: PgPool) {
    let repo = PostgresSessionRepository::new(pool);
    let tenant1 = create_test_tenant_id();
    let tenant2 = create_test_tenant_id();
    let user_id = create_test_user_id();

    let session1 = create_test_session(user_id.clone(), tenant1.clone());
    let session2 = create_test_session(user_id.clone(), tenant2.clone());

    repo.save(&session1).await.unwrap();
    repo.save(&session2).await.unwrap();

    // 租户 1 不能访问租户 2 的会话
    let found = repo.find_by_id(&session2.id, &tenant1).await.unwrap();
    assert!(found.is_none());

    // 租户 2 不能访问租户 1 的会话
    let found = repo.find_by_id(&session1.id, &tenant2).await.unwrap();
    assert!(found.is_none());
}

// ============================================================
// BackupCodeRepository 集成测试
// ============================================================

fn create_test_backup_code(user_id: UserId, tenant_id: TenantId, code: &str) -> BackupCode {
    BackupCode::new(user_id, tenant_id, code.to_string())
}

#[sqlx::test]
async fn test_backup_code_repository_save_and_find_by_id(pool: PgPool) {
    let repo = PostgresBackupCodeRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();
    let backup_code = create_test_backup_code(user_id, tenant_id.clone(), "ABC123");
    let code_id = backup_code.id.clone();

    // 保存备份码
    let save_result = repo.save(&backup_code).await;
    assert!(save_result.is_ok());

    // 查找备份码
    let found = repo.find_by_id(&code_id, &tenant_id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, code_id);
}

#[sqlx::test]
async fn test_backup_code_repository_save_batch(pool: PgPool) {
    let repo = PostgresBackupCodeRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();

    // 创建 5 个备份码
    let codes: Vec<BackupCode> = (0..5)
        .map(|i| create_test_backup_code(user_id.clone(), tenant_id.clone(), &format!("CODE{}", i)))
        .collect();

    // 批量保存
    let save_result = repo.save_batch(&codes).await;
    assert!(save_result.is_ok());

    // 验证所有备份码都已保存
    let found_codes = repo.find_available_by_user_id(&user_id, &tenant_id).await.unwrap();
    assert_eq!(found_codes.len(), 5);
}

#[sqlx::test]
async fn test_backup_code_repository_find_available_by_user_id(pool: PgPool) {
    let repo = PostgresBackupCodeRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();

    // 创建 3 个可用备份码
    for i in 0..3 {
        let code = create_test_backup_code(user_id.clone(), tenant_id.clone(), &format!("CODE{}", i));
        repo.save(&code).await.unwrap();
    }

    // 创建 1 个已使用的备份码
    let mut used_code = create_test_backup_code(user_id.clone(), tenant_id.clone(), "USED");
    used_code.mark_as_used();
    repo.save(&used_code).await.unwrap();

    // 查找可用备份码（应该只有 3 个）
    let available = repo.find_available_by_user_id(&user_id, &tenant_id).await.unwrap();
    assert_eq!(available.len(), 3);
}

#[sqlx::test]
async fn test_backup_code_repository_update(pool: PgPool) {
    let repo = PostgresBackupCodeRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();
    let mut backup_code = create_test_backup_code(user_id, tenant_id.clone(), "ABC123");

    repo.save(&backup_code).await.unwrap();

    // 标记为已使用
    backup_code.mark_as_used();
    let update_result = repo.update(&backup_code).await;
    assert!(update_result.is_ok());

    // 验证更新
    let found = repo.find_by_id(&backup_code.id, &tenant_id).await.unwrap().unwrap();
    assert!(found.is_used());
}

#[sqlx::test]
async fn test_backup_code_repository_delete_by_user_id(pool: PgPool) {
    let repo = PostgresBackupCodeRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();

    // 创建 3 个备份码
    for i in 0..3 {
        let code = create_test_backup_code(user_id.clone(), tenant_id.clone(), &format!("CODE{}", i));
        repo.save(&code).await.unwrap();
    }

    // 删除用户的所有备份码
    let delete_result = repo.delete_by_user_id(&user_id, &tenant_id).await;
    assert!(delete_result.is_ok());

    // 验证已删除
    let found = repo.find_available_by_user_id(&user_id, &tenant_id).await.unwrap();
    assert_eq!(found.len(), 0);
}

#[sqlx::test]
async fn test_backup_code_repository_count_available_by_user_id(pool: PgPool) {
    let repo = PostgresBackupCodeRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();

    // 初始计数为 0
    let count_before = repo.count_available_by_user_id(&user_id, &tenant_id).await.unwrap();
    assert_eq!(count_before, 0);

    // 创建 5 个备份码
    for i in 0..5 {
        let code = create_test_backup_code(user_id.clone(), tenant_id.clone(), &format!("CODE{}", i));
        repo.save(&code).await.unwrap();
    }

    // 计数应为 5
    let count_after = repo.count_available_by_user_id(&user_id, &tenant_id).await.unwrap();
    assert_eq!(count_after, 5);
}

#[sqlx::test]
async fn test_backup_code_repository_tenant_isolation(pool: PgPool) {
    let repo = PostgresBackupCodeRepository::new(pool);
    let tenant1 = create_test_tenant_id();
    let tenant2 = create_test_tenant_id();
    let user_id = create_test_user_id();

    let code1 = create_test_backup_code(user_id.clone(), tenant1.clone(), "CODE1");
    let code2 = create_test_backup_code(user_id.clone(), tenant2.clone(), "CODE2");

    repo.save(&code1).await.unwrap();
    repo.save(&code2).await.unwrap();

    // 租户 1 不能访问租户 2 的备份码
    let found = repo.find_by_id(&code2.id, &tenant1).await.unwrap();
    assert!(found.is_none());

    // 租户 2 不能访问租户 1 的备份码
    let found = repo.find_by_id(&code1.id, &tenant2).await.unwrap();
    assert!(found.is_none());
}

// ============================================================
// PasswordResetRepository 集成测试
// ============================================================

fn create_test_password_reset_token(user_id: UserId, tenant_id: TenantId) -> PasswordResetToken {
    PasswordResetToken::new(
        user_id,
        tenant_id,
        "test_token_hash".to_string(),
        Duration::hours(1),
    )
}

#[sqlx::test]
async fn test_password_reset_repository_save_and_find_by_id(pool: PgPool) {
    let repo = PostgresPasswordResetRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();
    let token = create_test_password_reset_token(user_id, tenant_id.clone());
    let token_id = token.id.clone();

    // 保存令牌
    let save_result = repo.save(&token).await;
    assert!(save_result.is_ok());

    // 查找令牌
    let found = repo.find_by_id(&token_id, &tenant_id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, token_id);
}

#[sqlx::test]
async fn test_password_reset_repository_find_by_token_hash(pool: PgPool) {
    let repo = PostgresPasswordResetRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();
    let token = create_test_password_reset_token(user_id, tenant_id.clone());
    let token_hash = token.token_hash.clone();

    repo.save(&token).await.unwrap();

    // 根据 token hash 查找
    let found = repo.find_by_token_hash(&token_hash, &tenant_id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().token_hash, token_hash);
}

#[sqlx::test]
async fn test_password_reset_repository_update(pool: PgPool) {
    let repo = PostgresPasswordResetRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();
    let mut token = create_test_password_reset_token(user_id, tenant_id.clone());

    repo.save(&token).await.unwrap();

    // 标记为已使用
    token.mark_as_used();
    let update_result = repo.update(&token).await;
    assert!(update_result.is_ok());

    // 验证更新
    let found = repo.find_by_id(&token.id, &tenant_id).await.unwrap().unwrap();
    assert!(found.is_used());
}

#[sqlx::test]
async fn test_password_reset_repository_mark_as_used(pool: PgPool) {
    let repo = PostgresPasswordResetRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();
    let token = create_test_password_reset_token(user_id, tenant_id.clone());
    let token_id = token.id.clone();

    repo.save(&token).await.unwrap();

    // 标记为已使用
    let mark_result = repo.mark_as_used(&token_id, &tenant_id).await;
    assert!(mark_result.is_ok());

    // 验证已标记
    let found = repo.find_by_id(&token_id, &tenant_id).await.unwrap().unwrap();
    assert!(found.is_used());
}

#[sqlx::test]
async fn test_password_reset_repository_delete_by_user_id(pool: PgPool) {
    let repo = PostgresPasswordResetRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();

    // 创建 3 个令牌
    for i in 0..3 {
        let mut token = create_test_password_reset_token(user_id.clone(), tenant_id.clone());
        token.token_hash = format!("token_hash_{}", i);
        repo.save(&token).await.unwrap();
    }

    // 删除用户的所有令牌
    let delete_result = repo.delete_by_user_id(&user_id, &tenant_id).await;
    assert!(delete_result.is_ok());

    // 验证已删除
    let count = repo.count_unused_by_user_id(&user_id, &tenant_id).await.unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test]
async fn test_password_reset_repository_delete_expired(pool: PgPool) {
    let repo = PostgresPasswordResetRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();

    // 创建一个过期的令牌
    let mut token = create_test_password_reset_token(user_id, tenant_id.clone());
    token.expires_at = Utc::now() - Duration::hours(1);
    repo.save(&token).await.unwrap();

    // 删除过期令牌
    let deleted = repo.delete_expired(&tenant_id).await.unwrap();
    assert_eq!(deleted, 1);

    // 验证已删除
    let found = repo.find_by_id(&token.id, &tenant_id).await.unwrap();
    assert!(found.is_none());
}

#[sqlx::test]
async fn test_password_reset_repository_count_unused_by_user_id(pool: PgPool) {
    let repo = PostgresPasswordResetRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();

    // 初始计数为 0
    let count_before = repo.count_unused_by_user_id(&user_id, &tenant_id).await.unwrap();
    assert_eq!(count_before, 0);

    // 创建 3 个未使用的令牌
    for i in 0..3 {
        let mut token = create_test_password_reset_token(user_id.clone(), tenant_id.clone());
        token.token_hash = format!("token_hash_{}", i);
        repo.save(&token).await.unwrap();
    }

    // 创建 1 个已使用的令牌
    let mut used_token = create_test_password_reset_token(user_id.clone(), tenant_id.clone());
    used_token.token_hash = "used_token".to_string();
    used_token.mark_as_used();
    repo.save(&used_token).await.unwrap();

    // 计数应为 3（不包括已使用的）
    let count_after = repo.count_unused_by_user_id(&user_id, &tenant_id).await.unwrap();
    assert_eq!(count_after, 3);
}

#[sqlx::test]
async fn test_password_reset_repository_tenant_isolation(pool: PgPool) {
    let repo = PostgresPasswordResetRepository::new(pool);
    let tenant1 = create_test_tenant_id();
    let tenant2 = create_test_tenant_id();
    let user_id = create_test_user_id();

    let token1 = create_test_password_reset_token(user_id.clone(), tenant1.clone());
    let token2 = create_test_password_reset_token(user_id.clone(), tenant2.clone());

    repo.save(&token1).await.unwrap();
    repo.save(&token2).await.unwrap();

    // 租户 1 不能访问租户 2 的令牌
    let found = repo.find_by_id(&token2.id, &tenant1).await.unwrap();
    assert!(found.is_none());

    // 租户 2 不能访问租户 1 的令牌
    let found = repo.find_by_id(&token1.id, &tenant2).await.unwrap();
    assert!(found.is_none());
}

// ============================================================
// WebAuthnCredentialRepository 集成测试
// ============================================================

fn create_test_webauthn_credential(user_id: UserId, tenant_id: TenantId) -> WebAuthnCredential {
    WebAuthnCredential::new(
        user_id,
        tenant_id,
        vec![1, 2, 3, 4, 5], // credential_id
        vec![6, 7, 8, 9, 10], // public_key
        0, // sign_count
        "Test Device".to_string(),
    )
}

#[sqlx::test]
async fn test_webauthn_credential_repository_save_and_find_by_id(pool: PgPool) {
    let repo = PostgresWebAuthnCredentialRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();
    let credential = create_test_webauthn_credential(user_id, tenant_id.clone());
    let credential_id = credential.id.clone();

    // 保存凭证
    let save_result = repo.save(&credential).await;
    assert!(save_result.is_ok());

    // 查找凭证
    let found = repo.find_by_id(&credential_id, &tenant_id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, credential_id);
}

#[sqlx::test]
async fn test_webauthn_credential_repository_find_by_credential_id(pool: PgPool) {
    let repo = PostgresWebAuthnCredentialRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();
    let credential = create_test_webauthn_credential(user_id, tenant_id.clone());
    let cred_id = credential.credential_id.clone();

    repo.save(&credential).await.unwrap();

    // 根据 credential_id 查找
    let found = repo.find_by_credential_id(&cred_id, &tenant_id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().credential_id, cred_id);
}

#[sqlx::test]
async fn test_webauthn_credential_repository_find_by_user_id(pool: PgPool) {
    let repo = PostgresWebAuthnCredentialRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();

    // 创建 3 个凭证
    for i in 0..3 {
        let mut credential = create_test_webauthn_credential(user_id.clone(), tenant_id.clone());
        credential.credential_id = vec![i, i + 1, i + 2];
        credential.name = format!("Device {}", i);
        repo.save(&credential).await.unwrap();
    }

    // 查找用户的所有凭证
    let credentials = repo.find_by_user_id(&user_id, &tenant_id).await.unwrap();
    assert_eq!(credentials.len(), 3);
}

#[sqlx::test]
async fn test_webauthn_credential_repository_update(pool: PgPool) {
    let repo = PostgresWebAuthnCredentialRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();
    let mut credential = create_test_webauthn_credential(user_id, tenant_id.clone());

    repo.save(&credential).await.unwrap();

    // 更新签名计数
    credential.increment_sign_count();
    let update_result = repo.update(&credential).await;
    assert!(update_result.is_ok());

    // 验证更新
    let found = repo.find_by_id(&credential.id, &tenant_id).await.unwrap().unwrap();
    assert_eq!(found.sign_count, 1);
}

#[sqlx::test]
async fn test_webauthn_credential_repository_delete(pool: PgPool) {
    let repo = PostgresWebAuthnCredentialRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();
    let credential = create_test_webauthn_credential(user_id, tenant_id.clone());
    let credential_id = credential.id.clone();

    repo.save(&credential).await.unwrap();

    // 删除凭证
    let delete_result = repo.delete(&credential_id, &tenant_id).await;
    assert!(delete_result.is_ok());

    // 验证已删除
    let found = repo.find_by_id(&credential_id, &tenant_id).await.unwrap();
    assert!(found.is_none());
}

#[sqlx::test]
async fn test_webauthn_credential_repository_has_credentials(pool: PgPool) {
    let repo = PostgresWebAuthnCredentialRepository::new(pool);
    let tenant_id = create_test_tenant_id();
    let user_id = create_test_user_id();

    // 保存前没有凭证
    let has_before = repo.has_credentials(&user_id, &tenant_id).await.unwrap();
    assert!(!has_before);

    // 创建凭证
    let credential = create_test_webauthn_credential(user_id.clone(), tenant_id.clone());
    repo.save(&credential).await.unwrap();

    // 保存后有凭证
    let has_after = repo.has_credentials(&user_id, &tenant_id).await.unwrap();
    assert!(has_after);
}

#[sqlx::test]
async fn test_webauthn_credential_repository_tenant_isolation(pool: PgPool) {
    let repo = PostgresWebAuthnCredentialRepository::new(pool);
    let tenant1 = create_test_tenant_id();
    let tenant2 = create_test_tenant_id();
    let user_id = create_test_user_id();

    let credential1 = create_test_webauthn_credential(user_id.clone(), tenant1.clone());
    let credential2 = create_test_webauthn_credential(user_id.clone(), tenant2.clone());

    repo.save(&credential1).await.unwrap();
    repo.save(&credential2).await.unwrap();

    // 租户 1 不能访问租户 2 的凭证
    let found = repo.find_by_id(&credential2.id, &tenant1).await.unwrap();
    assert!(found.is_none());

    // 租户 2 不能访问租户 1 的凭证
    let found = repo.find_by_id(&credential1.id, &tenant2).await.unwrap();
    assert!(found.is_none());
}
