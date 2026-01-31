# 多租户支持实施指南

## 概述

本文档说明如何在 ERP ERP 中实现完整的多租户隔离。

## 已实现功能

### 1. 数据库层

#### 租户表
- ✅ `tenants` 表：存储租户基本信息、状态、设置
- ✅ 支持试用期和订阅管理
- ✅ 租户状态：Trial, Active, Suspended, Cancelled

#### 租户隔离
- ✅ 所有业务表添加 `tenant_id` 字段
- ✅ 为 `tenant_id` 创建索引
- ✅ 启用 PostgreSQL 行级安全（RLS）
- ✅ 创建租户隔离策略

### 2. 领域层

#### 实体
- ✅ `Tenant` 聚合根
- ✅ `TenantStatus` 枚举
- ✅ `TenantSettings` 值对象
- ✅ `TenantContext` 值对象

#### 仓储
- ✅ `TenantRepository` trait
- ✅ `PostgresTenantRepository` 实现

### 3. 基础设施层

#### 中间件
- ✅ `extract_tenant_id()` - 从请求提取租户 ID
- ✅ `TenantValidationInterceptor` - 验证租户状态

#### 工具
- ✅ `set_tenant_context()` - 设置 RLS 上下文
- ✅ `with_tenant_context()` - 在事务中执行带租户上下文的操作

### 4. 测试
- ✅ 租户实体单元测试
- ✅ 租户仓储集成测试
- ✅ 租户隔离测试

## 使用指南

### 1. 运行数据库迁移

```bash
# 进入服务目录
cd services/iam-identity

# 运行迁移
sqlx migrate run
```

迁移文件：
- `20260126052917_create_tenants.sql` - 创建租户表
- `20260126052918_add_tenant_id_to_tables.sql` - 为业务表添加 tenant_id
- `20260126052919_enable_rls_policies.sql` - 启用 RLS 策略

### 2. 在 main.rs 中注入 TenantRepository

```rust
use iam_identity::shared::infrastructure::persistence::PostgresTenantRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run("config", |infra: Infrastructure| async move {
        let pool = infra.postgres_pool();
        
        // 创建租户仓储
        let tenant_repo: Arc<dyn TenantRepository> = 
            Arc::new(PostgresTenantRepository::new(pool.clone()));
        
        // 创建租户验证中间件
        let tenant_interceptor = TenantValidationInterceptor::new(tenant_repo.clone());
        
        // ... 其他初始化
    }).await
}
```

### 3. 在 gRPC 服务中使用租户中间件

```rust
use tonic::{Request, Response, Status};
use iam_identity::shared::infrastructure::middleware::extract_tenant_id;

impl MyService for MyServiceImpl {
    async fn my_method(
        &self,
        request: Request<MyRequest>,
    ) -> Result<Response<MyResponse>, Status> {
        // 提取租户 ID
        let tenant_id = extract_tenant_id(&request)?;
        
        // 验证租户（可选，如果已在拦截器中验证）
        // self.tenant_interceptor.validate_tenant(&tenant_id).await?;
        
        // 使用租户 ID 进行业务操作
        // ...
    }
}
```

### 4. 在仓储方法中使用租户上下文

```rust
use iam_identity::shared::infrastructure::persistence::with_tenant_context;

impl UserRepository for PostgresUserRepository {
    async fn find_by_email(
        &self,
        tenant_id: &TenantId,
        email: &str,
    ) -> AppResult<Option<User>> {
        with_tenant_context(&self.pool, tenant_id, |conn| {
            Box::pin(async move {
                sqlx::query_as::<_, UserRow>(
                    "SELECT * FROM users WHERE email = $1"
                )
                .bind(email)
                .fetch_optional(conn)
                .await
                .map(|row| row.map(Into::into))
                .map_err(|e| AppError::database(format!("Failed to find user: {}", e)))
            })
        })
        .await
    }
}
```

### 5. 创建默认租户（开发环境）

```sql
-- 创建默认租户用于开发
INSERT INTO tenants (
    id, 
    name, 
    display_name, 
    status, 
    trial_ends_at
) VALUES (
    '00000000-0000-0000-0000-000000000001',
    'default',
    'Default Tenant',
    'Active',
    NOW() + INTERVAL '365 days'
);
```

## 待完成任务

### 1. 更新所有 Repository 方法

需要为所有仓储方法添加 `tenant_id` 参数：

- [ ] `UserRepository`
- [ ] `SessionRepository`
- [ ] `LoginLogRepository`
- [ ] `PasswordResetRepository`
- [ ] `WebAuthnCredentialRepository`
- [ ] `BackupCodeRepository`
- [ ] `EmailVerificationRepository`
- [ ] `PhoneVerificationRepository`
- [ ] `OAuthClientRepository`
- [ ] `AuthorizationCodeRepository`
- [ ] `AccessTokenRepository`
- [ ] `RefreshTokenRepository`

