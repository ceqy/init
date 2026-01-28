//! 授权检查服务
//!
//! 统一访问控制决策点 (PDP)，聚合 RBAC 和 Policy 检查结果

use std::sync::Arc;

use cuba_common::TenantId;
use cuba_errors::AppResult;

use crate::domain::policy::{EvaluationRequest, PolicyEvaluator, PolicyRepository};
use crate::domain::role::{Permission, UserRoleRepository};

/// 授权检查请求
#[derive(Debug, Clone)]
pub struct AuthorizationCheckRequest {
    /// 用户 ID
    pub user_id: String,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 资源标识
    pub resource: String,
    /// 操作
    pub action: String,
    /// 上下文环境 (可选)
    pub context: Option<String>,
}

/// 授权检查结果
#[derive(Debug, Clone)]
pub struct AuthorizationCheckResult {
    /// 是否允许
    pub allowed: bool,
    /// 决策来源 (RBAC/Policy)
    pub decision_source: DecisionSource,
    /// 拒绝原因 (如果被拒绝)
    pub denied_reason: Option<String>,
    /// 匹配的权限代码 (如果是 RBAC)
    pub matched_permission: Option<String>,
    /// 匹配的策略 ID (如果是 Policy)
    pub matched_policy_id: Option<String>,
}

/// 决策来源
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionSource {
    /// RBAC 权限
    Rbac,
    /// ABAC 策略
    Policy,
    /// 默认拒绝
    DefaultDeny,
}

impl std::fmt::Display for DecisionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecisionSource::Rbac => write!(f, "RBAC"),
            DecisionSource::Policy => write!(f, "POLICY"),
            DecisionSource::DefaultDeny => write!(f, "DEFAULT_DENY"),
        }
    }
}

/// 授权服务
/// 
/// 决策逻辑:
/// 1. 首先检查 Policy (ABAC) - Deny 优先
/// 2. 如果 Policy 返回 Deny，直接拒绝
/// 3. 如果 Policy 返回 Allow，直接允许
/// 4. 如果 Policy 无匹配，检查 RBAC
/// 5. 如果 RBAC 允许，则允许
/// 6. 默认拒绝
pub struct AuthorizationService<PR, UR>
where
    PR: PolicyRepository,
    UR: UserRoleRepository,
{
    policy_repo: Arc<PR>,
    user_role_repo: Arc<UR>,
    cache: Option<Arc<crate::infrastructure::cache::AuthCache>>,
}

