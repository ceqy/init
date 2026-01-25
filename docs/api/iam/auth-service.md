# AuthService API 文档

## 服务定义

```protobuf
service AuthService {
  // 基础认证
  rpc Login(LoginRequest) returns (LoginResponse);
  rpc Logout(LogoutRequest) returns (LogoutResponse);
  rpc RefreshToken(RefreshTokenRequest) returns (RefreshTokenResponse);
  rpc ValidateToken(ValidateTokenRequest) returns (ValidateTokenResponse);
  
  // 密码管理
  rpc ChangePassword(ChangePasswordRequest) returns (ChangePasswordResponse);
  rpc RequestPasswordReset(RequestPasswordResetRequest) returns (RequestPasswordResetResponse);
  rpc ResetPassword(ResetPasswordRequest) returns (ResetPasswordResponse);
  
  // 双因子认证
  rpc Enable2FA(Enable2FARequest) returns (Enable2FAResponse);
  rpc Disable2FA(Disable2FARequest) returns (Disable2FAResponse);
  rpc Verify2FA(Verify2FARequest) returns (Verify2FAResponse);
  
  // 会话管理
  rpc GetActiveSessions(GetActiveSessionsRequest) returns (GetActiveSessionsResponse);
  rpc RevokeSession(RevokeSessionRequest) returns (RevokeSessionResponse);
  
  // WebAuthn 无密码登录
  rpc StartWebAuthnRegistration(StartWebAuthnRegistrationRequest) returns (StartWebAuthnRegistrationResponse);
  rpc FinishWebAuthnRegistration(FinishWebAuthnRegistrationRequest) returns (FinishWebAuthnRegistrationResponse);
  rpc StartWebAuthnAuthentication(StartWebAuthnAuthenticationRequest) returns (StartWebAuthnAuthenticationResponse);
  rpc FinishWebAuthnAuthentication(FinishWebAuthnAuthenticationRequest) returns (FinishWebAuthnAuthenticationResponse);
  rpc ListWebAuthnCredentials(ListWebAuthnCredentialsRequest) returns (ListWebAuthnCredentialsResponse);
  rpc DeleteWebAuthnCredential(DeleteWebAuthnCredentialRequest) returns (DeleteWebAuthnCredentialResponse);
}
```

## API 方法

### 1. Login - 用户登录

使用用户名和密码进行登录。

**请求**：
```protobuf
message LoginRequest {
  string username = 1;          // 用户名或邮箱（必填）
  string password = 2;          // 密码（必填）
  string tenant_id = 3;         // 租户 ID（必填）
  string device_info = 4;       // 设备信息（可选）
  string ip_address = 5;        // IP 地址（可选）
}
```

**响应**：
```protobuf
message LoginResponse {
  string access_token = 1;      // 访问令牌（JWT）
  string refresh_token = 2;     // 刷新令牌
  int64 expires_in = 3;         // 过期时间（秒）
  string token_type = 4;        // 令牌类型（Bearer）
  User user = 5;                // 用户信息
  bool require_2fa = 6;         // 是否需要 2FA 验证
  string session_id = 7;        // 会话 ID（2FA 时使用）
}
```

**示例**：
```bash
grpcurl -plaintext -d '{
  "username": "admin",
  "password": "SecurePass123!",
  "tenant_id": "default",
  "device_info": "Chrome 120 on macOS",
  "ip_address": "192.168.1.100"
}' localhost:50051 cuba.iam.auth.AuthService/Login
```

**错误码**：
- `INVALID_ARGUMENT`: 用户名或密码为空
- `UNAUTHENTICATED`: 用户名或密码错误
- `FAILED_PRECONDITION`: 账户已锁定或未激活
- `RESOURCE_EXHAUSTED`: 登录尝试次数过多

---

### 2. Logout - 用户登出

撤销当前会话或所有会话。

**请求**：
```protobuf
message LogoutRequest {
  string access_token = 1;      // 访问令牌（必填）
  bool logout_all_devices = 2;  // 是否登出所有设备（可选，默认 false）
}
```

**响应**：
```protobuf
message LogoutResponse {
  bool success = 1;             // 是否成功
}
```

**示例**：
```bash
grpcurl -plaintext -d '{
  "access_token": "eyJhbGc...",
  "logout_all_devices": false
}' localhost:50051 cuba.iam.auth.AuthService/Logout
```

---

### 3. RefreshToken - 刷新令牌

使用 Refresh Token 获取新的 Access Token。

**请求**：
```protobuf
message RefreshTokenRequest {
  string refresh_token = 1;     // 刷新令牌（必填）
}
```

