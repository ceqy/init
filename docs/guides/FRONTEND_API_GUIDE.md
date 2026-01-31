# ERP ERP 前端接入指南

## 服务端点

### IAM Identity 服务
- **gRPC 端点**: `localhost:50051`
- **健康检查**: `http://localhost:51051/health`
- **Metrics**: `http://localhost:51051/metrics`

### Gateway（如果使用）
- **HTTP 端点**: `http://localhost:8080`

## 认证流程

### 1. 用户注册

**Proto 定义**: `proto/iam/user.proto`

```protobuf
rpc RegisterUser(RegisterUserRequest) returns (RegisterUserResponse);

message RegisterUserRequest {
  string username = 1;      // 用户名
  string email = 2;         // 邮箱
  string password = 3;      // 密码
  string phone = 4;         // 手机号（可选）
}

message RegisterUserResponse {
  string user_id = 1;       // 用户 ID
  string message = 2;       // 提示信息
}
```

**前端调用示例（gRPC-Web）**:
```typescript
const request = new RegisterUserRequest();
request.setUsername("john_doe");
request.setEmail("john@example.com");
request.setPassword("SecurePass123!");
request.setPhone("+1234567890");

const metadata = { 'tenant-id': 'tenant-123' };

client.registerUser(request, metadata, (err, response) => {
  if (err) {
    console.error('注册失败:', err.message);
  } else {
    console.log('用户 ID:', response.getUserId());
    console.log('提示:', response.getMessage());
  }
});
```

### 2. 用户登录

**Proto 定义**: `proto/iam/auth.proto`

```protobuf
rpc Login(LoginRequest) returns (LoginResponse);

message LoginRequest {
  string username = 1;      // 用户名或邮箱
  string password = 2;      // 密码
  string ip_address = 3;    // IP 地址
  string user_agent = 4;    // User Agent
}

message LoginResponse {
  string access_token = 1;  // 访问令牌
  string refresh_token = 2; // 刷新令牌
  int64 expires_in = 3;     // 过期时间（秒）
  bool requires_2fa = 4;    // 是否需要 2FA
  string session_id = 5;    // 会话 ID
}
```

**前端调用示例**:
```typescript
const request = new LoginRequest();
request.setUsername("john_doe");
request.setPassword("SecurePass123!");
request.setIpAddress("192.168.1.100");
request.setUserAgent(navigator.userAgent);

const metadata = { 'tenant-id': 'tenant-123' };

client.login(request, metadata, (err, response) => {
  if (err) {
    console.error('登录失败:', err.message);
  } else {
    if (response.getRequires2fa()) {
      // 跳转到 2FA 验证页面
      showTwoFactorAuth(response.getSessionId());
    } else {
      // 保存 token
      localStorage.setItem('access_token', response.getAccessToken());
      localStorage.setItem('refresh_token', response.getRefreshToken());
      // 跳转到主页
      navigateToHome();
    }
  }
});
```

### 3. 双因素认证（2FA）

#### 3.1 启用 TOTP
```protobuf
rpc EnableTOTP(EnableTOTPRequest) returns (EnableTOTPResponse);

message EnableTOTPRequest {
  string user_id = 1;       // 用户 ID
}

message EnableTOTPResponse {
  string secret = 1;        // TOTP 密钥
  string qr_code_url = 2;   // 二维码 URL
  repeated string backup_codes = 3; // 备份码
}
```

**前端调用**:
```typescript
const request = new EnableTOTPRequest();
request.setUserId(userId);

client.enableTOTP(request, metadata, (err, response) => {
  if (!err) {
    // 显示二维码
    displayQRCode(response.getQrCodeUrl());
    // 显示备份码
    displayBackupCodes(response.getBackupCodesList());
  }
});
```

