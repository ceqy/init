//! 租户隔离集成测试

use cuba_common::{TenantId, UserId};
use iam_identity::shared::domain::entities::User;
use iam_identity::shared::domain::repositories::UserRepository;
use iam_identity::shared::domain::value_objects::{Email, HashedPassword, Username};
use iam_identity::shared::infrastructure::persistence::PostgresUserRepository;
use sqlx::PgPool;

/// 测试跨租户访问应该失败
#[sqlx::test]
async fn test_cross_tenant_access_should_fail(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool.clone());

    // 创建两个不同的租户
    let tenant1 = TenantId::new();
    let tenant2 = TenantId::new();

    // 在租户1中创建用户
    let user1 = User::new(
        Username::new("user1").unwrap(),
        Email::new("user1@example.com").unwrap(),
        HashedPassword::new("hashed_password".to_string()),
        tenant1.clone(),
    );

    repo.save(&user1).await.unwrap();

    // 尝试用租户2的身份访问租户1的用户（应该失败）
    let result = repo.find_by_id(&user1.id, &tenant2).await.unwrap();
    assert!(result.is_none(), "Should not find user from different tenant");

    // 用正确的租户ID访问（应该成功）
    let result = repo.find_by_id(&user1.id, &tenant1).await.unwrap();
    assert!(result.is_some(), "Should find user with correct tenant ID");
}

/// 测试租户隔离的用户名唯一性
#[sqlx::test]
async fn test_username_uniqueness_per_tenant(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool.clone());

    let tenant1 = TenantId::new();
    let tenant2 = TenantId::new();

    let username = Username::new("testuser").unwrap();

    // 在租户1中创建用户
    let user1 = User::new(
        username.clone(),
        Email::new("user1@example.com").unwrap(),
        HashedPassword::new("hashed_password".to_string()),
        tenant1.clone(),
    );
    repo.save(&user1).await.unwrap();

    // 在租户2中创建相同用户名的用户（应该成功，因为是不同租户）
    let user2 = User::new(
        username.clone(),
        Email::new("user2@example.com").unwrap(),
        HashedPassword::new("hashed_password".to_string()),
        tenant2.clone(),
    );
    repo.save(&user2).await.unwrap();

    // 验证两个租户都能找到各自的用户
    let found1 = repo.find_by_username(&username, &tenant1).await.unwrap();
    assert!(found1.is_some());
    assert_eq!(found1.unwrap().tenant_id, tenant1);

    let found2 = repo.find_by_username(&username, &tenant2).await.unwrap();
    assert!(found2.is_some());
    assert_eq!(found2.unwrap().tenant_id, tenant2);
}

/// 测试租户隔离的用户列表查询
#[sqlx::test]
async fn test_list_users_tenant_isolation(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool.clone());

    let tenant1 = TenantId::new();
    let tenant2 = TenantId::new();

    // 在租户1中创建2个用户
    for i in 1..=2 {
        let user = User::new(
            Username::new(&format!("tenant1_user{}", i)).unwrap(),
            Email::new(&format!("tenant1_user{}@example.com", i)).unwrap(),
            HashedPassword::new("hashed_password".to_string()),
            tenant1.clone(),
        );
        repo.save(&user).await.unwrap();
    }

    // 在租户2中创建3个用户
    for i in 1..=3 {
        let user = User::new(
            Username::new(&format!("tenant2_user{}", i)).unwrap(),
            Email::new(&format!("tenant2_user{}@example.com", i)).unwrap(),
            HashedPassword::new("hashed_password".to_string()),
            tenant2.clone(),
        );
        repo.save(&user).await.unwrap();
    }

    // 查询租户1的用户（应该只返回2个）
    let (users1, count1) = repo
        .list(&tenant1, None, None, &[], 1, 10)
        .await
        .unwrap();
    assert_eq!(count1, 2, "Tenant 1 should have 2 users");
    assert_eq!(users1.len(), 2);
    assert!(users1.iter().all(|u| u.tenant_id == tenant1));

    // 查询租户2的用户（应该只返回3个）
    let (users2, count2) = repo
        .list(&tenant2, None, None, &[], 1, 10)
        .await
        .unwrap();
    assert_eq!(count2, 3, "Tenant 2 should have 3 users");
    assert_eq!(users2.len(), 3);
    assert!(users2.iter().all(|u| u.tenant_id == tenant2));
}

