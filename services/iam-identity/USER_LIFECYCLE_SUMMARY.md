# 用户生命周期管理实现总结

## 实施完成情况

### ✅ 第一部分：邮箱验证（已完成）

**文件创建：**
- `src/shared/domain/entities/email_verification.rs` - 邮箱验证实体
- `src/shared/domain/repositories/email_verification_repository.rs` - 邮箱验证仓储接口
- `migrations/20260126070000_create_email_verifications_table.sql` - 数据库表

**功能实现：**
- ✅ EmailVerification 实体
  - 6位数字验证码生成
  - 10分钟过期时间
  - 状态管理（Pending/Verified/Expired）
  - 验证码验证逻辑
- ✅ User 实体新增字段
  - email_verified: 邮箱是否已验证
  - email_verified_at: 邮箱验证时间
  - mark_email_verified(): 标记邮箱已验证
  - is_email_verified(): 检查验证状态
- ✅ 数据库表和索引
  - email_verifications 表
  - 租户隔离（RLS）
  - 自动清理过期记录函数
- ✅ 完整的单元测试

### ✅ 第二部分：手机验证（已完成）

**文件创建：**
- `src/shared/domain/entities/phone_verification.rs` - 手机验证实体
- `src/shared/domain/repositories/phone_verification_repository.rs` - 手机验证仓储接口

**功能实现：**
- ✅ PhoneVerification 实体
  - 6位数字验证码生成
  - 5分钟过期时间（比邮箱更短）
  - 状态管理（Pending/Verified/Expired）
  - 验证码验证逻辑
- ✅ 完整的单元测试

**待创建：**
- [ ] 数据库迁移文件
- [ ] User 实体添加手机验证字段
- [ ] 短信服务集成（阿里云/腾讯云）

### 🔄 第三部分：社交账号绑定（待实现）

**需要创建的文件：**
1. `src/shared/domain/entities/social_account.rs` - 社交账号实体
2. `src/shared/domain/repositories/social_account_repository.rs` - 社交账号仓储
3. `migrations/20260126080000_create_social_accounts_table.sql` - 数据库表
4. `src/oauth/providers/` - OAuth 提供商实现
   - github_provider.rs
   - google_provider.rs
   - wechat_provider.rs

**功能设计：**

#### SocialAccount 实体
```rust
pub struct SocialAccount {
    pub id: SocialAccountId,
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub provider: SocialProvider,  // GitHub, Google, WeChat
    pub provider_user_id: String,
    pub provider_username: Option<String>,
    pub provider_email: Option<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub linked_at: DateTime<Utc>,
}

pub enum SocialProvider {
    GitHub,
    Google,
    WeChat,
    // 可扩展其他提供商
}
```

#### 数据库表设计
```sql
CREATE TABLE social_accounts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    provider_username VARCHAR(255),
    provider_email VARCHAR(255),
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_at TIMESTAMPTZ,
    linked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(provider, provider_user_id, tenant_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

#### OAuth 流程
1. **授权请求**
   ```rust
   // 生成授权 URL
   let auth_url = oauth_provider.get_authorization_url(redirect_uri, state);
   // 重定向用户到提供商
   ```

2. **回调处理**
   ```rust
   // 接收授权码
   let code = request.query("code");
   // 交换访问令牌
   let token = oauth_provider.exchange_code(code).await?;
   // 获取用户信息
   let user_info = oauth_provider.get_user_info(&token).await?;
   ```

3. **账号绑定**
   ```rust
   // 检查是否已绑定
   if let Some(existing) = social_account_repo
       .find_by_provider_user_id(provider, provider_user_id, tenant_id)
       .await? {
       // 已绑定，直接登录
       return Ok(existing.user_id);
   }
   
   // 创建新绑定
   let social_account = SocialAccount::new(
       user_id,
       tenant_id,
       provider,
       provider_user_id,
       access_token,
   );
   social_account_repo.save(&social_account).await?;
   ```

4. **社交登录**
   ```rust
   // 通过社交账号登录
   let social_account = social_account_repo
       .find_by_provider_user_id(provider, provider_user_id, tenant_id)
       .await?
       .ok_or(AppError::not_found("Social account not found"))?;
   
   // 获取用户
   let user = user_repo
       .find_by_id(&social_account.user_id, tenant_id)
       .await?
       .ok_or(AppError::not_found("User not found"))?;
   
   // 生成 Token
   let token = token_service.generate(&user);
   ```

## 完整的用户生命周期流程

### 1. 用户注册
```rust
// 1. 创建用户
let user = User::new(username, email, password_hash, tenant_id);
user_repo.save(&user).await?;

