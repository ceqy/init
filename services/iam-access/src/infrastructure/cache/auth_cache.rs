use std::sync::Arc;
use std::time::Duration;
use cuba_common::{TenantId, UserId};
use cuba_errors::AppResult;
use cuba_ports::CachePort;
use crate::domain::role::Role;

pub struct AuthCache {
    cache: Arc<dyn CachePort>,
    ttl: Duration,
}

impl AuthCache {
    pub fn new(cache: Arc<dyn CachePort>) -> Self {
        Self {
            cache,
            ttl: Duration::from_secs(300), // 5 minutes default
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    fn user_roles_key(tenant_id: &TenantId, user_id: &UserId) -> String {
        format!("iam:access:user_roles:{}:{}", tenant_id, user_id)
    }

    pub async fn get_user_roles(&self, tenant_id: &TenantId, user_id: &UserId) -> AppResult<Option<Vec<Role>>> {
        let key = Self::user_roles_key(tenant_id, user_id);
        let data = self.cache.get(&key).await?;
        
        match data {
            Some(json) => {
                let roles: Vec<Role> = serde_json::from_str(&json)
                    .map_err(|e| cuba_errors::AppError::internal(format!("Failed to deserialize roles from cache: {}", e)))?;
                Ok(Some(roles))
            }
            None => Ok(None),
        }
    }

    pub async fn set_user_roles(&self, tenant_id: &TenantId, user_id: &UserId, roles: &[Role]) -> AppResult<()> {
        let key = Self::user_roles_key(tenant_id, user_id);
        let json = serde_json::to_string(roles)
            .map_err(|e| cuba_errors::AppError::internal(format!("Failed to serialize roles for cache: {}", e)))?;
        
        self.cache.set(&key, &json, Some(self.ttl)).await
    }

    pub async fn invalidate_user_roles(&self, tenant_id: &TenantId, user_id: &UserId) -> AppResult<()> {
        let key = Self::user_roles_key(tenant_id, user_id);
        self.cache.delete(&key).await
    }

    // ============ Policy 缓存 ============

    fn tenant_policies_key(tenant_id: &TenantId) -> String {
        format!("iam:access:policies:{}", tenant_id)
    }

    /// 获取租户策略缓存
    pub async fn get_tenant_policies(&self, tenant_id: &TenantId) -> AppResult<Option<Vec<crate::domain::policy::Policy>>> {
        let key = Self::tenant_policies_key(tenant_id);
        let data = self.cache.get(&key).await?;
        
        match data {
            Some(json) => {
                let policies: Vec<crate::domain::policy::Policy> = serde_json::from_str(&json)
                    .map_err(|e| cuba_errors::AppError::internal(format!("Failed to deserialize policies from cache: {}", e)))?;
                Ok(Some(policies))
            }
            None => Ok(None),
        }
    }

    /// 设置租户策略缓存 (TTL: 10分钟)
    pub async fn set_tenant_policies(&self, tenant_id: &TenantId, policies: &[crate::domain::policy::Policy]) -> AppResult<()> {
        let key = Self::tenant_policies_key(tenant_id);
        let json = serde_json::to_string(policies)
            .map_err(|e| cuba_errors::AppError::internal(format!("Failed to serialize policies for cache: {}", e)))?;
        
        // 策略缓存使用较长的 TTL (10分钟)
        self.cache.set(&key, &json, Some(Duration::from_secs(600))).await
    }

    /// 失效租户策略缓存
    pub async fn invalidate_tenant_policies(&self, tenant_id: &TenantId) -> AppResult<()> {
        let key = Self::tenant_policies_key(tenant_id);
        self.cache.delete(&key).await
    }
}
