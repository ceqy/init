//! 策略评估器

use super::policy::{Effect, Policy};

/// 策略评估请求
#[derive(Debug, Clone)]
pub struct EvaluationRequest {
    /// 主体标识 (e.g., "user:123")
    pub subject: String,
    /// 主体拥有的角色 (e.g., ["admin", "editor"])
    pub roles: Vec<String>,
    /// 资源标识 (e.g., "order:456")
    pub resource: String,
    /// 操作 (e.g., "read")
    pub action: String,
    /// 上下文环境 (JSON 格式)
    pub context: Option<String>,
}

impl EvaluationRequest {
    pub fn new(subject: String, resource: String, action: String) -> Self {
        Self {
            subject,
            roles: Vec::new(),
            resource,
            action,
            context: None,
        }
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    /// 获取所有可能匹配的主体模式
    pub fn get_subject_patterns(&self) -> Vec<String> {
        let mut patterns = vec![self.subject.clone()];
        for role in &self.roles {
            patterns.push(format!("role:{}", role));
        }
        patterns
    }
}

/// 策略评估结果
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// 是否允许
    pub allowed: bool,
    /// 拒绝原因 (如果被拒绝)
    pub denied_reason: Option<String>,
    /// 匹配到的策略 ID
    pub matched_policy_id: Option<String>,
}

impl EvaluationResult {
    pub fn allow() -> Self {
        Self {
            allowed: true,
            denied_reason: None,
            matched_policy_id: None,
        }
    }

    pub fn deny(reason: String) -> Self {
        Self {
            allowed: false,
            denied_reason: Some(reason),
            matched_policy_id: None,
        }
    }

    pub fn with_policy(mut self, policy_id: String) -> Self {
        self.matched_policy_id = Some(policy_id);
        self
    }
}

/// 策略评估器
///
/// 使用 Deny-Override 策略: 如果有任何 Deny 策略匹配，则拒绝
pub struct PolicyEvaluator;

impl PolicyEvaluator {
    /// 评估策略
    ///
    /// 算法:
    /// 1. 收集所有匹配的策略
    /// 2. 按优先级排序
    /// 3. 如果存在 Deny 策略匹配，返回拒绝
    /// 4. 如果存在 Allow 策略匹配，返回允许
    /// 5. 默认拒绝 (deny by default)
    pub fn evaluate(policies: &[Policy], request: &EvaluationRequest) -> EvaluationResult {
        let subject_patterns = request.get_subject_patterns();

        // 收集匹配基本属性的策略
        let matching_policies: Vec<&Policy> = policies
            .iter()
            .filter(|p| {
                p.is_active
                    && subject_patterns.iter().any(|s| p.matches_subject(s))
                    && p.matches_resource(&request.resource)
                    && p.matches_action(&request.action)
            })
            .collect();

        // 进一步过滤满足条件的策略
        let mut final_policies: Vec<&Policy> = matching_policies
            .into_iter()
            .filter(|p| {
                if let Some(ref cond) = p.conditions {
                    Self::evaluate_conditions(cond, request)
                } else {
                    true
                }
            })
            .collect();

        // 按优先级降序排序
        final_policies.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Deny-Override: 检查是否有 Deny 策略
        for policy in &final_policies {
            if policy.effect == Effect::Deny {
                return EvaluationResult::deny(format!("Denied by policy: {}", policy.name))
                    .with_policy(policy.id.to_string());
            }
        }

        // 检查是否有 Allow 策略
        for policy in &final_policies {
            if policy.effect == Effect::Allow {
                return EvaluationResult::allow().with_policy(policy.id.to_string());
            }
        }

        // 默认拒绝
        EvaluationResult::deny("No matching policy found (deny by default)".to_string())
    }

    /// 评估条件表达式
    fn evaluate_conditions(conditions_str: &str, request: &EvaluationRequest) -> bool {
        let conditions: serde_json::Value = match serde_json::from_str(conditions_str) {
            Ok(v) => v,
            Err(_) => return false,
        };

        let context: serde_json::Value = request
            .context
            .as_ref()
            .and_then(|ctx| serde_json::from_str(ctx).ok())
            .unwrap_or(serde_json::Value::Null);

        if let Some(obj) = conditions.as_object() {
            for (key, expected_val) in obj {
                let actual_val = Self::get_value(key, &context, request);
                if !Self::match_value(expected_val, &actual_val, request) {
                    return false;
                }
            }
        }
        true
    }

    /// 从上下文或请求中获取值
    fn get_value(
        key: &str,
        context: &serde_json::Value,
        request: &EvaluationRequest,
    ) -> serde_json::Value {
        // 优先从 context 中查询 (支持级联 path: user.id)
        let mut current = context;
        for part in key.split('.') {
            if let Some(next) = current.get(part) {
                current = next;
            } else {
                // 没找到，尝试特殊变量
                return match key {
                    "subject.id" => serde_json::Value::String(request.subject.clone()),
                    "resource.id" => serde_json::Value::String(request.resource.clone()),
                    "action" => serde_json::Value::String(request.action.clone()),
                    _ => serde_json::Value::Null,
                };
            }
        }
        current.clone()
    }

