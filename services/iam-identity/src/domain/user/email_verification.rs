//! 邮箱验证实体

use chrono::{DateTime, Utc};
use common::{TenantId, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 邮箱验证 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmailVerificationId(pub Uuid);

impl EmailVerificationId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for EmailVerificationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EmailVerificationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 邮箱验证状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmailVerificationStatus {
    Pending,
    Verified,
    Expired,
}

/// 邮箱验证实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerification {
    /// 验证 ID
    pub id: EmailVerificationId,
    /// 用户 ID
    pub user_id: UserId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 邮箱地址
    pub email: String,
    /// 验证码（6位数字）
    pub code: String,
    /// 状态
    pub status: EmailVerificationStatus,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 验证时间
    pub verified_at: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl EmailVerification {
    /// 创建新的邮箱验证
    pub fn new(user_id: UserId, tenant_id: TenantId, email: String) -> Self {
        let code = Self::generate_code();
        let expires_at = Utc::now() + chrono::Duration::minutes(10);

        Self {
            id: EmailVerificationId::new(),
            user_id,
            tenant_id,
            email,
            code,
            status: EmailVerificationStatus::Pending,
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
    pub fn verify(&mut self, code: &str) -> Result<(), EmailVerificationError> {
        // 检查状态
        if self.status != EmailVerificationStatus::Pending {
            return Err(EmailVerificationError::AlreadyVerified);
        }

        // 检查是否过期
        if Utc::now() > self.expires_at {
            self.status = EmailVerificationStatus::Expired;
            return Err(EmailVerificationError::Expired);
        }

        // 验证码比对
        if self.code != code {
            return Err(EmailVerificationError::InvalidCode);
        }

        // 标记为已验证
        self.status = EmailVerificationStatus::Verified;
        self.verified_at = Some(Utc::now());

        Ok(())
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// 检查是否已验证
    pub fn is_verified(&self) -> bool {
        self.status == EmailVerificationStatus::Verified
    }

    /// 获取剩余有效时间（秒）
    pub fn get_remaining_seconds(&self) -> i64 {
        let remaining = self.expires_at.timestamp() - Utc::now().timestamp();
        remaining.max(0)
    }
}

/// 邮箱验证错误
#[derive(Debug, thiserror::Error)]
pub enum EmailVerificationError {
    #[error("Email verification has already been completed")]
    AlreadyVerified,

    #[error("Email verification code has expired")]
    Expired,

    #[error("Invalid verification code")]
    InvalidCode,
}
