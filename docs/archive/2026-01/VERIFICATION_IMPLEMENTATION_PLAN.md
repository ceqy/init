# 验证流程实现计划

## 已完成

### 1. 数据库迁移
- ✅ `20260126070000_create_email_verifications_table.sql` - 邮箱验证表（已存在）
- ✅ `20260126070001_create_phone_verifications_table.sql` - 手机验证表（新创建）

### 2. 领域层
- ✅ `EmailVerification` 实体（已存在）
- ✅ `PhoneVerification` 实体（已存在）
- ✅ `EmailVerificationRepository` 接口（已存在）
- ✅ `PhoneVerificationRepository` 接口（已存在）

### 3. 基础设施层
- ✅ `PostgresEmailVerificationRepository` - PostgreSQL 实现（新创建）
- ⏳ `PostgresPhoneVerificationRepository` - PostgreSQL 实现（待创建）

## 待完成组件

### 1. 基础设施层

#### PostgresPhoneVerificationRepository
```rust
// services/iam-identity/src/shared/infrastructure/persistence/postgres_phone_verification_repository.rs

// 实现与 PostgresEmailVerificationRepository 类似
// 主要方法：
// - save()
// - find_by_id()
// - find_latest_by_user_id()
// - find_latest_by_phone()
// - update()
// - delete_expired()
// - count_today_by_user()
```

#### 短信服务适配器
```rust
// crates/adapters/sms/Cargo.toml
// crates/adapters/sms/src/lib.rs
// crates/adapters/sms/src/client.rs

// 支持阿里云短信服务
pub trait SmsSender: Send + Sync {
    async fn send_verification_code(
        &self,
        phone: &str,
        code: &str,
    ) -> AppResult<()>;
}

pub struct AliyunSmsClient {
    access_key_id: String,
    access_key_secret: String,
    sign_name: String,
    template_code: String,
}
```

### 2. 领域服务

#### EmailVerificationService
```rust
// services/iam-identity/src/shared/domain/services/email_verification_service.rs

pub struct EmailVerificationService {
    email_verification_repo: Arc<dyn EmailVerificationRepository>,
    user_repo: Arc<dyn UserRepository>,
    email_sender: Arc<dyn EmailSender>,
}

impl EmailVerificationService {
    // 发送验证码
    pub async fn send_verification_code(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<()> {
        // 1. 检查今天发送次数（最多5次）
        // 2. 查找用户
        // 3. 创建验证记录
        // 4. 发送邮件
    }

    // 验证验证码
    pub async fn verify_code(
        &self,
        user_id: &UserId,
        code: &str,
        tenant_id: &TenantId,
    ) -> AppResult<()> {
        // 1. 查找最新的验证记录
        // 2. 验证验证码
        // 3. 更新用户的 email_verified 字段
    }
}
```

#### PhoneVerificationService
```rust
// services/iam-identity/src/shared/domain/services/phone_verification_service.rs

pub struct PhoneVerificationService {
    phone_verification_repo: Arc<dyn PhoneVerificationRepository>,
    user_repo: Arc<dyn UserRepository>,
    sms_sender: Arc<dyn SmsSender>,
}

impl PhoneVerificationService {
    // 发送验证码
    pub async fn send_verification_code(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> AppResult<()> {
        // 1. 检查今天发送次数（最多5次）
        // 2. 查找用户
        // 3. 创建验证记录
        // 4. 发送短信
    }

    // 验证验证码
    pub async fn verify_code(
        &self,
        user_id: &UserId,
        code: &str,
        tenant_id: &TenantId,
    ) -> AppResult<()> {
        // 1. 查找最新的验证记录
        // 2. 验证验证码
        // 3. 更新用户的 phone_verified 字段
    }
}
```

### 3. 应用层

#### 命令
```rust
// services/iam-identity/src/shared/application/commands/send_email_verification_command.rs
pub struct SendEmailVerificationCommand {
    pub user_id: String,
    pub tenant_id: String,
}

// services/iam-identity/src/shared/application/commands/verify_email_command.rs
pub struct VerifyEmailCommand {
    pub user_id: String,
    pub code: String,
    pub tenant_id: String,
}

// services/iam-identity/src/shared/application/commands/send_phone_verification_command.rs
pub struct SendPhoneVerificationCommand {
    pub user_id: String,
    pub tenant_id: String,
}

// services/iam-identity/src/shared/application/commands/verify_phone_command.rs
pub struct VerifyPhoneCommand {
    pub user_id: String,
    pub code: String,
    pub tenant_id: String,
}
```

#### 命令处理器
```rust
// services/iam-identity/src/shared/application/handlers/send_email_verification_handler.rs
pub struct SendEmailVerificationHandler {
    email_verification_service: Arc<EmailVerificationService>,
}

// services/iam-identity/src/shared/application/handlers/verify_email_handler.rs
pub struct VerifyEmailHandler {
    email_verification_service: Arc<EmailVerificationService>,
}

// services/iam-identity/src/shared/application/handlers/send_phone_verification_handler.rs
pub struct SendPhoneVerificationHandler {
    phone_verification_service: Arc<PhoneVerificationService>,
}

// services/iam-identity/src/shared/application/handlers/verify_phone_handler.rs
pub struct VerifyPhoneHandler {
    phone_verification_service: Arc<PhoneVerificationService>,
}
```

