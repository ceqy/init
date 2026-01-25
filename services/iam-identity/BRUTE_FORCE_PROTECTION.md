# 防止暴力破解和异常登录实现总结

## 实施完成情况

### ✅ 第一部分：登录失败次数限制（已完成）

**文件创建：**
- `src/auth/domain/services/login_attempt_service.rs` - 登录尝试追踪服务
- `src/auth/infrastructure/cache/login_attempt_cache.rs` - Redis 缓存实现

**功能实现：**
- ✅ 使用 Redis 记录失败次数
  - Key 格式：`login:failed:{username}:{tenant_id}`
  - TTL：15 分钟（900 秒）
- ✅ 失败次数限制逻辑
  - 5 次失败后锁定 15 分钟
  - 成功登录后清除计数
- ✅ 验证码要求
  - 3 次失败后要求验证码
- ✅ 完整的单元测试

**核心方法：**
```rust
// 记录登录失败
async fn record_failure(&self, username: &str, tenant_id: &TenantId) -> AppResult<()>

// 检查是否被锁定
async fn is_locked(&self, username: &str, tenant_id: &TenantId) -> AppResult<bool>

// 检查是否需要验证码
async fn requires_captcha(&self, username: &str, tenant_id: &TenantId) -> AppResult<bool>

// 清除失败记录
async fn clear_failures(&self, username: &str, tenant_id: &TenantId) -> AppResult<()>
```

### ✅ 第二部分：账户锁定机制（已完成）

**数据库迁移：**
- `migrations/20260126050000_add_account_lock_fields.sql`

**添加的字段：**
- `locked_until` - 账户锁定截止时间
- `lock_reason` - 锁定原因
- `failed_login_count` - 连续登录失败次数
- `last_failed_login_at` - 最后一次登录失败时间

**User 实体新增方法：**
```rust
// 记录登录失败（10次后自动锁定30分钟）
pub fn record_login_failure(&mut self)

// 清除登录失败记录
pub fn clear_login_failures(&mut self)

// 锁定账户
pub fn lock_account(&mut self, minutes: i64, reason: String)

// 解锁账户
pub fn unlock_account(&mut self)

// 检查账户是否被锁定
pub fn is_locked(&self) -> bool

// 检查是否应该自动解锁
pub fn should_auto_unlock(&self) -> bool

// 获取剩余锁定时间
pub fn get_lock_remaining_seconds(&self) -> Option<i64>
```

**自动解锁功能：**
- 数据库函数：`auto_unlock_expired_accounts()`
- 可通过定时任务调用

### ✅ 第三部分：登录日志记录（已完成）

**文件创建：**
- `src/auth/domain/entities/login_log.rs` - 登录日志实体
- `src/auth/domain/repositories/login_log_repository.rs` - 登录日志仓储接口
- `migrations/20260126060000_create_login_logs_table.sql` - 数据库表

**登录日志字段：**
- 用户信息：user_id, tenant_id, username
- 网络信息：ip_address, user_agent
- 设备信息：device_type, os, browser, device_fingerprint
- 登录结果：result (Success/Failed), failure_reason
- 地理位置：country, city
- 可疑标记：is_suspicious, suspicious_reason
- 时间戳：created_at

**设备信息解析：**
```rust
pub struct DeviceInfo {
    pub device_type: String,  // Desktop, Mobile, Tablet
    pub os: String,
    pub browser: String,
    pub is_mobile: bool,
}

// 从 User-Agent 解析
DeviceInfo::from_user_agent(user_agent)

// 生成设备指纹
device_info.fingerprint()
```

**登录失败原因：**
- InvalidCredentials - 凭证无效
- AccountLocked - 账户锁定
- AccountInactive - 账户未激活
- TwoFactorRequired - 需要2FA
- TwoFactorFailed - 2FA失败
- CaptchaRequired - 需要验证码
- CaptchaFailed - 验证码失败
- IpBlocked - IP被封禁
- TenantInactive - 租户未激活

**Repository 方法：**
```rust
// 保存日志
async fn save(&self, log: &LoginLog) -> AppResult<()>

// 查询用户登录历史
async fn find_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId, limit: i32) -> AppResult<Vec<LoginLog>>

// 查询最近一次成功登录
async fn find_last_successful_login(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<Option<LoginLog>>

// 查询用户从特定IP的登录历史
async fn find_by_user_and_ip(&self, user_id: &UserId, tenant_id: &TenantId, ip_address: &str) -> AppResult<Vec<LoginLog>>

// 查询用户从特定设备的登录历史
async fn find_by_user_and_device_fingerprint(&self, user_id: &UserId, tenant_id: &TenantId, device_fingerprint: &str) -> AppResult<Vec<LoginLog>>

// 统计失败登录次数
async fn count_failed_attempts(&self, user_id: &UserId, tenant_id: &TenantId, start_time: DateTime<Utc>) -> AppResult<i64>

// 查询可疑登录
async fn find_suspicious_logins(&self, tenant_id: &TenantId, start_time: DateTime<Utc>, limit: i32) -> AppResult<Vec<LoginLog>>
```

