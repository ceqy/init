use cuba_common::{TenantId, UserId};
use iam_identity::oauth::domain::entities::{OAuthClient, OAuthClientId};
use iam_identity::oauth::domain::repositories::OAuthClientRepository;
use iam_identity::oauth::infrastructure::persistence::PostgresOAuthClientRepository;
use iam_identity::shared::domain::entities::{Tenant, User};
use iam_identity::shared::domain::repositories::{TenantRepository, UserRepository};
use iam_identity::shared::domain::value_objects::{Email, HashedPassword, Username};
use iam_identity::shared::infrastructure::persistence::{
    PostgresTenantRepository, PostgresUserRepository,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

async fn migrate_db(pool: &PgPool) {
    let _ = sqlx::query("ALTER TABLE oauth_clients ADD COLUMN IF NOT EXISTS public_client BOOLEAN NOT NULL DEFAULT FALSE")
        .execute(pool)
        .await;
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS last_password_change_at TIMESTAMPTZ")
        .execute(pool)
        .await;
    // Add scopes if it was missing? Migration had allowed_scopes but repository uses scopes maps to allowed_scopes.
    // The previous error for scopes was fixed by code change mapping to allowed_scopes.
}

#[sqlx::test]
async fn test_user_repository_isolation(pool: PgPool) {
    migrate_db(&pool).await;

    let tenant_repo = PostgresTenantRepository::new(pool.clone());
    let user_repo = PostgresUserRepository::new(pool.clone());

    // 1. Create two tenants
    let tenant1 = Tenant::new("tenant-a".to_string(), "Tenant A".to_string()).unwrap();
    let tenant2 = Tenant::new("tenant-b".to_string(), "Tenant B".to_string()).unwrap();

    tenant_repo.save(&tenant1).await.unwrap();
    tenant_repo.save(&tenant2).await.unwrap();

    // 2. Create a user in Tenant 1
    let user1 = User::new(
        Username::new("user1").unwrap(),
        Email::new("user1@example.com").unwrap(),
        HashedPassword::from_plain("Password123!").unwrap(),
        tenant1.id.clone(),
    ); // Check if User::new returns Result. Waiting for view_file result. 
    // Assuming from previous error "no method unwrap found for struct User" that it returns User directly.
    // So removing .unwrap()
    
    // BUT wait, I should wait for view_file to be sure.
    // Let's hold on replacement until I see the file.


    user_repo.save(&user1).await.unwrap();

    // 3. Create a user in Tenant 2 with SAME username
    let user2 = User::new(
        Username::new("user1").unwrap(), // Same username
        Email::new("user2@example.com").unwrap(),
        HashedPassword::from_plain("Password123!").unwrap(),
        tenant2.id.clone(),
    );

    user_repo.save(&user2).await.unwrap();

    // 4. Verify isolation - Find by Username
    let found1 = user_repo
        .find_by_username(&user1.username, &tenant1.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found1.id, user1.id);
    assert_eq!(found1.tenant_id, tenant1.id);

    let found2 = user_repo
        .find_by_username(&user2.username, &tenant2.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found2.id, user2.id);
    assert_eq!(found2.tenant_id, tenant2.id);

    assert_ne!(found1.id, found2.id);

    // 5. Verify cross-tenant access fails
    let not_found = user_repo
        .find_by_id(&user1.id, &tenant2.id) // Looking for User 1 in Tenant 2
        .await
        .unwrap();
    assert!(not_found.is_none());

    // 6. Verify list isolation
    let (users1, total1) = user_repo
        .list(&tenant1.id, None, None, &[], 1, 10)
        .await
        .unwrap();
    assert_eq!(total1, 1);
    assert_eq!(users1[0].id, user1.id);

    let (users2, total2) = user_repo
        .list(&tenant2.id, None, None, &[], 1, 10)
        .await
        .unwrap();
    assert_eq!(total2, 1);
    assert_eq!(users2[0].id, user2.id);
}

#[sqlx::test]
async fn test_oauth_client_repository_isolation(pool: PgPool) {
    migrate_db(&pool).await;

    let tenant_repo = PostgresTenantRepository::new(pool.clone());
    let client_repo = PostgresOAuthClientRepository::new(pool.clone());

    // 1. Create two tenants
    let tenant1 = Tenant::new("tenant-c".to_string(), "Tenant C".to_string()).unwrap();
    let tenant2 = Tenant::new("tenant-d".to_string(), "Tenant D".to_string()).unwrap();

    tenant_repo.save(&tenant1).await.unwrap();
    tenant_repo.save(&tenant2).await.unwrap();

    let user_repo = PostgresUserRepository::new(pool.clone());
    
    // Create owner user
    let owner = User::new(
        Username::new("client_owner").unwrap(),
        Email::new("owner@example.com").unwrap(),
        HashedPassword::from_plain("Password123!").unwrap(),
        tenant1.id.clone(),
    );
    user_repo.save(&owner).await.unwrap();

    // 2. Create client in Tenant 1
    let client1 = OAuthClient::new(
        tenant1.id.clone(),
        owner.id.clone(), // Use real owner
        "Client 1".to_string(),
        iam_identity::oauth::domain::entities::OAuthClientType::Confidential,
        vec!["http://localhost/callback".to_string()],
    ).unwrap();

    client_repo.save(&client1).await.unwrap();

    // 3. Verify lookup in Tenant 1
    let found1 = client_repo
        .find_by_id(&client1.id, &tenant1.id)
        .await
        .unwrap();
    assert!(found1.is_some());

    // 4. Verify lookup in Tenant 2 fails
    let found2 = client_repo
        .find_by_id(&client1.id, &tenant2.id)
        .await
        .unwrap();
    assert!(found2.is_none());
}