#### 3.2 验证 TOTP
```protobuf
rpc VerifyTOTP(VerifyTOTPRequest) returns (VerifyTOTPResponse);

message VerifyTOTPRequest {
  string session_id = 1;    // 会话 ID
  string code = 2;          // 6 位验证码
}

message VerifyTOTPResponse {
  string access_token = 1;  // 访问令牌
  string refresh_token = 2; // 刷新令牌
  int64 expires_in = 3;     // 过期时间
}
```

### 4. 密码重置

#### 4.1 请求密码重置
```protobuf
rpc RequestPasswordReset(RequestPasswordResetRequest) returns (RequestPasswordResetResponse);

message RequestPasswordResetRequest {
  string email = 1;         // 邮箱
}

message RequestPasswordResetResponse {
  string message = 1;       // 提示信息
}
```

**前端调用**:
```typescript
const request = new RequestPasswordResetRequest();
request.setEmail("john@example.com");

client.requestPasswordReset(request, metadata, (err, response) => {
  if (!err) {
    showMessage('密码重置邮件已发送，请检查您的邮箱');
  }
});
```

#### 4.2 重置密码
```protobuf
rpc ResetPassword(ResetPasswordRequest) returns (ResetPasswordResponse);

message ResetPasswordRequest {
  string token = 1;         // 重置令牌（从邮件链接获取）
  string new_password = 2;  // 新密码
}

message ResetPasswordResponse {
  string message = 1;       // 提示信息
}
```

### 5. 邮箱验证

#### 5.1 发送验证邮件
```protobuf
rpc SendEmailVerification(SendEmailVerificationRequest) returns (SendEmailVerificationResponse);

message SendEmailVerificationRequest {
  string user_id = 1;       // 用户 ID
  string email = 2;         // 邮箱
}

message SendEmailVerificationResponse {
  string message = 1;       // 提示信息
}
```

#### 5.2 验证邮箱
```protobuf
rpc VerifyEmail(VerifyEmailRequest) returns (VerifyEmailResponse);

message VerifyEmailRequest {
  string user_id = 1;       // 用户 ID
  string code = 2;          // 6 位验证码
}

message VerifyEmailResponse {
  bool success = 1;         // 是否成功
  string message = 2;       // 提示信息
}
```

### 6. 手机验证

#### 6.1 发送验证短信
```protobuf
rpc SendPhoneVerification(SendPhoneVerificationRequest) returns (SendPhoneVerificationResponse);

message SendPhoneVerificationRequest {
  string user_id = 1;       // 用户 ID
  string phone = 2;         // 手机号
}

message SendPhoneVerificationResponse {
  string message = 1;       // 提示信息
}
```

#### 6.2 验证手机
```protobuf
rpc VerifyPhone(VerifyPhoneRequest) returns (VerifyPhoneResponse);

message VerifyPhoneRequest {
  string user_id = 1;       // 用户 ID
  string code = 2;          // 6 位验证码
}

message VerifyPhoneResponse {
  bool success = 1;         // 是否成功
  string message = 2;       // 提示信息
}
```

## OAuth2 授权流程

### 1. 创建 OAuth Client

```protobuf
rpc CreateClient(CreateClientRequest) returns (CreateClientResponse);

message CreateClientRequest {
  string name = 1;                    // Client 名称
  repeated string redirect_uris = 2;  // 重定向 URI
  repeated string grant_types = 3;    // 授权类型
  repeated string scopes = 4;         // 权限范围
  bool public_client = 6;             // 是否公开客户端
}

message CreateClientResponse {
  string client_id = 1;               // Client ID
  string client_secret = 2;           // Client Secret
}
```

### 2. 授权码流程

#### 2.1 获取授权码
```protobuf
rpc Authorize(AuthorizeRequest) returns (AuthorizeResponse);

message AuthorizeRequest {
  string client_id = 1;               // Client ID
  string redirect_uri = 2;            // 重定向 URI
  string response_type = 3;           // "code"
  string scope = 4;                   // 权限范围
  string state = 5;                   // 状态参数
  string code_challenge = 6;          // PKCE challenge
  string code_challenge_method = 7;   // "S256" 或 "plain"
  string user_id = 8;                 // 用户 ID（已认证）
}

message AuthorizeResponse {
  string code = 1;                    // 授权码
  string state = 2;                   // 状态参数
}
```

