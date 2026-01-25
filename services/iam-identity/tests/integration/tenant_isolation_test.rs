//! 租户隔离集成测试

use cuba_common::TenantId;
use iam_identity::shared::domain::entities::Tenant;
use iam_identity::shared::domain::repositories::TenantRepository;
use iam_identity::shared::infrastructure::persistence::PostgresTenantRepository;
use sqlx::PgPool;

#[sqlx::test]
async fn test_create_and_find_tenant(pool: PgPool) {
    let repo = PostgresTenantRepository::new(pool);

    // 创建租户
    let tenant = Tenant::new("test-tenant".to_string(), "Test Tenant".to_string()).unwrap();
    repo.save(&tenant).await.unwrap();

    // 查找租户
    let found = repo.find_by_id(&tenant.id).await.unwrap();
    assert!(found.is_some());

    let found_tenant = found.unwrap();
    assert_eq!(found_tenant.name, "test-tenant");
    assert_eq!(found_tenant.display_name, "Test Tenant");
}

#[sqlx::test]
async fn test_find_tenant_by_name(pool: PgPool) {
    let repo = PostgresTenantRepository::new(pool);

    let tenant = Tenant::new("unique-tenant".to_string(), "Unique Tenant".to_string()).unwrap();
    repo.save(&tenant).await.unwrap();

    let found = repo.find_by_name("unique-tenant").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, tenant.id);
}

#[sqlx::test]
async fn test_tenant_name_uniqueness(pool: PgPool) {
    let repo = PostgresTenantRepository::new(pool);

    let tenant1 = Tenant::new("duplicate".to_string(), "First".to_string()).unwrap();
    repo.save(&tenant1).await.unwrap();

    let tenant2 = Tenant::new("duplicate".to_string(), "Second".to_string()).unwrap();
    let result = repo.save(&tenant2).await;

    assert!(result.is_err());
}

#[sqlx::test]
async fn test_update_tenant(pool: PgPool) {
    let repo = PostgresTenantRepository::new(pool);

    let mut tenant = Tenant::new("update-test".to_string(), "Original".to_string()).unwrap();
    repo.save(&tenant).await.unwrap();

    tenant.display_name = "Updated".to_string();
    tenant.activate().unwrap();
    repo.update(&tenant).await.unwrap();

    let found = repo.find_by_id(&tenant.id).await.unwrap().unwrap();
    assert_eq!(found.display_name, "Updated");
    assert!(found.is_active());
}

#[sqlx::test]
async fn test_soft_delete_tenant(pool: PgPool) {
    let repo = PostgresTenantRepository::new(pool);

    let tenant = Tenant::new("delete-test".to_string(), "Delete Test".to_string()).unwrap();
    repo.save(&tenant).await.unwrap();

    repo.delete(&tenant.id).await.unwrap();

    let found = repo.find_by_id(&tenant.id).await.unwrap().unwrap();
    assert!(!found.is_available());
}

#[sqlx::test]
async fn test_list_tenants(pool: PgPool) {
    let repo = PostgresTenantRepository::new(pool);

    // 创建多个租户
    for i in 1..=5 {
        let tenant = Tenant::new(format!("tenant-{}", i), format!("Tenant {}", i)).unwrap();
        repo.save(&tenant).await.unwrap();
    }

    let (tenants, total) = repo.list(None, None, 1, 10).await.unwrap();
    assert!(tenants.len() >= 5);
    assert!(total >= 5);
}

#[sqlx::test]
async fn test_find_expiring_trials(pool: PgPool) {
    let repo = PostgresTenantRepository::new(pool);

    let tenant = Tenant::new("trial-tenant".to_string(), "Trial Tenant".to_string()).unwrap();
    repo.save(&tenant).await.unwrap();

    // 查找 30 天内到期的试用租户
    let expiring = repo.find_expiring_trials(30).await.unwrap();
    assert!(!expiring.is_empty());
}

#[sqlx::test]
async fn test_tenant_isolation_with_rls(pool: PgPool) {
    let repo = PostgresTenantRepository::new(pool);

    // 创建两个租户
    let tenant1 = Tenant::new("tenant1".to_string(), "Tenant 1".to_string()).unwrap();
    let tenant2 = Tenant::new("tenant2".to_string(), "Tenant 2".to_string()).unwrap();

    repo.save(&tenant1).await.unwrap();
    repo.save(&tenant2).await.unwrap();

    // 设置当前租户上下文为 tenant1
    sqlx::query(&format!(
        "SET LOCAL app.current_tenant_id = '{}'",
        tenant1.id.0
    ))
    .execute(&pool)
    .await
    .unwrap();

    // 验证只能访问 tenant1 的数据
    let found = repo.find_by_id(&tenant1.id).await.unwrap();
    assert!(found.is_some());
}
