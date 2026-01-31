# 安全最佳实践

## 概述

ERP 实施多层安全防护，确保系统和数据的安全性。

## 认证安全

### 密码安全

#### 密码哈希

使用 Argon2id 算法进行密码哈希：

```rust
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};

// 哈希密码
let salt = SaltString::generate(&mut OsRng);
let argon2 = Argon2::default();
let password_hash = argon2.hash_password(password.as_bytes(), &salt)?
    .to_string();

// 验证密码
let parsed_hash = PasswordHash::new(&password_hash)?;
argon2.verify_password(password.as_bytes(), &parsed_hash)?;
```

#### 密码强度要求

- 最小长度：8 字符
- 必须包含：大写字母、小写字母、数字
- 推荐包含：特殊字符 (!@#$%^&*)
- 禁止：常见密码、用户名、邮箱

#### 密码策略

```rust
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
    pub max_age_days: Option<u32>,
    pub history_count: usize,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: false,
            max_age_days: Some(90),
            history_count: 5,
        }
    }
}
```

### Token 安全

#### JWT 配置

```toml
[jwt]
secret = "your-super-secret-key-change-in-production"
algorithm = "HS256"
access_token_expires_in = 3600      # 1 小时
refresh_token_expires_in = 604800   # 7 天
```

#### Token 最佳实践

1. **使用强密钥**：至少 256 位随机密钥
2. **短期有效**：Access Token 1 小时，Refresh Token 7 天
3. **安全存储**：客户端使用 HttpOnly Cookie 或安全存储
4. **Token 撤销**：支持主动撤销 Token
5. **Token 轮换**：刷新时生成新的 Refresh Token

### 双因子认证（2FA）

#### TOTP（推荐）

基于时间的一次性密码：

```rust
use totp_rs::{Algorithm, TOTP};

// 生成 Secret
let totp = TOTP::new(
    Algorithm::SHA1,
    6,
    1,
    30,
    secret.as_bytes().to_vec(),
)?;

// 生成 QR 码
let qr_code_url = totp.get_qr_base64()?;

// 验证代码
let is_valid = totp.check_current(&code)?;
```

#### 备份码

生成 10 个一次性备份码：

```rust
use rand::Rng;

fn generate_backup_codes(count: usize) -> Vec<String> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| {
            format!("{:04}-{:04}-{:04}",
                rng.gen_range(0..10000),
                rng.gen_range(0..10000),
                rng.gen_range(0..10000))
        })
        .collect()
}
```

#### WebAuthn（FIDO2）

无密码认证，支持硬件密钥和生物识别：

- YubiKey
- Touch ID / Face ID
- Windows Hello
- Android 指纹

## 授权安全

### 基于角色的访问控制（RBAC）

```rust
pub struct Permission {
    pub resource: String,
    pub action: String,
}

pub struct Role {
    pub id: String,
    pub name: String,
    pub permissions: Vec<Permission>,
}

// 检查权限
fn has_permission(user: &User, resource: &str, action: &str) -> bool {
    user.roles.iter().any(|role| {
        role.permissions.iter().any(|perm| {
            perm.resource == resource && perm.action == action
        })
    })
}
```

### 租户隔离

#### Row-Level Security (RLS)

PostgreSQL 行级安全策略：

```sql
-- 启用 RLS
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

-- 创建策略
CREATE POLICY tenant_isolation ON users
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);

-- 设置租户上下文
SET app.current_tenant_id = '550e8400-e29b-41d4-a716-446655440000';
```

#### 应用层隔离

```rust
// 所有查询必须包含 tenant_id
async fn find_user(&self, user_id: &UserId, tenant_id: &TenantId) -> Result<User> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1 AND tenant_id = $2",
        user_id.0,
        tenant_id.0
    )
    .fetch_one(&self.pool)
    .await
}
```

## 账户保护

### 暴力破解防护

#### 登录失败锁定

```rust
pub struct LoginAttemptService {
    max_attempts: u32,
    lockout_duration: Duration,
}

impl LoginAttemptService {
    pub async fn check_and_record(&self, username: &str) -> Result<()> {
        let attempts = self.get_failed_attempts(username).await?;
        
        if attempts >= self.max_attempts {
            let lockout_until = Utc::now() + self.lockout_duration;
            return Err(AppError::AccountLocked { until: lockout_until });
        }
        
        Ok(())
    }
}
```

配置：
- 最大失败次数：5 次
- 锁定时长：15 分钟
- 锁定后：需要管理员解锁或等待超时

#### 速率限制

使用 Redis 实现：

```rust
use redis::AsyncCommands;

pub async fn check_rate_limit(
    redis: &mut redis::aio::Connection,
    key: &str,
    max_requests: u32,
    window_secs: u64,
) -> Result<bool> {
    let count: u32 = redis.incr(key, 1).await?;
    
    if count == 1 {
        redis.expire(key, window_secs as usize).await?;
    }
    
    Ok(count <= max_requests)
}
```

### 可疑登录检测

检测异常登录行为：

```rust
pub struct SuspiciousLoginDetector {
    // 检测规则
}

impl SuspiciousLoginDetector {
    pub fn is_suspicious(&self, login: &LoginAttempt, user: &User) -> bool {
        // 1. 新设备
        let is_new_device = !self.is_known_device(&login.device_info, user);
        
        // 2. 异常 IP
        let is_unusual_ip = !self.is_known_ip(&login.ip_address, user);
        
        // 3. 异常时间
        let is_unusual_time = self.is_unusual_login_time(&login.timestamp, user);
        
        // 4. 异常地理位置
        let is_unusual_location = self.is_unusual_location(&login.ip_address, user);
        
        is_new_device || is_unusual_ip || is_unusual_time || is_unusual_location
    }
}
```

检测到可疑登录时：
1. 发送邮件通知
2. 要求额外验证（2FA）
3. 记录安全日志

## 数据安全

### 敏感数据加密

#### 传输加密

- 所有 API 使用 TLS 1.3
- gRPC 使用 TLS 加密
- 禁用不安全的加密套件

#### 存储加密

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = Nonce::from_slice(b"unique nonce");
        self.cipher.encrypt(nonce, plaintext)
            .map_err(|e| AppError::EncryptionError(e.to_string()))
    }
    
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let nonce = Nonce::from_slice(b"unique nonce");
        self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| AppError::DecryptionError(e.to_string()))
    }
}
```

#### 敏感字段

需要加密的字段：
- 密码（哈希，不可逆）
- 身份证号
- 银行卡号
- 手机号（可选）
- 邮箱（可选）

### SQL 注入防护

使用参数化查询：

```rust
// ✅ 安全：使用参数化查询
sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE username = $1",
    username
)
.fetch_one(&pool)
.await?;