**前端流程**:
```typescript
// 1. 生成 PKCE 参数
const codeVerifier = generateRandomString(128);
const codeChallenge = await sha256(codeVerifier);

// 2. 保存 code_verifier
sessionStorage.setItem('code_verifier', codeVerifier);

// 3. 请求授权码
const request = new AuthorizeRequest();
request.setClientId("your-client-id");
request.setRedirectUri("http://localhost:3000/callback");
request.setResponseType("code");
request.setScope("read write");
request.setState(generateRandomString(32));
request.setCodeChallenge(codeChallenge);
request.setCodeChallengeMethod("S256");
request.setUserId(currentUserId);

client.authorize(request, metadata, (err, response) => {
  if (!err) {
    const code = response.getCode();
    const state = response.getState();
    // 重定向到回调 URL
    window.location.href = `${redirectUri}?code=${code}&state=${state}`;
  }
});
```

#### 2.2 交换 Token
```protobuf
rpc Token(TokenRequest) returns (TokenResponse);

message TokenRequest {
  string grant_type = 1;              // "authorization_code"
  string code = 2;                    // 授权码
  string redirect_uri = 3;            // 重定向 URI
  string client_id = 4;               // Client ID
  string client_secret = 5;           // Client Secret
  string code_verifier = 6;           // PKCE verifier
}

message TokenResponse {
  string access_token = 1;            // Access Token
  string token_type = 2;              // "Bearer"
  int64 expires_in = 3;               // 过期时间（秒）
  string refresh_token = 4;           // Refresh Token
  string scope = 5;                   // 权限范围
}
```

**前端流程**:
```typescript
// 从 URL 获取授权码
const urlParams = new URLSearchParams(window.location.search);
const code = urlParams.get('code');
const codeVerifier = sessionStorage.getItem('code_verifier');

const request = new TokenRequest();
request.setGrantType("authorization_code");
request.setCode(code);
request.setRedirectUri("http://localhost:3000/callback");
request.setClientId("your-client-id");
request.setClientSecret("your-client-secret");
request.setCodeVerifier(codeVerifier);

client.token(request, metadata, (err, response) => {
  if (!err) {
    localStorage.setItem('access_token', response.getAccessToken());
    localStorage.setItem('refresh_token', response.getRefreshToken());
  }
});
```

### 3. 刷新 Token
```protobuf
rpc RefreshToken(RefreshTokenRequest) returns (TokenResponse);

message RefreshTokenRequest {
  string refresh_token = 1;           // Refresh Token
  string client_id = 2;               // Client ID
  string client_secret = 3;           // Client Secret
}
```

### 4. 撤销 Token
```protobuf
rpc RevokeToken(RevokeTokenRequest) returns (google.protobuf.Empty);

message RevokeTokenRequest {
  string token = 1;                   // Token
  string client_id = 3;               // Client ID
  string client_secret = 4;           // Client Secret
}
```

## 用户管理

### 1. 获取用户信息
```protobuf
rpc GetUser(GetUserRequest) returns (GetUserResponse);

message GetUserRequest {
  string user_id = 1;                 // 用户 ID
}

message GetUserResponse {
  User user = 1;                      // 用户信息
}

message User {
  string id = 1;                      // 用户 ID
  string username = 2;                // 用户名
  string email = 3;                   // 邮箱
  string phone = 4;                   // 手机号
  bool email_verified = 5;            // 邮箱是否验证
  bool phone_verified = 6;            // 手机是否验证
  bool totp_enabled = 7;              // 是否启用 TOTP
  google.protobuf.Timestamp created_at = 8;  // 创建时间
}
```

