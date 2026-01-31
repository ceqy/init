// use chrono::{Duration, Utc};
use common::{TenantId, UserId};
use iam_identity::domain::oauth::{
    AccessToken, AuthorizationCode, OAuthClient, OAuthClientId, OAuthClientType, RefreshToken,
};
use iam_identity::domain::repositories::oauth::{
    AccessTokenRepository, AuthorizationCodeRepository, OAuthClientRepository,
    RefreshTokenRepository,
};
use iam_identity::infrastructure::persistence::oauth::{
    PostgresAccessTokenRepository, PostgresAuthorizationCodeRepository,
    PostgresOAuthClientRepository, PostgresRefreshTokenRepository,
};
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

async fn get_test_pool() -> PgPool {
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/cuba".to_string());
    PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to database")
}

async fn create_fixtures(
    pool: &PgPool,
    tenant_id: &TenantId,
    user_id: &UserId,
    client_id: &OAuthClientId,
) -> OAuthClient {
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

    // 3. Create OAuth Client manually using Repository to ensure consistency or manual SQL
    // Here we use manual SQL to avoid dependency on Repository logic being tested elsewhere,
    // but using the repo is also fine since we just verified it. Let's use the object and manual insert for speed/simplicity or just use the repo.
    // Using PostgresOAuthClientRepository to save the client is verified, so let's use it.
    let client_repo = PostgresOAuthClientRepository::new(pool.clone());

    // Construct client manually to specify ID
    let mut client = OAuthClient::new(
        tenant_id.clone(),
        user_id.clone(),
        "Test Client".to_string(),
        OAuthClientType::Confidential,
        vec!["https://example.com/callback".to_string()],
    )
    .unwrap();
    // Overwrite the random ID with our fixture ID
    client.id = client_id.clone();

    client_repo
        .save(&client)
        .await
        .expect("Failed to save fixture client");

    client
}

#[tokio::test]
async fn test_authorization_code_lifecycle() {
    let pool = get_test_pool().await;
    let repo = PostgresAuthorizationCodeRepository::new(pool.clone());

    let tenant_id = TenantId::new();
    let user_id = UserId::new();
    let client_id = OAuthClientId::new();

    create_fixtures(&pool, &tenant_id, &user_id, &client_id).await;

    let code_str = "auth_code_123".to_string();
    let auth_code = AuthorizationCode::new(
        code_str.clone(),
        client_id.clone(),
        user_id.clone(),
        tenant_id.clone(),
        "https://example.com/callback".to_string(),
        vec!["openid".to_string()],
        None,
        None,
    );

    // 1. Save
    repo.save(&auth_code)
        .await
        .expect("Failed to save auth code");

    // 2. Find by code
    let found = repo
        .find_by_code(&code_str, &tenant_id)
        .await
        .expect("Failed to find");
    assert!(found.is_some());
    let found_code = found.unwrap();
    assert_eq!(found_code.code, code_str);
    assert!(!found_code.used);

    // 3. Delete
    repo.delete(&code_str, &tenant_id)
        .await
        .expect("Failed to delete");

    // 4. Verify deleted
    let found_after = repo
        .find_by_code(&code_str, &tenant_id)
        .await
        .expect("Failed to find");
    assert!(found_after.is_none());
}

#[tokio::test]
async fn test_access_token_lifecycle() {
    let pool = get_test_pool().await;
    let repo = PostgresAccessTokenRepository::new(pool.clone());

    let tenant_id = TenantId::new();
    let user_id = UserId::new();
    let client_id = OAuthClientId::new();

    create_fixtures(&pool, &tenant_id, &user_id, &client_id).await;

    let token_str = format!("access_token_{}", Uuid::new_v4());
    let mut access_token = AccessToken::new(
        token_str.clone(),
        client_id.clone(),
        Some(user_id.clone()),
        tenant_id.clone(),
        vec!["openid".to_string()],
        3600,
    );

    // 1. Save
    repo.save(&access_token)
        .await
        .expect("Failed to save access token");

    // 2. Find
    let found = repo
        .find_by_token(&token_str, &tenant_id)
        .await
        .expect("Failed to find");
    assert!(found.is_some());
    assert_eq!(found.unwrap().token, token_str);

    // 3. Revoke (via update)
    access_token.revoke();
    repo.update(&access_token).await.expect("Failed to revoke");

    // 4. Verify revoked
    let found_revoked = repo
        .find_by_token(&token_str, &tenant_id)
        .await
        .expect("Failed to find");
    assert!(found_revoked.unwrap().revoked);
}

#[tokio::test]
async fn test_refresh_token_lifecycle() {
    let pool = get_test_pool().await;
    let repo = PostgresRefreshTokenRepository::new(pool.clone());

    let tenant_id = TenantId::new();
    let user_id = UserId::new();
    let client_id = OAuthClientId::new();

    create_fixtures(&pool, &tenant_id, &user_id, &client_id).await;

    // Create Access Token first (required by FK)
    let access_token_str = format!("access_token_ref_{}", Uuid::new_v4());
    let access_token = AccessToken::new(
        access_token_str.clone(),
        client_id.clone(),
        Some(user_id.clone()),
        tenant_id.clone(),
        vec!["openid".to_string()],
        3600,
    );
    let at_repo = PostgresAccessTokenRepository::new(pool.clone());
    at_repo
        .save(&access_token)
        .await
        .expect("Failed to save prerequisite access token");

    let token_str = format!("refresh_token_{}", Uuid::new_v4());
    let mut refresh_token = RefreshToken::new(
        token_str.clone(),
        access_token_str,
        client_id.clone(),
        user_id.clone(),
        tenant_id.clone(),
        vec!["openid".to_string()],
        3600 * 24,
    );

    // 1. Save
    repo.save(&refresh_token)
        .await
        .expect("Failed to save refresh token");

    // 2. Find
    let found = repo
        .find_by_token(&token_str, &tenant_id)
        .await
        .expect("Failed to find");
    assert!(found.is_some());
    assert_eq!(found.unwrap().token, token_str);

    // 3. Revoke (via update)
    refresh_token.revoke();
    repo.update(&refresh_token).await.expect("Failed to revoke");

    // 4. Verify revoked
    let found_revoked = repo
        .find_by_token(&token_str, &tenant_id)
        .await
        .expect("Failed to find");
    assert!(found_revoked.unwrap().revoked);
}
