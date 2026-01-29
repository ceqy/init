//! 用户等级检测
//!
//! 从 JWT claims 中提取用户等级

use crate::middleware::AuthContext;
use crate::rate_limit::types::UserTier;

impl UserTier {
    /// 从认证上下文检测用户等级
    ///
    /// # 优先级
    /// 1. Vip: 有 `rate_limit:vip` 权限
    /// 2. Premium: 有 `rate_limit:premium` 权限
    /// 3. Standard: 已认证但无特殊权限
    /// 4. Anonymous: 未认证
    pub fn from_auth_context(auth_context: Option<&AuthContext>) -> Self {
        if let Some(ctx) = auth_context {
            if ctx.claims.has_permission("rate_limit:vip") {
                return Self::Vip;
            }
            if ctx.claims.has_permission("rate_limit:premium") {
                return Self::Premium;
            }
            Self::Standard
        } else {
            Self::Anonymous
        }
    }

    /// 从 permissions 列表检测用户等级
    pub fn from_permissions(permissions: &[String]) -> Self {
        if permissions.contains(&"rate_limit:vip".to_string()) {
            return Self::Vip;
        }
        if permissions.contains(&"rate_limit:premium".to_string()) {
            return Self::Premium;
        }
        Self::Standard
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuba_auth_core::Claims;
    use std::collections::HashMap;

    fn create_test_claims(permissions: Vec<String>) -> Claims {
        Claims {
            sub: "user-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            exp: 9999999999,
            iat: 0,
            jti: "test-jti".to_string(),
            iss: "test-issuer".to_string(),
            aud: "test-audience".to_string(),
            token_type: "access".to_string(),
            permissions,
            roles: vec![],
        }
    }

    fn create_auth_context(permissions: Vec<String>) -> AuthContext {
        AuthContext {
            claims: create_test_claims(permissions),
            token: "test-token".to_string(),
        }
    }

    #[test]
    fn test_from_auth_context_vip() {
        let ctx = create_auth_context(vec![
            "rate_limit:vip".to_string(),
            "other:permission".to_string(),
        ]);
        let tier = UserTier::from_auth_context(Some(&ctx));
        assert_eq!(tier, UserTier::Vip);
    }

    #[test]
    fn test_from_auth_context_premium() {
        let ctx = create_auth_context(vec!["rate_limit:premium".to_string()]);
        let tier = UserTier::from_auth_context(Some(&ctx));
        assert_eq!(tier, UserTier::Premium);
    }

    #[test]
    fn test_from_auth_context_standard() {
        let ctx = create_auth_context(vec!["read:items".to_string(), "write:items".to_string()]);
        let tier = UserTier::from_auth_context(Some(&ctx));
        assert_eq!(tier, UserTier::Standard);
    }

    #[test]
    fn test_from_auth_context_anonymous() {
        let tier = UserTier::from_auth_context(None);
        assert_eq!(tier, UserTier::Anonymous);
    }

    #[test]
    fn test_from_permissions_vip() {
        let perms = vec!["rate_limit:vip".to_string()];
        assert_eq!(UserTier::from_permissions(&perms), UserTier::Vip);
    }

    #[test]
    fn test_from_permissions_premium() {
        let perms = vec!["rate_limit:premium".to_string()];
        assert_eq!(UserTier::from_permissions(&perms), UserTier::Premium);
    }

    #[test]
    fn test_from_permissions_standard() {
        let perms = vec!["read:items".to_string()];
        assert_eq!(UserTier::from_permissions(&perms), UserTier::Standard);
    }

    #[test]
    fn test_from_permissions_empty() {
        assert_eq!(UserTier::from_permissions(&[]), UserTier::Standard);
    }

    #[test]
    fn test_vip_overrides_premium() {
        let ctx = create_auth_context(vec![
            "rate_limit:vip".to_string(),
            "rate_limit:premium".to_string(),
        ]);
        let tier = UserTier::from_auth_context(Some(&ctx));
        assert_eq!(tier, UserTier::Vip);
    }
}