### 4. API 层（gRPC）

#### Proto 定义
```protobuf
// proto/iam/user.proto

service UserService {
  // 发送邮箱验证码
  rpc SendEmailVerification(SendEmailVerificationRequest) 
    returns (SendEmailVerificationResponse);

  // 验证邮箱
  rpc VerifyEmail(VerifyEmailRequest) 
    returns (VerifyEmailResponse);

  // 发送手机验证码
  rpc SendPhoneVerification(SendPhoneVerificationRequest) 
    returns (SendPhoneVerificationResponse);

  // 验证手机
  rpc VerifyPhone(VerifyPhoneRequest) 
    returns (VerifyPhoneResponse);
}

message SendEmailVerificationRequest {
  string user_id = 1;
}

message SendEmailVerificationResponse {
  bool success = 1;
  string message = 2;
  int32 expires_in_seconds = 3;
}

message VerifyEmailRequest {
  string user_id = 1;
  string code = 2;
}

message VerifyEmailResponse {
  bool success = 1;
  string message = 2;
}

// 类似的 Phone 消息定义...
```

### 5. 邮件模板

#### 邮箱验证模板
```html
<!-- crates/adapters/email/templates/email_verification.html -->
<!DOCTYPE html>
<html>
<head>
    <title>邮箱验证</title>
</head>
<body>
    <h1>邮箱验证</h1>
    <p>您好，{{ user_name }}！</p>
    <p>您的邮箱验证码是：</p>
    <h2 style="color: #3498db;">{{ code }}</h2>
    <p>验证码将在 {{ expires_in_minutes }} 分钟后失效。</p>
    <p>如果您没有请求验证，请忽略此邮件。</p>
</body>
</html>
```

```text
<!-- crates/adapters/email/templates/email_verification.txt -->
邮箱验证

您好，{{ user_name }}！

您的邮箱验证码是：{{ code }}

验证码将在 {{ expires_in_minutes }} 分钟后失效。

如果您没有请求验证，请忽略此邮件。
```

## 安全特性

### 1. 防滥用
- 每用户每天最多发送 5 次验证码
- 验证码有效期：邮箱 10 分钟，手机 5 分钟
- 验证码一次性使用

### 2. 验证码安全
- 6 位数字验证码
- 随机生成
- 不可预测

### 3. 租户隔离
- 所有操作强制租户隔离
- RLS 策略保护

## 配置参数

```toml
[verification]
# 邮箱验证
email_code_expires_minutes = 10
email_max_daily_sends = 5

# 手机验证
phone_code_expires_minutes = 5
phone_max_daily_sends = 5

[sms]
# 阿里云短信配置
provider = "aliyun"
access_key_id = "your_access_key_id"
access_key_secret = "your_access_key_secret"
sign_name = "ERP"
template_code = "SMS_123456789"
```

## 测试计划

### 单元测试
- ✅ EmailVerification 实体测试（已存在）
- ✅ PhoneVerification 实体测试（已存在）
- ✅ PostgresEmailVerificationRepository 测试（已创建）
- ⏳ PostgresPhoneVerificationRepository 测试
- ⏳ EmailVerificationService 测试
- ⏳ PhoneVerificationService 测试
- ⏳ 命令处理器测试

### 集成测试
- ⏳ 完整的邮箱验证流程测试
- ⏳ 完整的手机验证流程测试
- ⏳ 防滥用测试
- ⏳ 租户隔离测试

## 实施步骤

### 第一阶段：基础设施层（1-2天）
1. ✅ 创建数据库迁移
2. ✅ 实现 PostgresEmailVerificationRepository
3. ⏳ 实现 PostgresPhoneVerificationRepository
4. ⏳ 创建短信服务适配器

### 第二阶段：领域服务（1天）
1. ⏳ 实现 EmailVerificationService
2. ⏳ 实现 PhoneVerificationService
3. ⏳ 编写单元测试

### 第三阶段：应用层（1天）
1. ⏳ 创建命令和命令处理器
2. ⏳ 编写单元测试

### 第四阶段：API 层（1天）
1. ⏳ 更新 Proto 定义
2. ⏳ 实现 gRPC 方法
3. ⏳ 编写集成测试

### 第五阶段：邮件模板（0.5天）
1. ⏳ 创建邮箱验证邮件模板
2. ⏳ 测试邮件发送

## 预期成果

完成后将提供：
- 完整的邮箱验证流程
- 完整的手机验证流程
- 防滥用机制
- 租户隔离
- 完整的测试覆盖
- API 文档

## 注意事项

1. **短信服务**：需要申请阿里云短信服务并配置模板
2. **邮件服务**：需要配置 SMTP 服务器
3. **测试环境**：建议使用 Mock 短信服务进行测试
4. **生产环境**：需要监控验证码发送量和成功率
