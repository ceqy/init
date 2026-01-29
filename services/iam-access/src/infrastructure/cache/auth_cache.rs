use crate::domain::role::Role;
use cuba_common::{TenantId, UserId};
use cuba_errors::AppResult;
use cuba_ports::CachePort;
use std::sync::Arc;
use std::time::Duration;

/// 认证缓存配置
#[derive(Clone)]
pub struct AuthCacheConfig {
    /// 用户角色缓存 TTL（秒）
    pub user_roles_ttl_secs: u64,
    /// 角色缓存 TTL（秒）
    pub role_ttl_secs: u64,
    /// 策略缓存 TTL（秒）
    pub policy_ttl_secs: u64,
}

impl Default for AuthCacheConfig {
    fn default() -> Self {
        Self {
            user_roles_ttl_secs: 300, // 5 分钟
            role_ttl_secs: 600,       // 10 分钟
            policy_ttl_secs: 600,     // 10 分钟
        }
    }
}

pub struct AuthCache {
    cache: Arc<dyn CachePort>,
    config: AuthCacheConfig,
}

impl AuthCache {
    pub fn new(cache: Arc<dyn CachePort>) -> Self {
        Self {
            cache,
            config: AuthCacheConfig::default(),
        }
    }

    pub fn with_config(mut self, config: AuthCacheConfig) -> Self {
        self.config = config;
        self
    }

    /// 兼容旧的 API
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.config.user_roles_ttl_secs = ttl.as_secs();
        self
    }

    fn user_roles_key(tenant_id: &TenantId, user_id: &UserId) -> String {
        format!("iam:access:user_roles:{}:{}", tenant_id, user_id)
    }

    pub async fn get_user_roles(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> AppResult<Option<Vec<Role>>> {
        let key = Self::user_roles_key(tenant_id, user_id);
        let data = self.cache.get(&key).await?;

        match data {
            Some(json) => {
                let roles: Vec<Role> = serde_json::from_str(&json).map_err(|e| {
                    cuba_errors::AppError::internal(format!(
                        "Failed to deserialize roles from cache: {}",
                        e
                    ))
                })?;
                Ok(Some(roles))
            }
            None => Ok(None),
        }
    }

    pub async fn set_user_roles(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        roles: &[Role],
    ) -> AppResult<()> {
        let key = Self::user_roles_key(tenant_id, user_id);
        let json = serde_json::to_string(roles).map_err(|e| {
            cuba_errors::AppError::internal(format!("Failed to serialize roles for cache: {}", e))
        })?;

        let ttl = Duration::from_secs(self.config.user_roles_ttl_secs);
        self.cache.set(&key, &json, Some(ttl)).await
    }

    pub async fn invalidate_user_roles(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> AppResult<()> {
        let key = Self::user_roles_key(tenant_id, user_id);
        self.cache.delete(&key).await
    }

    // ============ Role 单条缓存 ============

    fn role_by_id_key(role_id: &uuid::Uuid) -> String {
        format!("iam:access:role:id:{}", role_id)
    }

    fn role_by_code_key(tenant_id: &TenantId, code: &str) -> String {
        format!("iam:access:role:code:{}:{}", tenant_id, code)
    }

    /// 获取角色 (按 ID)
    pub async fn get_role_by_id(&self, role_id: &uuid::Uuid) -> AppResult<Option<Role>> {
        let key = Self::role_by_id_key(role_id);
        let data = self.cache.get(&key).await?;

        match data {
            Some(json) => {
                let role: Role = serde_json::from_str(&json).map_err(|e| {
                    cuba_errors::AppError::internal(format!("Failed to deserialize role: {}", e))
                })?;
                Ok(Some(role))
            }
            None => Ok(None),
        }
    }

    /// 设置角色缓存 (按 ID)
    pub async fn set_role(&self, role: &Role) -> AppResult<()> {
        let key_id = Self::role_by_id_key(&role.id.0);
        let key_code = Self::role_by_code_key(&role.tenant_id, &role.code);
        let json = serde_json::to_string(role).map_err(|e| {
            cuba_errors::AppError::internal(format!("Failed to serialize role: {}", e))
        })?;

        let ttl = Duration::from_secs(self.config.role_ttl_secs);
        // 存储两个键
        self.cache.set(&key_id, &json, Some(ttl)).await?;
        self.cache.set(&key_code, &json, Some(ttl)).await
    }

    /// 获取角色 (按 code)
    pub async fn get_role_by_code(
        &self,
        tenant_id: &TenantId,
        code: &str,
    ) -> AppResult<Option<Role>> {
        let key = Self::role_by_code_key(tenant_id, code);
        let data = self.cache.get(&key).await?;

        match data {
            Some(json) => {
                let role: Role = serde_json::from_str(&json).map_err(|e| {
                    cuba_errors::AppError::internal(format!("Failed to deserialize role: {}", e))
                })?;
                Ok(Some(role))
            }
            None => Ok(None),
        }
    }

    /// 失效角色缓存
    pub async fn invalidate_role(&self, role: &Role) -> AppResult<()> {
        let key_id = Self::role_by_id_key(&role.id.0);
        let key_code = Self::role_by_code_key(&role.tenant_id, &role.code);
        self.cache.delete(&key_id).await?;
        self.cache.delete(&key_code).await
    }

    // ============ Policy 缓存 ============

    fn tenant_policies_key(tenant_id: &TenantId) -> String {
        format!("iam:access:policies:{}", tenant_id)
    }

    /// 获取租户策略缓存
    pub async fn get_tenant_policies(
        &self,
        tenant_id: &TenantId,
    ) -> AppResult<Option<Vec<crate::domain::policy::Policy>>> {
        let key = Self::tenant_policies_key(tenant_id);
        let data = self.cache.get(&key).await?;

        match data {
            Some(json) => {
                let policies: Vec<crate::domain::policy::Policy> = serde_json::from_str(&json)
                    .map_err(|e| {
                        cuba_errors::AppError::internal(format!(
                            "Failed to deserialize policies from cache: {}",
                            e
                        ))
                    })?;
                Ok(Some(policies))
            }
            None => Ok(None),
        }
    }

    /// 设置租户策略缓存
    pub async fn set_tenant_policies(
        &self,
        tenant_id: &TenantId,
        policies: &[crate::domain::policy::Policy],
    ) -> AppResult<()> {
        let key = Self::tenant_policies_key(tenant_id);
        let json = serde_json::to_string(policies).map_err(|e| {
            cuba_errors::AppError::internal(format!(
                "Failed to serialize policies for cache: {}",
                e
            ))
        })?;

        let ttl = Duration::from_secs(self.config.policy_ttl_secs);
        self.cache.set(&key, &json, Some(ttl)).await
    }

    /// 失效租户策略缓存
    pub async fn invalidate_tenant_policies(&self, tenant_id: &TenantId) -> AppResult<()> {
        let key = Self::tenant_policies_key(tenant_id);
        self.cache.delete(&key).await
    }
}