// 2. 发送邮箱验证
let verification = EmailVerification::new(user.id, tenant_id, email);
email_verification_repo.save(&verification).await?;
email_service.send_verification_code(&email, &verification.code).await?;

// 3. 用户状态为 PendingVerification
```

### 2. 邮箱验证
```rust
// 1. 查找验证记录
let mut verification = email_verification_repo
    .find_latest_by_user_id(&user_id, &tenant_id)
    .await?
    .ok_or(AppError::not_found("Verification not found"))?;

// 2. 验证验证码
verification.verify(&code)?;
email_verification_repo.update(&verification).await?;

// 3. 更新用户状态
user.mark_email_verified();
user.activate();
user_repo.update(&user).await?;
```

### 3. 手机验证
```rust
// 1. 发送短信验证码
let verification = PhoneVerification::new(user.id, tenant_id, phone);
phone_verification_repo.save(&verification).await?;
sms_service.send_verification_code(&phone, &verification.code).await?;

// 2. 验证验证码
let mut verification = phone_verification_repo
    .find_latest_by_user_id(&user_id, &tenant_id)
    .await?
    .ok_or(AppError::not_found("Verification not found"))?;

verification.verify(&code)?;
phone_verification_repo.update(&verification).await?;

// 3. 更新用户手机号
user.phone = Some(phone);
user.phone_verified = true;
user_repo.update(&user).await?;
```

### 4. 社交账号绑定
```rust
// 1. OAuth 授权
let auth_url = github_provider.get_authorization_url(redirect_uri, state);
// 重定向用户...

// 2. 回调处理
let token = github_provider.exchange_code(code).await?;
let user_info = github_provider.get_user_info(&token).await?;

// 3. 绑定账号
let social_account = SocialAccount::new(
    user.id,
    tenant_id,
    SocialProvider::GitHub,
    user_info.id,
    token.access_token,
);
social_account_repo.save(&social_account).await?;
```

### 5. 社交登录
```rust
// 1. OAuth 授权和回调（同上）

// 2. 查找社交账号
let social_account = social_account_repo
    .find_by_provider_user_id(provider, provider_user_id, tenant_id)
    .await?;

// 3. 如果不存在，自动创建用户
let user = if let Some(account) = social_account {
    user_repo.find_by_id(&account.user_id, tenant_id).await?
} else {
    // 自动创建用户
    let user = User::new(
        Username::new(&user_info.username)?,
        Email::new(&user_info.email)?,
        HashedPassword::new("".to_string()), // 社交登录无密码
        tenant_id,
    );
    user.mark_email_verified(); // 信任社交提供商的邮箱
    user.activate();
    user_repo.save(&user).await?;
    
    // 创建社交账号绑定
    let social_account = SocialAccount::new(
        user.id,
        tenant_id,
        provider,
        user_info.id,
        token.access_token,
    );
    social_account_repo.save(&social_account).await?;
    
    user
};

// 4. 生成 Token
let token = token_service.generate(&user);
```

## API 设计

### 邮箱验证 API

```protobuf
service UserService {
    // 发送邮箱验证码
    rpc SendEmailVerification(SendEmailVerificationRequest) returns (SendEmailVerificationResponse);
    
    // 验证邮箱
    rpc VerifyEmail(VerifyEmailRequest) returns (VerifyEmailResponse);
}

message SendEmailVerificationRequest {
    string user_id = 1;
}

message SendEmailVerificationResponse {
    string verification_id = 1;
    int64 expires_in_seconds = 2;
}

message VerifyEmailRequest {
    string user_id = 1;
    string code = 2;
}

message VerifyEmailResponse {
    bool success = 1;
}
```

### 手机验证 API

```protobuf
service UserService {
    // 发送手机验证码
    rpc SendPhoneVerification(SendPhoneVerificationRequest) returns (SendPhoneVerificationResponse);
    
    // 验证手机
    rpc VerifyPhone(VerifyPhoneRequest) returns (VerifyPhoneResponse);
}

message SendPhoneVerificationRequest {
    string user_id = 1;
    string phone = 2;
}

message SendPhoneVerificationResponse {
    string verification_id = 1;
    int64 expires_in_seconds = 2;
}

message VerifyPhoneRequest {
    string user_id = 1;
    string code = 2;
}

