# 多租户指南

## 概述

ERP 采用多租户架构，支持多个组织（租户）共享同一套系统，同时确保数据完全隔离。

## 租户模型

### 租户实体

```rust
pub struct Tenant {
    pub id: TenantId,
    pub code: String,           // 租户代码（唯一）
    pub name: String,           // 租户名称
    pub status: TenantStatus,   // 状态
    pub settings: TenantSettings, // 租户配置
    pub audit_info: AuditInfo,
}

pub enum TenantStatus {
    Active,      // 活跃
    Suspended,   // 暂停
    Inactive,    // 停用
}

pub struct TenantSettings {
    pub max_users: Option<u32>,
    pub features: Vec<String>,
    pub custom_domain: Option<String>,
}
```

### 租户上下文

```rust
pub struct TenantContext {
    pub tenant_id: TenantId,
    pub tenant_code: String,
}

impl TenantContext {
    pub fn new(tenant_id: TenantId, tenant_code: String) -> Self {
        Self { tenant_id, tenant_code }
    }
}
```

## 数据隔离

### 数据库层隔离

#### Row-Level Security (RLS)

PostgreSQL 行级安全策略确保数据隔离：

```sql
-- 启用 RLS
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;

-- 创建隔离策略
CREATE POLICY tenant_isolation_policy ON users
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);

CREATE POLICY tenant_isolation_policy ON sessions
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);

CREATE POLICY tenant_isolation_policy ON orders
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);
```

#### 设置租户上下文

```rust
pub async fn set_tenant_context(
    pool: &PgPool,
    tenant_id: &TenantId,
) -> Result<()> {
    sqlx::query("SET app.current_tenant_id = $1")
        .bind(tenant_id.0)
        .execute(pool)
        .await?;
    Ok(())
}
```

### 应用层隔离

#### Repository 层

所有 Repository 方法必须包含 `tenant_id` 参数：

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,  // 必须参数
    ) -> Result<Option<User>>;
    
    async fn save(
        &self,
        user: &User,
        tenant_id: &TenantId,  // 必须参数
    ) -> Result<()>;
    
    async fn list(
        &self,
        tenant_id: &TenantId,  // 必须参数
        page: u32,
        page_size: u32,
    ) -> Result<Vec<User>>;
}
```

#### 实现示例

```rust
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> Result<Option<User>> {
        sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users
            WHERE id = $1 AND tenant_id = $2
            "#,
            user_id.0,
            tenant_id.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(e.to_string()))
    }
}
```

### API 层隔离

#### gRPC Interceptor

从请求中提取租户信息：

```rust
pub struct TenantInterceptor;

impl Interceptor for TenantInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        // 从 metadata 中提取租户 ID
        let tenant_id = request
            .metadata()
            .get("x-tenant-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing tenant ID"))?;
        
        // 验证租户 ID
        let tenant_id = TenantId::from_str(tenant_id)
            .map_err(|_| Status::invalid_argument("Invalid tenant ID"))?;
        
        // 存储到 request extensions
        request.extensions_mut().insert(tenant_id);
        
        Ok(request)
    }
}
```

#### 使用示例

```rust
pub async fn get_user(
    &self,
    request: Request<GetUserRequest>,
) -> Result<Response<GetUserResponse>, Status> {
    // 从 request 中获取租户 ID
    let tenant_id = request
        .extensions()
        .get::<TenantId>()
        .ok_or_else(|| Status::unauthenticated("Missing tenant context"))?;
    
    let user_id = UserId::from_str(&request.get_ref().user_id)
        .map_err(|_| Status::invalid_argument("Invalid user ID"))?;
    
    // 查询时必须传入 tenant_id
    let user = self.user_repo
        .find_by_id(&user_id, tenant_id)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("User not found"))?;
    
    Ok(Response::new(GetUserResponse {
        user: Some(user.into()),
    }))
}
```

## 租户识别

### 方式 1：子域名

```
tenant1.erp.com
tenant2.erp.com
```

从 Host header 提取租户代码：

```rust
pub fn extract_tenant_from_host(host: &str) -> Option<String> {
    host.split('.')
        .next()
        .filter(|s| !s.is_empty() && *s != "www")
        .map(|s| s.to_string())
}
```

### 方式 2：路径前缀

```
erp.com/tenant1/api/...
erp.com/tenant2/api/...
```

从路径提取租户代码：

```rust
pub fn extract_tenant_from_path(path: &str) -> Option<String> {
    path.split('/')
        .nth(1)
        .map(|s| s.to_string())
}
```

### 方式 3：Header

```
X-Tenant-ID: 550e8400-e29b-41d4-a716-446655440000
X-Tenant-Code: tenant1
```

从 header 提取：

```rust
pub fn extract_tenant_from_header(
    headers: &HeaderMap,
) -> Option<String> {
    headers
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}
```

## 租户管理

### 创建租户

```rust
pub async fn create_tenant(
    &self,
    code: String,
    name: String,
    settings: TenantSettings,
) -> Result<Tenant> {
    // 1. 验证租户代码唯一性
    if self.tenant_repo.exists_by_code(&code).await? {
        return Err(AppError::already_exists("Tenant code already exists"));
    }
    
    // 2. 创建租户
    let tenant = Tenant::new(code, name, settings)?;
    
    // 3. 保存租户
    self.tenant_repo.save(&tenant).await?;
    
    // 4. 初始化租户数据
    self.initialize_tenant_data(&tenant).await?;
    
    Ok(tenant)
}