impl<PR, UR> AuthorizationService<PR, UR>
where
    PR: PolicyRepository,
    UR: UserRoleRepository,
{
    pub fn new(policy_repo: Arc<PR>, user_role_repo: Arc<UR>) -> Self {
        Self {
            policy_repo,
            user_role_repo,
            cache: None,
        }
    }

    pub fn with_cache(mut self, cache: Arc<crate::infrastructure::cache::AuthCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// 执行授权检查
    pub async fn check(&self, request: AuthorizationCheckRequest) -> AppResult<AuthorizationCheckResult> {
        use metrics::{counter, histogram};
        let start = std::time::Instant::now();
        
        let result = self.check_internal(&request).await;
        
        // 记录指标
        if let Ok(ref res) = result {
            counter!("authorization_checks_total", 
                "decision" => res.decision_source.to_string(),
                "allowed" => res.allowed.to_string()
            ).increment(1);
        } else {
            counter!("authorization_checks_errors_total").increment(1);
        }
        
        histogram!("authorization_check_duration_ms")
            .record(start.elapsed().as_millis() as f64);
        
        result
    }

    /// 内部授权检查逻辑
    async fn check_internal(&self, request: &AuthorizationCheckRequest) -> AppResult<AuthorizationCheckResult> {
        // 1. 获取用户的角色
        let user_roles = self.user_role_repo
            .get_user_roles(&request.user_id, &request.tenant_id)
            .await?;
        
        let role_codes: Vec<String> = user_roles.iter().map(|r| r.code.clone()).collect();

        // 2. 加载所有激活的策略 (优先使用缓存)
        let policies = self.get_cached_policies(&request.tenant_id).await?;


        // 3. 构建策略评估请求
        let eval_request = EvaluationRequest::new(
            format!("user:{}", request.user_id),
            request.resource.clone(),
            request.action.clone(),
        )
        .with_roles(role_codes.clone());

        let eval_request = if let Some(ctx) = request.context.clone() {
            eval_request.with_context(ctx)
        } else {
            eval_request
        };

        // 4. 执行策略评估
        let policy_result = PolicyEvaluator::evaluate(&policies, &eval_request);

        // 5. 检查策略结果
        if policy_result.matched_policy_id.is_some() {
            // 策略有匹配
            return Ok(AuthorizationCheckResult {
                allowed: policy_result.allowed,
                decision_source: DecisionSource::Policy,
                denied_reason: policy_result.denied_reason,
                matched_permission: None,
                matched_policy_id: policy_result.matched_policy_id,
            });
        }

        // 6. 策略无匹配，检查 RBAC
        let permission_code = format!("{}:{}", request.resource, request.action);
        let has_permission = self.user_role_repo
            .user_has_permission(&request.user_id, &request.tenant_id, &permission_code)
            .await?;

        if has_permission {
            return Ok(AuthorizationCheckResult {
                allowed: true,
                decision_source: DecisionSource::Rbac,
                denied_reason: None,
                matched_permission: Some(permission_code),
                matched_policy_id: None,
            });
        }

        // 7. 检查通配符权限 (resource:*)
        let wildcard_permission = format!("{}:*", request.resource);
        let has_wildcard = self.user_role_repo
            .user_has_permission(&request.user_id, &request.tenant_id, &wildcard_permission)
            .await?;

        if has_wildcard {
            return Ok(AuthorizationCheckResult {
                allowed: true,
                decision_source: DecisionSource::Rbac,
                denied_reason: None,
                matched_permission: Some(wildcard_permission),
                matched_policy_id: None,
            });
        }

        // 8. 默认拒绝
        Ok(AuthorizationCheckResult {
            allowed: false,
            decision_source: DecisionSource::DefaultDeny,
            denied_reason: Some("No matching permission or policy found".to_string()),
            matched_permission: None,
            matched_policy_id: None,
        })
    }

    /// 批量检查 (并发执行)
    pub async fn batch_check(&self, requests: Vec<AuthorizationCheckRequest>) -> AppResult<Vec<AuthorizationCheckResult>> {
        use futures::stream::{self, StreamExt};
        
        let results: Vec<AppResult<AuthorizationCheckResult>> = stream::iter(requests)
            .map(|req| self.check(req))
            .buffer_unordered(10) // 最多 10 个并发
            .collect()
            .await;
        
        results.into_iter().collect()
    }

    /// 获取用户在指定租户下授予的所有权限
    pub async fn get_user_granted_permissions(
        &self,
        user_id: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<Permission>> {
        self.user_role_repo
            .get_user_permissions(user_id, tenant_id)
            .await
    }

    /// 获取用户在指定租户下的所有角色
    pub async fn get_user_roles(
        &self,
        user_id: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<crate::domain::role::Role>> {
        self.user_role_repo
            .get_user_roles(user_id, tenant_id)
            .await
    }

    /// 获取策略 (优先使用缓存)
    async fn get_cached_policies(&self, tenant_id: &TenantId) -> AppResult<Vec<crate::domain::policy::Policy>> {
        // 尝试从缓存获取
        if let Some(cache) = &self.cache {
            if let Ok(Some(policies)) = cache.get_tenant_policies(tenant_id).await {
                return Ok(policies);
            }
        }

        // 缓存未命中，从数据库加载
        let policies = self.policy_repo
            .list_active_by_tenant(tenant_id)
            .await?;

        // 写入缓存
        if let Some(cache) = &self.cache {
            let _ = cache.set_tenant_policies(tenant_id, &policies).await;
        }

        Ok(policies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::domain::role::{Role, RoleId};
    use crate::domain::policy::Policy;

    // --- Mocks ---

    struct MockPolicyRepository {
        policies: Vec<Policy>,
    }

    #[async_trait]
    impl PolicyRepository for MockPolicyRepository {
        async fn create(&self, _policy: &Policy) -> AppResult<()> { Ok(()) }
        async fn update(&self, _policy: &Policy) -> AppResult<()> { Ok(()) }
        async fn delete(&self, _id: &crate::domain::policy::PolicyId) -> AppResult<()> { Ok(()) }
        async fn find_by_id(&self, _id: &crate::domain::policy::PolicyId) -> AppResult<Option<Policy>> { Ok(None) }
        async fn list_active_by_tenant(&self, _tenant_id: &TenantId) -> AppResult<Vec<Policy>> {
            Ok(self.policies.clone())
        }
        
        async fn list_by_tenant(&self, _tenant_id: &TenantId, _page: u32, _page_size: u32) -> AppResult<(Vec<Policy>, i64)> {
            Ok((self.policies.clone(), self.policies.len() as i64))
        }
        async fn find_by_subject(&self, _tenant_id: &TenantId, _subject: &str) -> AppResult<Vec<Policy>> { Ok(vec![]) }
        async fn find_by_resource(&self, _tenant_id: &TenantId, _resource: &str) -> AppResult<Vec<Policy>> { Ok(vec![]) }
        async fn exists_by_name(&self, _tenant_id: &TenantId, _name: &str) -> AppResult<bool> { Ok(false) }
    }

    struct MockUserRoleRepository {
        roles: Vec<Role>,
        permissions: Vec<Permission>,
    }

    #[async_trait]
    impl UserRoleRepository for MockUserRoleRepository {
        async fn assign_roles(&self, _user_id: &str, _tenant_id: &TenantId, _role_ids: &[RoleId]) -> AppResult<()> { Ok(()) }
        async fn remove_roles(&self, _user_id: &str, _tenant_id: &TenantId, _role_ids: &[RoleId]) -> AppResult<()> { Ok(()) }
        async fn get_user_roles(&self, _user_id: &str, _tenant_id: &TenantId) -> AppResult<Vec<Role>> {
            Ok(self.roles.clone())
        }
        async fn get_user_permissions(&self, _user_id: &str, _tenant_id: &TenantId) -> AppResult<Vec<Permission>> {
            Ok(self.permissions.clone())
        }
        async fn clear_user_roles(&self, _user_id: &str, _tenant_id: &TenantId) -> AppResult<()> { Ok(()) }
        async fn user_has_role(&self, _user_id: &str, _tenant_id: &TenantId, _role_id: &RoleId) -> AppResult<bool> { Ok(false) }
        async fn user_has_permission(&self, _user_id: &str, _tenant_id: &TenantId, _permission_code: &str) -> AppResult<bool> {
            Ok(self.permissions.iter().any(|p| p.code == _permission_code))
        }
    }

    // --- Tests ---

    #[tokio::test]
    async fn test_rbac_allow() {
        let tenant_id = TenantId::new();
        
        // Setup Roles & Permissions
        let perm = Permission::new("res:read".to_string(), "name".to_string(), None, "res".to_string(), "read".to_string(), "mod".to_string());
        let mock_user_role_repo = Arc::new(MockUserRoleRepository {
            roles: vec![],
            permissions: vec![perm],
        });
        
        // Setup Policies (Empty)
        let mock_policy_repo = Arc::new(MockPolicyRepository { policies: vec![] });
        
        let service = AuthorizationService::new(mock_policy_repo, mock_user_role_repo);
        
        let req = AuthorizationCheckRequest {
            user_id: "user1".to_string(),
            tenant_id,
            resource: "res".to_string(),
            action: "read".to_string(),
            context: None,
        };
        
        let result = service.check(req).await.unwrap();
        assert!(result.allowed);
        assert!(matches!(result.decision_source, DecisionSource::Rbac));
    }

    #[tokio::test]
    async fn test_policy_deny_override() {
        let tenant_id = TenantId::new();
        
        // RBAC allows
        let perm = Permission::new("code".to_string(), "name".to_string(), None, "res".to_string(), "read".to_string(), "mod".to_string());
        let mock_user_role_repo = Arc::new(MockUserRoleRepository {
            roles: vec![],
            permissions: vec![perm],
        });
        
        // Policy denies
        let policy = Policy::deny(
            tenant_id.clone(), "deny-all".to_string(), 
            vec!["*".to_string()], vec!["*".to_string()], vec!["*".to_string()]
        );
        let mock_policy_repo = Arc::new(MockPolicyRepository { policies: vec![policy] });
        
        let service = AuthorizationService::new(mock_policy_repo, mock_user_role_repo);
        
        let req = AuthorizationCheckRequest {
            user_id: "user1".to_string(),
            tenant_id,
            resource: "res".to_string(),
            action: "read".to_string(),
            context: None,
        };
        
        let result = service.check(req).await.unwrap();
        assert!(!result.allowed);
        assert!(matches!(result.decision_source, DecisionSource::Policy));
    }

    #[tokio::test]
    async fn test_default_deny() {
        let tenant_id = TenantId::new();
        
        // No RBAC, No Policies
        let mock_user_role_repo = Arc::new(MockUserRoleRepository {
            roles: vec![],
            permissions: vec![],
        });
        let mock_policy_repo = Arc::new(MockPolicyRepository { policies: vec![] });
        
        let service = AuthorizationService::new(mock_policy_repo, mock_user_role_repo);
        
        let req = AuthorizationCheckRequest {
            user_id: "user1".to_string(),
            tenant_id,
            resource: "res".to_string(),
            action: "read".to_string(),
            context: None,
        };
        
        let result = service.check(req).await.unwrap();
        assert!(!result.allowed);
        assert!(matches!(result.decision_source, DecisionSource::DefaultDeny));
    }
}
