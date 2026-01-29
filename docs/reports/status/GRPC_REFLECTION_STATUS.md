# gRPC 反射实现状态

## ✅ 实现完成

gRPC 反射功能已成功实现并验证！

## 已完成的工作

1. ✅ 添加 `tonic-reflection = "0.13"` 依赖（匹配 tonic 0.13 版本）
2. ✅ 更新 `build.rs` 生成文件描述符集（`.bin` 文件）
3. ✅ 在 `mod.rs` 中导出 `FILE_DESCRIPTOR_SET`
4. ✅ 在 `main.rs` 中注册反射服务
5. ✅ 修复 `build()` 弃用警告（改用 `build_v1()`）
6. ✅ 修复 proto 模块导入冲突
7. ✅ 修复 3 个 `tenant_id` 字段访问错误
8. ✅ 添加 `extract_tenant_id()` 辅助方法
9. ✅ 修复版本兼容性问题（tonic-reflection 0.12 → 0.13）
10. ✅ 服务编译成功并启动
11. ✅ 反射 API 验证通过

## 修复的问题

### 1. tenant_id 字段访问错误

**问题**：代码尝试从请求体中访问 `tenant_id` 字段，但这些 proto 消息中没有定义该字段。

**修复**：
- 添加 `extract_tenant_id()` 辅助方法从 metadata 获取 tenant_id
- 在 `reset_password()` 中：在 `into_inner()` 之前提取 tenant_id
- 在 `verify_2fa()` 中：在 `into_inner()` 之前提取 tenant_id
- 在 `finish_webauthn_authentication()` 中：在 `into_inner()` 之前提取 tenant_id

### 2. 版本兼容性问题

**问题**：`tonic-reflection = "0.12"` 与 `tonic = "0.13"` 不兼容

**修复**：升级到 `tonic-reflection = "0.13"`

## 临时解决方案（测试反射）

如果只是想测试反射功能，可以：

### 方案 1：使用 proto 文件直接测试

```bash
# 使用 proto 文件（不需要反射）
grpcurl -plaintext \
  -proto proto/iam/auth.proto \
  -import-path proto \
  -d '{"username":"test","password":"pass","ip_address":"127.0.0.1","user_agent":"grpcurl"}' \
  -H 'tenant-id: tenant-123' \
  localhost:50051 \
  cuba.iam.auth.AuthService/Login
```

### 方案 2：注释掉错误的方法

临时注释掉 `auth_service_impl.rs` 中的这 3 个方法：
- `reset_password()`
- `verify_2fa()` 
- `finish_webauthn_authentication()`

然后重新编译测试反射。

### 方案 3：修复这 3 个错误

找到并修复访问 `.tenant_id` 的代码。

## 验证反射是否工作

一旦编译通过，使用以下命令验证：

```bash
# 1. 列出所有服务
grpcurl -plaintext localhost:50051 list

# 预期输出：
# cuba.iam.auth.AuthService
# cuba.iam.user.UserService
# grpc.reflection.v1.ServerReflection  ← 这个表示反射已启用

# 2. 列出服务方法
grpcurl -plaintext localhost:50051 list cuba.iam.auth.AuthService

# 3. 查看方法详情
grpcurl -plaintext localhost:50051 describe cuba.iam.auth.AuthService.Login

# 4. 测试调用
grpcurl -plaintext \
  -d '{"username":"test","password":"pass","ip_address":"127.0.0.1","user_agent":"grpcurl"}' \
  -H 'tenant-id: tenant-123' \
  localhost:50051 \
  cuba.iam.auth.AuthService/Login
```

## 反射实现代码

### build.rs
```rust
tonic_build::configure()
    .build_server(true)
    .build_client(false)
    .file_descriptor_set_path("src/auth/api/grpc/auth_descriptor.bin")
    .out_dir("src/auth/api/grpc")
    .compile_protos(&["../../proto/iam/auth.proto"], &["../../proto"])
    .expect("Failed to compile auth.proto");
```

### mod.rs
```rust
pub mod proto {
    tonic::include_proto!("cuba.iam.auth");
    
    pub const FILE_DESCRIPTOR_SET: &[u8] = 
        include_bytes!("auth_descriptor.bin");
}
```

