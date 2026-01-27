//! CQRS 查询测试

use cuba_common::TenantId;
use iam_identity::application::queries::user::{
    GetUserByIdQuery, GetUserByUsernameQuery, GetUserByEmailQuery,
    GetCurrentUserQuery, ListUsersQuery, UserQueryResult, ListUsersResult,
};
use iam_identity::application::queries::oauth::{
    GetOAuthClientByIdQuery, ListUserOAuthClientsQuery, ListTenantOAuthClientsQuery,
    ValidateOAuthClientQuery, ListUserAuthorizedAppsQuery,
    OAuthClientQueryResult, ListOAuthClientsResult, ValidateOAuthClientResult,
};
use cuba_common::UserId;
use cuba_cqrs_core::Query;

// ============ User Queries Tests ============

#[test]
fn test_get_user_by_id_query_creation() {
    let query = GetUserByIdQuery {
        user_id: UserId::new(),
        tenant_id: TenantId::new(),
    };
    
    assert!(!query.user_id.0.is_nil());
    assert!(!query.tenant_id.0.is_nil());
}

#[test]
fn test_get_user_by_username_query_creation() {
    let query = GetUserByUsernameQuery {
        username: "testuser".to_string(),
        tenant_id: TenantId::new(),
    };
    
    assert_eq!(query.username, "testuser");
}

#[test]
fn test_get_user_by_email_query_creation() {
    let query = GetUserByEmailQuery {
        email: "test@example.com".to_string(),
        tenant_id: TenantId::new(),
    };
    
    assert_eq!(query.email, "test@example.com");
}

#[test]
fn test_get_current_user_query_creation() {
    let query = GetCurrentUserQuery {
        access_token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...".to_string(),
    };
    
    assert!(query.access_token.starts_with("eyJ"));
}

#[test]
fn test_list_users_query_creation() {
    let query = ListUsersQuery {
        tenant_id: TenantId::new(),
        page: 1,
        page_size: 20,
        status_filter: Some("Active".to_string()),
        search: Some("john".to_string()),
    };
    
    assert_eq!(query.page, 1);
    assert_eq!(query.page_size, 20);
    assert_eq!(query.status_filter, Some("Active".to_string()));
}

#[test]
fn test_user_query_result() {
    let result = UserQueryResult {
        id: "user-123".to_string(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        display_name: Some("Test User".to_string()),
        status: "Active".to_string(),
        tenant_id: "tenant-456".to_string(),
        email_verified: true,
        phone_verified: false,
        two_factor_enabled: false,
        created_at: chrono::Utc::now(),
        last_login_at: Some(chrono::Utc::now()),
    };
    
    assert_eq!(result.username, "testuser");
    assert!(result.email_verified);
}

#[test]
fn test_list_users_result() {
    let result = ListUsersResult {
        users: vec![],
        total: 100,
        page: 1,
        page_size: 20,
    };
    
    assert_eq!(result.total, 100);
    assert!(result.users.is_empty());
}

// ============ OAuth Queries Tests ============

#[test]
fn test_get_oauth_client_by_id_query() {
    let query = GetOAuthClientByIdQuery {
        client_id: "client-123".to_string(),
        tenant_id: TenantId::new(),
    };
    
    assert_eq!(query.client_id, "client-123");
}

#[test]
fn test_list_user_oauth_clients_query() {
    let query = ListUserOAuthClientsQuery {
        owner_id: "user-123".to_string(),
        tenant_id: TenantId::new(),
        page: 1,
        page_size: 10,
    };
    
    assert_eq!(query.owner_id, "user-123");
    assert_eq!(query.page_size, 10);
}

#[test]
fn test_list_tenant_oauth_clients_query() {
    let query = ListTenantOAuthClientsQuery {
        tenant_id: TenantId::new(),
        page: 2,
        page_size: 50,
    };
    
    assert_eq!(query.page, 2);
    assert_eq!(query.page_size, 50);
}

#[test]
fn test_validate_oauth_client_query() {
    let query = ValidateOAuthClientQuery {
        client_id: "client-123".to_string(),
        client_secret: Some("secret-456".to_string()),
        redirect_uri: Some("https://example.com/callback".to_string()),
    };
    
    assert_eq!(query.client_id, "client-123");
    assert!(query.client_secret.is_some());
}

#[test]
fn test_list_user_authorized_apps_query() {
    let query = ListUserAuthorizedAppsQuery {
        user_id: "user-123".to_string(),
        tenant_id: TenantId::new(),
    };
    
    assert_eq!(query.user_id, "user-123");
}

#[test]
fn test_oauth_client_query_result() {
    let result = OAuthClientQueryResult {
        id: "id-123".to_string(),
        client_id: "client-123".to_string(),
        name: "My App".to_string(),
        client_type: "Confidential".to_string(),
        redirect_uris: vec!["https://example.com/callback".to_string()],
        allowed_scopes: vec!["openid".to_string(), "profile".to_string()],
        owner_id: "user-456".to_string(),
        tenant_id: "tenant-789".to_string(),
        is_active: true,
        created_at: chrono::Utc::now(),
    };
    
    assert_eq!(result.name, "My App");
    assert!(result.is_active);
    assert_eq!(result.redirect_uris.len(), 1);
}

#[test]
fn test_validate_oauth_client_result_valid() {
    let result = ValidateOAuthClientResult {
        valid: true,
        client: Some(OAuthClientQueryResult {
            id: "id-123".to_string(),
            client_id: "client-123".to_string(),
            name: "My App".to_string(),
            client_type: "Confidential".to_string(),
            redirect_uris: vec![],
            allowed_scopes: vec![],
            owner_id: "user-456".to_string(),
            tenant_id: "tenant-789".to_string(),
            is_active: true,
            created_at: chrono::Utc::now(),
        }),
        error: None,
    };
    
    assert!(result.valid);
    assert!(result.client.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_validate_oauth_client_result_invalid() {
    let result = ValidateOAuthClientResult {
        valid: false,
        client: None,
        error: Some("Invalid client secret".to_string()),
    };
    
    assert!(!result.valid);
    assert!(result.client.is_none());
    assert_eq!(result.error, Some("Invalid client secret".to_string()));
}