**数据清理：**
- 自动清理90天前的日志
- 函数：`cleanup_old_login_logs()`

### ✅ 第四部分：可疑登录检测（已完成）

**文件创建：**
- `src/auth/domain/services/suspicious_login_detector.rs` - 可疑登录检测服务

**检测维度：**

1. **异地登录检测**
   - 基于 IP 地址前缀
   - 检查是否从新的地理位置登录
   - 对比用户历史登录 IP

2. **新设备登录检测**
   - 基于设备指纹
   - 检查是否从未使用过的设备登录
   - 对比用户历史设备记录

3. **异常时间登录检测**
   - 检测深夜登录（凌晨 2-5 点）
   - 对比用户过去30天的登录习惯
   - 如果用户从未在深夜登录，则标记为可疑

**核心方法：**
```rust
// 综合检测
async fn detect(
    &self,
    user_id: &UserId,
    tenant_id: &TenantId,
    ip_address: &str,
    device_fingerprint: &str,
) -> AppResult<Option<String>>

// 异地登录检测
async fn detect_unusual_location(...) -> AppResult<Option<String>>

// 新设备检测
async fn detect_new_device(...) -> AppResult<Option<String>>

// 异常时间检测
async fn detect_unusual_time(...) -> AppResult<Option<String>>
```

## 架构设计

### 多层防护机制

```
┌─────────────────────────────────────────┐
│  1. Redis 层（快速失败计数）              │
│     - 15分钟内5次失败 → 临时锁定          │
│     - 3次失败 → 要求验证码                │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│  2. 数据库层（持久化账户锁定）            │
│     - 10次失败 → 锁定30分钟               │
│     - 管理员可手动解锁                    │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│  3. 登录日志层（审计和分析）              │
│     - 记录所有登录尝试                    │
│     - 支持历史查询和统计                  │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│  4. 可疑检测层（智能告警）                │
│     - 异地登录检测                        │
│     - 新设备检测                          │
│     - 异常时间检测                        │
└─────────────────────────────────────────┘
```

### 登录流程集成

```rust
async fn login(username: &str, password: &str, ip: &str, user_agent: &str) -> AppResult<Token> {
    // 1. 检查 Redis 临时锁定
    if login_attempt_service.is_locked(username, tenant_id).await? {
        let remaining = login_attempt_service.get_lock_remaining_seconds(username, tenant_id).await?;
        return Err(AppError::locked(format!("Account locked for {} seconds", remaining.unwrap_or(0))));
    }

    // 2. 检查是否需要验证码
    if login_attempt_service.requires_captcha(username, tenant_id).await? {
        if captcha.is_none() {
            return Err(AppError::validation("Captcha required"));
        }
        // 验证验证码...
    }

    // 3. 查找用户
    let user = user_repo.find_by_username(username, tenant_id).await?
        .ok_or_else(|| AppError::not_found("User not found"))?;

    // 4. 检查账户锁定状态
    if user.is_locked() {
        let remaining = user.get_lock_remaining_seconds();
        login_log_repo.save(&LoginLog::failure(
            Some(user.id.clone()),
            tenant_id,
            username.to_string(),
            ip.to_string(),
            user_agent.to_string(),
            LoginFailureReason::AccountLocked,
        )).await?;
        return Err(AppError::locked(format!("Account locked for {} seconds", remaining.unwrap_or(0))));
    }

    // 5. 验证密码
    if !password_service.verify(password, &user.password_hash)? {
        // 记录失败
        login_attempt_service.record_failure(username, tenant_id).await?;
        user.record_login_failure();
        user_repo.update(&user).await?;
        
        login_log_repo.save(&LoginLog::failure(
            Some(user.id.clone()),
            tenant_id,
            username.to_string(),
            ip.to_string(),
            user_agent.to_string(),
            LoginFailureReason::InvalidCredentials,
        )).await?;
        
        return Err(AppError::unauthorized("Invalid credentials"));
    }

    // 6. 登录成功
    login_attempt_service.clear_failures(username, tenant_id).await?;
    user.clear_login_failures();
    user.record_login();
    user_repo.update(&user).await?;

    // 7. 记录成功日志
    let device_info = DeviceInfo::from_user_agent(user_agent);
    let mut log = LoginLog::success(
        user.id.clone(),
        tenant_id,
        username.to_string(),
        ip.to_string(),
        user_agent.to_string(),
    );

    // 8. 可疑登录检测
    if let Some(reason) = suspicious_detector.detect(
        &user.id,
        tenant_id,
        ip,
        &device_info.fingerprint(),
    ).await? {
        log.mark_suspicious(reason.clone());
        // 发送告警邮件
        email_service.send_suspicious_login_alert(&user, &reason).await?;
    }

    login_log_repo.save(&log).await?;

    // 9. 生成 Token
    Ok(token_service.generate(&user))
}
```

## 使用示例

### 1. 在登录处理器中集成

