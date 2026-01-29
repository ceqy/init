use cuba_common::{TenantId, UserId};
use iam_identity::domain::oauth::{OAuthClient, OAuthClientType};
use iam_identity::domain::repositories::oauth::OAuthClientRepository;
use iam_identity::infrastructure::persistence::oauth::PostgresOAuthClientRepository;
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
    let repo = PostgresOAuthClientRepository::new(pool.clone());

    let tenant_id = TenantId::new();
    let owner_id = UserId::new();

    create_dummy_user(&pool, &owner_id, &tenant_id, "oauth_owner").await;

    let client = OAuthClient::new(
        tenant_id.clone(),
        owner_id.clone(),
        "Test Client".to_string(),
        OAuthClientType::Confidential,
        vec!["https://example.com/callback".to_string()],
    )
    .expect("Failed to create client");

    // Save
    repo.save(&client)
        .await
        .expect("Failed to save oauth client");

    // Find by ID
    let found = repo
        .find_by_id(&client.id, &tenant_id)
        .await
        .expect("Failed to find oauth client");
    assert!(found.is_some());
    let found_client = found.unwrap();
    assert_eq!(found_client.id, client.id);
    assert_eq!(found_client.name, "Test Client");
    assert_eq!(found_client.tenant_id, tenant_id);
}

#[tokio::test]
async fn test_tenant_isolation() {
    let pool = get_test_pool().await;
    let repo = PostgresOAuthClientRepository::new(pool.clone());

    let tenant_a = TenantId::new();
    let owner_a = UserId::new();
    create_dummy_user(&pool, &owner_a, &tenant_a, "owner_a").await;

    let tenant_b = TenantId::new();
    let owner_b = UserId::new();
    create_dummy_user(&pool, &owner_b, &tenant_b, "owner_b").await;

    let client_a = OAuthClient::new(
        tenant_a.clone(),
        owner_a.clone(),
        "Client A".to_string(),
        OAuthClientType::Confidential,
        vec!["https://a.com".to_string()],
    )
    .unwrap();

    repo.save(&client_a).await.expect("Failed to save client A");

    // Tenant B try to find Client A
    let found_by_b = repo
        .find_by_id(&client_a.id, &tenant_b)
        .await
        .expect("Query failed");
    assert!(found_by_b.is_none());

    // Tenant A find Client A
    let found_by_a = repo
        .find_by_id(&client_a.id, &tenant_a)
        .await
        .expect("Query failed");
    assert!(found_by_a.is_some());
}

#[tokio::test]
async fn test_list_by_tenant() {
    let pool = get_test_pool().await;
    let repo = PostgresOAuthClientRepository::new(pool.clone());

    let tenant_id = TenantId::new();
    let owner_id = UserId::new();
    create_dummy_user(&pool, &owner_id, &tenant_id, "owner_list").await;

    for i in 0..5 {
        let client = OAuthClient::new(
            tenant_id.clone(),
            owner_id.clone(),
            format!("Client {}", i),
            OAuthClientType::Confidential,
            vec!["https://example.com".to_string()],
        )
        .unwrap();
        repo.save(&client).await.expect("Failed to save");
    }

    let clients = repo
        .list_by_tenant(&tenant_id, 1, 10)
        .await
        .expect("Failed to list");
    assert_eq!(clients.len(), 5);
}
