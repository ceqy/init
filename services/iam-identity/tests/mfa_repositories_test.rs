use cuba_common::{TenantId, UserId};
use iam_identity::domain::auth::{BackupCode, WebAuthnCredential};
use iam_identity::domain::repositories::auth::{
    BackupCodeRepository, WebAuthnCredentialRepository,
};
use iam_identity::infrastructure::persistence::auth::{
    PostgresBackupCodeRepository, PostgresWebAuthnCredentialRepository,
};
use sqlx::PgPool;
use std::env;

async fn get_test_pool() -> PgPool {
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/cuba".to_string());
    PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to database")
}

async fn create_user_fixture(pool: &PgPool, tenant_id: &TenantId, user_id: &UserId) {
    // 1. Create Tenant
    let _ = sqlx::query(
        "INSERT INTO tenants (id, name, display_name) VALUES ($1, $2, $2) ON CONFLICT (id) DO NOTHING",
    )
    .bind(tenant_id.0)
    .bind(format!("tenant_{}", tenant_id.0))
    .execute(pool)
    .await;

    // 2. Create User
    let _ = sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, tenant_id, status) VALUES ($1, $2, $3, 'hash', $4, 'Active') ON CONFLICT (id) DO NOTHING",
    )
    .bind(user_id.0)
    .bind(format!("user_{}", user_id.0))
    .bind(format!("{}@example.com", user_id.0))
    .bind(tenant_id.0)
    .execute(pool)
    .await
    .expect("Failed to create user");
}

#[tokio::test]
async fn test_backup_code_lifecycle() {
    let pool = get_test_pool().await;
    let repo = PostgresBackupCodeRepository::new(pool.clone());

    let tenant_id = TenantId::new();
    let user_id = UserId::new();

    create_user_fixture(&pool, &tenant_id, &user_id).await;

    // 1. Save
    let code_hash = "hashed_code_123".to_string();
    let backup_code = BackupCode::new(user_id.clone(), tenant_id.clone(), code_hash.clone());

    repo.save(&backup_code)
        .await
        .expect("Failed to save backup code");

    // 2. Find Available
    let codes = repo
        .find_available_by_user_id(&user_id, &tenant_id)
        .await
        .expect("Failed to find codes");
    assert_eq!(codes.len(), 1);
    assert_eq!(codes[0].code_hash, code_hash);

    // 3. Mark as used (Update)
    let mut code_to_use = codes[0].clone();
    code_to_use.mark_as_used();
    repo.update(&code_to_use).await.expect("Failed to update");

    // 4. Verify used
    let available_after = repo
        .find_available_by_user_id(&user_id, &tenant_id)
        .await
        .expect("Failed to find");
    assert!(available_after.is_empty());

    // 5. Find by ID (should still exist)
    let found = repo
        .find_by_id(&code_to_use.id, &tenant_id)
        .await
        .expect("Failed to find by id");
    assert!(found.is_some());
    assert!(found.unwrap().used);

    // 6. Delete
    repo.delete_by_user_id(&user_id, &tenant_id)
        .await
        .expect("Failed to delete");
    let found_after_delete = repo
        .find_by_id(&code_to_use.id, &tenant_id)
        .await
        .expect("Failed to find");
    assert!(found_after_delete.is_none());
}

#[tokio::test]
async fn test_webauthn_credential_lifecycle() {
    let pool = get_test_pool().await;
    let repo = PostgresWebAuthnCredentialRepository::new(pool.clone());

    let tenant_id = TenantId::new();
    let user_id = UserId::new();

    create_user_fixture(&pool, &tenant_id, &user_id).await;

    let credential_id = vec![1, 2, 3, 4];
    let public_key = vec![5, 6, 7, 8];

    let credential = WebAuthnCredential::new(
        user_id.0,
        credential_id.clone(),
        public_key.clone(),
        0,
        "My Key".to_string(),
        None,
        vec![],
        false,
        false,
        tenant_id.clone(),
    );

    // 1. Save
    repo.save(&credential)
        .await
        .expect("Failed to save credential");

    // 2. Find by User
    let creds = repo
        .find_by_user_id(&user_id, &tenant_id)
        .await
        .expect("Failed to find by user");
    assert_eq!(creds.len(), 1);
    assert_eq!(creds[0].name, "My Key");

    // 3. Find by Credential ID (bytes) (if supported) or ID
    // Check specific query method if exists, otherwise find_by_id
    let found = repo
        .find_by_id(&credential.id, &tenant_id)
        .await
        .expect("Failed to find by id");
    assert!(found.is_some());

    // 4. Update counter
    let mut cred_to_update = found.unwrap();
    cred_to_update.update_counter(10);
    repo.update(&cred_to_update)
        .await
        .expect("Failed to update");

    let updated = repo
        .find_by_id(&credential.id, &tenant_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.counter, 10);

    // 5. Delete
    repo.delete(&credential.id, &tenant_id)
        .await
        .expect("Failed to delete");

    let deleted = repo
        .find_by_id(&credential.id, &tenant_id)
        .await
        .expect("Failed to find");
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_tenant_isolation_2fa() {
    let pool = get_test_pool().await;
    let repo = PostgresBackupCodeRepository::new(pool.clone());

    let tenant_a = TenantId::new();
    let user_a = UserId::new();
    create_user_fixture(&pool, &tenant_a, &user_a).await;

    let tenant_b = TenantId::new();
    let user_b = UserId::new();
    create_user_fixture(&pool, &tenant_b, &user_b).await;

    let backup_code = BackupCode::new(user_a.clone(), tenant_a.clone(), "hash".to_string());
    repo.save(&backup_code).await.expect("Save failed");

    // Tenant B try to find User A's code
    let found_by_b = repo
        .find_by_id(&backup_code.id, &tenant_b)
        .await
        .expect("Query failed");
    assert!(found_by_b.is_none());

    // Tenant A find User A's code
    let found_by_a = repo
        .find_by_id(&backup_code.id, &tenant_a)
        .await
        .expect("Query failed");
    assert!(found_by_a.is_some());
}
