# REST API 测试指南（通过 Gateway）

## 当前状态

✅ **Gateway 已完成基础集成**

Gateway 已成功集成 gRPC 客户端，可以通过 REST API 访问后端服务。

### 已实现
- ✅ Gateway 服务启动（端口 8080）
- ✅ gRPC 客户端集成
- ✅ 用户注册 API
- ✅ 用户登录 API
- ✅ 用户登出 API
- ✅ Token 刷新 API
- ✅ 请求/响应转换（REST ↔ gRPC）
- ✅ 错误处理和映射
- ✅ CORS 支持
- ✅ HTTP 追踪

### 待实现
- ⏳ Token 验证中间件
- ⏳ 租户隔离中间件（从 header 提取）
- ⏳ 更多用户管理 API
- ⏳ 密码管理 API
- ⏳ 会话管理 API
- ⏳ 2FA API

## 服务信息

- **Gateway 地址**: `http://localhost:8080`
- **IAM Identity gRPC**: `localhost:50051`
- **健康检查**: `http://localhost:8080/health`

## 当前可用端点

### 健康检查

#### GET /health

```bash
curl http://localhost:8080/health
```

响应：
```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

#### GET /ready

```bash
curl http://localhost:8080/ready
```

响应：
```json
{
  "ready": true,
  "checks": [
    {
      "name": "iam-identity",
      "healthy": true
    }
  ]
}
```

### 认证 API（已实现）

#### POST /api/auth/register

注册新用户

```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "test_user",
    "email": "test@example.com",
    "password": "TestPass123!",
    "display_name": "Test User"
  }'
```

响应：
```json
{
  "user_id": "019bf98a-2478-79d2-81f6-fbe41aef4c56",
  "user": {
    "id": "019bf98a-2478-79d2-81f6-fbe41aef4c56",
    "username": "test_user",
    "email": "test@example.com",
    "display_name": "Test User",
    "status": "PendingVerification"
  }
}
```

#### POST /api/auth/login

用户登录

```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "test_user",
    "password": "TestPass123!"
  }'
```

响应：
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "expires_in": "3600",
  "token_type": "Bearer",
  "user": {
    "id": "019bf98a-2478-79d2-81f6-fbe41aef4c56",
    "username": "test_user",
    "email": "test@example.com",
    "display_name": "Test User",
    "status": "Active"
  }
}
```

#### POST /api/auth/logout

用户登出

```bash
curl -X POST http://localhost:8080/api/auth/logout \
  -H "Content-Type: application/json" \
  -d '{
    "access_token": "<your_access_token>",
    "logout_all_devices": false
  }'
```

响应：
```json
{
  "success": true,
  "message": null
}
```

#### POST /api/auth/refresh

刷新 Token

```bash
curl -X POST http://localhost:8080/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "<your_refresh_token>"
  }'
```

响应：
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "expires_in": "3600"
}
```

#### GET /api/auth/me

获取当前用户信息（待实现）

```bash
curl http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer <access_token>"
```

## 完整测试流程

### 步骤 1: 注册用户

```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe",
    "email": "john@example.com",
    "password": "SecurePass123!",
    "display_name": "John Doe"
  }'
```

### 步骤 2: 激活用户（临时方案）

由于用户注册后状态为 `PendingVerification`，需要手动激活：

```bash
docker exec -i cuba-postgres psql -U postgres -d cuba -c \
  "UPDATE users SET status = 'Active' WHERE username = 'john_doe';"
```

### 步骤 3: 登录

```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe",
    "password": "SecurePass123!"
  }'
```

保存返回的 `access_token` 和 `refresh_token`。

### 步骤 4: 使用 Access Token

```bash
# 保存 token
export ACCESS_TOKEN="<your_access_token>"

# 使用 token 调用需要认证的 API（待实现）
curl http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

### 步骤 5: 刷新 Token

```bash
export REFRESH_TOKEN="<your_refresh_token>"

curl -X POST http://localhost:8080/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\": \"$REFRESH_TOKEN\"}"
```

