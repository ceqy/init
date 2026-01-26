# gRPC API 测试指南

## 前提条件

1. 服务已启动：`just iam` 或 `cargo run --package iam-identity`
2. 数据库已迁移：`sqlx migrate run`
3. 安装 grpcurl：`brew install grpcurl` (macOS)

## 重要说明

### Tenant ID 格式

**必须使用 UUID 格式**，不能使用简单字符串如 "tenant-123"。

✅ 正确：`00000000-0000-0000-0000-000000000001`
❌ 错误：`tenant-123`

### Metadata Header

所有需要租户隔离的 API 都需要在 metadata 中传递 `tenant-id`：

```bash
-H 'tenant-id: 00000000-0000-0000-0000-000000000001'
```

## 使用 gRPC 反射

### 1. 列出所有服务

```bash
grpcurl -plaintext localhost:50051 list
```

输出：
```
cuba.iam.auth.AuthService
cuba.iam.user.UserService
grpc.reflection.v1.ServerReflection
```

### 2. 列出服务的所有方法

```bash
grpcurl -plaintext localhost:50051 list cuba.iam.auth.AuthService
```

### 3. 查看方法签名

```bash
grpcurl -plaintext localhost:50051 describe cuba.iam.auth.AuthService.Login
```

### 4. 查看消息结构

```bash
grpcurl -plaintext localhost:50051 describe cuba.iam.auth.LoginRequest
```

## 完整测试流程

### 步骤 1: 注册用户

```bash
grpcurl -plaintext \
  -d '{
    "username": "john_doe",
    "email": "john@example.com",
    "password": "SecurePass123!",
    "display_name": "John Doe",
    "tenant_id": "00000000-0000-0000-0000-000000000001"
  }' \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.user.UserService/Register
```

响应示例：
```json
{
  "userId": "019bf97d-1687-7030-b05f-f47491d3e406",
  "user": {
    "id": "019bf97d-1687-7030-b05f-f47491d3e406",
    "username": "john_doe",
    "email": "john@example.com",
    "displayName": "John Doe",
    "tenantId": "00000000-0000-0000-0000-000000000001",
    "status": "PendingVerification",
    "language": "zh-CN",
    "timezone": "Asia/Shanghai"
  }
}
```

### 步骤 2: 激活用户（临时方案）

由于 ActivateUser 需要管理员权限，暂时使用数据库直接激活：

```bash
docker exec -i cuba-postgres psql -U postgres -d cuba -c \
  "UPDATE users SET status = 'Active' WHERE username = 'john_doe';"
```

### 步骤 3: 登录

```bash
grpcurl -plaintext \
  -d '{
    "username": "john_doe",
    "password": "SecurePass123!",
    "tenant_id": "00000000-0000-0000-0000-000000000001",
    "ip_address": "127.0.0.1",
    "device_info": "grpcurl-test"
  }' \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.auth.AuthService/Login
```

响应示例：
```json
{
  "accessToken": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "refreshToken": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "expiresIn": "3600",
  "tokenType": "Bearer",
  "user": {
    "id": "019bf97d-1687-7030-b05f-f47491d3e406",
    "username": "john_doe",
    "email": "john@example.com",
    "displayName": "John Doe",
    "status": "Active"
  },
  "sessionId": "019bf97e-f66c-7193-93e0-1f2658e91669"
}
```

### 步骤 4: 使用 Access Token 调用需要认证的 API

```bash
# 保存 token
export ACCESS_TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."

# 获取当前用户信息
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.user.UserService/GetCurrentUser
```

### 步骤 5: 验证 Token

```bash
grpcurl -plaintext \
  -d "{\"access_token\": \"$ACCESS_TOKEN\"}" \
  localhost:50051 \
  cuba.iam.auth.AuthService/ValidateToken
```

### 步骤 6: 刷新 Token

```bash
export REFRESH_TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."

grpcurl -plaintext \
  -d "{\"refresh_token\": \"$REFRESH_TOKEN\"}" \
  localhost:50051 \
  cuba.iam.auth.AuthService/RefreshToken
```

### 步骤 7: 登出

```bash
# 登出当前设备
grpcurl -plaintext \
  -d "{\"access_token\": \"$ACCESS_TOKEN\", \"logout_all_devices\": false}" \
  localhost:50051 \
  cuba.iam.auth.AuthService/Logout

# 登出所有设备
grpcurl -plaintext \
  -d "{\"access_token\": \"$ACCESS_TOKEN\", \"logout_all_devices\": true}" \
  localhost:50051 \
  cuba.iam.auth.AuthService/Logout
```

## 常见 API 测试

### 用户管理

#### 获取用户信息

```bash
grpcurl -plaintext \
  -d '{"user_id": "019bf97d-1687-7030-b05f-f47491d3e406"}' \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.user.UserService/GetUser
```

#### 更新用户资料