**响应**：
```protobuf
message RefreshTokenResponse {
  string access_token = 1;      // 新的访问令牌
  string refresh_token = 2;     // 新的刷新令牌
  int64 expires_in = 3;         // 过期时间（秒）
}
```

**示例**：
```bash
grpcurl -plaintext -d '{
  "refresh_token": "eyJhbGc..."
}' localhost:50051 cuba.iam.auth.AuthService/RefreshToken
```

**错误码**：
- `UNAUTHENTICATED`: Refresh Token 无效或已过期

---

### 4. ValidateToken - 验证令牌

验证 Access Token 的有效性并返回用户信息。

**请求**：
```protobuf
message ValidateTokenRequest {
  string access_token = 1;      // 访问令牌（必填）
}
```

**响应**：
```protobuf
message ValidateTokenResponse {
  bool valid = 1;               // 是否有效
  string user_id = 2;           // 用户 ID
  string tenant_id = 3;         // 租户 ID
  repeated string permissions = 4; // 权限列表
  google.protobuf.Timestamp expires_at = 5; // 过期时间
}
```

**示例**：
```bash
grpcurl -plaintext -d '{
  "access_token": "eyJhbGc..."
}' localhost:50051 cuba.iam.auth.AuthService/ValidateToken
```

---

### 5. ChangePassword - 修改密码

修改当前用户的密码。

**请求**：
```protobuf
message ChangePasswordRequest {
  string user_id = 1;           // 用户 ID（必填）
  string old_password = 2;      // 旧密码（必填）
  string new_password = 3;      // 新密码（必填）
}
```

**响应**：
```protobuf
message ChangePasswordResponse {
  bool success = 1;             // 是否成功
  string message = 2;           // 消息
}
```

**示例**：
```bash
grpcurl -plaintext -d '{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "old_password": "OldPass123!",
  "new_password": "NewPass456!"
}' localhost:50051 cuba.iam.auth.AuthService/ChangePassword
```

**错误码**：
- `INVALID_ARGUMENT`: 密码强度不足
- `UNAUTHENTICATED`: 旧密码错误
- `FAILED_PRECONDITION`: 新密码与旧密码相同

---

### 6. RequestPasswordReset - 请求重置密码

发送密码重置邮件。

**请求**：
```protobuf
message RequestPasswordResetRequest {
  string email = 1;             // 邮箱（必填）
  string tenant_id = 2;         // 租户 ID（必填）
}
```

**响应**：
```protobuf
message RequestPasswordResetResponse {
  bool success = 1;             // 是否成功
  string message = 2;           // 消息
}
```

**示例**：
```bash
grpcurl -plaintext -d '{
  "email": "user@example.com",
  "tenant_id": "default"
}' localhost:50051 cuba.iam.auth.AuthService/RequestPasswordReset
```

**注意**：为防止用户枚举攻击，无论邮箱是否存在都返回成功。

---

### 7. ResetPassword - 重置密码

使用重置令牌设置新密码。

**请求**：
```protobuf
message ResetPasswordRequest {
  string email = 1;             // 邮箱（必填）
  string reset_token = 2;       // 重置令牌（必填）
  string new_password = 3;      // 新密码（必填）
}
```

**响应**：
```protobuf
message ResetPasswordResponse {
  bool success = 1;             // 是否成功
  string message = 2;           // 消息
}
```

**示例**：
```bash
grpcurl -plaintext -d '{
  "email": "user@example.com",
  "reset_token": "abc123...",
  "new_password": "NewPass789!"
}' localhost:50051 cuba.iam.auth.AuthService/ResetPassword
```

**错误码**：
- `INVALID_ARGUMENT`: 令牌格式错误或密码强度不足
- `UNAUTHENTICATED`: 令牌无效或已过期
- `NOT_FOUND`: 用户不存在

---

### 8. Enable2FA - 启用双因子认证

为用户启用 TOTP 双因子认证。

**请求**：
```protobuf
message Enable2FARequest {
  string user_id = 1;           // 用户 ID（必填）
  string method = 2;            // 方法：TOTP, SMS, EMAIL（必填）
  string verification_code = 3; // 验证码（可选，用于确认启用）
}
```

**响应**：
```protobuf
message Enable2FAResponse {
  string secret = 1;            // TOTP secret（Base32 编码）
  string qr_code_url = 2;       // QR 码 URL
  repeated string backup_codes = 3; // 备份码（仅首次返回）
  bool enabled = 4;             // 是否已启用
}
```

**示例**：

步骤 1：获取 QR 码
```bash
grpcurl -plaintext -d '{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "method": "TOTP"
}' localhost:50051 cuba.iam.auth.AuthService/Enable2FA
```