### 步骤 6: 登出

```bash
# 登出当前设备
curl -X POST http://localhost:8080/api/auth/logout \
  -H "Content-Type: application/json" \
  -d "{
    \"access_token\": \"$ACCESS_TOKEN\",
    \"logout_all_devices\": false
  }"

# 登出所有设备
curl -X POST http://localhost:8080/api/auth/logout \
  -H "Content-Type: application/json" \
  -d "{
    \"access_token\": \"$ACCESS_TOKEN\",
    \"logout_all_devices\": true
  }"
```

## 未来的 REST API 设计

### 认证 API

#### POST /api/auth/register

注册新用户

```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
  -d '{
    "username": "john_doe",
    "email": "john@example.com",
    "password": "SecurePass123!",
    "display_name": "John Doe"
  }'
```

#### POST /api/auth/login

用户登录

```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
  -d '{
    "username": "john_doe",
    "password": "SecurePass123!"
  }'
```

响应：
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "expires_in": 3600,
  "token_type": "Bearer",
  "user": {
    "id": "019bf97d-1687-7030-b05f-f47491d3e406",
    "username": "john_doe",
    "email": "john@example.com",
    "display_name": "John Doe",
    "status": "Active"
  }
}
```

#### POST /api/auth/logout

用户登出

```bash
curl -X POST http://localhost:8080/api/auth/logout \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access_token>" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
  -d '{
    "logout_all_devices": false
  }'
```

#### POST /api/auth/refresh

刷新 Token

```bash
curl -X POST http://localhost:8080/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "<refresh_token>"
  }'
```

#### GET /api/auth/me

获取当前用户信息

```bash
curl http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer <access_token>" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001"
```

### 用户管理 API

#### GET /api/users/:id

获取用户详情

```bash
curl http://localhost:8080/api/users/019bf97d-1687-7030-b05f-f47491d3e406 \
  -H "Authorization: Bearer <access_token>" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001"
```

#### PUT /api/users/profile

更新用户资料

```bash
curl -X PUT http://localhost:8080/api/users/profile \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access_token>" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
  -d '{
    "display_name": "John Smith",
    "phone": "+1234567890",
    "language": "en-US",
    "timezone": "America/New_York"
  }'
```

#### GET /api/users

列出用户

```bash
curl "http://localhost:8080/api/users?page=1&page_size=10" \
  -H "Authorization: Bearer <access_token>" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001"
```

### 密码管理 API

#### POST /api/auth/password/change

修改密码

```bash
curl -X POST http://localhost:8080/api/auth/password/change \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access_token>" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
  -d '{
    "old_password": "SecurePass123!",
    "new_password": "NewSecurePass456!"
  }'
```

#### POST /api/auth/password/reset/request

请求密码重置

```bash
curl -X POST http://localhost:8080/api/auth/password/reset/request \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
  -d '{
    "email": "john@example.com"
  }'
```

#### POST /api/auth/password/reset

重置密码

```bash
curl -X POST http://localhost:8080/api/auth/password/reset \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
  -d '{
    "email": "john@example.com",
    "reset_token": "token-from-email",
    "new_password": "NewPassword123!"
  }'
```

### 会话管理 API

#### GET /api/auth/sessions

获取活跃会话

```bash
curl http://localhost:8080/api/auth/sessions \
  -H "Authorization: Bearer <access_token>" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001"
```

#### DELETE /api/auth/sessions/:id

撤销会话

```bash
curl -X DELETE http://localhost:8080/api/auth/sessions/019bf97e-f66c-7193-93e0-1f2658e91669 \
  -H "Authorization: Bearer <access_token>" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001"
```

### 2FA 管理 API

#### POST /api/auth/2fa/enable

启用 2FA

```bash
curl -X POST http://localhost:8080/api/auth/2fa/enable \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access_token>" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
  -d '{
    "method": "totp"
  }'
