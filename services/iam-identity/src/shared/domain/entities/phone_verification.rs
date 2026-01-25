//! 手机验证实体

use chrono::{DateTime, Utc};
use cuba_common::{TenantId, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 手机验证 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhoneVerificationId(pub Uuid);

impl PhoneVerificationId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for PhoneVerificationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PhoneVerificationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 手机验证状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhoneVerificationStatus {
    Pending,
    Verified,
    Expired,
}

/// 手机验证实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneVerification {
    /// 验证 ID
    pub id: PhoneVerificationId,
    /// 用户 ID
    pub user_id: UserId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 手机号码
    pub phone: String,
    /// 验证码（6位数字）
    pub code: String,
    /// 状态
    pub status: PhoneVerificationStatus,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 验证时间
    pub verified_at: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl PhoneVerification {
    /// 创建新的手机验证
    pub fn new(user_id: UserId, tenant_id: TenantId, phone: String) -> Self {
        let code = Self::generate_code();
        let expires_at = Utc::now() + chrono::Duration::minutes(5); // 5分钟过期

        Self {
            id: PhoneVerificationId::new(),
            user_id,
            tenant_id,
            phone,
            code,
            status: PhoneVerificationStatus::Pending,
            expires_at,
            verified_at: None,
            created_at: Utc::now(),
        }
    }

    /// 生成6位数字验证码
    fn generate_code() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(0..1000000))
    }

    /// 验证验证码
    pub fn verify(&mut self, code: &str) -> Result<(), PhoneVerificationError> {
        // 检查状态
        if self.status != PhoneVerificationStatus::Pending {
            return Err(PhoneVerificationError::AlreadyVerified);
        }

        // 检查是否过期
        if Utc::now() > self.expires_at {
            self.status = PhoneVerificationStatus::Expired;
            return Err(PhoneVerificationError::Expired);
        }

        // 验证码比对
        if self.code != code {
            return Err(PhoneVerificationError::InvalidCode);
        }

        // 标记为已验证
        self.status = PhoneVerificationStatus::Verified;
        self.verified_at = Some(Utc::now());

        Ok(())
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// 检查是否已验证
    pub fn is_verified(&self) -> bool {
        self.status == PhoneVerificationStatus::Verified
    }

    /// 获取剩余有效时间（秒）
    pub fn get_remaining_seconds(&self) -> i64 {
        let remaining = self.expires_at.timestamp() - Utc::now().timestamp();
        remaining.max(0)
    }
}

/// 手机验证错误
#[derive(Debug, thiserror::Error)]
pub enum PhoneVerificationError {
    #[error("Phone verification has already been completed")]
    AlreadyVerified,

    #[error("Phone verification code has expired")]
    Expired,

    #[error("Invalid verification code")]
    InvalidCode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_phone_verification() {
        let user_id = UserId::new();
        let tenant_id = TenantId::new();
        let verification = PhoneVerification::new(
            user_id,
            tenant_id,
            "+8613800138000".to_string(),
        );

        assert_eq!(verification.status, PhoneVerificationStatus::Pending);
        assert_eq!(verification.code.len(), 6);
        assert!(verification.code.chars().all(|c| c.is_ascii_digit()));
        assert!(!verification.is_expired());
    }

    #[test]
    fn test_verify_with_correct_code() {
        let user_id = UserId::new();
        let tenant_id = TenantId::new();
        let mut verification = PhoneVerification::new(
            user_id,
            tenant_id,
            "+8613800138000".to_string(),
        );

        let code = verification.code.clone();
        let result = verification.verify(&code);

        assert!(result.is_ok());
        assert_eq!(verification.status, PhoneVerificationStatus::Verified);
        assert!(verification.verified_at.is_some());
    }

    #[test]
    fn test_verify_with_incorrect_code() {
        let user_id = UserId::new();
        let tenant_id = TenantId::new();
        let mut verification = PhoneVerification::new(
            user_id,
            tenant_id,
            "+8613800138000".to_string(),
        );

        let result = verification.verify("000000");

        assert!(result.is_err());
        assert_eq!(verification.status, PhoneVerificationStatus::Pending);
    }
}
