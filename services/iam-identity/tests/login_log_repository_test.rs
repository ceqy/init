use common::{TenantId, UserId};
use iam_identity::domain::auth::LoginLog;
use iam_identity::domain::repositories::auth::LoginLogRepository;
use iam_identity::infrastructure::persistence::auth::PostgresLoginLogRepository;
use sqlx::PgPool;
use std::env;

async fn get_test_pool() -> PgPool {
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/cuba".to_string());
    PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to database")
}

async fn create_dummy_user(pool: &PgPool, user_id: &UserId, tenant_id: &TenantId, username: &str) {
    // Insert Tenant
    let _ = sqlx::query("INSERT INTO tenants (id, name, display_name) VALUES ($1, $2, $2) ON CONFLICT (id) DO NOTHING")
        .bind(tenant_id.0)
        .bind(format!("tenant_{}", tenant_id.0))
        .execute(pool)
        .await;

    // Insert User
    let _ = sqlx::query("INSERT INTO users (id, username, email, password_hash, tenant_id, status) VALUES ($1, $2, $3, 'hash', $4, 'Active') ON CONFLICT (id) DO NOTHING")
        .bind(user_id.0)
        .bind(username)
        .bind(format!("{}@example.com", username))
        .bind(tenant_id.0)
        .execute(pool)
        .await
        .expect("Failed to create user");
}

#[tokio::test]
async fn test_save_and_find() {
    let pool = get_test_pool().await;
    let repo = PostgresLoginLogRepository::new(pool.clone());

    let tenant_id = TenantId::new();
    let user_id = UserId::new();

    create_dummy_user(&pool, &user_id, &tenant_id, "test_user").await;

    let login_log = LoginLog::success(
        user_id.clone(),
        tenant_id.clone(),
        "test_user".to_string(),
        "127.0.0.1".to_string(),
        "Mozilla/5.0".to_string(),
    );

    // Save
    repo.save(&login_log)
        .await
        .expect("Failed to save login log");

    // Find by ID
    let found = repo
        .find_by_id(&login_log.id, &tenant_id)
        .await
        .expect("Failed to find login log");
    assert!(found.is_some());
    let found_log = found.unwrap();
    assert_eq!(found_log.id, login_log.id);
    assert_eq!(found_log.user_id, Some(user_id));
    assert_eq!(found_log.tenant_id, tenant_id);
    assert_eq!(found_log.username, "test_user");
}

#[tokio::test]
async fn test_tenant_isolation() {
    let pool = get_test_pool().await;
    let repo = PostgresLoginLogRepository::new(pool.clone());

    let tenant_a = TenantId::new();
    let tenant_b = TenantId::new();
    let user_id = UserId::new();

    create_dummy_user(&pool, &user_id, &tenant_a, "user_a").await;
    // Also create tenant_b
    let _ = sqlx::query("INSERT INTO tenants (id, name, display_name) VALUES ($1, $2, $2) ON CONFLICT (id) DO NOTHING")
        .bind(tenant_b.0)
        .bind(format!("tenant_{}", tenant_b.0))
        .execute(&pool)
        .await;

    let log_a = LoginLog::success(
        user_id.clone(),
        tenant_a.clone(),
        "user_a".to_string(),
        "127.0.0.1".to_string(),
        "Agent A".to_string(),
    );

    repo.save(&log_a).await.expect("Failed to save log A");

    // Attempt to find log_a using tenant_b
    let found_by_b = repo
        .find_by_id(&log_a.id, &tenant_b)
        .await
        .expect("Query failed");
    assert!(
        found_by_b.is_none(),
        "Tenant B should not see Tenant A's log"
    );

    // Find correctly with tenant_a
    let found_by_a = repo
        .find_by_id(&log_a.id, &tenant_a)
        .await
        .expect("Query failed");
    assert!(found_by_a.is_some(), "Tenant A should see its own log");
}

#[tokio::test]
async fn test_find_by_user_id() {
    let pool = get_test_pool().await;
    let repo = PostgresLoginLogRepository::new(pool.clone());

    let tenant_id = TenantId::new();
    let user_id = UserId::new();

    create_dummy_user(&pool, &user_id, &tenant_id, "user_multi").await;

    for _ in 0..5 {
        let log = LoginLog::success(
            user_id.clone(),
            tenant_id.clone(),
            "user_multi".to_string(),
            "127.0.0.1".to_string(),
            "Agent".to_string(),
        );
        repo.save(&log).await.expect("Failed to save");
    }

    let logs = repo
        .find_by_user_id(&user_id, &tenant_id, 10)
        .await
        .expect("Failed to list");
    assert_eq!(logs.len(), 5);
}