message VerifyPhoneResponse {
    bool success = 1;
}
```

### 社交账号 API

```protobuf
service UserService {
    // 获取 OAuth 授权 URL
    rpc GetOAuthAuthorizationUrl(GetOAuthAuthorizationUrlRequest) returns (GetOAuthAuthorizationUrlResponse);
    
    // OAuth 回调处理
    rpc HandleOAuthCallback(HandleOAuthCallbackRequest) returns (HandleOAuthCallbackResponse);
    
    // 绑定社交账号
    rpc LinkSocialAccount(LinkSocialAccountRequest) returns (LinkSocialAccountResponse);
    
    // 解绑社交账号
    rpc UnlinkSocialAccount(UnlinkSocialAccountRequest) returns (UnlinkSocialAccountResponse);
    
    // 列出社交账号
    rpc ListSocialAccounts(ListSocialAccountsRequest) returns (ListSocialAccountsResponse);
}

message GetOAuthAuthorizationUrlRequest {
    string provider = 1;  // github, google, wechat
    string redirect_uri = 2;
}

message GetOAuthAuthorizationUrlResponse {
    string authorization_url = 1;
    string state = 2;
}

message HandleOAuthCallbackRequest {
    string provider = 1;
    string code = 2;
    string state = 3;
}

message HandleOAuthCallbackResponse {
    string access_token = 1;
    string refresh_token = 2;
    UserInfo user_info = 3;
}

message LinkSocialAccountRequest {
    string user_id = 1;
    string provider = 2;
    string access_token = 3;
}

message LinkSocialAccountResponse {
    string social_account_id = 1;
}

message UnlinkSocialAccountRequest {
    string user_id = 1;
    string social_account_id = 2;
}

message UnlinkSocialAccountResponse {
    bool success = 1;
}

message ListSocialAccountsRequest {
    string user_id = 1;
}

message ListSocialAccountsResponse {
    repeated SocialAccount accounts = 1;
}

message SocialAccount {
    string id = 1;
    string provider = 2;
    string provider_username = 3;
    string linked_at = 4;
}
```

## 安全考虑

### 验证码安全
1. **限制发送频率**
   - 同一用户每天最多发送10次
   - 同一IP每小时最多发送20次
   - 使用 Redis 计数器实现

2. **验证码复杂度**
   - 邮箱：6位数字
   - 手机：6位数字
   - 使用加密随机数生成器

3. **过期时间**
   - 邮箱验证码：10分钟
   - 手机验证码：5分钟（更短，因为短信成本高）

4. **防止暴力破解**
   - 验证失败3次后要求等待
   - 记录验证尝试日志

### OAuth 安全
1. **State 参数**
   - 防止 CSRF 攻击
   - 使用随机生成的 state
   - 验证回调时的 state

2. **Token 存储**
   - access_token 加密存储
   - refresh_token 加密存储
   - 定期刷新 token

3. **权限范围**
   - 只请求必要的权限
   - 用户可见权限列表

## 监控指标

1. **验证码发送量**
   - 每日邮箱验证码发送量
   - 每日短信验证码发送量
   - 发送成功率

2. **验证成功率**
   - 邮箱验证成功率
   - 手机验证成功率
   - 平均验证时间

3. **社交账号**
   - 各提供商绑定数量
   - 社交登录占比
   - OAuth 成功率

## 下一步工作

### 必须完成
- [ ] 创建手机验证数据库迁移
- [ ] 实现邮件发送服务集成
- [ ] 实现短信发送服务集成
- [ ] 创建社交账号实体和仓储
- [ ] 实现 OAuth 提供商（GitHub/Google/WeChat）
- [ ] 创建社交账号数据库迁移
- [ ] 实现验证码发送频率限制
- [ ] 添加验证相关的 gRPC API

### 推荐完成
- [ ] 实现邮件模板系统
- [ ] 实现短信模板系统
- [ ] 添加验证码发送日志
- [ ] 实现验证码发送统计
- [ ] 添加社交账号管理界面
- [ ] 实现 Token 自动刷新
- [ ] 添加更多 OAuth 提供商

## 总结

用户生命周期管理的核心功能已经完成：
- ✅ 邮箱验证实体和仓储
- ✅ 手机验证实体和仓储
- ✅ User 实体支持邮箱验证
- ✅ 完整的单元测试

下一步需要完成社交账号绑定功能和各种服务集成（邮件、短信、OAuth）。