/// 测试租户用户数量统计
#[sqlx::test]
async fn test_count_users_by_tenant(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool.clone());

    let tenant1 = TenantId::new();
    let tenant2 = TenantId::new();

    // 在租户1中创建5个用户
    for i in 1..=5 {
        let user = User::new(
            Username::new(&format!("tenant1_user{}", i)).unwrap(),
            Email::new(&format!("tenant1_user{}@example.com", i)).unwrap(),
            HashedPassword::new("hashed_password".to_string()),
            tenant1.clone(),
        );
        repo.save(&user).await.unwrap();
    }

    // 在租户2中创建3个用户
    for i in 1..=3 {
        let user = User::new(
            Username::new(&format!("tenant2_user{}", i)).unwrap(),
            Email::new(&format!("tenant2_user{}@example.com", i)).unwrap(),
            HashedPassword::new("hashed_password".to_string()),
            tenant2.clone(),
        );
        repo.save(&user).await.unwrap();
    }

    // 统计各租户的用户数量
    let count1 = repo.count_by_tenant(&tenant1).await.unwrap();
    assert_eq!(count1, 5, "Tenant 1 should have 5 users");

    let count2 = repo.count_by_tenant(&tenant2).await.unwrap();
    assert_eq!(count2, 3, "Tenant 2 should have 3 users");
}

/// 测试删除用户的租户隔离
#[sqlx::test]
async fn test_delete_user_tenant_isolation(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool.clone());

    let tenant1 = TenantId::new();
    let tenant2 = TenantId::new();

    // 在租户1中创建用户
    let user = User::new(
        Username::new("testuser").unwrap(),
        Email::new("test@example.com").unwrap(),
        HashedPassword::new("hashed_password".to_string()),
        tenant1.clone(),
    );
    repo.save(&user).await.unwrap();

    // 尝试用租户2的身份删除租户1的用户（应该失败或无效）
    let result = repo.delete(&user.id, &tenant2).await;
    // 根据实现，这可能成功但不删除任何内容，或者返回错误

    // 验证用户仍然存在
    let found = repo.find_by_id(&user.id, &tenant1).await.unwrap();
    assert!(found.is_some(), "User should still exist");

    // 用正确的租户ID删除（应该成功）
    repo.delete(&user.id, &tenant1).await.unwrap();

    // 验证用户已被删除
    let found = repo.find_by_id(&user.id, &tenant1).await.unwrap();
    assert!(found.is_none(), "User should be deleted");
}

/// 测试更新用户的租户隔离
#[sqlx::test]
async fn test_update_user_tenant_isolation(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool.clone());

    let tenant1 = TenantId::new();
    let tenant2 = TenantId::new();

    // 在租户1中创建用户
    let mut user = User::new(
        Username::new("testuser").unwrap(),
        Email::new("test@example.com").unwrap(),
        HashedPassword::new("hashed_password".to_string()),
        tenant1.clone(),
    );
    repo.save(&user).await.unwrap();

    // 尝试修改用户的租户ID（应该被阻止）
    user.tenant_id = tenant2.clone();
    let result = repo.update(&user).await;
    // 根据实现，这应该失败或被忽略

    // 验证用户的租户ID没有改变
    let found = repo.find_by_id(&user.id, &tenant1).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().tenant_id, tenant1);
}

/// 性能测试：大量租户数据查询
#[sqlx::test]
async fn test_performance_with_many_tenants(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool.clone());

    // 创建10个租户，每个租户10个用户
    let mut tenants = Vec::new();
    for _ in 0..10 {
        let tenant_id = TenantId::new();
        tenants.push(tenant_id.clone());

        for i in 1..=10 {
            let user = User::new(
                Username::new(&format!("user{}_{}", tenant_id.0, i)).unwrap(),
                Email::new(&format!("user{}_{}@example.com", tenant_id.0, i)).unwrap(),
                HashedPassword::new("hashed_password".to_string()),
                tenant_id.clone(),
            );
            repo.save(&user).await.unwrap();
        }
    }

    // 测试查询性能
    use std::time::Instant;

    for tenant_id in &tenants {
        let start = Instant::now();
        let (users, count) = repo
            .list(tenant_id, None, None, &[], 1, 10)
            .await
            .unwrap();
        let duration = start.elapsed();

        assert_eq!(count, 10);
        assert_eq!(users.len(), 10);
        assert!(users.iter().all(|u| &u.tenant_id == tenant_id));

        // 查询应该在合理时间内完成（< 100ms）
        assert!(
            duration.as_millis() < 100,
            "Query took too long: {:?}",
            duration
        );
    }
}
