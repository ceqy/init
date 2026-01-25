# UserService API 文档

## 服务定义

```protobuf
service UserService {
  // 用户 CRUD 操作
  rpc Register(RegisterRequest) returns (RegisterResponse);
  rpc GetUser(GetUserRequest) returns (GetUserResponse);
  rpc GetCurrentUser(GetCurrentUserRequest) returns (GetCurrentUserResponse);
  rpc UpdateUser(UpdateUserRequest) returns (UpdateUserResponse);
  rpc UpdateProfile(UpdateProfileRequest) returns (UpdateProfileResponse);
  rpc DeleteUser(DeleteUserRequest) returns (google.protobuf.Empty);
  rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
  
  // 用户状态管理
  rpc ActivateUser(ActivateUserRequest) returns (ActivateUserResponse);
  rpc DeactivateUser(DeactivateUserRequest) returns (DeactivateUserResponse);
  rpc LockUser(LockUserRequest) returns (LockUserResponse);
  rpc UnlockUser(UnlockUserRequest) returns (UnlockUserResponse);
  
  // 用户角色管理
  rpc AssignRoles(AssignRolesRequest) returns (AssignRolesResponse);
  rpc RemoveRoles(RemoveRolesRequest) returns (RemoveRolesResponse);
  rpc GetUserRoles(GetUserRolesRequest) returns (GetUserRolesResponse);
}
```

## API 方法

### 1. Register - 注册新用户

创建新用户账户。

**请求**：
```protobuf
message RegisterRequest {
  string username = 1;          // 用户名（必填，3-32 字符）
  string email = 2;             // 邮箱（必填）
  string password = 3;          // 密码（必填，最少 8 字符）
  string display_name = 4;      // 显示名称（可选）
  string phone = 5;             // 电话（可选）
  string tenant_id = 6;         // 租户 ID（必填）
  repeated string role_ids = 7; // 角色 ID 列表（可选）
}
```

**响应**：
```protobuf
message RegisterResponse {
  string user_id = 1;           // 用户 ID
  User user = 2;                // 用户信息
}
```

**示例**：
```bash
grpcurl -plaintext -d '{
  "username": "john.doe",
  "email": "john@example.com",
  "password": "SecurePass123!",
  "display_name": "John Doe",
  "phone": "+1234567890",
  "tenant_id": "default",
  "role_ids": ["user"]
}' localhost:50051 cuba.iam.user.UserService/Register
```

**错误码**：
- `INVALID_ARGUMENT`: 参数验证失败（用户名格式、密码强度等）
- `ALREADY_EXISTS`: 用户名或邮箱已存在

---

### 2. GetUser - 获取用户信息

根据用户 ID 获取用户详细信息。

**请求**：
```protobuf
message GetUserRequest {
  string user_id = 1;           // 用户 ID（必填）
}
```

**响应**：
```protobuf
message GetUserResponse {
  User user = 1;                // 用户信息
}
```

**示例**：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{"user_id": "550e8400-e29b-41d4-a716-446655440000"}' \
  localhost:50051 cuba.iam.user.UserService/GetUser
```

**错误码**：
- `NOT_FOUND`: 用户不存在
- `PERMISSION_DENIED`: 无权访问该用户信息

---

### 3. GetCurrentUser - 获取当前用户

获取当前登录用户的信息。

**请求**：
```protobuf
message GetCurrentUserRequest {
  string access_token = 1;      // 访问令牌（必填）
}
```

**响应**：
```protobuf
message GetCurrentUserResponse {
  User user = 1;                // 用户信息
}
```

**示例**：
```bash
grpcurl -plaintext -d '{
  "access_token": "eyJhbGc..."
}' localhost:50051 cuba.iam.user.UserService/GetCurrentUser
```

---

### 4. UpdateUser - 更新用户信息

更新用户的完整信息（管理员操作）。

**请求**：
```protobuf
message UpdateUserRequest {
  string user_id = 1;           // 用户 ID（必填）
  string username = 2;          // 用户名（可选）
  string email = 3;             // 邮箱（可选）
  string display_name = 4;      // 显示名称（可选）
  string phone = 5;             // 电话（可选）
  string avatar_url = 6;        // 头像 URL（可选）
  string language = 7;          // 语言（可选）
  string timezone = 8;          // 时区（可选）
  string status = 9;            // 状态（可选）
}
```

**响应**：
```protobuf
message UpdateUserResponse {
  User user = 1;                // 更新后的用户信息
}
```

**示例**：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "display_name": "John Smith",
    "phone": "+1234567891",
    "language": "zh-CN",
    "timezone": "Asia/Shanghai"
  }' localhost:50051 cuba.iam.user.UserService/UpdateUser
```

**错误码**：
- `NOT_FOUND`: 用户不存在
- `ALREADY_EXISTS`: 用户名或邮箱已被其他用户使用
- `PERMISSION_DENIED`: 无权更新该用户

---

### 5. UpdateProfile - 更新个人资料

