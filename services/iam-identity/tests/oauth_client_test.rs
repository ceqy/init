//! OAuth 客户端实体测试

use cuba_common::{TenantId, UserId};
use iam_identity::domain::oauth::{GrantType, OAuthClient, OAuthClientType};

/// 创建测试 OAuth 客户端
fn create_test_client(tenant_id: TenantId, owner_id: UserId) -> OAuthClient {
    OAuthClient::new(
        tenant_id,
        owner_id,
        "Test App".to_string(),
        OAuthClientType::Confidential,
        vec!["https://example.com/callback".to_string()],
    )
    .expect("Valid client")
}

#[test]
fn test_oauth_client_creation() {
    let tenant_id = TenantId::new();
    let owner_id = UserId::new();

    let client = create_test_client(tenant_id.clone(), owner_id.clone());

    assert!(!client.id.0.is_nil());
    assert_eq!(client.tenant_id, tenant_id);
    assert_eq!(client.owner_id, owner_id);
    assert_eq!(client.name, "Test App");
    assert!(client.is_active);
}

#[test]
fn test_oauth_client_empty_name_fails() {
    let tenant_id = TenantId::new();
    let owner_id = UserId::new();

    let result = OAuthClient::new(
        tenant_id,
        owner_id,
        "".to_string(),
        OAuthClientType::Confidential,
        vec!["https://example.com/callback".to_string()],
    );

    assert!(result.is_err());
}

#[test]
fn test_oauth_client_no_redirect_uris_fails() {
    let tenant_id = TenantId::new();
    let owner_id = UserId::new();

    let result = OAuthClient::new(
        tenant_id,
        owner_id,
        "Test App".to_string(),
        OAuthClientType::Confidential,
        vec![],
    );

    assert!(result.is_err());
}

#[test]
fn test_oauth_client_http_localhost_allowed() {
    let tenant_id = TenantId::new();
    let owner_id = UserId::new();

    let result = OAuthClient::new(
        tenant_id,
        owner_id,
        "Dev App".to_string(),
        OAuthClientType::Public,
        vec!["http://localhost:3000/callback".to_string()],
    );

    assert!(result.is_ok());
}

#[test]
fn test_oauth_client_redirect_uri_validation() {
    let tenant_id = TenantId::new();
    let owner_id = UserId::new();
    let client = create_test_client(tenant_id, owner_id);

    assert!(client.validate_redirect_uri_match("https://example.com/callback"));
    assert!(!client.validate_redirect_uri_match("https://malicious.com/callback"));
}

#[test]
fn test_oauth_client_grant_type_check() {
    let tenant_id = TenantId::new();
    let owner_id = UserId::new();
    let client = create_test_client(tenant_id, owner_id);

    // 默认只有 AuthorizationCode
    assert!(client.is_grant_type_allowed(&GrantType::AuthorizationCode));
    assert!(!client.is_grant_type_allowed(&GrantType::ClientCredentials));
}

#[test]
fn test_oauth_client_scope_validation() {
    let tenant_id = TenantId::new();
    let owner_id = UserId::new();
    let client = create_test_client(tenant_id, owner_id);

    // 默认 scopes: openid, profile, email
    assert!(client.validate_scopes(&["openid".to_string(), "profile".to_string()]));
    assert!(!client.validate_scopes(&["admin".to_string()]));
}

#[test]
fn test_oauth_client_activation() {
    let tenant_id = TenantId::new();
    let owner_id = UserId::new();
    let mut client = create_test_client(tenant_id, owner_id);

    assert!(client.is_active);

    client.deactivate();
    assert!(!client.is_active);

    client.activate();
    assert!(client.is_active);
}

#[test]
fn test_oauth_client_update() {
    let tenant_id = TenantId::new();
    let owner_id = UserId::new();
    let mut client = create_test_client(tenant_id, owner_id);

    let original_updated_at = client.updated_at;

    client
        .update(
            Some("Updated App".to_string()),
            Some("New description".to_string()),
            None,
            Some(vec!["openid".to_string(), "offline_access".to_string()]),
        )
        .expect("Update should succeed");

    assert_eq!(client.name, "Updated App");
    assert_eq!(client.description, Some("New description".to_string()));
    assert!(
        client
            .allowed_scopes
            .contains(&"offline_access".to_string())
    );
    assert!(client.updated_at > original_updated_at);
}

#[test]
fn test_oauth_client_secret_management() {
    let tenant_id = TenantId::new();
    let owner_id = UserId::new();
    let mut client = create_test_client(tenant_id, owner_id);

    assert!(client.client_secret_hash.is_none());

    client.set_client_secret("secret123".to_string());
    assert!(client.client_secret_hash.is_some());

    client.rotate_client_secret("new_secret456".to_string());
    assert_eq!(client.client_secret_hash, Some("new_secret456".to_string()));
}

#[test]
fn test_oauth_client_public_vs_confidential() {
    let tenant_id = TenantId::new();
    let owner_id = UserId::new();

    // Public client
    let public_client = OAuthClient::new(
        tenant_id.clone(),
        owner_id.clone(),
        "Public App".to_string(),
        OAuthClientType::Public,
        vec!["https://example.com/callback".to_string()],
    )
    .expect("Valid client");

    assert!(public_client.public_client);

    // Confidential client
    let confidential_client = OAuthClient::new(
        tenant_id.clone(),
        owner_id.clone(),
        "Confidential App".to_string(),
        OAuthClientType::Confidential,
        vec!["https://example.com/callback".to_string()],
    )
    .expect("Valid client");

    assert!(!confidential_client.public_client);
}