### main.rs
```rust
use tonic_reflection::server::Builder as ReflectionBuilder;

let reflection_service = ReflectionBuilder::configure()
    .register_encoded_file_descriptor_set(auth::api::grpc::proto::FILE_DESCRIPTOR_SET)
    .register_encoded_file_descriptor_set(user::api::grpc::proto::FILE_DESCRIPTOR_SET)
    .build_v1()
    .map_err(|e| cuba_errors::AppError::internal(format!("Failed to build reflection service: {}", e)))?;

server
    .add_service(AuthServiceServer::new(auth_service))
    .add_service(UserServiceServer::new(user_service))
    .add_service(reflection_service)  // 添加反射服务
    .serve_with_shutdown(addr, cuba_bootstrap::shutdown_signal())
    .await
```

## 下一步

1. 修复 3 个 `tenant_id` 字段访问错误
2. 重新编译服务
3. 启动服务并测试反射功能
4. 验证 grpcurl 可以正常工作

## 相关文档

- [GRPC_REFLECTION_FIX.md](GRPC_REFLECTION_FIX.md) - 完整的反射实现指南
- [FRONTEND_API_GUIDE.md](FRONTEND_API_GUIDE.md) - 前端接入指南


## 验证结果

### 1. 列出所有服务

```bash
❯ grpcurl -plaintext localhost:50051 list
cuba.iam.auth.AuthService
cuba.iam.user.UserService
grpc.reflection.v1.ServerReflection  ← 反射服务已启用
```

### 2. 列出服务方法

```bash
❯ grpcurl -plaintext localhost:50051 list cuba.iam.auth.AuthService
cuba.iam.auth.AuthService.ChangePassword
cuba.iam.auth.AuthService.DeleteWebAuthnCredential
cuba.iam.auth.AuthService.Disable2FA
cuba.iam.auth.AuthService.Enable2FA
cuba.iam.auth.AuthService.FinishWebAuthnAuthentication
cuba.iam.auth.AuthService.FinishWebAuthnRegistration
cuba.iam.auth.AuthService.GetActiveSessions
cuba.iam.auth.AuthService.ListWebAuthnCredentials
cuba.iam.auth.AuthService.Login
cuba.iam.auth.AuthService.Logout
cuba.iam.auth.AuthService.RefreshToken
cuba.iam.auth.AuthService.RequestPasswordReset
cuba.iam.auth.AuthService.ResetPassword
cuba.iam.auth.AuthService.RevokeSession
cuba.iam.auth.AuthService.StartWebAuthnAuthentication
cuba.iam.auth.AuthService.StartWebAuthnRegistration
cuba.iam.auth.AuthService.ValidateToken
cuba.iam.auth.AuthService.Verify2FA
```

### 3. 查看方法详情

```bash
❯ grpcurl -plaintext localhost:50051 describe cuba.iam.auth.AuthService.Login
cuba.iam.auth.AuthService.Login is a method:
// 用户登录
rpc Login ( .cuba.iam.auth.LoginRequest ) returns ( .cuba.iam.auth.LoginResponse );
```

### 4. 查看消息结构

```bash
❯ grpcurl -plaintext localhost:50051 describe cuba.iam.auth.LoginRequest
cuba.iam.auth.LoginRequest is a message:
// 登录请求
message LoginRequest {
  string username = 1;
  string password = 2;
  string tenant_id = 3;
  string device_info = 4;
  string ip_address = 5;
}
```

### 5. 测试 API 调用

```bash
❯ grpcurl -plaintext \
  -d '{"username":"test","password":"pass","tenant_id":"00000000-0000-0000-0000-000000000001","ip_address":"127.0.0.1","device_info":"grpcurl"}' \
  -H 'tenant-id: 00000000-0000-0000-0000-000000000001' \
  localhost:50051 \
  cuba.iam.auth.AuthService/Login

# 服务正常响应（虽然有数据库错误，但说明 gRPC 调用成功）
ERROR:
  Code: Internal
  Message: Database error: ...
```

## 反射实现代码