### 2. 更新用户信息
```protobuf
rpc UpdateUser(UpdateUserRequest) returns (UpdateUserResponse);

message UpdateUserRequest {
  string user_id = 1;                 // 用户 ID
  string email = 2;                   // 新邮箱
  string phone = 3;                   // 新手机号
}
```

### 3. 修改密码
```protobuf
rpc ChangePassword(ChangePasswordRequest) returns (ChangePasswordResponse);

message ChangePasswordRequest {
  string user_id = 1;                 // 用户 ID
  string old_password = 2;            // 旧密码
  string new_password = 3;            // 新密码
}
```

## 请求头（Metadata）

所有请求必须包含以下 metadata：

```typescript
const metadata = {
  'tenant-id': 'your-tenant-id',      // 租户 ID（必需）
  'authorization': `Bearer ${token}`, // 访问令牌（需要认证的接口）
};
```

## 错误处理

### gRPC 错误码

| 错误码 | 说明 | 处理方式 |
|--------|------|----------|
| `UNAUTHENTICATED` | 未认证 | 跳转到登录页 |
| `PERMISSION_DENIED` | 权限不足 | 显示权限错误 |
| `INVALID_ARGUMENT` | 参数错误 | 显示表单验证错误 |
| `NOT_FOUND` | 资源不存在 | 显示 404 |
| `ALREADY_EXISTS` | 资源已存在 | 显示冲突错误 |
| `INTERNAL` | 服务器错误 | 显示通用错误 |

### 错误处理示例

```typescript
client.login(request, metadata, (err, response) => {
  if (err) {
    switch (err.code) {
      case grpc.status.UNAUTHENTICATED:
        showError('用户名或密码错误');
        break;
      case grpc.status.INVALID_ARGUMENT:
        showError('请求参数错误');
        break;
      case grpc.status.INTERNAL:
        showError('服务器错误，请稍后重试');
        break;
      default:
        showError(err.message);
    }
  } else {
    // 处理成功响应
  }
});
```

## Token 管理

### 1. 存储 Token
```typescript
// 登录成功后
localStorage.setItem('access_token', accessToken);
localStorage.setItem('refresh_token', refreshToken);
localStorage.setItem('token_expires_at', Date.now() + expiresIn * 1000);
```

### 2. 自动刷新 Token
```typescript
async function refreshAccessToken() {
  const refreshToken = localStorage.getItem('refresh_token');
  
  const request = new RefreshTokenRequest();
  request.setRefreshToken(refreshToken);
  request.setClientId(clientId);
  request.setClientSecret(clientSecret);
  
  return new Promise((resolve, reject) => {
    client.refreshToken(request, metadata, (err, response) => {
      if (err) {
        // Token 刷新失败，跳转到登录页
        logout();
        reject(err);
      } else {
        localStorage.setItem('access_token', response.getAccessToken());
        localStorage.setItem('refresh_token', response.getRefreshToken());
        localStorage.setItem('token_expires_at', Date.now() + response.getExpiresIn() * 1000);
        resolve(response);
      }
    });
  });
}

// 在请求拦截器中检查 token 是否过期
async function getValidToken() {
  const expiresAt = localStorage.getItem('token_expires_at');
  const now = Date.now();
  
  // 提前 5 分钟刷新
  if (now + 5 * 60 * 1000 > expiresAt) {
    await refreshAccessToken();
  }
  
  return localStorage.getItem('access_token');
}
```

### 3. 登出
```typescript
async function logout() {
  const token = localStorage.getItem('access_token');
  
  // 撤销 token
  const request = new RevokeTokenRequest();
  request.setToken(token);
  request.setClientId(clientId);
  request.setClientSecret(clientSecret);
  
  client.revokeToken(request, metadata, () => {
    // 清除本地存储
    localStorage.removeItem('access_token');
    localStorage.removeItem('refresh_token');
    localStorage.removeItem('token_expires_at');
    
    // 跳转到登录页
    window.location.href = '/login';
  });
}
```