步骤 2：扫描 QR 码后，使用验证码确认
```bash
grpcurl -plaintext -d '{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "method": "TOTP",
  "verification_code": "123456"
}' localhost:50051 cuba.iam.auth.AuthService/Enable2FA
```

---

### 9. Disable2FA - 禁用双因子认证

禁用用户的双因子认证。

**请求**：
```protobuf
message Disable2FARequest {
  string user_id = 1;           // 用户 ID（必填）
  string password = 2;          // 密码验证（必填）
}
```

**响应**：
```protobuf
message Disable2FAResponse {
  bool success = 1;             // 是否成功
  string message = 2;           // 消息
}
```

**示例**：
```bash
grpcurl -plaintext -d '{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "password": "SecurePass123!"
}' localhost:50051 cuba.iam.auth.AuthService/Disable2FA
```

---

### 10. Verify2FA - 验证双因子认证

在登录后验证 2FA 代码。

**请求**：
```protobuf
message Verify2FARequest {
  string user_id = 1;           // 用户 ID（必填）
  string code = 2;              // 验证码（必填）
  string session_id = 3;        // 会话 ID（必填）
}
```

**响应**：
```protobuf
message Verify2FAResponse {
  bool success = 1;             // 是否成功
  string access_token = 2;      // 访问令牌
  string refresh_token = 3;     // 刷新令牌
  int64 expires_in = 4;         // 过期时间（秒）
}
```

**示例**：
```bash
grpcurl -plaintext -d '{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "code": "123456",
  "session_id": "session-abc123"
}' localhost:50051 cuba.iam.auth.AuthService/Verify2FA
```

**错误码**：
- `INVALID_ARGUMENT`: 验证码格式错误
- `UNAUTHENTICATED`: 验证码错误或已过期
- `NOT_FOUND`: 会话不存在

---

### 11. GetActiveSessions - 获取活跃会话

获取用户的所有活跃会话。

**请求**：
```protobuf
message GetActiveSessionsRequest {
  string user_id = 1;           // 用户 ID（必填）
}
```

**响应**：
```protobuf
message GetActiveSessionsResponse {
  repeated Session sessions = 1; // 会话列表
}

message Session {
  string id = 1;                // 会话 ID
  string user_id = 2;           // 用户 ID
  string device_info = 3;       // 设备信息
  string ip_address = 4;        // IP 地址
  string user_agent = 5;        // User Agent
  bool is_current = 6;          // 是否当前会话
  google.protobuf.Timestamp created_at = 7;     // 创建时间
  google.protobuf.Timestamp expires_at = 8;     // 过期时间
  google.protobuf.Timestamp last_activity_at = 9; // 最后活动时间
}
```

**示例**：
```bash
grpcurl -plaintext -d '{
  "user_id": "550e8400-e29b-41d4-a716-446655440000"
}' localhost:50051 cuba.iam.auth.AuthService/GetActiveSessions
```

---

### 12. RevokeSession - 撤销会话

撤销指定的会话。

**请求**：
```protobuf
message RevokeSessionRequest {
  string user_id = 1;           // 用户 ID（必填）
  string session_id = 2;        // 会话 ID（必填）
}
```

**响应**：
```protobuf
message RevokeSessionResponse {
  bool success = 1;             // 是否成功
}
```

**示例**：
```bash
grpcurl -plaintext -d '{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "session_id": "session-abc123"
}' localhost:50051 cuba.iam.auth.AuthService/RevokeSession
```

---

## WebAuthn API

### 13. StartWebAuthnRegistration - 开始 WebAuthn 注册

开始注册 WebAuthn 凭证（如 YubiKey、Touch ID）。

**请求**：
```protobuf
message StartWebAuthnRegistrationRequest {
  string user_id = 1;           // 用户 ID（必填）
  string credential_name = 2;   // 凭证名称（必填，如 "YubiKey 5"）
}
```

**响应**：
```protobuf
message StartWebAuthnRegistrationResponse {
  string challenge = 1;         // 挑战（Base64 编码）
  string rp_id = 2;             // Relying Party ID
  string rp_name = 3;           // Relying Party 名称
  string user_id = 4;           // 用户 ID（Base64 编码）
  string user_name = 5;         // 用户名
  string user_display_name = 6; // 用户显示名称
  repeated string exclude_credentials = 7; // 排除的凭证 ID
  string registration_state = 8; // 注册状态（用于完成注册）
}
```

**示例**：
```bash
grpcurl -plaintext -d '{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "credential_name": "YubiKey 5 NFC"
}' localhost:50051 cuba.iam.auth.AuthService/StartWebAuthnRegistration
```

