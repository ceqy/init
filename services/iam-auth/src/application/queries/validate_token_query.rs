//! 验证令牌查询

use cuba_cqrs_core::Query;

/// 验证令牌查询
#[derive(Debug, Clone)]
pub struct ValidateTokenQuery {
    pub access_token: String,
}

impl Query for ValidateTokenQuery {
    type Result = ValidateTokenResult;
}

/// 验证令牌结果
#[derive(Debug, Clone)]
pub struct ValidateTokenResult {
    pub valid: bool,
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub permissions: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}
