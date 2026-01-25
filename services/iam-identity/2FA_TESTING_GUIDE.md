# 2FA 功能测试指南

## 🧪 测试环境准备

### 1. 启动依赖服务
```bash
# 启动 PostgreSQL 和 Redis
docker-compose up -d postgres redis
```

### 2. 运行数据库迁移
```bash
# 设置数据库 URL
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/cuba"

# 运行迁移
sqlx migrate run --source services/iam-identity/migrations
```

### 3. 启动 IAM Identity 服务
```bash
cargo run -p iam-identity
```

服务将在以下端口启动：
- gRPC: `127.0.0.1:50051`
- 健康检查: `127.0.0.1:51051`

---

## 📱 测试工具

### 推荐工具
1. **grpcurl** - 命令行 gRPC 客户端
2. **BloomRPC** - GUI gRPC 客户端
3. **Google Authenticator** - 移动端 TOTP 应用
4. **Authy** - 移动端 TOTP 应用（备选）

### 安装 grpcurl
```bash
# macOS
brew install grpcurl

# Linux
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest
```

---

## 🧪 测试场景

### 场景 1：启用 2FA（完整流程）

#### 步骤 1：创建测试用户（如果还没有）
```bash
grpcurl -plaintext \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "Test123456!",
    "tenant_id": "00000000-0000-0000-0000-000000000001"
  }' \
  localhost:50052 cuba.iam.user.UserService/CreateUser
```

#### 步骤 2：登录获取用户 ID
```bash
grpcurl -plaintext \
  -d '{
    "username": "testuser",
    "password": "Test123456!",
    "tenant_id": "00000000-0000-0000-0000-000000000001"
  }' \
  localhost:50051 cuba.iam.auth.AuthService/Login
```

保存返回的 `user.id`，例如：`550e8400-e29b-41d4-a716-446655440000`

#### 步骤 3：启用 2FA（第一步 - 获取 QR 码）
```bash
grpcurl -plaintext \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "method": "totp",
    "verification_code": ""
  }' \
  localhost:50051 cuba.iam.auth.AuthService/Enable2FA
```

**预期响应**：
```json
{
  "secret": "JBSWY3DPEHPK3PXP",
  "qr_code_url": "otpauth://totp/Cuba%20ERP:testuser?secret=JBSWY3DPEHPK3PXP&issuer=Cuba%20ERP&algorithm=SHA1&digits=6&period=30",
  "backup_codes": [],
  "enabled": false
}
```

#### 步骤 4：扫描 QR 码
1. 打开 Google Authenticator 应用
2. 点击 "+" 添加账户
3. 选择 "扫描 QR 码"
4. 将 `qr_code_url` 转换为 QR 码图片并扫描
   - 或者手动输入 secret: `JBSWY3DPEHPK3PXP`

#### 步骤 5：启用 2FA（第二步 - 验证并启用）
```bash
# 从 Google Authenticator 获取 6 位验证码
grpcurl -plaintext \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "method": "totp",
    "verification_code": "123456"
  }' \
  localhost:50051 cuba.iam.auth.AuthService/Enable2FA
```

**预期响应**：
```json
{
  "secret": "JBSWY3DPEHPK3PXP",
  "qr_code_url": "otpauth://...",
  "backup_codes": [
    "12345678",
    "87654321",
    "11223344",
    "55667788",
    "99887766",
    "44332211",
    "66778899",
    "22334455",
    "88776655",
    "33445566"
  ],
  "enabled": true
}
```

**重要**：保存这 10 个备份码！

---

### 场景 2：使用 TOTP 码验证 2FA

#### 步骤 1：登录（会返回 require_2fa=true）
```bash
grpcurl -plaintext \
  -d '{
    "username": "testuser",
    "password": "Test123456!",
    "tenant_id": "00000000-0000-0000-0000-000000000001"
  }' \
  localhost:50051 cuba.iam.auth.AuthService/Login
```

**预期响应**：
```json
{
  "require_2fa": true,
  "session_id": "temporary-session-id"
}
```

#### 步骤 2：验证 2FA
```bash
# 从 Google Authenticator 获取当前验证码
grpcurl -plaintext \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "code": "123456"
  }' \
  localhost:50051 cuba.iam.auth.AuthService/Verify2FA
```

**预期响应**：
```json
{
  "success": true,
  "access_token": "eyJhbGc...",
  "refresh_token": "eyJhbGc...",
  "expires_in": 3600
}
```

---

### 场景 3：使用备份码验证 2FA

#### 步骤 1：登录
```bash
grpcurl -plaintext \
  -d '{
    "username": "testuser",
    "password": "Test123456!",
    "tenant_id": "00000000-0000-0000-0000-000000000001"
  }' \
  localhost:50051 cuba.iam.auth.AuthService/Login
```