## 前端库推荐

### gRPC-Web
```bash
npm install grpc-web
npm install google-protobuf
```

### 生成客户端代码
```bash
# 安装 protoc 和 protoc-gen-grpc-web
brew install protobuf

# 生成 JavaScript 客户端
protoc -I=proto \
  --js_out=import_style=commonjs:./src/generated \
  --grpc-web_out=import_style=typescript,mode=grpcwebtext:./src/generated \
  proto/iam/auth.proto \
  proto/iam/user.proto \
  proto/iam/oauth.proto
```

### TypeScript 客户端示例
```typescript
import { AuthServiceClient } from './generated/AuthServiceClientPb';
import { LoginRequest } from './generated/auth_pb';

const client = new AuthServiceClient('http://localhost:8080');

async function login(username: string, password: string) {
  const request = new LoginRequest();
  request.setUsername(username);
  request.setPassword(password);
  request.setIpAddress(await getClientIP());
  request.setUserAgent(navigator.userAgent);
  
  const metadata = {
    'tenant-id': getTenantId(),
  };
  
  return new Promise((resolve, reject) => {
    client.login(request, metadata, (err, response) => {
      if (err) {
        reject(err);
      } else {
        resolve(response.toObject());
      }
    });
  });
}
```

## 完整登录流程示例

```typescript
// 1. 用户输入用户名和密码
async function handleLogin(username: string, password: string) {
  try {
    const response = await login(username, password);
    
    if (response.requires2fa) {
      // 需要 2FA 验证
      showTwoFactorAuthPage(response.sessionId);
    } else {
      // 登录成功
      saveTokens(response.accessToken, response.refreshToken, response.expiresIn);
      navigateToHome();
    }
  } catch (error) {
    handleLoginError(error);
  }
}

// 2. 2FA 验证
async function handleTwoFactorAuth(sessionId: string, code: string) {
  try {
    const response = await verifyTOTP(sessionId, code);
    saveTokens(response.accessToken, response.refreshToken, response.expiresIn);
    navigateToHome();
  } catch (error) {
    showError('验证码错误');
  }
}

// 3. 保存 tokens
function saveTokens(accessToken: string, refreshToken: string, expiresIn: number) {
  localStorage.setItem('access_token', accessToken);
  localStorage.setItem('refresh_token', refreshToken);
  localStorage.setItem('token_expires_at', (Date.now() + expiresIn * 1000).toString());
}

// 4. 获取用户信息
async function getUserProfile() {
  const token = await getValidToken();
  const userId = parseJWT(token).sub;
  
  const request = new GetUserRequest();
  request.setUserId(userId);
  
  const metadata = {
    'tenant-id': getTenantId(),
    'authorization': `Bearer ${token}`,
  };
  
  return new Promise((resolve, reject) => {
    client.getUser(request, metadata, (err, response) => {
      if (err) reject(err);
      else resolve(response.getUser().toObject());
    });
  });
}
```

## 测试工具

### grpcurl（命令行测试）
```bash
# 安装
brew install grpcurl

# 列出所有服务
grpcurl -plaintext localhost:50051 list

# 列出服务方法
grpcurl -plaintext localhost:50051 list cuba.iam.auth.AuthService

# 调用登录接口
grpcurl -plaintext \
  -d '{"username":"john_doe","password":"SecurePass123!"}' \
  -H 'tenant-id: tenant-123' \
  localhost:50051 \
  cuba.iam.auth.AuthService/Login
```

### Postman（图形化测试）
1. 创建 gRPC 请求
2. 导入 proto 文件
3. 设置 metadata
4. 发送请求

## 相关文档

- [API 文档](docs/api/README.md)
- [认证服务 API](docs/api/iam/auth-service.md)
- [用户服务 API](docs/api/iam/user-service.md)
- [安全最佳实践](docs/guides/security.md)
- [多租户指南](docs/guides/multi-tenancy.md)