```

#### POST /api/auth/2fa/verify

验证 2FA

```bash
curl -X POST http://localhost:8080/api/auth/2fa/verify \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
  -d '{
    "user_id": "019bf97d-1687-7030-b05f-f47491d3e406",
    "code": "123456",
    "session_id": "session-id-from-login"
  }'
```

#### POST /api/auth/2fa/disable

禁用 2FA

```bash
curl -X POST http://localhost:8080/api/auth/2fa/disable \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access_token>" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
  -d '{
    "password": "SecurePass123!"
  }'
```

## HTTP Headers

### 必需 Headers

- `Content-Type: application/json` - 所有 POST/PUT 请求
- `X-Tenant-ID: <uuid>` - 租户隔离（所有需要租户上下文的请求）
- `Authorization: Bearer <token>` - 需要认证的请求

### 可选 Headers

- `Accept-Language: zh-CN` - 指定响应语言
- `X-Request-ID: <uuid>` - 请求追踪 ID

## 错误响应格式

```json
{
  "error": {
    "code": "INVALID_CREDENTIALS",
    "message": "用户名或密码错误",
    "details": {
      "field": "password",
      "reason": "incorrect"
    }
  }
}
```

### 常见错误码

| HTTP 状态码 | 错误码 | 说明 |
|------------|--------|------|
| 400 | INVALID_REQUEST | 请求参数无效 |
| 401 | UNAUTHENTICATED | 未认证或 Token 无效 |
| 403 | PERMISSION_DENIED | 权限不足 |
| 404 | NOT_FOUND | 资源不存在 |
| 409 | ALREADY_EXISTS | 资源已存在 |
| 429 | RATE_LIMIT_EXCEEDED | 请求频率超限 |
| 500 | INTERNAL_ERROR | 服务器内部错误 |

## 使用 Postman 测试

### 1. 导入环境变量

创建环境变量：
- `base_url`: `http://localhost:8080`
- `tenant_id`: `00000000-0000-0000-0000-000000000001`
- `access_token`: （登录后自动设置）

### 2. 设置 Pre-request Script

在 Collection 级别添加：

```javascript
pm.request.headers.add({
    key: 'X-Tenant-ID',
    value: pm.environment.get('tenant_id')
});
```

### 3. 设置 Tests Script

在登录请求中添加：

```javascript
if (pm.response.code === 200) {
    const response = pm.response.json();
    pm.environment.set('access_token', response.access_token);
    pm.environment.set('refresh_token', response.refresh_token);
}
```

## 开发计划

### Phase 1: 基础集成（当前）
- [ ] 实现 gRPC 客户端连接
- [ ] 实现登录/登出 API
- [ ] 实现 Token 验证中间件
- [ ] 实现租户隔离中间件

### Phase 2: 完整认证
- [ ] 实现注册 API
- [ ] 实现密码管理 API
- [ ] 实现会话管理 API
- [ ] 实现 2FA API

### Phase 3: 用户管理
- [ ] 实现用户 CRUD API
- [ ] 实现角色管理 API
- [ ] 实现权限管理 API

### Phase 4: 高级功能
- [ ] 实现 WebAuthn API
- [ ] 实现 OAuth2 API
- [ ] 实现审计日志 API
- [ ] 实现速率限制

## 相关文档

- [GRPC_API_TESTING_GUIDE.md](GRPC_API_TESTING_GUIDE.md) - gRPC API 测试指南（当前推荐）
- [FRONTEND_API_GUIDE.md](FRONTEND_API_GUIDE.md) - 前端集成指南
- [PROJECT_STARTUP_CHECKLIST.md](PROJECT_STARTUP_CHECKLIST.md) - 项目启动指南

## 注意事项

1. **Tenant ID**：当前默认使用 `00000000-0000-0000-0000-000000000001`，可在请求中指定
2. **Token 格式**：使用 JWT，需要在 Authorization header 中传递
3. **CORS**：Gateway 已配置 permissive CORS，支持跨域请求
4. **用户激活**：注册后需要手动激活用户（未来将实现邮件验证）
5. **错误处理**：Gateway 会将 gRPC 错误码映射为 HTTP 状态码
