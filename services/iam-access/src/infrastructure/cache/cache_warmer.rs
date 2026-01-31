//! 缓存预热
//!
//! 在应用启动时预加载热点数据到缓存，避免冷启动时的缓存雪崩

use crate::domain::policy::Policy;
use crate::domain::role::Role;
use crate::infrastructure::cache::AuthCache;
use common::TenantId;
use errors::AppResult;
use std::sync::Arc;
use tracing::{info, warn};

/// 缓存预热器
pub struct CacheWarmer {
    auth_cache: Arc<AuthCache>,
}

impl CacheWarmer {
    pub fn new(auth_cache: Arc<AuthCache>) -> Self {
        Self { auth_cache }
    }

    /// 预热所有租户的策略缓存
    pub async fn warm_policies(
        &self,
        policy_loader: impl Fn(
            &TenantId,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = AppResult<Vec<Policy>>> + Send>,
        >,
        tenant_ids: &[TenantId],
    ) -> AppResult<()> {
        info!(
            "Starting policy cache warming for {} tenants",
            tenant_ids.len()
        );

        let mut success_count = 0;
        let mut error_count = 0;

        for tenant_id in tenant_ids {
            match policy_loader(tenant_id).await {
                Ok(policies) => {
                    if let Err(e) = self
                        .auth_cache
                        .set_tenant_policies(tenant_id, &policies)
                        .await
                    {
                        warn!(
                            tenant_id = %tenant_id,
                            error = %e,
                            "Failed to warm policy cache"
                        );
                        error_count += 1;
                    } else {
                        success_count += 1;
                    }
                }
                Err(e) => {
                    warn!(
                        tenant_id = %tenant_id,
                        error = %e,
                        "Failed to load policies for warming"
                    );
                    error_count += 1;
                }
            }
        }

        info!(
            "Policy cache warming completed: {} success, {} errors",
            success_count, error_count
        );

        Ok(())
    }

    /// 预热热点角色缓存
    pub async fn warm_roles(
        &self,
        role_loader: impl Fn() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = AppResult<Vec<Role>>> + Send>,
        >,
    ) -> AppResult<()> {
        info!("Starting role cache warming");

        match role_loader().await {
            Ok(roles) => {
                let mut success_count = 0;
                let mut error_count = 0;

                for role in roles {
                    if let Err(e) = self.auth_cache.set_role(&role).await {
                        warn!(
                            role_id = %role.id.0,
                            error = %e,
                            "Failed to warm role cache"
                        );
                        error_count += 1;
                    } else {
                        success_count += 1;
                    }
                }

                info!(
                    "Role cache warming completed: {} success, {} errors",
                    success_count, error_count
                );
            }
            Err(e) => {
                warn!(error = %e, "Failed to load roles for warming");
            }
        }

        Ok(())
    }

    /// 预热所有缓存（策略 + 角色）
    pub async fn warm_all(
        &self,
        policy_loader: impl Fn(
            &TenantId,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = AppResult<Vec<Policy>>> + Send>,
        >,
        role_loader: impl Fn() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = AppResult<Vec<Role>>> + Send>,
        >,
        tenant_ids: &[TenantId],
    ) -> AppResult<()> {
        info!("Starting cache warming");

        // 并发预热策略和角色
        let policies_future = self.warm_policies(policy_loader, tenant_ids);
        let roles_future = self.warm_roles(role_loader);

        let (policies_result, roles_result) = tokio::join!(policies_future, roles_future);

        policies_result?;
        roles_result?;

        info!("Cache warming completed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache::auth_cache::AuthCacheConfig;
    use ports::CachePort;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockCache {
        data: Arc<Mutex<HashMap<String, String>>>,
    }

    #[async_trait::async_trait]
    impl CachePort for MockCache {
        async fn get(&self, key: &str) -> AppResult<Option<String>> {
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        async fn set(
            &self,
            key: &str,
            value: &str,
            _ttl: Option<std::time::Duration>,
        ) -> AppResult<()> {
            self.data
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_string());
            Ok(())
        }

        async fn delete(&self, key: &str) -> AppResult<()> {
            self.data.lock().unwrap().remove(key);
            Ok(())
        }

        async fn exists(&self, key: &str) -> AppResult<bool> {
            Ok(self.data.lock().unwrap().contains_key(key))
        }

        async fn expire(&self, _key: &str, _ttl: std::time::Duration) -> AppResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_warm_policies() {
        let mock_cache = Arc::new(MockCache {
            data: Arc::new(Mutex::new(HashMap::new())),
        }) as Arc<dyn CachePort>;
        let auth_cache =
            Arc::new(AuthCache::new(mock_cache.clone()).with_config(AuthCacheConfig::default()));
        let warmer = CacheWarmer::new(auth_cache);

        let tenant_ids = vec![TenantId::new()];

        let result = warmer
            .warm_policies(|_tenant_id| Box::pin(async { Ok(vec![]) }), &tenant_ids)
            .await;

        assert!(result.is_ok());
    }
}
