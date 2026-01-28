# IAM Access Service

访问控制服务 - 统一授权决策点 (PDP)

## 功能

- **RBAC** - 基于角色的访问控制
- **ABAC** - 基于属性的访问控制 (Policy)
- **统一授权** - 聚合 RBAC 和 Policy 决策

## 架构

```
┌─────────────────────────────────────────────────┐
│              Authorization Service               │
│  ┌─────────────┐        ┌─────────────────────┐ │
│  │   RBAC      │        │   Policy (ABAC)     │ │
│  │ Permission  │        │ Subject/Resource/   │ │
│  │   Check     │        │   Action Matching   │ │
│  └─────────────┘        └─────────────────────┘ │
└─────────────────────────────────────────────────┘
           ↓ Deny 优先  ↓
        授权决策结果
```

## API 端点

### RBAC 服务

| 方法 | 描述 |
|------|------|
| `CreateRole` | 创建角色 |
| `UpdateRole` | 更新角色 |
| `DeleteRole` | 删除角色 |
| `AssignPermissionsToRole` | 分配权限到角色 |
| `AssignRolesToUser` | 分配角色给用户 |

### Authorization 服务

| 方法 | 描述 |
|------|------|
| `Check` | 单次授权检查 |
| `BatchCheck` | 批量授权检查 |
| `GetUserGrantedPermissions` | 获取用户权限列表 |

## 决策逻辑

1. 首先检查 Policy (ABAC)
2. 如果 Policy 返回 **Deny** → 直接拒绝
3. 如果 Policy 返回 **Allow** → 直接允许
4. 如果 Policy 无匹配 → 检查 RBAC
5. 如果 RBAC 允许 → 允许
6. 默认拒绝

## 使用示例

```rust
// 授权检查请求
let request = CheckRequest {
    tenant_id: "tenant-uuid".to_string(),
    subject: "user:user-uuid".to_string(),
    resource: "order".to_string(),
    action: "read".to_string(),
    context: None,
};

// 调用授权服务
let response = client.check(request).await?;
println!("Allowed: {}", response.allowed);
```

## 配置

通过环境变量配置：

| 变量 | 描述 | 默认值 |
|------|------|--------|
| `DATABASE_URL` | PostgreSQL 连接 | - |
| `REDIS_URL` | Redis 连接 | - |
| `GRPC_PORT` | gRPC 端口 | 50051 |

## Metrics

服务暴露以下指标：

- `authorization_checks_total` - 授权检查总数
- `authorization_check_duration_ms` - 检查延迟
- `authorization_checks_errors_total` - 错误数