用户更新自己的个人资料。

**请求**：
```protobuf
message UpdateProfileRequest {
  string user_id = 1;           // 用户 ID（必填）
  string display_name = 2;      // 显示名称（可选）
  string email = 3;             // 邮箱（可选）
  string phone = 4;             // 电话（可选）
  string avatar_url = 5;        // 头像 URL（可选）
  string language = 6;          // 语言（可选）
  string timezone = 7;          // 时区（可选）
}
```

**响应**：
```protobuf
message UpdateProfileResponse {
  User user = 1;                // 更新后的用户信息
}
```

**示例**：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "display_name": "John Doe",
    "avatar_url": "https://example.com/avatar.jpg",
    "language": "en-US",
    "timezone": "America/New_York"
  }' localhost:50051 cuba.iam.user.UserService/UpdateProfile
```

---

### 6. DeleteUser - 删除用户

删除用户账户（软删除）。

**请求**：
```protobuf
message DeleteUserRequest {
  string user_id = 1;           // 用户 ID（必填）
}
```

**响应**：
```protobuf
google.protobuf.Empty
```

**示例**：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{"user_id": "550e8400-e29b-41d4-a716-446655440000"}' \
  localhost:50051 cuba.iam.user.UserService/DeleteUser
```

**错误码**：
- `NOT_FOUND`: 用户不存在
- `PERMISSION_DENIED`: 无权删除该用户
- `FAILED_PRECONDITION`: 无法删除（如最后一个管理员）

---

### 7. ListUsers - 用户列表查询

分页查询用户列表，支持多种过滤条件。

**请求**：
```protobuf
message ListUsersRequest {
  string tenant_id = 1;         // 租户 ID 过滤（可选）
  string status = 2;            // 状态过滤（可选）
  string search = 3;            // 搜索关键词（可选）
  repeated string role_ids = 4; // 角色 ID 过滤（可选）
  int32 page = 100;             // 页码（默认 1）
  int32 page_size = 101;        // 每页大小（默认 20，最大 100）
}
```

**响应**：
```protobuf
message ListUsersResponse {
  repeated User users = 1;      // 用户列表
  int32 page = 2;               // 当前页码
  int32 page_size = 3;          // 每页大小
  int64 total = 4;              // 总记录数
  int32 total_pages = 5;        // 总页数
}
```

**示例**：

查询所有用户：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{"page": 1, "page_size": 20}' \
  localhost:50051 cuba.iam.user.UserService/ListUsers
```

按状态过滤：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{
    "status": "ACTIVE",
    "page": 1,
    "page_size": 20
  }' localhost:50051 cuba.iam.user.UserService/ListUsers
```

搜索用户：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{
    "search": "john",
    "page": 1,
    "page_size": 20
  }' localhost:50051 cuba.iam.user.UserService/ListUsers
```

按角色过滤：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{
    "role_ids": ["admin", "manager"],
    "page": 1,
    "page_size": 20
  }' localhost:50051 cuba.iam.user.UserService/ListUsers
```

---

## 用户状态管理

### 8. ActivateUser - 激活用户

激活已停用的用户账户。

**请求**：
```protobuf
message ActivateUserRequest {
  string user_id = 1;           // 用户 ID（必填）
}
```

**响应**：
```protobuf
message ActivateUserResponse {
  User user = 1;                // 用户信息
}
```

**示例**：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{"user_id": "550e8400-e29b-41d4-a716-446655440000"}' \
  localhost:50051 cuba.iam.user.UserService/ActivateUser
```

---

### 9. DeactivateUser - 停用用户

停用用户账户，用户将无法登录。

**请求**：
```protobuf
message DeactivateUserRequest {
  string user_id = 1;           // 用户 ID（必填）
  string reason = 2;            // 停用原因（可选）
}
```

**响应**：
```protobuf
message DeactivateUserResponse {
  User user = 1;                // 用户信息
}
```

**示例**：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "reason": "长期未使用"
  }' localhost:50051 cuba.iam.user.UserService/DeactivateUser
```

---

### 10. LockUser - 锁定用户

锁定用户账户（通常因安全原因）。

**请求**：
```protobuf
message LockUserRequest {
  string user_id = 1;           // 用户 ID（必填）
  string reason = 2;            // 锁定原因（可选）
}
```

**响应**：
```protobuf
message LockUserResponse {
  User user = 1;                // 用户信息
}
```

**示例**：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "reason": "可疑活动"
  }' localhost:50051 cuba.iam.user.UserService/LockUser
```

---

### 11. UnlockUser - 解锁用户

解锁被锁定的用户账户。

**请求**：
```protobuf
message UnlockUserRequest {
  string user_id = 1;           // 用户 ID（必填）
}
```

**响应**：
```protobuf
message UnlockUserResponse {
  User user = 1;                // 用户信息
}
```

**示例**：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{"user_id": "550e8400-e29b-41d4-a716-446655440000"}' \
  localhost:50051 cuba.iam.user.UserService/UnlockUser
```