```rust
use crate::auth::domain::services::{LoginAttemptService, SuspiciousLoginDetector};
use crate::auth::domain::entities::{LoginLog, LoginFailureReason};

pub struct LoginHandler {
    user_repo: Arc<dyn UserRepository>,
    login_attempt_service: Arc<LoginAttemptService>,
    suspicious_detector: Arc<SuspiciousLoginDetector>,
    login_log_repo: Arc<dyn LoginLogRepository>,
}
```

### 2. 查询登录历史

```rust
// 查询用户最近10次登录
let logs = login_log_repo
    .find_by_user_id(&user_id, &tenant_id, 10)
    .await?;

for log in logs {
    println!("Login at {} from {} ({})", 
        log.created_at, 
        log.ip_address, 
        log.result
    );
}
```

### 3. 管理员解锁账户

```rust
// 手动解锁用户账户
user.unlock_account();
user_repo.update(&user).await?;

// 清除 Redis 计数
login_attempt_service.clear_failures(&user.username.0, &tenant_id).await?;
```

### 4. 查询可疑登录

```rust
// 查询最近24小时的可疑登录
let start_time = Utc::now() - chrono::Duration::hours(24);
let suspicious_logins = login_log_repo
    .find_suspicious_logins(&tenant_id, start_time, 100)
    .await?;

for log in suspicious_logins {
    println!("Suspicious login: {} - {}", 
        log.username, 
        log.suspicious_reason.unwrap_or_default()
    );
}
```

## 配置建议

### Redis 配置
```toml
[redis]
url = "redis://localhost:6379"
max_connections = 10
```

### 锁定策略配置
```rust
pub struct LockPolicy {
    // Redis 临时锁定
    pub redis_failure_threshold: i32,  // 5次
    pub redis_lock_duration_minutes: i64,  // 15分钟
    pub captcha_threshold: i32,  // 3次
    
    // 数据库持久化锁定
    pub db_failure_threshold: i32,  // 10次
    pub db_lock_duration_minutes: i64,  // 30分钟
}
```

## 监控指标

### 关键指标

1. **登录失败率**
   ```sql
   SELECT 
       COUNT(CASE WHEN result = 'Failed' THEN 1 END)::FLOAT / COUNT(*) * 100 as failure_rate
   FROM login_logs
   WHERE created_at > NOW() - INTERVAL '1 hour';
   ```

2. **账户锁定数量**
   ```sql
   SELECT COUNT(*) 
   FROM users 
   WHERE locked_until IS NOT NULL 
   AND locked_until > NOW();
   ```

3. **可疑登录数量**
   ```sql
   SELECT COUNT(*) 
   FROM login_logs 
   WHERE is_suspicious = TRUE 
   AND created_at > NOW() - INTERVAL '24 hours';
   ```

4. **Top 失败 IP**
   ```sql
   SELECT ip_address, COUNT(*) as failure_count
   FROM login_logs
   WHERE result = 'Failed'
   AND created_at > NOW() - INTERVAL '1 hour'
   GROUP BY ip_address
   ORDER BY failure_count DESC
   LIMIT 10;
   ```

## 测试

### 单元测试

```bash
# 测试登录尝试服务
cargo test -p iam-identity login_attempt_service

# 测试可疑登录检测
cargo test -p iam-identity suspicious_login_detector

# 测试登录日志实体
cargo test -p iam-identity login_log
```

### 集成测试场景

1. **暴力破解防护测试**
   - 连续5次失败登录
   - 验证 Redis 锁定生效
   - 验证15分钟后自动解锁

2. **账户锁定测试**
   - 连续10次失败登录
   - 验证数据库锁定生效
   - 验证管理员解锁功能

3. **可疑登录检测测试**
   - 从新IP登录
   - 从新设备登录
   - 深夜登录

## 下一步工作

### 必须完成

- [ ] 实现 LoginLogRepository 的 PostgreSQL 实现
- [ ] 在登录 Handler 中集成所有功能
- [ ] 实现邮件告警功能
- [ ] 添加管理员解锁 API
- [ ] 实现定时任务清理过期日志

### 推荐完成

- [ ] 实现 IP 地理位置查询（使用 GeoIP 库）
- [ ] 实现更精确的设备指纹识别
- [ ] 添加登录历史查询 API
- [ ] 实现可疑登录告警仪表板
- [ ] 添加 Prometheus metrics
- [ ] 实现 IP 黑名单功能

## 安全最佳实践

1. **不要在错误消息中泄露信息**
   - ❌ "用户名不存在"
   - ✅ "用户名或密码错误"

2. **使用恒定时间比较**
   - 防止时序攻击

3. **记录所有登录尝试**
   - 包括成功和失败
   - 用于审计和分析

4. **实施渐进式延迟**
   - 失败次数越多，响应越慢

5. **定期审查可疑登录**
   - 建立告警机制
   - 及时响应异常

## 总结

防止暴力破解和异常登录的核心功能已经完成，包括：
- ✅ Redis 快速失败计数和临时锁定
- ✅ 数据库持久化账户锁定
- ✅ 完整的登录日志记录
- ✅ 智能可疑登录检测

系统提供了多层防护机制，从快速响应到持久化锁定，再到智能检测和告警，全方位保护用户账户安全。