### build.rs
```rust
fn main() {
    // 编译 auth.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("src/auth/api/grpc/auth_descriptor.bin")
        .out_dir("src/auth/api/grpc")
        .compile_protos(&["../../proto/iam/auth.proto"], &["../../proto"])
        .expect("Failed to compile auth.proto");

    // 编译 user.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("src/user/api/grpc/user_descriptor.bin")
        .out_dir("src/user/api/grpc")
        .compile_protos(&["../../proto/iam/user.proto"], &["../../proto"])
        .expect("Failed to compile user.proto");

    // 编译 oauth.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("src/oauth/api/grpc/oauth_descriptor.bin")
        .out_dir("src/oauth/api/grpc")
        .compile_protos(&["../../proto/iam/oauth.proto"], &["../../proto"])
        .expect("Failed to compile oauth.proto");

    println!("cargo:rerun-if-changed=../../proto/iam/auth.proto");
    println!("cargo:rerun-if-changed=../../proto/iam/user.proto");
    println!("cargo:rerun-if-changed=../../proto/iam/oauth.proto");
}
```

### auth/api/grpc/mod.rs
```rust
pub mod proto {
    tonic::include_proto!("cuba.iam.auth");
    
    pub const FILE_DESCRIPTOR_SET: &[u8] = 
        include_bytes!("auth_descriptor.bin");
}
```

### user/api/grpc/mod.rs
```rust
pub mod proto {
    tonic::include_proto!("cuba.iam.user");
    
    pub const FILE_DESCRIPTOR_SET: &[u8] = 
        include_bytes!("user_descriptor.bin");
}
```

### main.rs
```rust
use tonic_reflection::server::Builder as ReflectionBuilder;

// 构建反射服务
let reflection_service = ReflectionBuilder::configure()
    .register_encoded_file_descriptor_set(auth::api::grpc::proto::FILE_DESCRIPTOR_SET)
    .register_encoded_file_descriptor_set(user::api::grpc::proto::FILE_DESCRIPTOR_SET)
    .build_v1()
    .map_err(|e| cuba_errors::AppError::internal(format!("Failed to build reflection service: {}", e)))?;

server
    .add_service(AuthServiceServer::new(auth_service))
    .add_service(UserServiceServer::new(user_service))
    .add_service(reflection_service)  // 添加反射服务
    .serve_with_shutdown(addr, cuba_bootstrap::shutdown_signal())
    .await
```

### auth_service_impl.rs - extract_tenant_id 辅助方法
```rust
impl AuthServiceImpl {
    /// 从请求 metadata 中提取 tenant_id
    fn extract_tenant_id<T>(request: &Request<T>) -> Result<TenantId, Status> {
        let tenant_id_str = request
            .metadata()
            .get("tenant-id")
            .and_then(|t| t.to_str().ok())
            .ok_or_else(|| Status::invalid_argument("Missing tenant-id in metadata"))?;

        let uuid = Uuid::parse_str(tenant_id_str)
            .map_err(|_| Status::invalid_argument("Invalid tenant ID format"))?;

        Ok(TenantId::from_uuid(uuid))
    }
}
```

## 使用指南

### 不使用反射（需要 proto 文件）

```bash
grpcurl -plaintext \
  -proto proto/iam/auth.proto \
  -import-path proto \
  -d '{"username":"test","password":"pass"}' \
  -H 'tenant-id: tenant-123' \
  localhost:50051 \
  cuba.iam.auth.AuthService/Login
```

### 使用反射（无需 proto 文件）

```bash
# 1. 列出服务
grpcurl -plaintext localhost:50051 list

# 2. 列出方法
grpcurl -plaintext localhost:50051 list cuba.iam.auth.AuthService

# 3. 查看方法签名
grpcurl -plaintext localhost:50051 describe cuba.iam.auth.AuthService.Login

# 4. 调用方法
grpcurl -plaintext \
  -d '{"username":"test","password":"pass","tenant_id":"xxx","ip_address":"127.0.0.1"}' \
  -H 'tenant-id: xxx' \
  localhost:50051 \
  cuba.iam.auth.AuthService/Login
```

## 总结

✅ gRPC 反射功能已完全实现并验证通过
✅ 可以使用 grpcurl 无需 proto 文件即可测试 API
✅ 所有编译错误已修复
✅ 服务正常启动并响应请求

## 相关文档

- [GRPC_REFLECTION_FIX.md](GRPC_REFLECTION_FIX.md) - 完整的反射实现指南
- [FRONTEND_API_GUIDE.md](FRONTEND_API_GUIDE.md) - 前端接入指南