示例：

```rust
// 修改前
async fn find_by_id(&self, id: &UserId) -> AppResult<Option<User>>;

// 修改后
async fn find_by_id(
    &self, 
    tenant_id: &TenantId, 
    id: &UserId
) -> AppResult<Option<User>>;
```

### 2. 更新所有 Command/Query Handler

在所有命令和查询处理器中传递 `tenant_id`：

```rust
impl CommandHandler<CreateUserCommand> for CreateUserHandler {
    async fn handle(&self, command: CreateUserCommand) -> AppResult<UserId> {
        // 从命令中获取 tenant_id
        let tenant_id = command.tenant_id;
        
        // 传递给仓储
        self.user_repo.save(&tenant_id, &user).await?;
        
        Ok(user.id)
    }
}
```

### 3. 更新 gRPC 服务实现

在所有 gRPC 服务方法中提取和验证租户：

```rust
async fn create_user(
    &self,
    request: Request<CreateUserRequest>,
) -> Result<Response<CreateUserResponse>, Status> {
    // 提取租户 ID
    let tenant_id = extract_tenant_id(&request)?;
    
    // 构建命令（包含 tenant_id）
    let command = CreateUserCommand {
        tenant_id,
        email: request.get_ref().email.clone(),
        // ...
    };
    
    // 执行命令
    let user_id = self.handler.handle(command).await?;
    
    Ok(Response::new(CreateUserResponse { 
        user_id: user_id.to_string() 
    }))
}
```

### 4. 添加租户管理 API

创建租户管理的 gRPC 服务：

- [ ] `CreateTenant` - 创建租户
- [ ] `GetTenant` - 获取租户信息
- [ ] `UpdateTenant` - 更新租户
- [ ] `ListTenants` - 列表查询
- [ ] `ActivateTenant` - 激活租户
- [ ] `SuspendTenant` - 暂停租户
- [ ] `ExtendSubscription` - 延长订阅

### 5. 添加租户配额管理

实现租户资源配额检查：

```rust
pub struct TenantQuotaService {
    tenant_repo: Arc<dyn TenantRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl TenantQuotaService {
    /// 检查是否可以创建新用户
    pub async fn can_create_user(&self, tenant_id: &TenantId) -> AppResult<bool> {
        let tenant = self.tenant_repo.find_by_id(tenant_id).await?
            .ok_or_else(|| AppError::not_found("Tenant not found"))?;
        
        if let Some(max_users) = tenant.settings.max_users {
            let current_users = self.user_repo.count_by_tenant(tenant_id).await?;
            Ok(current_users < max_users)
        } else {
            Ok(true)
        }
    }
}
```

### 6. 添加租户缓存

使用 Redis 缓存租户信息以提高性能：

```rust
pub struct CachedTenantRepository {
    repo: Arc<dyn TenantRepository>,
    cache: Arc<dyn CachePort>,
}

impl CachedTenantRepository {
    async fn find_by_id(&self, id: &TenantId) -> AppResult<Option<Tenant>> {
        let cache_key = format!("tenant:{}", id);
        
        // 尝试从缓存获取
        if let Some(cached) = self.cache.get::<Tenant>(&cache_key).await? {
            return Ok(Some(cached));
        }
        
        // 从数据库获取
        if let Some(tenant) = self.repo.find_by_id(id).await? {
            // 缓存 1 小时
            self.cache.set(&cache_key, &tenant, 3600).await?;
            Ok(Some(tenant))
        } else {
            Ok(None)
        }
    }
}
```

## 测试

### 运行单元测试

```bash
cargo test -p iam-identity --lib
```

### 运行集成测试

```bash
cargo test -p iam-identity --test '*'
```

### 测试租户隔离

```bash
cargo test -p iam-identity tenant_isolation
```

## 注意事项

### 1. RLS 性能

- RLS 策略会在每次查询时执行，可能影响性能
- 建议为 `tenant_id` 创建索引
- 考虑使用连接池级别的租户上下文设置

### 2. 默认租户 ID

- 迁移中使用 `00000000-0000-0000-0000-000000000000` 作为默认值
- 生产环境应该为所有现有数据分配真实的租户 ID

### 3. 超级管理员

- 某些操作（如租户管理）需要跨租户访问
- 考虑创建特殊的超级管理员角色，绕过 RLS

### 4. 数据迁移

- 为现有数据分配租户 ID
- 更新所有 INSERT 语句包含 `tenant_id`

## 参考资料

- [PostgreSQL Row Level Security](https://www.postgresql.org/docs/current/ddl-rowsecurity.html)
- [Multi-tenancy Patterns](https://docs.microsoft.com/en-us/azure/architecture/patterns/multi-tenancy)