```bash
grpcurl -plaintext \
  -d '{
    "display_name": "John Smith",
    "phone": "+1234567890",
    "language": "en-US",
    "timezone": "America/New_York"
  }' \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.user.UserService/UpdateProfile
```

#### 列出用户

```bash
grpcurl -plaintext \
  -d '{"page_size": 10, "page": 1}' \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.user.UserService/ListUsers
```

### 密码管理

#### 修改密码

```bash
grpcurl -plaintext \
  -d '{
    "user_id": "019bf97d-1687-7030-b05f-f47491d3e406",
    "old_password": "SecurePass123!",
    "new_password": "NewSecurePass456!"
  }' \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.auth.AuthService/ChangePassword
```

#### 请求密码重置

```bash
grpcurl -plaintext \
  -d '{
    "email": "john@example.com",
    "tenant_id": "00000000-0000-0000-0000-000000000001"
  }' \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.auth.AuthService/RequestPasswordReset
```

#### 重置密码

```bash
grpcurl -plaintext \
  -d '{
    "email": "john@example.com",
    "reset_token": "token-from-email",
    "new_password": "NewPassword123!"
  }' \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.auth.AuthService/ResetPassword
```

### 会话管理

#### 获取活跃会话

```bash
grpcurl -plaintext \
  -d '{"user_id": "019bf97d-1687-7030-b05f-f47491d3e406"}' \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.auth.AuthService/GetActiveSessions
```

#### 撤销会话

```bash
grpcurl -plaintext \
  -d '{
    "user_id": "019bf97d-1687-7030-b05f-f47491d3e406",
    "session_id": "019bf97e-f66c-7193-93e0-1f2658e91669"
  }' \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.auth.AuthService/RevokeSession
```

### 2FA 管理

#### 启用 2FA

```bash
grpcurl -plaintext \
  -d '{
    "user_id": "019bf97d-1687-7030-b05f-f47491d3e406",
    "method": "totp"
  }' \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.auth.AuthService/Enable2FA
```

#### 验证 2FA

```bash
grpcurl -plaintext \
  -d '{
    "user_id": "019bf97d-1687-7030-b05f-f47491d3e406",
    "code": "123456",
    "session_id": "session-id-from-login"
  }' \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.auth.AuthService/Verify2FA
```

#### 禁用 2FA

```bash
grpcurl -plaintext \
  -d '{
    "user_id": "019bf97d-1687-7030-b05f-f47491d3e406",
    "password": "SecurePass123!"
  }' \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.auth.AuthService/Disable2FA
```

## 常见错误

### 1. Invalid tenant ID

**错误**：
```
ERROR:
  Code: InvalidArgument
  Message: Invalid tenant ID
```

**原因**：tenant-id 不是有效的 UUID 格式

**解决**：使用 UUID 格式，如 `00000000-0000-0000-0000-000000000001`

### 2. Missing tenant-id in metadata

**错误**：
```
ERROR:
  Code: InvalidArgument
  Message: Missing tenant-id in metadata
```

**原因**：请求中缺少 tenant-id header

**解决**：添加 `-H 'tenant-id: <uuid>'`

### 3. Invalid credentials

**错误**：
```
ERROR:
  Code: Unauthenticated
  Message: Invalid credentials
```

**原因**：用户名或密码错误，或用户不存在

**解决**：检查用户名密码，或先注册用户

### 4. User account is not active

**错误**：
```
ERROR:
  Code: PermissionDenied
  Message: User account is not active
```

**原因**：用户状态不是 Active

**解决**：激活用户或修改数据库状态

### 5. Missing or invalid token

**错误**：
```
ERROR:
  Code: Unauthenticated
  Message: Missing or invalid token
```

**原因**：需要认证的 API 缺少 Authorization header

**解决**：添加 `-H "authorization: Bearer $ACCESS_TOKEN"`

## 使用 Proto 文件（不使用反射）

如果不想使用反射，可以直接指定 proto 文件：

```bash
grpcurl -plaintext \
  -proto proto/iam/auth.proto \
  -import-path proto \
  -d '{
    "username": "john_doe",
    "password": "SecurePass123!",
    "tenant_id": "00000000-0000-0000-0000-000000000001",
    "ip_address": "127.0.0.1"
  }' \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.auth.AuthService/Login
```

## 健康检查

服务在 gRPC 端口 + 1000 上提供 HTTP 健康检查：

```bash
# 存活检查
curl http://localhost:51051/health

# 就绪检查
curl http://localhost:51051/ready

# Prometheus metrics
curl http://localhost:51051/metrics
```

## 相关文档

- [GRPC_REFLECTION_STATUS.md](GRPC_REFLECTION_STATUS.md) - 反射实现状态
- [FRONTEND_API_GUIDE.md](FRONTEND_API_GUIDE.md) - 前端接入指南
- [PROJECT_STARTUP_CHECKLIST.md](PROJECT_STARTUP_CHECKLIST.md) - 项目启动指南