    /// 匹配值 (支持占位符 ${subject.id})
    fn match_value(
        expected: &serde_json::Value,
        actual: &serde_json::Value,
        request: &EvaluationRequest,
    ) -> bool {
        if let Some(s) = expected.as_str()
            && s.starts_with("${")
            && s.ends_with('}')
        {
            let var_path = &s[2..s.len() - 1];
            let var_val = match var_path {
                "subject.id" => request.subject.clone(),
                "resource.id" => request.resource.clone(),
                _ => return false,
            };
            return actual.as_str() == Some(&var_val);
        }
        expected == actual
    }

    /// 批量评估
    pub fn batch_evaluate(
        policies: &[Policy],
        requests: &[EvaluationRequest],
    ) -> Vec<EvaluationResult> {
        requests
            .iter()
            .map(|req| Self::evaluate(policies, req))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::TenantId;

    fn setup_policies() -> Vec<Policy> {
        let tenant_id = TenantId::new();
        vec![
            Policy::allow(
                tenant_id.clone(),
                "Admin Full Access".to_string(),
                vec!["role:admin".to_string()],
                vec!["*".to_string()],
                vec!["*".to_string()],
            )
            .with_priority(100),
            Policy::allow(
                tenant_id.clone(),
                "Editor Article Access".to_string(),
                vec!["role:editor".to_string()],
                vec!["article:*".to_string()],
                vec!["read".to_string(), "write".to_string()],
            )
            .with_priority(50),
            Policy::deny(
                tenant_id.clone(),
                "Deny Delete for Everyone".to_string(),
                vec!["*".to_string()],
                vec!["*".to_string()],
                vec!["delete".to_string()],
            )
            .with_priority(200), // Higher priority than admin
        ]
    }

    #[test]
    fn test_admin_allow() {
        let policies = setup_policies();
        let request = EvaluationRequest::new(
            "user:123".to_string(),
            "article:456".to_string(),
            "read".to_string(),
        )
        .with_roles(vec!["admin".to_string()]);

        let result = PolicyEvaluator::evaluate(&policies, &request);
        assert!(result.allowed);
    }

    #[test]
    fn test_editor_allow() {
        let policies = setup_policies();
        let request = EvaluationRequest::new(
            "user:456".to_string(),
            "article:123".to_string(),
            "write".to_string(),
        )
        .with_roles(vec!["editor".to_string()]);

        let result = PolicyEvaluator::evaluate(&policies, &request);
        assert!(result.allowed);
    }

    #[test]
    fn test_deny_override() {
        let policies = setup_policies();
        // Admin 尝试删除，但 Deny 策略优先级更高
        let request = EvaluationRequest::new(
            "user:123".to_string(),
            "article:456".to_string(),
            "delete".to_string(),
        )
        .with_roles(vec!["admin".to_string()]);

        let result = PolicyEvaluator::evaluate(&policies, &request);
        assert!(!result.allowed);
        assert!(result.denied_reason.is_some());
    }

    #[test]
    fn test_policy_with_conditions() {
        let tenant_id = TenantId::new();
        let policies = vec![
            Policy::allow(
                tenant_id.clone(),
                "Owner Only".to_string(),
                vec!["*".to_string()],
                vec!["document:*".to_string()],
                vec!["read".to_string()],
            )
            .with_conditions(r#"{"owner_id": "${subject.id}"}"#.to_string()),
        ];

        // 匹配：owner_id 与 subject.id 相同
        let request_ok = EvaluationRequest::new(
            "user:123".to_string(),
            "document:abc".to_string(),
            "read".to_string(),
        )
        .with_context(r#"{"owner_id": "user:123"}"#.to_string());

        let result_ok = PolicyEvaluator::evaluate(&policies, &request_ok);
        assert!(
            result_ok.allowed,
            "Should be allowed if owner_id matches subject.id"
        );

        // 不匹配：owner_id 与 subject.id 不同
        let request_err = EvaluationRequest::new(
            "user:456".to_string(),
            "document:abc".to_string(),
            "read".to_string(),
        )
        .with_context(r#"{"owner_id": "user:123"}"#.to_string());

        let result_err = PolicyEvaluator::evaluate(&policies, &request_err);
        assert!(
            !result_err.allowed,
            "Should be denied if owner_id does not match subject.id"
        );
    }

    #[test]
    fn test_no_matching_policy() {
        let policies = setup_policies();
        let request = EvaluationRequest::new(
            "user:999".to_string(),
            "article:123".to_string(),
            "read".to_string(),
        ); // No roles

        let result = PolicyEvaluator::evaluate(&policies, &request);
        assert!(!result.allowed);
    }
}