async fn initialize_tenant_data(&self, tenant: &Tenant) -> Result<()> {
    // 创建默认角色
    self.create_default_roles(tenant).await?;
    
    // 创建管理员用户
    self.create_admin_user(tenant).await?;
    
    // 初始化配置
    self.initialize_settings(tenant).await?;
    
    Ok(())
}
```

### 暂停租户

```rust
pub async fn suspend_tenant(
    &self,
    tenant_id: &TenantId,
    reason: String,
) -> Result<()> {
    // 1. 更新租户状态
    let mut tenant = self.tenant_repo
        .find_by_id(tenant_id)
        .await?
        .ok_or_else(|| AppError::not_found("Tenant not found"))?;
    
    tenant.suspend(reason)?;
    self.tenant_repo.update(&tenant).await?;
    
    // 2. 撤销所有会话
    self.session_repo.revoke_all_by_tenant(tenant_id).await?;
    
    // 3. 发送通知
    self.notification_service
        .notify_tenant_suspended(tenant_id)
        .await?;
    
    Ok(())
}
```

### 删除租户

```rust
pub async fn delete_tenant(
    &self,
    tenant_id: &TenantId,
) -> Result<()> {
    // 1. 软删除租户
    self.tenant_repo.soft_delete(tenant_id).await?;
    
    // 2. 归档数据（可选）
    self.archive_tenant_data(tenant_id).await?;
    
    // 3. 计划物理删除（30 天后）
    self.schedule_physical_deletion(tenant_id, 30).await?;
    
    Ok(())
}
```

## 租户配置

### 功能开关

```rust
pub struct TenantFeatures {
    pub two_factor_auth: bool,
    pub webauthn: bool,
    pub oauth2: bool,
    pub api_access: bool,
    pub custom_branding: bool,
}

impl Tenant {
    pub fn has_feature(&self, feature: &str) -> bool {
        self.settings.features.contains(&feature.to_string())
    }
}
```

### 配额管理

```rust
pub struct TenantQuota {
    pub max_users: u32,
    pub max_storage_gb: u32,
    pub max_api_calls_per_day: u32,
}

pub async fn check_quota(
    &self,
    tenant_id: &TenantId,
    quota_type: QuotaType,
) -> Result<bool> {
    let tenant = self.tenant_repo.find_by_id(tenant_id).await?
        .ok_or_else(|| AppError::not_found("Tenant not found"))?;
    
    match quota_type {
        QuotaType::Users => {
            let user_count = self.user_repo.count_by_tenant(tenant_id).await?;
            Ok(user_count < tenant.settings.max_users.unwrap_or(u32::MAX))
        }
        QuotaType::Storage => {
            // 检查存储配额
            todo!()
        }
        QuotaType::ApiCalls => {
            // 检查 API 调用配额
            todo!()
        }
    }
}
```

## 跨租户操作

### 超级管理员

超级管理员可以访问所有租户数据：

```rust
pub struct SuperAdminContext {
    pub admin_id: UserId,
    pub target_tenant_id: Option<TenantId>,
}

impl SuperAdminContext {
    pub fn can_access_tenant(&self, tenant_id: &TenantId) -> bool {
        // 超级管理员可以访问任何租户
        true
    }
}
```

### 审计和监控

跨租户的审计和监控：

```rust
pub async fn get_system_metrics(&self) -> Result<SystemMetrics> {
    SystemMetrics {
        total_tenants: self.tenant_repo.count().await?,
        active_tenants: self.tenant_repo.count_by_status(TenantStatus::Active).await?,
        total_users: self.user_repo.count_all().await?,
        total_sessions: self.session_repo.count_all().await?,
    }
}
```

## 测试

### 租户隔离测试

```rust
#[sqlx::test]
async fn test_tenant_isolation(pool: PgPool) {
    let tenant1 = TenantId::new();
    let tenant2 = TenantId::new();
    
    // 创建租户 1 的用户
    let user1 = User::new("user1", "user1@tenant1.com", tenant1)?;
    user_repo.save(&user1, &tenant1).await?;
    
    // 创建租户 2 的用户
    let user2 = User::new("user2", "user2@tenant2.com", tenant2)?;
    user_repo.save(&user2, &tenant2).await?;
    
    // 租户 1 不能访问租户 2 的用户
    let result = user_repo.find_by_id(&user2.id, &tenant1).await?;
    assert!(result.is_none());
    
    // 租户 2 不能访问租户 1 的用户
    let result = user_repo.find_by_id(&user1.id, &tenant2).await?;
    assert!(result.is_none());
}
```

## 最佳实践

### 1. 始终验证租户上下文

```rust
// ✅ 正确
async fn get_user(&self, user_id: &UserId, tenant_id: &TenantId) -> Result<User> {
    self.repo.find_by_id(user_id, tenant_id).await
}

// ❌ 错误：缺少租户验证
async fn get_user(&self, user_id: &UserId) -> Result<User> {
    self.repo.find_by_id(user_id).await
}
```

### 2. 使用 RLS 作为最后防线

即使应用层有租户隔离，也要启用数据库 RLS。

### 3. 审计跨租户访问

记录所有跨租户访问（如超级管理员操作）。

### 4. 测试租户隔离

为所有 Repository 方法编写租户隔离测试。

### 5. 监控租户配额

定期检查租户配额使用情况，防止滥用。

## 相关文档

- [安全最佳实践](./security.md)
- [IAM Identity 服务](../api/iam/README.md)
- [租户隔离测试指南](../../services/iam-identity/TENANT_ISOLATION_GUIDE.md)
