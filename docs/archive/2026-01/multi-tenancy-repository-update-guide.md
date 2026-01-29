# 多租户 Repository 更新脚本

## 需要更新的 Repository Trait

### 1. LoginLogRepository
```rust
// services/iam-identity/src/auth/domain/repositories/login_log_repository.rs

#[async_trait]
pub trait LoginLogRepository: Send + Sync {
    async fn save(&self, log: &LoginLog) -> AppResult<()>;
    async fn find_by_user_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        page: i32,
        page_size: i32,
    ) -> AppResult<(Vec<LoginLog>, i64)>;
    async fn find_failed_attempts(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        since: DateTime<Utc>,
    ) -> AppResult<Vec<LoginLog>>;
}
```

### 2. PasswordResetRepository
```rust
// services/iam-identity/src/auth/domain/repositories/password_reset_repository.rs

#[async_trait]
pub trait PasswordResetRepository: Send + Sync {
    async fn save(&self, reset: &PasswordReset) -> AppResult<()>;
    async fn find_by_token(&self, token: &str, tenant_id: &TenantId) -> AppResult<Option<PasswordReset>>;
    async fn find_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<Vec<PasswordReset>>;
    async fn mark_as_used(&self, token: &str, tenant_id: &TenantId) -> AppResult<()>;
    async fn cleanup_expired(&self, tenant_id: &TenantId) -> AppResult<u64>;
}
```

### 3. WebAuthnCredentialRepository
```rust
// services/iam-identity/src/auth/domain/repositories/webauthn_credential_repository.rs

#[async_trait]
pub trait WebAuthnCredentialRepository: Send + Sync {
    async fn save(&self, credential: &WebAuthnCredential) -> AppResult<()>;
    async fn find_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<Vec<WebAuthnCredential>>;
    async fn find_by_credential_id(&self, credential_id: &[u8], tenant_id: &TenantId) -> AppResult<Option<WebAuthnCredential>>;
    async fn update_counter(&self, credential_id: &[u8], tenant_id: &TenantId, counter: u32) -> AppResult<()>;
    async fn delete(&self, credential_id: &[u8], tenant_id: &TenantId) -> AppResult<()>;
}
```

### 4. BackupCodeRepository
```rust
// services/iam-identity/src/auth/domain/repositories/backup_code_repository.rs

#[async_trait]
pub trait BackupCodeRepository: Send + Sync {
    async fn save_batch(&self, codes: &[BackupCode]) -> AppResult<()>;
    async fn find_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<Vec<BackupCode>>;
    async fn find_by_code_hash(&self, code_hash: &str, tenant_id: &TenantId) -> AppResult<Option<BackupCode>>;
    async fn mark_as_used(&self, code_hash: &str, tenant_id: &TenantId) -> AppResult<()>;
    async fn delete_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<()>;
}
```

### 5. EmailVerificationRepository
```rust
// services/iam-identity/src/shared/domain/repositories/email_verification_repository.rs

#[async_trait]
pub trait EmailVerificationRepository: Send + Sync {
    async fn save(&self, verification: &EmailVerification) -> AppResult<()>;
    async fn find_by_token(&self, token: &str, tenant_id: &TenantId) -> AppResult<Option<EmailVerification>>;
    async fn find_by_email(&self, email: &str, tenant_id: &TenantId) -> AppResult<Option<EmailVerification>>;
    async fn mark_as_verified(&self, token: &str, tenant_id: &TenantId) -> AppResult<()>;
    async fn cleanup_expired(&self, tenant_id: &TenantId) -> AppResult<u64>;
}
```

### 6. PhoneVerificationRepository
```rust
// services/iam-identity/src/shared/domain/repositories/phone_verification_repository.rs

#[async_trait]
pub trait PhoneVerificationRepository: Send + Sync {
    async fn save(&self, verification: &PhoneVerification) -> AppResult<()>;
    async fn find_by_phone(&self, phone: &str, tenant_id: &TenantId) -> AppResult<Option<PhoneVerification>>;
    async fn verify_code(&self, phone: &str, code: &str, tenant_id: &TenantId) -> AppResult<bool>;
    async fn cleanup_expired(&self, tenant_id: &TenantId) -> AppResult<u64>;
}
```

## 实体更新

所有实体需要添加 `tenant_id` 字段：

```rust
pub struct SomeEntity {
    pub id: SomeId,
    pub tenant_id: TenantId,  // 添加这个
    // ... 其他字段
}
```

## 更新步骤

1. 更新实体添加 `tenant_id` 字段
2. 更新 Repository trait 添加 `tenant_id` 参数
3. 更新 Repository 实现的所有 SQL 查询
4. 更新 Command/Query 添加 `tenant_id` 字段
5. 更新 Handler 传递 `tenant_id`
6. 更新 gRPC 服务提取 `tenant_id`

## 测试验证

```bash
# 编译检查
cargo check -p iam-identity

# 运行测试
cargo test -p iam-identity

# 运行租户隔离测试
cargo test -p iam-identity tenant_isolation
```