#### 步骤 2：使用备份码验证
```bash
# 使用之前保存的备份码之一
grpcurl -plaintext \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "code": "12345678"
  }' \
  localhost:50051 cuba.iam.auth.AuthService/Verify2FA
```

**预期响应**：
```json
{
  "success": true,
  "access_token": "eyJhbGc...",
  "refresh_token": "eyJhbGc...",
  "expires_in": 3600
}
```

**注意**：这个备份码现在已经被使用，不能再次使用！

---

### 场景 4：禁用 2FA

```bash
grpcurl -plaintext \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "password": "Test123456!"
  }' \
  localhost:50051 cuba.iam.auth.AuthService/Disable2FA
```

**预期响应**：
```json
{
  "success": true,
  "message": "2FA disabled successfully"
}
```

---

## ✅ 测试检查清单

### 功能测试
- [ ] 可以启用 2FA（两步流程）
- [ ] QR 码可以被 Google Authenticator 扫描
- [ ] TOTP 码验证成功
- [ ] 备份码验证成功
- [ ] 备份码只能使用一次
- [ ] 可以禁用 2FA
- [ ] 禁用 2FA 需要密码验证

### 错误处理测试
- [ ] 无效的 TOTP 码返回错误
- [ ] 无效的备份码返回错误
- [ ] 已使用的备份码返回错误
- [ ] 错误的密码无法禁用 2FA
- [ ] 未启用 2FA 的用户无法验证 2FA

### 安全测试
- [ ] TOTP secret 只在启用时返回一次
- [ ] 备份码只在启用时返回一次
- [ ] 验证失败不泄露具体原因
- [ ] 禁用 2FA 需要密码验证

---

## 🐛 常见问题

### 问题 1：TOTP 码验证失败
**原因**：时间不同步
**解决**：
1. 确保服务器时间正确：`date`
2. 确保手机时间自动同步
3. TOTP 有 30 秒的时间窗口，可能需要等待下一个码

### 问题 2：QR 码无法扫描
**原因**：URL 格式问题
**解决**：
1. 检查 `qr_code_url` 是否完整
2. 可以使用在线 QR 码生成器：https://www.qr-code-generator.com/
3. 或者手动输入 secret

### 问题 3：备份码验证失败
**原因**：备份码已被使用
**解决**：
1. 检查数据库：`SELECT * FROM backup_codes WHERE user_id = '...'`
2. 使用其他未使用的备份码
3. 如果所有备份码都用完，需要禁用并重新启用 2FA

---

## 📊 数据库验证

### 查看用户的 2FA 状态
```sql
SELECT 
  id,
  username,
  two_factor_enabled,
  two_factor_secret IS NOT NULL as has_secret
FROM users
WHERE username = 'testuser';
```

### 查看备份码
```sql
SELECT 
  id,
  user_id,
  is_used,
  used_at,
  created_at
FROM backup_codes
WHERE user_id = '550e8400-e29b-41d4-a716-446655440000'
ORDER BY created_at DESC;
```

### 统计可用备份码
```sql
SELECT 
  COUNT(*) as total,
  SUM(CASE WHEN is_used THEN 1 ELSE 0 END) as used,
  SUM(CASE WHEN NOT is_used THEN 1 ELSE 0 END) as available
FROM backup_codes
WHERE user_id = '550e8400-e29b-41d4-a716-446655440000';
```

---

## 🎯 性能测试

### 测试 TOTP 验证性能
```bash
# 使用 hey 进行压力测试
hey -n 1000 -c 10 \
  -m POST \
  -H "Content-Type: application/grpc" \
  -d '{"user_id":"...","code":"123456"}' \
  http://localhost:50051/cuba.iam.auth.AuthService/Verify2FA
```

### 预期性能指标
- 响应时间：< 100ms (p95)
- 吞吐量：> 100 req/s
- 错误率：0%

---

## 📝 测试报告模板

```markdown
# 2FA 功能测试报告

**测试日期**: 2026-01-26
**测试人员**: [姓名]
**环境**: Development

## 测试结果

### 功能测试
- [x] 启用 2FA: ✅ 通过
- [x] TOTP 验证: ✅ 通过
- [x] 备份码验证: ✅ 通过
- [x] 禁用 2FA: ✅ 通过

### 错误处理测试
- [x] 无效 TOTP: ✅ 通过
- [x] 无效备份码: ✅ 通过
- [x] 错误密码: ✅ 通过

### 性能测试
- 响应时间 (p95): 85ms
- 吞吐量: 120 req/s
- 错误率: 0%

## 问题
无

## 结论
2FA 功能测试通过，可以部署到生产环境。
```

---

## 🚀 下一步

测试通过后：
1. 更新用户文档
2. 添加监控和告警
3. 部署到测试环境
4. 进行用户验收测试
5. 部署到生产环境