---

### 14. FinishWebAuthnRegistration - 完成 WebAuthn 注册

完成 WebAuthn 凭证注册。

**请求**：
```protobuf
message FinishWebAuthnRegistrationRequest {
  string user_id = 1;           // 用户 ID（必填）
  string credential_name = 2;   // 凭证名称（必填）
  string registration_state = 3; // 注册状态（必填）
  string credential_response = 4; // 凭证响应（JSON 格式，必填）
}
```

**响应**：
```protobuf
message FinishWebAuthnRegistrationResponse {
  bool success = 1;             // 是否成功
  string credential_id = 2;     // 凭证 ID
  string message = 3;           // 消息
}
```

---

### 15. StartWebAuthnAuthentication - 开始 WebAuthn 认证

开始 WebAuthn 认证流程。

**请求**：
```protobuf
message StartWebAuthnAuthenticationRequest {
  string username = 1;          // 用户名或邮箱（必填）
  string tenant_id = 2;         // 租户 ID（必填）
}
```

**响应**：
```protobuf
message StartWebAuthnAuthenticationResponse {
  string challenge = 1;         // 挑战（Base64 编码）
  string rp_id = 2;             // Relying Party ID
  repeated string allow_credentials = 3; // 允许的凭证 ID
  string authentication_state = 4; // 认证状态（用于完成认证）
  string user_id = 5;           // 用户 ID
}
```

---

### 16. FinishWebAuthnAuthentication - 完成 WebAuthn 认证

完成 WebAuthn 认证并获取 Token。

**请求**：
```protobuf
message FinishWebAuthnAuthenticationRequest {
  string authentication_state = 1; // 认证状态（必填）
  string credential_response = 2;  // 凭证响应（JSON 格式，必填）
  string device_info = 3;       // 设备信息（可选）
  string ip_address = 4;        // IP 地址（可选）
}
```

**响应**：
```protobuf
message FinishWebAuthnAuthenticationResponse {
  string access_token = 1;      // 访问令牌
  string refresh_token = 2;     // 刷新令牌
  int64 expires_in = 3;         // 过期时间（秒）
  string token_type = 4;        // 令牌类型
  User user = 5;                // 用户信息
}
```

---

### 17. ListWebAuthnCredentials - 列出 WebAuthn 凭证

列出用户的所有 WebAuthn 凭证。

**请求**：
```protobuf
message ListWebAuthnCredentialsRequest {
  string user_id = 1;           // 用户 ID（必填）
}
```

**响应**：
```protobuf
message ListWebAuthnCredentialsResponse {
  repeated WebAuthnCredential credentials = 1;
}

message WebAuthnCredential {
  string id = 1;                // 凭证 ID
  string name = 2;              // 凭证名称
  repeated string transports = 3; // 传输方式
  bool backup_eligible = 4;     // 是否可备份
  bool backup_state = 5;        // 是否已备份
  google.protobuf.Timestamp created_at = 6;     // 创建时间
  google.protobuf.Timestamp last_used_at = 7;   // 最后使用时间
}
```

---

### 18. DeleteWebAuthnCredential - 删除 WebAuthn 凭证

删除指定的 WebAuthn 凭证。

**请求**：
```protobuf
message DeleteWebAuthnCredentialRequest {
  string user_id = 1;           // 用户 ID（必填）
  string credential_id = 2;     // 凭证 ID（必填）
}
```

**响应**：
```protobuf
message DeleteWebAuthnCredentialResponse {
  bool success = 1;             // 是否成功
  string message = 2;           // 消息
}
```

---

## 数据模型

### User - 用户

```protobuf
message User {
  string id = 1;                // 用户 ID
  string username = 2;          // 用户名
  string email = 3;             // 邮箱
  string display_name = 4;      // 显示名称
  string phone = 5;             // 电话
  string avatar_url = 6;        // 头像 URL
  string tenant_id = 7;         // 租户 ID
  repeated string role_ids = 8; // 角色 ID 列表
  string status = 9;            // 状态
  string language = 10;         // 语言
  string timezone = 11;         // 时区
  bool two_factor_enabled = 12; // 是否启用 2FA
  google.protobuf.Timestamp last_login_at = 13; // 最后登录时间
  AuditInfo audit_info = 14;    // 审计信息
}
```

### AuditInfo - 审计信息

```protobuf
message AuditInfo {
  google.protobuf.Timestamp created_at = 1;  // 创建时间
  string created_by = 2;                      // 创建人
  google.protobuf.Timestamp updated_at = 3;  // 更新时间
  string updated_by = 4;                      // 更新人
}
```