---

## 用户角色管理

### 12. AssignRoles - 分配角色

为用户分配一个或多个角色。

**请求**：
```protobuf
message AssignRolesRequest {
  string user_id = 1;           // 用户 ID（必填）
  repeated string role_ids = 2; // 角色 ID 列表（必填）
}
```

**响应**：
```protobuf
message AssignRolesResponse {
  User user = 1;                // 用户信息
}
```

**示例**：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "role_ids": ["admin", "manager"]
  }' localhost:50051 cuba.iam.user.UserService/AssignRoles
```

---

### 13. RemoveRoles - 移除角色

移除用户的一个或多个角色。

**请求**：
```protobuf
message RemoveRolesRequest {
  string user_id = 1;           // 用户 ID（必填）
  repeated string role_ids = 2; // 角色 ID 列表（必填）
}
```

**响应**：
```protobuf
message RemoveRolesResponse {
  User user = 1;                // 用户信息
}
```

**示例**：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "role_ids": ["manager"]
  }' localhost:50051 cuba.iam.user.UserService/RemoveRoles
```

---

### 14. GetUserRoles - 获取用户角色

获取用户的所有角色。

**请求**：
```protobuf
message GetUserRolesRequest {
  string user_id = 1;           // 用户 ID（必填）
}
```

**响应**：
```protobuf
message GetUserRolesResponse {
  repeated Role roles = 1;      // 角色列表
}

message Role {
  string id = 1;                // 角色 ID
  string code = 2;              // 角色代码
  string name = 3;              // 角色名称
  string description = 4;       // 角色描述
}
```

**示例**：
```bash
grpcurl -plaintext \
  -H "Authorization: Bearer eyJhbGc..." \
  -d '{"user_id": "550e8400-e29b-41d4-a716-446655440000"}' \
  localhost:50051 cuba.iam.user.UserService/GetUserRoles
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
  string status = 9;            // 状态 (ACTIVE, INACTIVE, LOCKED)
  string language = 10;         // 语言
  string timezone = 11;         // 时区
  bool two_factor_enabled = 12; // 是否启用双因子认证
  google.protobuf.Timestamp last_login_at = 13; // 最后登录时间
  AuditInfo audit_info = 90;    // 审计信息
}
```

### 用户状态

| 状态 | 说明 |
|------|------|
| ACTIVE | 活跃 - 可以正常登录 |
| INACTIVE | 停用 - 无法登录，可以重新激活 |
| LOCKED | 锁定 - 因安全原因锁定，需要管理员解锁 |

## 业务规则

### 用户名规则

- 长度：3-32 字符
- 允许字符：字母、数字、下划线、连字符
- 不允许：空格、特殊字符
- 唯一性：租户内唯一

### 邮箱规则

- 必须是有效的邮箱格式
- 唯一性：租户内唯一
- 用于密码重置和通知

### 密码规则

- 最小长度：8 字符
- 必须包含：大写字母、小写字母、数字
- 推荐包含：特殊字符
- 不能与用户名相同

### 角色分配规则

- 用户可以拥有多个角色
- 角色权限累加
- 至少保留一个管理员用户

## 使用场景

### 场景 1：用户自助注册

```javascript
// 1. 用户填写注册表单
const registerRequest = {
  username: 'john.doe',
  email: 'john@example.com',
  password: 'SecurePass123!',
  display_name: 'John Doe',
  tenant_id: 'default'
};

// 2. 调用注册 API
const response = await userService.register(registerRequest);

// 3. 发送验证邮件（可选）
await emailService.sendVerificationEmail(response.user.email);

// 4. 自动登录或跳转到登录页
navigate('/login');
```

### 场景 2：管理员创建用户

```javascript
// 1. 管理员填写用户信息
const registerRequest = {
  username: 'employee001',
  email: 'employee@company.com',
  password: generateRandomPassword(),
  display_name: 'Employee Name',
  tenant_id: getCurrentTenantId(),
  role_ids: ['employee', 'sales']
};

// 2. 创建用户
const response = await userService.register(registerRequest);

// 3. 发送欢迎邮件（包含临时密码）
await emailService.sendWelcomeEmail(
  response.user.email,
  registerRequest.password
);
```

### 场景 3：用户列表管理

```javascript
// 1. 加载用户列表
const listRequest = {
  tenant_id: getCurrentTenantId(),
  status: 'ACTIVE',
  page: 1,
  page_size: 20
};

const response = await userService.listUsers(listRequest);

// 2. 显示用户列表
displayUserTable(response.users);

// 3. 分页控制
displayPagination({
  current: response.page,
  total: response.total_pages
});
```

### 场景 4：用户状态管理

```javascript
// 停用长期未登录的用户
const inactiveUsers = await findInactiveUsers(90); // 90 天未登录

for (const user of inactiveUsers) {
  await userService.deactivateUser({
    user_id: user.id,
    reason: '90 天未登录'
  });
  
  await emailService.sendDeactivationNotice(user.email);
}
```
