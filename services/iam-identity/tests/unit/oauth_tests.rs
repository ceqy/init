//! OAuth 实体单元测试

use cuba_common::{TenantId, UserId};
use iam_identity::oauth::domain::entities::*;

#[cfg(test)]
mod oauth_client_tests {
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
    fn test_create_oauth_client() {
        let client = create_test_client();
        assert_eq!(client.name, "Test App");
        assert_eq!(client.client_type, OAuthClientType::Confidential);
        assert!(client.is_active);
        assert!(client.require_pkce);
    }

    #[test]
    fn test_oauth_client_invalid_name() {
        let result = OAuthClient::new(
            TenantId::new(),
            UserId::new(),
            "".to_string(),
            OAuthClientType::Confidential,
            vec!["https://example.com/callback".to_string()],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_oauth_client_no_redirect_uris() {
        let result = OAuthClient::new(
            TenantId::new(),
            UserId::new(),
            "Test App".to_string(),
            OAuthClientType::Confidential,
            vec![],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_oauth_client_invalid_redirect_uri_http() {
        let result = OAuthClient::new(
            TenantId::new(),
            UserId::new(),
            "Test App".to_string(),
            OAuthClientType::Confidential,
            vec!["http://example.com/callback".to_string()],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_oauth_client_valid_redirect_uri_localhost() {
        let result = OAuthClient::new(
            TenantId::new(),
            UserId::new(),
            "Test App".to_string(),
            OAuthClientType::Confidential,
            vec!["http://localhost:3000/callback".to_string()],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_oauth_client_invalid_redirect_uri_fragment() {
        let result = OAuthClient::new(
            TenantId::new(),
            UserId::new(),
            "Test App".to_string(),
            OAuthClientType::Confidential,
            vec!["https://example.com/callback#fragment".to_string()],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_oauth_client_set_secret() {
        let mut client = create_test_client();
        assert!(client.client_secret_hash.is_none());
        
        client.set_client_secret("hashed_secret".to_string());
        assert_eq!(client.client_secret_hash, Some("hashed_secret".to_string()));
    }

    #[test]
    fn test_oauth_client_rotate_secret() {
        let mut client = create_test_client();
        client.set_client_secret("old_secret".to_string());
        
        client.rotate_client_secret("new_secret".to_string());
        assert_eq!(client.client_secret_hash, Some("new_secret".to_string()));
    }

    #[test]
    fn test_oauth_client_validate_redirect_uri_match() {
        let client = create_test_client();
        
        assert!(client.validate_redirect_uri_match("https://example.com/callback"));
        assert!(!client.validate_redirect_uri_match("https://evil.com/callback"));
    }

    #[test]
    fn test_oauth_client_is_grant_type_allowed() {
        let client = create_test_client();
        
        assert!(client.is_grant_type_allowed(&GrantType::AuthorizationCode));
        assert!(!client.is_grant_type_allowed(&GrantType::ClientCredentials));
    }

    #[test]
    fn test_oauth_client_validate_scopes() {
        let client = create_test_client();
        
        assert!(client.validate_scopes(&["openid".to_string(), "profile".to_string()]));
        assert!(!client.validate_scopes(&["admin".to_string()]));
    }

    #[test]
    fn test_oauth_client_update() {
        let mut client = create_test_client();
        
        let result = client.update(
            Some("Updated App".to_string()),
            Some("New description".to_string()),
            None,
            None,
        );
        
        assert!(result.is_ok());
        assert_eq!(client.name, "Updated App");
        assert_eq!(client.description, Some("New description".to_string()));
    }

    #[test]
    fn test_oauth_client_activate_deactivate() {
        let mut client = create_test_client();
        
        client.deactivate();
        assert!(!client.is_active);
        
        client.activate();
        assert!(client.is_active);
    }
}

#[cfg(test)]
mod authorization_code_tests {
    use super::*;

    fn create_test_authorization_code() -> AuthorizationCode {
        AuthorizationCode::new(
            "test_code".to_string(),
            OAuthClientId::new(),
            UserId::new(),
            TenantId::new(),
            "https://example.com/callback".to_string(),
            vec!["openid".to_string()],
            None,
            None,
        )
    }

    #[test]
    fn test_create_authorization_code() {
        let code = create_test_authorization_code();
        assert_eq!(code.code, "test_code");
        assert!(!code.is_expired());
        assert!(!code.is_used());
    }

    #[test]
    fn test_authorization_code_mark_as_used() {
        let mut code = create_test_authorization_code();
        assert!(!code.is_used());
        
        code.mark_as_used();
        assert!(code.is_used());
    }

    #[test]
    fn test_authorization_code_verify_pkce_s256() {
        use base64::Engine;
        use sha2::{Digest, Sha256};

        let code_verifier = "test_verifier_1234567890";
        let hash = Sha256::digest(code_verifier.as_bytes());
        let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&hash);

        let code = AuthorizationCode::new(
            "test_code".to_string(),
            OAuthClientId::new(),
            UserId::new(),
            TenantId::new(),
            "https://example.com/callback".to_string(),
            vec!["openid".to_string()],
            Some(code_challenge),
            Some("S256".to_string()),
        );

        assert!(code.verify_code_verifier(code_verifier).unwrap());
        assert!(!code.verify_code_verifier("wrong_verifier").unwrap());
    }

    #[test]
    fn test_authorization_code_verify_pkce_plain() {
        let code_verifier = "test_verifier";

        let code = AuthorizationCode::new(
            "test_code".to_string(),
            OAuthClientId::new(),
            UserId::new(),
            TenantId::new(),
            "https://example.com/callback".to_string(),
            vec!["openid".to_string()],
            Some(code_verifier.to_string()),
            Some("plain".to_string()),
        );

        assert!(code.verify_code_verifier(code_verifier).unwrap());
        assert!(!code.verify_code_verifier("wrong_verifier").unwrap());
    }

    #[test]
    fn test_authorization_code_no_pkce() {
        let code = create_test_authorization_code();
        
        // 没有 PKCE 要求时，任何 verifier 都应该通过
        assert!(code.verify_code_verifier("any_verifier").unwrap());
    }
}

#[cfg(test)]
mod access_token_tests {
    use super::*;

    fn create_test_access_token() -> AccessToken {
        AccessToken::new(
            "test_token".to_string(),
            OAuthClientId::new(),
            Some(UserId::new()),
            TenantId::new(),
            vec!["openid".to_string(), "profile".to_string()],
            3600,
        )
    }

    #[test]
    fn test_create_access_token() {
        let token = create_test_access_token();
        assert_eq!(token.token, "test_token");
        assert!(!token.is_expired());
        assert!(!token.is_revoked());
        assert!(token.is_valid());
    }

    #[test]
    fn test_access_token_has_scope() {
        let token = create_test_access_token();
        
        assert!(token.has_scope("openid"));
        assert!(token.has_scope("profile"));
        assert!(!token.has_scope("admin"));
    }

    #[test]
    fn test_access_token_revoke() {
        let mut token = create_test_access_token();
        assert!(token.is_valid());
        
        token.revoke();
        assert!(token.is_revoked());
        assert!(!token.is_valid());
    }

    #[test]
    fn test_access_token_get_remaining_seconds() {
        let token = create_test_access_token();
        let remaining = token.get_remaining_seconds();
        
        // 应该接近 3600 秒
        assert!(remaining > 3590 && remaining <= 3600);
    }
}

#[cfg(test)]
mod refresh_token_tests {
    use super::*;

    fn create_test_refresh_token() -> RefreshToken {
        RefreshToken::new(
            "test_refresh_token".to_string(),
            "test_access_token".to_string(),
            OAuthClientId::new(),
            UserId::new(),
            TenantId::new(),
            vec!["openid".to_string()],
            2592000, // 30 days
        )
    }

    #[test]
    fn test_create_refresh_token() {
        let token = create_test_refresh_token();
        assert_eq!(token.token, "test_refresh_token");
        assert_eq!(token.access_token, "test_access_token");
        assert!(!token.is_expired());
        assert!(!token.is_revoked());
        assert!(token.is_valid());
    }

    #[test]
    fn test_refresh_token_revoke() {
        let mut token = create_test_refresh_token();
        assert!(token.is_valid());
        
        token.revoke();
        assert!(token.is_revoked());
        assert!(!token.is_valid());
    }

    #[test]
    fn test_refresh_token_get_remaining_seconds() {
        let token = create_test_refresh_token();
        let remaining = token.get_remaining_seconds();
        
        // 应该接近 30 天（2592000 秒）
        assert!(remaining > 2591900 && remaining <= 2592000);
    }
}