// ❌ 危险：字符串拼接
let query = format!("SELECT * FROM users WHERE username = '{}'", username);
sqlx::query(&query).fetch_one(&pool).await?;
```

### XSS 防护

输出转义：

```rust
use html_escape::encode_text;

// 转义 HTML
let safe_output = encode_text(&user_input);
```

### CSRF 防护

使用 CSRF Token：

```rust
use uuid::Uuid;

pub struct CsrfToken {
    token: String,
    expires_at: DateTime<Utc>,
}

impl CsrfToken {
    pub fn new() -> Self {
        Self {
            token: Uuid::new_v4().to_string(),
            expires_at: Utc::now() + Duration::hours(1),
        }
    }
    
    pub fn verify(&self, token: &str) -> bool {
        self.token == token && Utc::now() < self.expires_at
    }
}
```

## 会话安全

### 会话管理

```rust
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub device_info: String,
    pub ip_address: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
}

impl Session {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    pub fn is_idle(&self, max_idle: Duration) -> bool {
        Utc::now() - self.last_activity_at > max_idle
    }
}
```

### 会话配置

```toml
[session]
max_age = 86400           # 24 小时
max_idle = 3600           # 1 小时无活动
max_sessions_per_user = 5 # 每用户最多 5 个会话
```

### 会话撤销

支持以下场景的会话撤销：
1. 用户主动登出
2. 密码修改
3. 管理员强制登出
4. 检测到可疑活动

## 审计日志

### 安全事件记录

```rust
pub enum SecurityEvent {
    LoginSuccess { user_id: String, ip: String },
    LoginFailed { username: String, ip: String, reason: String },
    PasswordChanged { user_id: String },
    AccountLocked { user_id: String, reason: String },
    PermissionDenied { user_id: String, resource: String, action: String },
    SuspiciousActivity { user_id: String, description: String },
}

pub async fn log_security_event(event: SecurityEvent) {
    // 记录到数据库
    // 发送到 SIEM 系统
    // 触发告警（如需要）
}
```

### 审计日志查询

```sql
-- 查询失败登录
SELECT * FROM login_logs
WHERE success = false
  AND created_at > NOW() - INTERVAL '1 day'
ORDER BY created_at DESC;

-- 查询账户锁定
SELECT * FROM users
WHERE locked_at IS NOT NULL
  AND locked_at > NOW() - INTERVAL '7 days';

-- 查询权限拒绝
SELECT * FROM audit_logs
WHERE event_type = 'PERMISSION_DENIED'
  AND created_at > NOW() - INTERVAL '1 day';
```

## 安全配置检查清单

### 生产环境

- [ ] 使用强 JWT 密钥（至少 256 位）
- [ ] 启用 TLS/HTTPS
- [ ] 配置防火墙规则
- [ ] 启用数据库 RLS
- [ ] 配置速率限制
- [ ] 启用审计日志
- [ ] 配置备份策略
- [ ] 定期安全扫描
- [ ] 更新依赖包
- [ ] 配置监控告警

### 开发环境

- [ ] 不使用生产密钥
- [ ] 使用测试数据
- [ ] 启用详细日志
- [ ] 配置本地 TLS（可选）

## 安全更新

### 依赖更新

```bash
# 检查过期依赖
cargo outdated

# 更新依赖
cargo update

# 安全审计
cargo audit
```

### 漏洞响应

1. 监控安全公告
2. 评估影响范围
3. 测试补丁
4. 部署更新
5. 通知用户

## 相关资源

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [gRPC Security](https://grpc.io/docs/guides/auth/)
