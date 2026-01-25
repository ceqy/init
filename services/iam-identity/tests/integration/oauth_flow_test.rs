//! OAuth2 流程集成测试

use cuba_common::{TenantId, UserId};
use iam_identity::oauth::domain::entities::*;

#[cfg(test)]
mod authorization_code_flow_tests {
    use super::*;

    fn create_test_client() -> OAuthClient {
        OAuthClient::new(
            TenantId::new(),
            UserId::new(),
            "Test App".to_string(),
            OAuthClientType::Confidential,
            vec!["https://example.com/callback".to_string()],
        ).unwrap()
    }

    #[test]
    fn test_authorization_code_flow_with_pkce() {
        use base64::Engine;
        use sha2::{Digest, Sha256};

        // 1. 创建 OAuth Client
        let client = create_test_client();
        assert!(client.require_pkce);
        
        // 2. 生成 PKCE 参数
        let code_verifier = "test_verifier_1234567890_abcdefghijklmnopqrstuvwxyz";
        let hash = Sha256::digest(code_verifier.as_bytes());
        let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&hash);
        
        // 3. 创建授权码
        let auth_code = AuthorizationCode::new(
            "auth_code_123".to_string(),
            client.id.clone(),
            UserId::new(),
            client.tenant_id.clone(),
            "https://example.com/callback".to_string(),
            vec!["openid".to_string(), "profile".to_string()],
            Some(code_challenge),
            Some("S256".to_string()),
        );
        
        assert!(!auth_code.is_expired());
        assert!(!auth_code.is_used());
        
        // 4. 验证 PKCE
        assert!(auth_code.verify_code_verifier(code_verifier).unwrap());
        
        // 5. 标记授权码为已使用
        let mut auth_code = auth_code;
        auth_code.mark_as_used();
        assert!(auth_code.is_used());
        
        // 6. 创建 Access Token
        let access_token = AccessToken::new(
            "access_token_123".to_string(),
            client.id.clone(),
            Some(auth_code.user_id.clone()),
            client.tenant_id.clone(),
            auth_code.scopes.clone(),
            client.access_token_lifetime,
        );
        
        assert!(access_token.is_valid());
        assert!(access_token.has_scope("openid"));
        assert!(access_token.has_scope("profile"));
        
        // 7. 创建 Refresh Token
        let refresh_token = RefreshToken::new(
            "refresh_token_123".to_string(),
            access_token.token.clone(),
            client.id.clone(),
            auth_code.user_id.clone(),
            client.tenant_id.clone(),
            auth_code.scopes.clone(),
            client.refresh_token_lifetime,
        );
        
        assert!(refresh_token.is_valid());
    }

    #[test]
    fn test_authorization_code_flow_without_pkce() {
        let mut client = create_test_client();
        client.require_pkce = false;
        
        // 创建不带 PKCE 的授权码
        let auth_code = AuthorizationCode::new(
            "auth_code_123".to_string(),
            client.id.clone(),
            UserId::new(),
            client.tenant_id.clone(),
            "https://example.com/callback".to_string(),
            vec!["openid".to_string()],
            None,
            None,
        );
        
        // 不需要 PKCE 验证
        assert!(auth_code.verify_code_verifier("any_verifier").unwrap());
    }

    #[test]
    fn test_authorization_code_reuse_prevention() {
        let client = create_test_client();
        let mut auth_code = AuthorizationCode::new(
            "auth_code_123".to_string(),
            client.id.clone(),
            UserId::new(),
            client.tenant_id.clone(),
            "https://example.com/callback".to_string(),
            vec!["openid".to_string()],
            None,
            None,
        );
        
        // 第一次使用
        assert!(!auth_code.is_used());
        auth_code.mark_as_used();
        
        // 第二次使用应该被拒绝
        assert!(auth_code.is_used());
    }
}

#[cfg(test)]
mod client_credentials_flow_tests {
    use super::*;

    #[test]
    fn test_client_credentials_flow() {
        // 1. 创建 OAuth Client
        let mut client = OAuthClient::new(
            TenantId::new(),
            UserId::new(),
            "Service App".to_string(),
            OAuthClientType::Confidential,
            vec!["https://example.com/callback".to_string()],
        ).unwrap();
        
        // 2. 设置 Client Secret
        client.set_client_secret("hashed_secret".to_string());
        
        // 3. 添加 Client Credentials 授权类型
        client.grant_types.push(GrantType::ClientCredentials);
        
        // 4. 验证授权类型
        assert!(client.is_grant_type_allowed(&GrantType::ClientCredentials));
        
        // 5. 创建 Access Token（无用户 ID）
        let access_token = AccessToken::new(
            "access_token_123".to_string(),
            client.id.clone(),
            None, // Client Credentials 流程没有用户
            client.tenant_id.clone(),
            vec!["api:read".to_string(), "api:write".to_string()],
            client.access_token_lifetime,
        );
        
        assert!(access_token.is_valid());
        assert!(access_token.user_id.is_none());
        assert!(access_token.has_scope("api:read"));
    }
}

#[cfg(test)]
mod refresh_token_flow_tests {
    use super::*;

    #[test]
    fn test_refresh_token_flow() {
        let client = create_test_client();
        let user_id = UserId::new();
        
        // 1. 创建原始 Access Token
        let access_token = AccessToken::new(
            "access_token_123".to_string(),
            client.id.clone(),
            Some(user_id.clone()),
            client.tenant_id.clone(),
            vec!["openid".to_string(), "profile".to_string()],
            client.access_token_lifetime,
        );
        
        // 2. 创建 Refresh Token
        let refresh_token = RefreshToken::new(
            "refresh_token_123".to_string(),
            access_token.token.clone(),
            client.id.clone(),
            user_id.clone(),
            client.tenant_id.clone(),
            access_token.scopes.clone(),
            client.refresh_token_lifetime,
        );
        
        assert!(refresh_token.is_valid());
        
        // 3. 使用 Refresh Token 创建新的 Access Token
        let new_access_token = AccessToken::new(
            "access_token_456".to_string(),
            client.id.clone(),
            Some(user_id.clone()),
            client.tenant_id.clone(),
            refresh_token.scopes.clone(),
            client.access_token_lifetime,
        );
        
        assert!(new_access_token.is_valid());
        assert_ne!(new_access_token.token, access_token.token);
    }

    #[test]
    fn test_refresh_token_revocation() {
        let client = create_test_client();
        let mut refresh_token = RefreshToken::new(
            "refresh_token_123".to_string(),
            "access_token_123".to_string(),
            client.id.clone(),
            UserId::new(),
            client.tenant_id.clone(),
            vec!["openid".to_string()],
            client.refresh_token_lifetime,
        );
        
        // 撤销 Refresh Token
        assert!(refresh_token.is_valid());
        refresh_token.revoke();
        assert!(!refresh_token.is_valid());
        assert!(refresh_token.is_revoked());
    }
}

#[cfg(test)]
mod token_revocation_tests {
    use super::*;

    #[test]
    fn test_access_token_revocation() {
        let client = create_test_client();
        let mut access_token = AccessToken::new(
            "access_token_123".to_string(),
            client.id.clone(),
            Some(UserId::new()),
            client.tenant_id.clone(),
            vec!["openid".to_string()],
            client.access_token_lifetime,
        );
        
        assert!(access_token.is_valid());
        
        // 撤销 Token
        access_token.revoke();
        assert!(!access_token.is_valid());
        assert!(access_token.is_revoked());
    }

    #[test]
    fn test_cascade_token_revocation() {
        let client = create_test_client();
        let user_id = UserId::new();
        
        // 创建 Access Token 和 Refresh Token
        let mut access_token = AccessToken::new(
            "access_token_123".to_string(),
            client.id.clone(),
            Some(user_id.clone()),
            client.tenant_id.clone(),
            vec!["openid".to_string()],
            client.access_token_lifetime,
        );
        
        let mut refresh_token = RefreshToken::new(
            "refresh_token_123".to_string(),
            access_token.token.clone(),
            client.id.clone(),
            user_id.clone(),
            client.tenant_id.clone(),
            access_token.scopes.clone(),
            client.refresh_token_lifetime,
        );
        
        // 撤销 Refresh Token 应该同时撤销关联的 Access Token
        refresh_token.revoke();
        access_token.revoke();
        
        assert!(!refresh_token.is_valid());
        assert!(!access_token.is_valid());
    }
}

#[cfg(test)]
mod oauth_client_management_tests {
    use super::*;

    #[test]
    fn test_oauth_client_lifecycle() {
        // 1. 创建 Client
        let mut client = OAuthClient::new(
            TenantId::new(),
            UserId::new(),
            "Test App".to_string(),
            OAuthClientType::Confidential,
            vec!["https://example.com/callback".to_string()],
        ).unwrap();
        
        assert!(client.is_active);
        
        // 2. 更新 Client
        let result = client.update(
            Some("Updated App".to_string()),
            Some("New description".to_string()),
            Some(vec![
                "https://example.com/callback".to_string(),
                "https://example.com/callback2".to_string(),
            ]),
            Some(vec!["openid".to_string(), "profile".to_string(), "email".to_string()]),
        );
        
        assert!(result.is_ok());
        assert_eq!(client.name, "Updated App");
        assert_eq!(client.redirect_uris.len(), 2);
        assert_eq!(client.allowed_scopes.len(), 3);
        
        // 3. 停用 Client
        client.deactivate();
        assert!(!client.is_active);
        
        // 4. 重新激活 Client
        client.activate();
        assert!(client.is_active);
    }

    #[test]
    fn test_oauth_client_secret_rotation() {
        let mut client = create_test_client();
        
        // 设置初始 Secret
        client.set_client_secret("old_secret".to_string());
        assert_eq!(client.client_secret_hash, Some("old_secret".to_string()));
        
        // 轮换 Secret
        client.rotate_client_secret("new_secret".to_string());
        assert_eq!(client.client_secret_hash, Some("new_secret".to_string()));
    }

    #[test]
    fn test_oauth_client_scope_validation() {
        let client = create_test_client();
        
        // 有效的 Scope
        assert!(client.validate_scopes(&["openid".to_string()]));
        assert!(client.validate_scopes(&["openid".to_string(), "profile".to_string()]));
        
        // 无效的 Scope
        assert!(!client.validate_scopes(&["admin".to_string()]));
        assert!(!client.validate_scopes(&["openid".to_string(), "admin".to_string()]));
    }
}

fn create_test_client() -> OAuthClient {
    OAuthClient::new(
        TenantId::new(),
        UserId::new(),
        "Test App".to_string(),
        OAuthClientType::Confidential,
        vec!["https://example.com/callback".to_string()],
    ).unwrap()
}
